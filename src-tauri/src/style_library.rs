//! Content-verified style discovery and recoverable, explicitly confirmed cleanup.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const KEYBOARD_ROOT: &str = "/home/data/Documents/Ritmos de Teclados";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StyleFingerprint {
    pub path: String,
    pub size: u64,
    pub sha256: String,
    pub state: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub keep: StyleFingerprint,
    pub redundant: Vec<StyleFingerprint>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DuplicateScan {
    pub root: String,
    pub groups: Vec<DuplicateGroup>,
    pub scanned_files: usize,
    pub redundant_files: usize,
    pub reclaimable_bytes: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CleanupReport {
    pub manifest_path: String,
    pub moved_files: usize,
    pub restored_files: usize,
    pub quarantined_bytes: u64,
    pub status: String,
    pub errors: Vec<String>,
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn path_string(p: &Path) -> Result<String, String> {
    p.to_str()
        .map(str::to_owned)
        .ok_or_else(|| "Caminho não UTF-8".into())
}
fn style(p: &Path) -> bool {
    p.extension()
        .and_then(|s| s.to_str())
        .map(|s| {
            matches!(
                s.to_ascii_lowercase().as_str(),
                "sty" | "prs" | "sst" | "bcs"
            )
        })
        .unwrap_or(false)
}

/// Reject symbolic links in every existing component, including ancestor directories.
fn checked_path(p: &Path, allow_missing: bool) -> Result<(), String> {
    if !p.is_absolute() {
        return Err(format!("Caminho não absoluto: {}", p.display()));
    }
    let mut current = PathBuf::new();
    let mut missing = false;
    for c in p.components() {
        if matches!(c, Component::ParentDir | Component::CurDir) {
            return Err("Componentes relativos não permitidos".into());
        }
        current.push(c.as_os_str());
        if missing {
            continue;
        }
        match fs::symlink_metadata(&current) {
            Ok(m) if m.file_type().is_symlink() => {
                return Err(format!(
                    "Link simbólico não permitido: {}",
                    current.display()
                ))
            }
            Ok(_) => (),
            Err(e) if allow_missing && e.kind() == std::io::ErrorKind::NotFound => missing = true,
            Err(e) => return Err(format!("{}: {e}", current.display())),
        }
    }
    Ok(())
}
fn absolute(p: &Path) -> Result<PathBuf, String> {
    if p.is_absolute() {
        Ok(p.to_owned())
    } else {
        Ok(std::env::current_dir().map_err(err)?.join(p))
    }
}
fn canonical_safe(p: &Path) -> Result<PathBuf, String> {
    let p = absolute(p)?;
    checked_path(&p, false)?;
    fs::canonicalize(p).map_err(err)
}
fn state(m: &fs::Metadata) -> String {
    #[cfg(unix)]
    {
        format!(
            "{}:{}:{}:{}:{}",
            m.dev(),
            m.ino(),
            m.len(),
            m.mtime(),
            m.mtime_nsec()
        )
    }
    #[cfg(not(unix))]
    {
        format!("{}:{:?}", m.len(), m.modified())
    }
}
fn fingerprint(p: &Path) -> Result<StyleFingerprint, String> {
    checked_path(p, false)?;
    fingerprint_file(File::open(p).map_err(err)?, p, true)
}
fn fingerprint_file(mut f: File, p: &Path, verify_path: bool) -> Result<StyleFingerprint, String> {
    let before = f.metadata().map_err(err)?;
    if !before.is_file() {
        return Err("Esperado arquivo regular".into());
    }
    let mut hash = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = f.read(&mut buf).map_err(err)?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    let after = f.metadata().map_err(err)?;
    if state(&before) != state(&after)
        || (verify_path && state(&after) != state(&fs::metadata(p).map_err(err)?))
    {
        return Err(format!("Arquivo mudou durante leitura: {}", p.display()));
    }
    Ok(StyleFingerprint {
        path: path_string(p)?,
        size: after.len(),
        sha256: format!("{:x}", hash.finalize()),
        state: state(&after),
    })
}
fn unchanged(expected: &StyleFingerprint) -> Result<(), String> {
    let actual = fingerprint(Path::new(&expected.path))?;
    if &actual != expected {
        return Err(format!(
            "Arquivo alterado desde a análise: {}",
            expected.path
        ));
    }
    Ok(())
}
fn same_bytes(a: &Path, b: &Path) -> Result<bool, String> {
    checked_path(a, false)?;
    checked_path(b, false)?;
    let mut a = File::open(a).map_err(err)?;
    let mut b = File::open(b).map_err(err)?;
    if a.metadata().map_err(err)?.len() != b.metadata().map_err(err)?.len() {
        return Ok(false);
    }
    let mut x = [0u8; 65536];
    let mut y = [0u8; 65536];
    loop {
        let nx = a.read(&mut x).map_err(err)?;
        // read_exact avoids assuming independent reads have identical chunk boundaries.
        if nx == 0 {
            return Ok(b.read(&mut y[..1]).map_err(err)? == 0);
        }
        if let Err(e) = b.read_exact(&mut y[..nx]) {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                return Ok(false);
            }
            return Err(err(e));
        }
        if x[..nx] != y[..nx] {
            return Ok(false);
        }
    }
}

/// Recurses deterministically, never following links (including a linked root).
pub fn collect_style_files(roots: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    fn visit(
        p: &Path,
        seen: &mut BTreeSet<PathBuf>,
        files: &mut BTreeSet<PathBuf>,
    ) -> Result<(), String> {
        let m = fs::symlink_metadata(p).map_err(err)?;
        if m.file_type().is_symlink() {
            return Ok(());
        }
        let p = canonical_safe(p)?;
        if m.is_file() {
            if style(&p) {
                files.insert(p);
            }
            return Ok(());
        }
        if !m.is_dir() || !seen.insert(p.clone()) {
            return Ok(());
        }
        let mut children = fs::read_dir(&p)
            .map_err(err)?
            .map(|e| e.map(|e| e.path()).map_err(err))
            .collect::<Result<Vec<_>, _>>()?;
        children.sort();
        for child in children {
            visit(&child, seen, files)?;
        }
        Ok(())
    }
    let mut seen = BTreeSet::new();
    let mut files = BTreeSet::new();
    for root in roots {
        let root = absolute(root)?;
        checked_path(&root, false)?;
        visit(&root, &mut seen, &mut files)?;
    }
    Ok(files.into_iter().collect())
}
fn grouped(paths: &[PathBuf]) -> Result<(Vec<StyleFingerprint>, Vec<DuplicateGroup>), String> {
    let mut sorted = BTreeSet::new();
    for p in paths {
        let p = canonical_safe(p)?;
        if style(&p) {
            sorted.insert(p);
        }
    }
    let mut buckets: BTreeMap<(u64, String), Vec<usize>> = BTreeMap::new();
    let mut keep: Vec<StyleFingerprint> = Vec::new();
    let mut duplicates: BTreeMap<usize, Vec<StyleFingerprint>> = BTreeMap::new();
    for p in sorted {
        let fp = fingerprint(&p)?;
        let key = (fp.size, fp.sha256.clone());
        let mut matched = None;
        if let Some(indices) = buckets.get(&key) {
            for &i in indices {
                if same_bytes(Path::new(&keep[i].path), &p)? {
                    unchanged(&keep[i])?;
                    unchanged(&fp)?;
                    matched = Some(i);
                    break;
                }
            }
        }
        if let Some(i) = matched {
            duplicates.entry(i).or_default().push(fp);
        } else {
            buckets.entry(key).or_default().push(keep.len());
            keep.push(fp);
        }
    }
    let groups = duplicates
        .into_iter()
        .map(|(i, redundant)| DuplicateGroup {
            keep: keep[i].clone(),
            redundant,
        })
        .collect();
    Ok((keep, groups))
}
/// Non-style files are preserved, never used as duplicate candidates.
pub fn deduplicate_style_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let (keep, _) = grouped(paths)?;
    let mut result: BTreeSet<PathBuf> = keep.into_iter().map(|f| PathBuf::from(f.path)).collect();
    for p in paths {
        if !style(p) {
            result.insert(canonical_safe(p)?);
        }
    }
    Ok(result.into_iter().collect())
}
fn scan_root(root: &Path) -> Result<DuplicateScan, String> {
    let root = canonical_safe(root)?;
    if !root.is_dir() {
        return Err("Raiz de estilos não é diretório".into());
    }
    let paths = collect_style_files(&[root.clone()])?;
    let (_, groups) = grouped(&paths)?;
    let redundant_files = groups.iter().map(|g| g.redundant.len()).sum();
    let reclaimable_bytes = groups
        .iter()
        .flat_map(|g| &g.redundant)
        .map(|f| f.size)
        .sum();
    Ok(DuplicateScan {
        root: path_string(&root)?,
        groups,
        scanned_files: paths.len(),
        redundant_files,
        reclaimable_bytes,
    })
}
pub fn scan_keyboard_duplicates(root: &Path) -> Result<DuplicateScan, String> {
    scan_root(root)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JournalEntry {
    original: StyleFingerprint,
    keep: StyleFingerprint,
    quarantined: String,
    phase: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Journal {
    version: u32,
    root: String,
    status: String,
    entries: Vec<JournalEntry>,
}
fn quarantine_dir() -> Result<PathBuf, String> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .ok_or("Diretório de dados indisponível")?;
    Ok(base.join("piano-tect-tauri/style-quarantine"))
}
fn report(path: &Path) -> CleanupReport {
    CleanupReport {
        manifest_path: path.to_string_lossy().into_owned(),
        moved_files: 0,
        restored_files: 0,
        quarantined_bytes: 0,
        status: "unchanged".into(),
        errors: Vec::new(),
    }
}
fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path).and_then(|f| f.sync_all()).map_err(err)
}
fn save_journal(path: &Path, journal: &Journal) -> Result<(), String> {
    checked_path(path.parent().ok_or("Manifesto sem diretório")?, false)?;
    checked_path(path, true)?;
    // A crash can leave a temporary journal behind. Use an exclusive unique
    // name so such an orphan never prevents a later recovery attempt.
    static NEXT_JOURNAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nonce = NEXT_JOURNAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(err)?
        .as_nanos();
    let temp = path.with_extension(format!("json.{}-{stamp}-{nonce}.new", std::process::id()));
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(err)?;
    let result = (|| {
        serde_json::to_writer_pretty(&mut f, journal).map_err(err)?;
        f.write_all(b"\n").map_err(err)?;
        f.sync_all().map_err(err)?;
        fs::rename(&temp, path).map_err(err)?;
        sync_directory(path.parent().unwrap())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
fn content_matches(path: &Path, expected: &StyleFingerprint) -> Result<bool, String> {
    let fp = fingerprint(path)?;
    Ok(fp.size == expected.size && fp.sha256 == expected.sha256 && fp.state == expected.state)
}
// No source unlink is ever used for style files. Linux renameat2 moves the
// actual directory entry atomically, refuses existing destinations and EXDEV.
#[cfg(target_os = "linux")]
mod anchored_move {
    use super::*;
    use std::{
        ffi::CString,
        os::{
            raw::{c_char, c_int, c_uint},
            unix::{
                ffi::OsStrExt,
                io::{AsRawFd, FromRawFd},
            },
        },
    };
    const O_NOFOLLOW: c_int = 0o400000;
    const O_DIRECTORY: c_int = 0o200000;
    const O_CLOEXEC: c_int = 0o2000000;
    const O_NONBLOCK: c_int = 0o4000;
    extern "C" {
        fn openat(dirfd: c_int, pathname: *const c_char, flags: c_int, ...) -> c_int;
        fn renameat2(
            oldfd: c_int,
            old: *const c_char,
            newfd: c_int,
            new: *const c_char,
            flags: c_uint,
        ) -> c_int;
    }
    fn name(name: &std::ffi::OsStr) -> Result<CString, String> {
        CString::new(name.as_bytes()).map_err(err)
    }
    fn open(dir: &File, name: &CString, flags: c_int) -> Result<File, String> {
        // SAFETY: valid directory fd and NUL-terminated pathname; no O_CREAT.
        let fd = unsafe {
            openat(
                dir.as_raw_fd(),
                name.as_ptr(),
                flags | O_NOFOLLOW | O_CLOEXEC,
            )
        };
        if fd < 0 {
            Err(err(std::io::Error::last_os_error()))
        } else {
            // SAFETY: openat returned a new owned descriptor.
            Ok(unsafe { File::from_raw_fd(fd) })
        }
    }
    fn parent(path: &Path) -> Result<(File, CString), String> {
        checked_path(path, true)?;
        let mut dir = File::open("/").map_err(err)?;
        for c in path.parent().ok_or("Caminho sem diretório")?.components() {
            match c {
                Component::RootDir => (),
                Component::Normal(n) => dir = open(&dir, &name(n)?, O_DIRECTORY)?,
                _ => return Err("Componente não permitido".into()),
            }
        }
        Ok((dir, name(path.file_name().ok_or("Caminho sem nome")?)?))
    }
    fn rename(from: &File, old: &CString, to: &File, new: &CString) -> Result<(), String> {
        // SAFETY: both descriptors and C strings remain alive for the syscall.
        if unsafe {
            renameat2(
                from.as_raw_fd(),
                old.as_ptr(),
                to.as_raw_fd(),
                new.as_ptr(),
                1,
            )
        } != 0
        {
            return Err(err(std::io::Error::last_os_error()));
        }
        Ok(())
    }
    fn matches(file: File, expected: &StyleFingerprint) -> Result<bool, String> {
        let fp = fingerprint_file(file, Path::new(&expected.path), false)?;
        Ok(fp == *expected)
    }
    pub(super) fn move_file(
        from: &Path,
        to: &Path,
        expected: &StyleFingerprint,
    ) -> Result<(), String> {
        move_with_hook(from, to, expected, || {})
    }
    pub(super) fn move_with_hook(
        from: &Path,
        to: &Path,
        expected: &StyleFingerprint,
        before_rename: impl FnOnce(),
    ) -> Result<(), String> {
        let (source_dir, source_name) = parent(from)?;
        let (target_dir, target_name) = parent(to)?;
        let source = open(&source_dir, &source_name, O_NONBLOCK)?;
        source.sync_all().map_err(err)?;
        if !matches(source, expected)? {
            return Err("Origem alterada antes de movimentação".into());
        }
        before_rename();
        rename(&source_dir, &source_name, &target_dir, &target_name)?;
        let verification =
            open(&target_dir, &target_name, O_NONBLOCK).and_then(|f| matches(f, expected));
        if !matches!(verification, Ok(true)) {
            // Preserve an unexpectedly replaced object. Never overwrite a newly
            // created source; if rollback collides, the object remains quarantined.
            let rollback = rename(&target_dir, &target_name, &source_dir, &source_name);
            let _ = source_dir.sync_all();
            let _ = target_dir.sync_all();
            return Err(format!(
                "Objeto mudou durante movimentação; rollback: {}",
                rollback.err().unwrap_or_else(|| "concluído".into())
            ));
        }
        target_dir.sync_all().map_err(err)?;
        source_dir.sync_all().map_err(err)
    }
}
fn move_no_replace(from: &Path, to: &Path, expected: &StyleFingerprint) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        anchored_move::move_file(from, to, expected)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (from, to, expected);
        Err("Movimentação segura exige Linux renameat2".into())
    }
}

fn inside_style(root: &Path, fp: &StyleFingerprint) -> Result<(), String> {
    let p = Path::new(&fp.path);
    checked_path(p, true)?;
    if !p.starts_with(root) || p == root || !style(p) {
        return Err("Arquivo fora da raiz ou extensão não suportada".into());
    }
    Ok(())
}
fn validate_scan(scan: &DuplicateScan, allowed: &Path) -> Result<PathBuf, String> {
    let root = canonical_safe(Path::new(&scan.root))?;
    if root != canonical_safe(allowed)? {
        return Err("Limpeza permitida somente na biblioteca de teclados".into());
    }
    let mut seen = BTreeSet::new();
    let mut count = 0usize;
    let mut bytes = 0u64;
    for group in &scan.groups {
        inside_style(&root, &group.keep)?;
        if !seen.insert(group.keep.path.clone()) {
            return Err("Caminho repetido na análise".into());
        }
        unchanged(&group.keep)?;
        for fp in &group.redundant {
            inside_style(&root, fp)?;
            if !seen.insert(fp.path.clone()) {
                return Err("Caminho repetido na análise".into());
            }
            unchanged(fp)?;
            if fp.size != group.keep.size
                || fp.sha256 != group.keep.sha256
                || !same_bytes(Path::new(&group.keep.path), Path::new(&fp.path))?
            {
                return Err("Arquivos não são duplicados idênticos".into());
            }
            count = count.checked_add(1).ok_or("Contagem excedida")?;
            bytes = bytes.checked_add(fp.size).ok_or("Tamanho excedido")?;
        }
        unchanged(&group.keep)?;
    }
    if count != scan.redundant_files
        || bytes != scan.reclaimable_bytes
        || scan.scanned_files < seen.len()
    {
        return Err("Resumo de análise inconsistente".into());
    }
    Ok(root)
}
fn prepare_base(base: &Path, root: &Path) -> Result<PathBuf, String> {
    let base = absolute(base)?;
    checked_path(&base, true)?;
    if base.starts_with(root) || root.starts_with(&base) {
        return Err("Quarentena deve ficar fora da biblioteca".into());
    }
    fs::create_dir_all(&base).map_err(err)?;
    let base = canonical_safe(&base)?;
    if base.starts_with(root) || root.starts_with(&base) {
        return Err("Quarentena sobrepõe biblioteca".into());
    }
    Ok(base)
}
pub fn quarantine_keyboard_duplicates(
    scan: &DuplicateScan,
    confirmed: bool,
    quarantine_base: &Path,
) -> Result<CleanupReport, String> {
    let expected = absolute(&quarantine_dir()?)?;
    if absolute(quarantine_base)? != expected {
        return Err("Diretório de quarentena não autorizado".into());
    }
    cleanup_at(scan, confirmed, quarantine_base, Path::new(KEYBOARD_ROOT))
}
fn cleanup_at(
    scan: &DuplicateScan,
    confirmed: bool,
    base: &Path,
    allowed: &Path,
) -> Result<CleanupReport, String> {
    if !confirmed {
        return Err("Confirmação explícita obrigatória".into());
    }
    let root = validate_scan(scan, allowed)?;
    if scan.redundant_files == 0 {
        return Ok(report(Path::new("")));
    }
    let base = prepare_base(base, &root)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(err)?
        .as_nanos();
    let mut run = None;
    for sequence in 0..100u32 {
        let p = base.join(format!("{}-{stamp}-{sequence}", std::process::id()));
        match fs::create_dir(&p) {
            Ok(()) => {
                run = Some(p);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(err(e)),
        }
    }
    let run = run.ok_or("Não foi possível criar quarentena exclusiva")?;
    let files = run.join("files");
    fs::create_dir(&files).map_err(err)?;
    let manifest = run.join("manifest.json");
    let mut journal = Journal {
        version: 1,
        root: path_string(&root)?,
        status: "prepared".into(),
        entries: Vec::new(),
    };
    for group in &scan.groups {
        for fp in &group.redundant {
            let quarantined =
                path_string(&files.join(format!("{:08}.bin", journal.entries.len())))?;
            journal.entries.push(JournalEntry {
                original: fp.clone(),
                keep: group.keep.clone(),
                quarantined,
                phase: "planned".into(),
            });
        }
    }
    // All intent is durable before the first library mutation.
    save_journal(&manifest, &journal)?;
    sync_directory(&base)?;
    let mut result = report(&manifest);
    let operation = (|| -> Result<(), String> {
        for index in 0..journal.entries.len() {
            let entry = &journal.entries[index];
            unchanged(&entry.keep)?;
            unchanged(&entry.original)?;
            if !same_bytes(Path::new(&entry.keep.path), Path::new(&entry.original.path))? {
                return Err("Duplicado mudou antes da movimentação".into());
            }
            unchanged(&entry.keep)?;
            unchanged(&entry.original)?;
            journal.status = "moving".into();
            journal.entries[index].phase = "moving".into();
            save_journal(&manifest, &journal)?;
            let entry = &journal.entries[index];
            move_no_replace(
                Path::new(&entry.original.path),
                Path::new(&entry.quarantined),
                &entry.original,
            )?;
            result.moved_files += 1;
            result.quarantined_bytes += entry.original.size;
            journal.entries[index].phase = "moved".into();
            save_journal(&manifest, &journal)?;
        }
        journal.status = "quarantined".into();
        save_journal(&manifest, &journal)
    })();
    match operation {
        Ok(()) => result.status = "quarantined".into(),
        Err(error) => {
            result.errors.push(error);
            let recovery = recover(&manifest, &mut journal);
            result.restored_files = recovery.restored_files;
            result.quarantined_bytes = recovery.quarantined_bytes;
            result.errors.extend(recovery.errors);
            result.status = if recovery.status == "restored" {
                "rolled_back"
            } else {
                "recovery_required"
            }
            .into();
        }
    }
    Ok(result)
}

fn validate_journal(
    manifest: &Path,
    journal: &Journal,
    allowed: &Path,
    base: &Path,
) -> Result<(), String> {
    if journal.version != 1 {
        return Err("Versão de manifesto não suportada".into());
    }
    let root = canonical_safe(allowed)?;
    if Path::new(&journal.root) != root {
        return Err("Raiz do manifesto não autorizada".into());
    }
    let base = canonical_safe(base)?;
    if base.starts_with(&root) || root.starts_with(&base) {
        return Err("Quarentena sobrepõe biblioteca".into());
    }
    let run = manifest.parent().ok_or("Manifesto sem diretório")?;
    if manifest.file_name().and_then(|n| n.to_str()) != Some("manifest.json")
        || run.parent() != Some(base.as_path())
    {
        return Err("Manifesto fora da quarentena autorizada".into());
    }
    checked_path(manifest, false)?;
    let files = run.join("files");
    checked_path(&files, false)?;
    let mut original_paths = BTreeSet::new();
    let mut quarantine_paths = BTreeSet::new();
    for entry in &journal.entries {
        inside_style(&root, &entry.original)?;
        inside_style(&root, &entry.keep)?;
        if entry.original.path == entry.keep.path
            || entry.original.size != entry.keep.size
            || entry.original.sha256 != entry.keep.sha256
        {
            return Err("Entrada de manifesto inconsistente".into());
        }
        let q = Path::new(&entry.quarantined);
        checked_path(q, true)?;
        if q.parent() != Some(files.as_path())
            || !original_paths.insert(entry.original.path.clone())
            || !quarantine_paths.insert(q.to_owned())
        {
            return Err("Entrada fora da quarentena ou repetida".into());
        }
        if !matches!(
            entry.phase.as_str(),
            "planned" | "moving" | "moved" | "restoring" | "restored"
        ) {
            return Err("Estado de entrada inválido".into());
        }
    }
    Ok(())
}
fn exists_regular(path: &Path) -> Result<bool, String> {
    checked_path(path, true)?;
    match fs::symlink_metadata(path) {
        Ok(m) if m.is_file() => Ok(true),
        Ok(_) => Err(format!("Esperado arquivo regular: {}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(err(e)),
    }
}
fn recover(manifest: &Path, journal: &mut Journal) -> CleanupReport {
    let mut result = report(manifest);
    journal.status = "restoring".into();
    if let Err(error) = save_journal(manifest, journal) {
        result.errors.push(error);
        result.status = "recovery_required".into();
        return result;
    }
    for index in (0..journal.entries.len()).rev() {
        let restore_one = (|| -> Result<bool, String> {
            let entry = &journal.entries[index];
            let original = Path::new(&entry.original.path);
            let q = Path::new(&entry.quarantined);
            let has_original = exists_regular(original)?;
            let has_q = exists_regular(q)?;
            if !has_q {
                if !has_original || !content_matches(original, &entry.original)? {
                    return Err(format!(
                        "Arquivo ausente ou alterado: {}",
                        original.display()
                    ));
                }
                return Ok(false);
            }
            if !content_matches(q, &entry.original)? {
                return Err(format!("Arquivo de quarentena alterado: {}", q.display()));
            }
            if has_original {
                // Only remove the second hard link of our original inode. An unrelated
                // file at the original name is a collision, even with identical bytes.
                if !content_matches(original, &entry.original)? || !same_bytes(original, q)? {
                    return Err(format!(
                        "Destino já existe; preservado: {}",
                        original.display()
                    ));
                }
                journal.entries[index].phase = "restoring".into();
                save_journal(manifest, journal)?;
                let entry = &journal.entries[index];
                if !content_matches(Path::new(&entry.original.path), &entry.original)?
                    || !content_matches(Path::new(&entry.quarantined), &entry.original)?
                {
                    return Err("Arquivo alterado durante recuperação".into());
                }
                // Legacy hard-link journals can have both names. Archive the
                // redundant name without unlinking anything: an atomic no-clobber
                // move preserves a concurrent replacement rather than deleting it.
                let archived = Path::new(&entry.quarantined).with_extension("recovered-link");
                move_no_replace(Path::new(&entry.quarantined), &archived, &entry.original)?;
                return Ok(false);
            }
            // Never recreate missing ancestor directories: changes since the scan
            // require user intervention instead of potentially restoring elsewhere.
            checked_path(original.parent().ok_or("Destino sem diretório")?, false)?;
            journal.entries[index].phase = "restoring".into();
            save_journal(manifest, journal)?;
            let entry = &journal.entries[index];
            move_no_replace(
                Path::new(&entry.quarantined),
                Path::new(&entry.original.path),
                &entry.original,
            )?;
            Ok(true)
        })();
        match restore_one {
            Ok(restored) => {
                if restored {
                    result.restored_files += 1;
                }
                journal.entries[index].phase = "restored".into();
                if let Err(e) = save_journal(manifest, journal) {
                    result.errors.push(e);
                }
            }
            Err(e) => result.errors.push(e),
        }
    }
    for entry in &journal.entries {
        if fs::symlink_metadata(&entry.quarantined).is_ok() {
            result.quarantined_bytes = result.quarantined_bytes.saturating_add(entry.original.size);
        }
    }
    result.status = if result.errors.is_empty() {
        "restored"
    } else {
        "recovery_required"
    }
    .into();
    journal.status = result.status.clone();
    if let Err(e) = save_journal(manifest, journal) {
        result.errors.push(e);
        result.status = "recovery_required".into();
    }
    result
}
pub fn restore_quarantine(manifest: &Path) -> Result<CleanupReport, String> {
    restore_at(manifest, Path::new(KEYBOARD_ROOT), &quarantine_dir()?)
}
fn restore_at(manifest: &Path, allowed: &Path, base: &Path) -> Result<CleanupReport, String> {
    let manifest = canonical_safe(manifest)?;
    let metadata = fs::metadata(&manifest).map_err(err)?;
    if !metadata.is_file() || metadata.len() > 64 * 1024 * 1024 {
        return Err("Manifesto inválido ou excessivo".into());
    }
    let journal_file = File::open(&manifest).map_err(err)?;
    let mut journal: Journal = serde_json::from_reader(journal_file).map_err(err)?;
    validate_journal(&manifest, &journal, allowed, base)?;
    Ok(recover(&manifest, &mut journal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture {
        dir: PathBuf,
        root: PathBuf,
        quarantine: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!(
                "piano-style-test-{}-{stamp}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let root = dir.join("styles");
            let quarantine = dir.join("quarantine");
            fs::create_dir_all(&root).unwrap();
            Self {
                dir,
                root,
                quarantine,
            }
        }
        fn write(&self, name: &str, contents: &[u8]) -> PathBuf {
            let path = self.root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, contents).unwrap();
            path
        }
        fn scan(&self) -> DuplicateScan {
            scan_root(&self.root).unwrap()
        }
        fn clean(&self, scan: &DuplicateScan) -> Result<CleanupReport, String> {
            cleanup_at(scan, true, &self.quarantine, &self.root)
        }
        fn restore(&self, result: &CleanupReport) -> Result<CleanupReport, String> {
            restore_at(
                Path::new(&result.manifest_path),
                &self.root,
                &self.quarantine,
            )
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }
    #[test]
    fn content_only_deterministic_style_deduplication() {
        let f = Fixture::new();
        let a = f.write("a.STY", b"same");
        let b = f.write("z.prs", b"same");
        let c = f.write("c.sst", b"else");
        let d = f.write("d.bcs", b"same");
        let other = f.write("e.txt", b"same");
        let collected = collect_style_files(&[f.root.clone(), f.root.clone()]).unwrap();
        assert_eq!(collected.len(), 4);
        assert_eq!(
            deduplicate_style_paths(&[b, c.clone(), a.clone(), d, other.clone()]).unwrap(),
            vec![a.clone(), c, other]
        );
        let scan = f.scan();
        assert_eq!(scan.groups.len(), 1);
        assert_eq!(scan.groups[0].keep.path, a.to_str().unwrap());
        assert_eq!(scan.redundant_files, 2);
        assert_eq!(scan.reclaimable_bytes, 8);
    }
    #[test]
    fn confirmed_quarantine_restores_and_is_idempotent() {
        let f = Fixture::new();
        let a = f.write("a.sty", b"duplicate");
        let b = f.write("nested/b.prs", b"duplicate");
        let scan = f.scan();
        assert!(cleanup_at(&scan, false, &f.quarantine, &f.root).is_err());
        assert!(a.exists() && b.exists());
        let result = f.clean(&scan).unwrap();
        assert_eq!(result.status, "quarantined");
        assert_eq!(result.moved_files, 1);
        assert!(a.exists());
        assert!(!b.exists());
        let restored = f.restore(&result).unwrap();
        assert_eq!(restored.status, "restored");
        assert_eq!(restored.restored_files, 1);
        assert_eq!(fs::read(&b).unwrap(), b"duplicate");
        let again = f.restore(&result).unwrap();
        assert_eq!(again.status, "restored");
        assert_eq!(again.restored_files, 0);
    }
    #[test]
    fn changed_candidate_or_keeper_aborts_before_mutation() {
        let f = Fixture::new();
        let a = f.write("a.sty", b"duplicate");
        let b = f.write("b.sty", b"duplicate");
        let scan = f.scan();
        fs::write(&b, b"different").unwrap();
        assert!(f.clean(&scan).is_err());
        assert_eq!(fs::read(&a).unwrap(), b"duplicate");
        assert_eq!(fs::read(&b).unwrap(), b"different");
        fs::write(&b, b"duplicate").unwrap();
        let scan = f.scan();
        fs::write(&a, b"different").unwrap();
        assert!(f.clean(&scan).is_err());
        assert!(b.exists());
    }
    #[test]
    fn restore_never_overwrites_existing_destination() {
        let f = Fixture::new();
        f.write("a.sty", b"duplicate");
        let b = f.write("b.sty", b"duplicate");
        let result = f.clean(&f.scan()).unwrap();
        fs::write(&b, b"new content").unwrap();
        let restored = f.restore(&result).unwrap();
        assert_eq!(restored.status, "recovery_required");
        assert_eq!(restored.restored_files, 0);
        assert_eq!(fs::read(&b).unwrap(), b"new content");
        assert!(restored.quarantined_bytes > 0);
        fs::remove_file(&b).unwrap();
        assert_eq!(f.restore(&result).unwrap().status, "restored");
    }
    #[test]
    fn altered_quarantine_is_preserved_for_manual_recovery() {
        let f = Fixture::new();
        f.write("a.sty", b"duplicate");
        let b = f.write("b.sty", b"duplicate");
        let result = f.clean(&f.scan()).unwrap();
        let journal: Journal =
            serde_json::from_reader(File::open(&result.manifest_path).unwrap()).unwrap();
        fs::write(&journal.entries[0].quarantined, b"tampered").unwrap();
        let restored = f.restore(&result).unwrap();
        assert_eq!(restored.status, "recovery_required");
        assert!(!b.exists());
        assert_eq!(
            fs::read(&journal.entries[0].quarantined).unwrap(),
            b"tampered"
        );
    }
    #[test]
    fn duplicate_metadata_cannot_authorize_unequal_bytes() {
        let f = Fixture::new();
        let a = f.write("a.sty", b"same");
        let b = f.write("b.sty", b"else");
        let keep = fingerprint(&a).unwrap();
        let mut candidate = fingerprint(&b).unwrap();
        candidate.sha256 = keep.sha256.clone();
        let scan = DuplicateScan {
            root: f.root.to_str().unwrap().into(),
            groups: vec![DuplicateGroup {
                keep,
                redundant: vec![candidate],
            }],
            scanned_files: 2,
            redundant_files: 1,
            reclaimable_bytes: 4,
        };
        assert!(f.clean(&scan).is_err());
        assert_eq!(fs::read(&b).unwrap(), b"else");
    }
    #[test]
    fn rejects_duplicate_candidates_and_in_tree_quarantine() {
        let f = Fixture::new();
        f.write("a.sty", b"same");
        f.write("b.sty", b"same");
        let mut scan = f.scan();
        let extra = scan.groups[0].redundant[0].clone();
        scan.groups[0].redundant.push(extra);
        assert!(f.clean(&scan).is_err());
        assert!(cleanup_at(&f.scan(), true, &f.root.join("quarantine"), &f.root).is_err());
    }
    #[test]
    fn journal_recovers_interrupted_move_with_both_links() {
        let f = Fixture::new();
        f.write("a.sty", b"same");
        let b = f.write("b.sty", b"same");
        let result = f.clean(&f.scan()).unwrap();
        let journal: Journal =
            serde_json::from_reader(File::open(&result.manifest_path).unwrap()).unwrap();
        fs::hard_link(&journal.entries[0].quarantined, &b).unwrap();
        let restored = f.restore(&result).unwrap();
        assert_eq!(restored.status, "restored");
        assert_eq!(fs::read(&b).unwrap(), b"same");
        assert!(!Path::new(&journal.entries[0].quarantined).exists());
    }
    #[cfg(unix)]
    #[test]
    fn does_not_follow_links_or_directory_cycles() {
        use std::os::unix::fs::symlink;
        let f = Fixture::new();
        let a = f.write("a.sty", b"same");
        f.write("nested/b.sty", b"different");
        symlink(&f.root, f.root.join("nested/cycle")).unwrap();
        symlink(&a, f.root.join("linked.sty")).unwrap();
        assert_eq!(collect_style_files(&[f.root.clone()]).unwrap().len(), 2);
        let alias = f.dir.join("alias");
        symlink(&f.root, &alias).unwrap();
        assert!(collect_style_files(&[alias.clone()]).is_err());
        assert!(collect_style_files(&[alias.join("a.sty")]).is_err());
        let scan = f.scan();
        assert_eq!(scan.redundant_files, 0);
    }
    #[cfg(unix)]
    #[test]
    fn replaced_ancestor_symlink_blocks_cleanup_and_restore() {
        use std::os::unix::fs::symlink;
        let f = Fixture::new();
        f.write("a.sty", b"same");
        let b = f.write("nested/b.sty", b"same");
        let scan = f.scan();
        let moved = f.root.join("nested-old");
        fs::rename(f.root.join("nested"), &moved).unwrap();
        symlink(&moved, f.root.join("nested")).unwrap();
        assert!(f.clean(&scan).is_err());
        assert!(b.exists());
        fs::remove_file(f.root.join("nested")).unwrap();
        fs::rename(&moved, f.root.join("nested")).unwrap();
        let result = f.clean(&f.scan()).unwrap();
        fs::rename(f.root.join("nested"), &moved).unwrap();
        symlink(&moved, f.root.join("nested")).unwrap();
        assert!(f.restore(&result).is_err());
        assert!(!b.exists());
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn atomic_move_rolls_back_replaced_source_without_deletion() {
        let f = Fixture::new();
        let source = f.write("a.sty", b"expected");
        let expected = fingerprint(&source).unwrap();
        fs::create_dir(&f.quarantine).unwrap();
        let target = f.quarantine.join("candidate.bin");
        let saved = f.root.join("saved.sty");
        let result = anchored_move::move_with_hook(&source, &target, &expected, || {
            fs::rename(&source, &saved).unwrap();
            fs::write(&source, b"unrelated replacement").unwrap();
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&source).unwrap(), b"unrelated replacement");
        assert_eq!(fs::read(&saved).unwrap(), b"expected");
        assert!(!target.exists());
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn atomic_move_never_overwrites_racing_destination() {
        let f = Fixture::new();
        let source = f.write("a.sty", b"expected");
        let expected = fingerprint(&source).unwrap();
        fs::create_dir(&f.quarantine).unwrap();
        let target = f.quarantine.join("candidate.bin");
        let result = anchored_move::move_with_hook(&source, &target, &expected, || {
            fs::write(&target, b"destination replacement").unwrap();
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&source).unwrap(), b"expected");
        assert_eq!(fs::read(&target).unwrap(), b"destination replacement");
    }
    #[cfg(target_os = "linux")]
    #[test]
    fn anchored_move_does_not_follow_racing_parent_symlink() {
        use std::os::unix::fs::symlink;
        let f = Fixture::new();
        let source = f.write("nested/a.sty", b"expected");
        let foreign = f.dir.join("foreign");
        fs::create_dir(&foreign).unwrap();
        fs::write(foreign.join("a.sty"), b"foreign").unwrap();
        let expected = fingerprint(&source).unwrap();
        fs::create_dir(&f.quarantine).unwrap();
        let target = f.quarantine.join("candidate.bin");
        let saved_dir = f.root.join("old-nested");
        anchored_move::move_with_hook(&source, &target, &expected, || {
            fs::rename(f.root.join("nested"), &saved_dir).unwrap();
            symlink(&foreign, f.root.join("nested")).unwrap();
        })
        .unwrap();
        assert_eq!(fs::read(foreign.join("a.sty")).unwrap(), b"foreign");
        assert_eq!(fs::read(&target).unwrap(), b"expected");
        assert!(!saved_dir.join("a.sty").exists());
    }
    #[test]
    fn partial_recovery_preserves_collision_and_restores_other_files() {
        let f = Fixture::new();
        f.write("a.sty", b"same");
        let b = f.write("b.sty", b"same");
        let c = f.write("c.sty", b"same");
        let result = f.clean(&f.scan()).unwrap();
        fs::write(&b, b"foreign").unwrap();
        let recovered = f.restore(&result).unwrap();
        assert_eq!(recovered.status, "recovery_required");
        assert_eq!(recovered.restored_files, 1);
        assert_eq!(fs::read(&b).unwrap(), b"foreign");
        assert_eq!(fs::read(&c).unwrap(), b"same");
        assert_eq!(recovered.quarantined_bytes, 4);
        fs::remove_file(&b).unwrap();
        let complete = f.restore(&result).unwrap();
        assert_eq!(complete.status, "restored");
        assert_eq!(complete.restored_files, 1);
    }
    #[test]
    fn orphaned_journal_temp_does_not_block_recovery() {
        let f = Fixture::new();
        f.write("a.sty", b"same");
        f.write("b.sty", b"same");
        let result = f.clean(&f.scan()).unwrap();
        fs::write(
            Path::new(&result.manifest_path).with_extension("json.new"),
            b"interrupted",
        )
        .unwrap();
        assert_eq!(f.restore(&result).unwrap().status, "restored");
    }
    #[test]
    fn manifest_root_and_quarantine_paths_are_validated() {
        let f = Fixture::new();
        f.write("a.sty", b"same");
        f.write("b.sty", b"same");
        let result = f.clean(&f.scan()).unwrap();
        let mut journal: Journal =
            serde_json::from_reader(File::open(&result.manifest_path).unwrap()).unwrap();
        journal.entries[0].quarantined = f.dir.join("foreign.bin").to_str().unwrap().into();
        save_journal(Path::new(&result.manifest_path), &journal).unwrap();
        assert!(f.restore(&result).is_err());
        assert!(quarantine_keyboard_duplicates(&f.scan(), true, &f.quarantine).is_err());
    }
}
