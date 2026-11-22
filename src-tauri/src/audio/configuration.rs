use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};

pub(super) const VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct RuntimeCapture {
    pub params: EngineParams,
    pub instrument: InstrumentModel,
    pub arranger: MusicPackageSummary,
    pub package: Option<Arc<LoadedMusicPackage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct AssetReference {
    pub path: PathBuf,
    pub sha256: String,
}

impl AssetReference {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let path = std::fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        let mut hash = Sha256::new();
        let mut chunk = [0_u8; 65536];
        loop {
            let len = file.read(&mut chunk).map_err(|e| e.to_string())?;
            if len == 0 { break; }
            hash.update(&chunk[..len]);
        }
        Ok(Self { path, sha256: format!("{:x}", hash.finalize()) })
    }

    pub fn validate(&self) -> Result<(), String> {
        if Self::from_path(&self.path)?.sha256 != self.sha256 {
            return Err(format!("Saved asset changed: {}", self.path.display()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct SavedConfiguration {
    pub version: u32,
    pub params: EngineParams,
    pub instrument_id: String,
    pub preset_id: String,
    pub preset_b_id: Option<String>,
    pub midi_config: MidiConfig,
    pub requested_buffer_size: u32,
    pub arranger: MusicPackageSummary,
    pub package_sources: Vec<AssetReference>,
    pub instrument_names: Vec<String>,
    pub soundfont: Option<AssetReference>,
    pub package_uses_soundfont: bool,
    pub imports: Vec<(AssetReference, String)>,
}

pub(super) fn valid_buffer_size(size: u32) -> bool {
    matches!(size, 128 | 150 | 190 | 210 | 256 | 512 | 1024)
}

pub(super) fn default_configuration_path() -> PathBuf {
    let root = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    root.join("piano-tect-tauri/current-setup.json")
}

pub(super) fn read(path: &Path) -> Result<Option<SavedConfiguration>, String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let setup: SavedConfiguration = serde_json::from_slice(&bytes).map_err(|e| format!("Invalid saved configuration: {e}"))?;
    if setup.version != VERSION { return Err(format!("Unsupported configuration version {}", setup.version)); }
    if !valid_buffer_size(setup.requested_buffer_size) { return Err("Invalid saved buffer size".into()); }
    for asset in setup.package_sources.iter().chain(setup.soundfont.iter()).chain(setup.imports.iter().map(|(asset, _)| asset)) { asset.validate()?; }
    Ok(Some(setup))
}

pub(super) fn write_atomic(path: &Path, setup: &SavedConfiguration) -> Result<(), String> {
    let parent = path.parent().ok_or("Configuration path has no parent")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    static NEXT_WRITE: AtomicU64 = AtomicU64::new(0);
    let serial = NEXT_WRITE.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(".current-setup-{}-{serial}.tmp", std::process::id()));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(&mut file, setup).map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        std::fs::rename(&temporary, path).map_err(|e| e.to_string())?;
        std::fs::File::open(parent).and_then(|directory| directory.sync_all()).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if result.is_err() { let _ = std::fs::remove_file(&temporary); }
    result
}

pub(super) fn clear(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub(super) fn restore_arranger(arranger: &mut MusicStudioArrangerEngine, state: &MusicPackageSummary) {
    arranger.set_transport(false, Some(false), None);
    arranger.set_style(state.active_style_index);
    arranger.set_section(&state.active_section, true);
    arranger.set_auto_fill(state.auto_fill);
    arranger.set_tempo_locked(state.tempo_locked);
    arranger.set_transport(false, Some(state.sync_start), Some(state.bpm));
    for (slot, index) in [("upper1", Some(state.upper1_instrument)), ("upper2", state.upper2_instrument), ("lower", state.lower_instrument)] {
        arranger.set_keyboard_voice(slot, index, Some(state.split_key), Some(state.split_enabled), Some(state.chord_scan_lower), Some(state.chord_scan_upper), Some(state.upper_octave), Some(state.transpose), Some(&state.chord_mode));
    }
    for slot in 0..8 {
        arranger.set_track_volume(slot, state.track_volumes[slot]);
        arranger.set_track_instrument(slot, state.track_instruments[slot]);
    }
    arranger.all_notes_off();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;
    static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("piano-tect-setup-{}-{id}", std::process::id()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
    }

    fn setup() -> SavedConfiguration {
        let instruments = build_factory_instruments();
        SavedConfiguration {
            version: VERSION, params: instruments[0].default_params.clone(),
            instrument_id: instruments[0].id.clone(), preset_id: "test".into(), preset_b_id: None,
            midi_config: MidiConfig::default(), requested_buffer_size: 150,
            arranger: MusicPackageSummary::default(), package_sources: Vec::new(), instrument_names: Vec::new(), soundfont: None, imports: Vec::new(), package_uses_soundfont: false,
        }
    }

    #[test]
    fn atomic_roundtrip_clear_and_no_sample_payload() {
        let directory = TestDirectory::new();
        let path = directory.0.join("config/current.json");
        assert!(read(&path).unwrap().is_none());
        let mut saved = setup();
        write_atomic(&path, &saved).unwrap();
        saved.arranger.split_key = 54;
        saved.params.volume_db = -7.5;
        write_atomic(&path, &saved).unwrap();
        let restored = read(&path).unwrap().unwrap();
        assert_eq!(restored.requested_buffer_size, 150);
        assert_eq!(restored.params.volume_db, -7.5);
        assert!(!restored.arranger.rhythm_playing);
        let json = std::fs::read_to_string(&path).unwrap();
        assert!(!json.contains("pcm_l"));
        assert!(!json.contains("pcm_r"));
        clear(&path).unwrap();
        clear(&path).unwrap();
        assert!(read(&path).unwrap().is_none());
        // Clearing the file cannot mutate the supplied live setup.
        assert_eq!(saved.params.volume_db, -7.5);
    }

    #[test]
    fn rejects_changed_missing_assets_and_unknown_versions() {
        let directory = TestDirectory::new();
        let path = directory.0.join("setup.json");
        let asset_path = directory.0.join("style.sty");
        std::fs::write(&asset_path, b"same style").unwrap();
        let mut saved = setup();
        saved.package_sources.push(AssetReference::from_path(&asset_path).unwrap());
        write_atomic(&path, &saved).unwrap();
        assert!(read(&path).is_ok());
        std::fs::write(&asset_path, b"changed style").unwrap();
        assert!(read(&path).unwrap_err().contains("changed"));
        std::fs::remove_file(&asset_path).unwrap();
        assert!(read(&path).is_err());
        saved.package_sources.clear();
        saved.version = VERSION + 1;
        write_atomic(&path, &saved).unwrap();
        assert!(read(&path).unwrap_err().contains("version"));
        std::fs::write(&path, b"{").unwrap();
        assert!(read(&path).unwrap_err().contains("Invalid"));
    }
}
