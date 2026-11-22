//! Zero-dependency pure-Rust SoundFont 2 (`.sf2` RIFF `sfbk`) parser and lazy on-demand sample loader.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;

const DEFAULT_SOUNDFONT_PATHS: [&str; 3] = [
    "/usr/share/sounds/sf2/FluidR3_GM.sf2",
    "/usr/share/sounds/sf2/default-GM.sf2",
    "/usr/share/sounds/sf2/TimGM6mb.sf2",
];

// SF2 generator operator IDs.
const GEN_START_ADDRS_OFFSET: u16 = 0;
const GEN_END_ADDRS_OFFSET: u16 = 1;
const GEN_STARTLOOP_ADDRS_OFFSET: u16 = 2;
const GEN_ENDLOOP_ADDRS_OFFSET: u16 = 3;
const GEN_START_ADDRS_COARSE_OFFSET: u16 = 4;
const GEN_END_ADDRS_COARSE_OFFSET: u16 = 12;
const GEN_PAN: u16 = 17;
const GEN_INSTRUMENT: u16 = 41;
const GEN_KEY_RANGE: u16 = 43;
const GEN_VEL_RANGE: u16 = 44;
const GEN_STARTLOOP_ADDRS_COARSE_OFFSET: u16 = 45;
const GEN_INITIAL_ATTENUATION: u16 = 48;
const GEN_ENDLOOP_ADDRS_COARSE_OFFSET: u16 = 50;
const GEN_COARSE_TUNE: u16 = 51;
const GEN_FINE_TUNE: u16 = 52;
const GEN_SAMPLE_ID: u16 = 53;
const GEN_SAMPLE_MODES: u16 = 54;
const GEN_OVERRIDING_ROOT_KEY: u16 = 58;

const SAMPLE_TYPE_MONO: u16 = 1;
const SAMPLE_TYPE_RIGHT: u16 = 2;
const SAMPLE_TYPE_LEFT: u16 = 4;
const SAMPLE_TYPE_ROM_BIT: u16 = 0x8000;

/// Locate the first existing default General MIDI SoundFont on disk.
pub fn find_default_soundfont() -> Option<PathBuf> {
    DEFAULT_SOUNDFONT_PATHS
        .iter()
        .map(Path::new)
        .find(|p| p.is_file())
        .map(Path::to_path_buf)
}

/// Summary metadata for a preset entry declared in a SoundFont bank.
#[derive(Debug, Clone)]
pub struct Sf2PresetInfo {
    pub bank: u16,
    pub program: u16,
    pub name: String,
    pub is_drumkit: bool,
}

/// Loaded PCM sample and synthesis zone parameters for one SF2 key/velocity split.
#[derive(Debug, Clone)]
pub struct Sf2ZoneData {
    pub root_key: u8,
    pub key_low: u8,
    pub key_high: u8,
    pub vel_low: u8,
    pub vel_high: u8,
    pub name: String,
    pub sample_rate: u32,
    pub loop_start: usize,
    pub loop_end: usize,
    pub loop_enabled: bool,
    pub gain: f32,
    pub pan: i32,
    pub fine_tune_cents: i32,
    pub pcm_l: Arc<Vec<f32>>,
    pub pcm_r: Arc<Vec<f32>>,
}

/// Complete decoded zone list for an SF2 preset.
#[derive(Debug, Clone)]
pub struct Sf2InstrumentData {
    pub bank: u16,
    pub program: u16,
    pub name: String,
    pub is_drumkit: bool,
    pub zones: Vec<Sf2ZoneData>,
}

#[derive(Debug, Clone)]
struct ParsedPreset {
    info: Sf2PresetInfo,
    bag_start: usize,
    bag_end: usize,
}

#[derive(Debug, Clone, Copy)]
struct ParsedBag {
    gen_start: usize,
    gen_end: usize,
}

#[derive(Debug, Clone, Copy)]
struct RawGen {
    oper: u16,
    amount: u16,
}

impl RawGen {
    #[inline]
    fn as_i16(self) -> i16 {
        self.amount as i16
    }

    #[inline]
    fn as_range(self) -> (u8, u8) {
        ((self.amount & 0xFF) as u8, ((self.amount >> 8) & 0xFF) as u8)
    }
}

#[derive(Debug, Clone, Copy)]
struct ParsedInst {
    bag_start: usize,
    bag_end: usize,
}

#[derive(Debug, Clone)]
struct SampleHeader {
    name: String,
    start: u32,
    end: u32,
    start_loop: u32,
    end_loop: u32,
    sample_rate: u32,
    original_pitch: u8,
    pitch_correction: i8,
    sample_link: u16,
    sample_type: u16,
}

#[derive(Debug, Clone, Default)]
struct PresetGenState {
    key_range: Option<(u8, u8)>,
    vel_range: Option<(u8, u8)>,
    pan: i32,
    initial_attenuation: i32,
    coarse_tune: i32,
    fine_tune: i32,
}

impl PresetGenState {
    fn apply(&mut self, gen: RawGen) {
        match gen.oper {
            GEN_PAN => self.pan = gen.as_i16() as i32,
            GEN_KEY_RANGE => self.key_range = Some(gen.as_range()),
            GEN_VEL_RANGE => self.vel_range = Some(gen.as_range()),
            GEN_INITIAL_ATTENUATION => self.initial_attenuation = gen.as_i16() as i32,
            GEN_COARSE_TUNE => self.coarse_tune = gen.as_i16() as i32,
            GEN_FINE_TUNE => self.fine_tune = gen.as_i16() as i32,
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
struct InstGenState {
    key_range: Option<(u8, u8)>,
    vel_range: Option<(u8, u8)>,
    pan: i32,
    initial_attenuation: i32,
    coarse_tune: i32,
    fine_tune: i32,
    sample_modes: u16,
    overriding_root_key: i16,
    start_offset: i32,
    start_coarse_offset: i32,
    end_offset: i32,
    end_coarse_offset: i32,
    startloop_offset: i32,
    startloop_coarse_offset: i32,
    endloop_offset: i32,
    endloop_coarse_offset: i32,
}

impl Default for InstGenState {
    fn default() -> Self {
        Self {
            key_range: None,
            vel_range: None,
            pan: 0,
            initial_attenuation: 0,
            coarse_tune: 0,
            fine_tune: 0,
            sample_modes: 0,
            overriding_root_key: -1,
            start_offset: 0,
            start_coarse_offset: 0,
            end_offset: 0,
            end_coarse_offset: 0,
            startloop_offset: 0,
            startloop_coarse_offset: 0,
            endloop_offset: 0,
            endloop_coarse_offset: 0,
        }
    }
}

impl InstGenState {
    fn apply(&mut self, gen: RawGen) {
        match gen.oper {
            GEN_START_ADDRS_OFFSET => self.start_offset = gen.as_i16() as i32,
            GEN_END_ADDRS_OFFSET => self.end_offset = gen.as_i16() as i32,
            GEN_STARTLOOP_ADDRS_OFFSET => self.startloop_offset = gen.as_i16() as i32,
            GEN_ENDLOOP_ADDRS_OFFSET => self.endloop_offset = gen.as_i16() as i32,
            GEN_START_ADDRS_COARSE_OFFSET => self.start_coarse_offset = gen.as_i16() as i32,
            GEN_END_ADDRS_COARSE_OFFSET => self.end_coarse_offset = gen.as_i16() as i32,
            GEN_PAN => self.pan = gen.as_i16() as i32,
            GEN_KEY_RANGE => self.key_range = Some(gen.as_range()),
            GEN_VEL_RANGE => self.vel_range = Some(gen.as_range()),
            GEN_STARTLOOP_ADDRS_COARSE_OFFSET => self.startloop_coarse_offset = gen.as_i16() as i32,
            GEN_INITIAL_ATTENUATION => self.initial_attenuation = gen.as_i16() as i32,
            GEN_ENDLOOP_ADDRS_COARSE_OFFSET => self.endloop_coarse_offset = gen.as_i16() as i32,
            GEN_COARSE_TUNE => self.coarse_tune = gen.as_i16() as i32,
            GEN_FINE_TUNE => self.fine_tune = gen.as_i16() as i32,
            GEN_SAMPLE_MODES => self.sample_modes = gen.amount,
            GEN_OVERRIDING_ROOT_KEY => self.overriding_root_key = gen.as_i16(),
            _ => {}
        }
    }

    #[inline]
    fn effective_start_offset(&self) -> i64 {
        (self.start_offset as i64) + (self.start_coarse_offset as i64) * 32_768
    }

    #[inline]
    fn effective_end_offset(&self) -> i64 {
        (self.end_offset as i64) + (self.end_coarse_offset as i64) * 32_768
    }

    #[inline]
    fn effective_startloop_offset(&self) -> i64 {
        (self.startloop_offset as i64) + (self.startloop_coarse_offset as i64) * 32_768
    }

    #[inline]
    fn effective_endloop_offset(&self) -> i64 {
        (self.endloop_offset as i64) + (self.endloop_coarse_offset as i64) * 32_768
    }
}

#[derive(Debug, Clone)]
struct PendingZone {
    pbag_idx: usize,
    inst_idx: u16,
    sample_id: u16,
    sample_link: u16,
    sample_type_low: u16,
    root_key: u8,
    key_low: u8,
    key_high: u8,
    vel_low: u8,
    vel_high: u8,
    name: String,
    sample_rate: u32,
    abs_start: u32,
    abs_end: u32,
    rel_loop_start: usize,
    rel_loop_end: usize,
    loop_enabled: bool,
    gain: f32,
    pan: i32,
    coarse_tune: i32,
    fine_tune_cents: i32,
}

/// Lazy SoundFont 2 bank keeping metadata in RAM while paging PCM samples on demand.
pub struct SoundFontBank {
    name: String,
    path: PathBuf,
    smpl_file_offset: u64,
    smpl_byte_len: u64,
    presets: Vec<ParsedPreset>,
    pbags: Vec<ParsedBag>,
    pgens: Vec<RawGen>,
    insts: Vec<ParsedInst>,
    ibags: Vec<ParsedBag>,
    igens: Vec<RawGen>,
    shdrs: Vec<SampleHeader>,
    preset_cache: Mutex<HashMap<(u16, u16), Arc<Sf2InstrumentData>>>,
    sample_cache: Mutex<HashMap<(u32, u32), Arc<Vec<f32>>>>,
}

impl SoundFontBank {
    /// Parse the RIFF `sfbk` directory (`INFO`, `sdta`/`smpl` bounds, and `pdta` hydra)
    /// without reading the `smpl` PCM payload into memory.
    pub fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path)
            .map_err(|e| format!("failed to open soundfont '{}': {e}", path.display()))?;
        let file_len = file
            .metadata()
            .map_err(|e| format!("failed to stat soundfont '{}': {e}", path.display()))?
            .len();

        let mut reader = BufReader::new(file);
        let mut header = [0u8; 12];
        reader
            .read_exact(&mut header)
            .map_err(|e| format!("failed to read RIFF header: {e}"))?;

        if &header[0..4] != b"RIFF" || &header[8..12] != b"sfbk" {
            return Err("invalid SoundFont file: expected RIFF/sfbk header".to_string());
        }

        let riff_size = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as u64;
        let riff_end = (8u64.saturating_add(riff_size)).min(file_len);

        let mut inam: Option<String> = None;
        let mut smpl_file_offset: Option<u64> = None;
        let mut smpl_byte_len: u64 = 0;

        let mut raw_phdr = Vec::new();
        let mut raw_pbag = Vec::new();
        let mut raw_pgen = Vec::new();
        let mut raw_inst = Vec::new();
        let mut raw_ibag = Vec::new();
        let mut raw_igen = Vec::new();
        let mut raw_shdr = Vec::new();
        let mut seen_pdta = false;

        let mut pos = 12u64;
        while pos.saturating_add(8) <= riff_end {
            reader
                .seek(SeekFrom::Start(pos))
                .map_err(|e| format!("seek failed at {pos}: {e}"))?;
            let Some((cid, csize)) = read_chunk_header(&mut reader)? else {
                break;
            };
            let payload_start = pos + 8;
            let payload_end = payload_start.saturating_add(csize).min(file_len);

            if &cid == b"LIST" && payload_start.saturating_add(4) <= payload_end {
                let mut list_type = [0u8; 4];
                reader
                    .read_exact(&mut list_type)
                    .map_err(|e| format!("failed to read LIST type: {e}"))?;

                match &list_type {
                    b"INFO" => {
                        inam = parse_info_inam(&mut reader, payload_start + 4, payload_end)?;
                    }
                    b"sdta" => {
                        if let Some((off, len)) =
                            locate_smpl_subchunk(&mut reader, payload_start + 4, payload_end)?
                        {
                            smpl_file_offset = Some(off);
                            smpl_byte_len = len;
                        }
                    }
                    b"pdta" => {
                        seen_pdta = true;
                        read_pdta_subchunks(
                            &mut reader,
                            payload_start + 4,
                            payload_end,
                            &mut raw_phdr,
                            &mut raw_pbag,
                            &mut raw_pgen,
                            &mut raw_inst,
                            &mut raw_ibag,
                            &mut raw_igen,
                            &mut raw_shdr,
                        )?;
                    }
                    _ => {}
                }
            }

            pos = advance_riff_offset(payload_start, csize, file_len);
        }

        let smpl_file_offset = smpl_file_offset
            .ok_or_else(|| "missing sdta/smpl chunk in SoundFont".to_string())?;
        if !seen_pdta || raw_phdr.is_empty() || raw_inst.is_empty() || raw_shdr.is_empty() {
            return Err("missing or incomplete pdta chunk in SoundFont".to_string());
        }

        let pgens = parse_gens(&raw_pgen);
        let igens = parse_gens(&raw_igen);
        let pbags = parse_bags(&raw_pbag, pgens.len());
        let ibags = parse_bags(&raw_ibag, igens.len());
        let mut presets = parse_presets(&raw_phdr, pbags.len())?;
        presets.sort_by_key(|p| (p.info.bank, p.info.program));
        let insts = parse_insts(&raw_inst, ibags.len())?;
        let shdrs = parse_shdrs(&raw_shdr)?;

        let fallback_name = path
            .file_stem()
            .or_else(|| path.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "SoundFont".to_string());
        let name = inam.filter(|s| !s.is_empty()).unwrap_or(fallback_name);

        Ok(Self {
            name,
            path: path.to_path_buf(),
            smpl_file_offset,
            smpl_byte_len,
            presets,
            pbags,
            pgens,
            insts,
            ibags,
            igens,
            shdrs,
            preset_cache: Mutex::new(HashMap::new()),
            sample_cache: Mutex::new(HashMap::new()),
        })
    }

    /// Display name of the bank (`INAM` chunk if present, else filename stem).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Filesystem path of the loaded SoundFont file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// List all presets in the bank sorted by `(bank, program)`.
    pub fn list_presets(&self) -> Vec<Sf2PresetInfo> {
        self.presets.iter().map(|p| p.info.clone()).collect()
    }

    /// Lazily resolve a preset `(bank, program)` and page its PCM zones from disk on demand.
    pub fn load_preset(&self, bank: u16, program: u16) -> Option<Arc<Sf2InstrumentData>> {
        if let Some(cached) = self.preset_cache.lock().get(&(bank, program)) {
            return Some(Arc::clone(cached));
        }

        let preset = self.resolve_preset(bank, program)?;
        let resolved_key = (preset.info.bank, preset.info.program);

        if resolved_key != (bank, program) {
            let mut cache = self.preset_cache.lock();
            if let Some(existing) = cache.get(&resolved_key) {
                let cloned = Arc::clone(existing);
                cache.insert((bank, program), Arc::clone(&cloned));
                return Some(cloned);
            }
        }

        let pending = self.expand_preset_zones(preset);
        let pcm_map = self.load_required_samples(&pending)?;
        let zones = assemble_zones(&pending, &pcm_map);

        let data = Arc::new(Sf2InstrumentData {
            bank: preset.info.bank,
            program: preset.info.program,
            name: preset.info.name.clone(),
            is_drumkit: preset.info.is_drumkit,
            zones,
        });

        let mut cache = self.preset_cache.lock();
        let entry = cache
            .entry(resolved_key)
            .or_insert_with(|| Arc::clone(&data));
        let result = Arc::clone(entry);
        if resolved_key != (bank, program) {
            cache.entry((bank, program)).or_insert_with(|| Arc::clone(&result));
        }
        Some(result)
    }

    fn resolve_preset(&self, bank: u16, program: u16) -> Option<&ParsedPreset> {
        if self.presets.is_empty() {
            return None;
        }
        if let Some(p) = self
            .presets
            .iter()
            .find(|p| p.info.bank == bank && p.info.program == program)
        {
            return Some(p);
        }
        if bank == 128 {
            if let Some(p) = self
                .presets
                .iter()
                .find(|p| p.info.bank == 128 && p.info.program == 0)
            {
                return Some(p);
            }
            if let Some(p) = self.presets.iter().find(|p| p.info.bank == 128) {
                return Some(p);
            }
        } else if let Some(p) = self
            .presets
            .iter()
            .find(|p| p.info.bank == 0 && p.info.program == program)
        {
            return Some(p);
        }

        self.presets
            .iter()
            .filter(|p| p.info.bank == 0)
            .min_by_key(|p| p.info.program.abs_diff(program))
            .or_else(|| self.presets.first())
    }

    fn expand_preset_zones(&self, preset: &ParsedPreset) -> Vec<PendingZone> {
        let mut pending = Vec::new();
        if preset.bag_start >= preset.bag_end || preset.bag_end > self.pbags.len() {
            return pending;
        }

        let mut pbag_idx = preset.bag_start;
        let mut global_preset = PresetGenState::default();

        let first_gens = self.bag_pgens(pbag_idx);
        if !first_gens.iter().any(|g| g.oper == GEN_INSTRUMENT) {
            for &gen in first_gens {
                global_preset.apply(gen);
            }
            pbag_idx += 1;
        }

        let max_smpl_samples = (self.smpl_byte_len / 2) as i64;

        for bi in pbag_idx..preset.bag_end {
            let gens = self.bag_pgens(bi);
            let mut local_preset = global_preset.clone();
            let mut inst_idx: Option<u16> = None;

            for &gen in gens {
                if gen.oper == GEN_INSTRUMENT {
                    inst_idx = Some(gen.amount);
                    break;
                }
                local_preset.apply(gen);
            }

            let Some(inst_id) = inst_idx else {
                continue;
            };
            let Some(inst) = self.insts.get(inst_id as usize) else {
                continue;
            };
            if inst.bag_start >= inst.bag_end || inst.bag_end > self.ibags.len() {
                continue;
            }

            let mut ibag_idx = inst.bag_start;
            let mut global_inst = InstGenState::default();

            let first_igens = self.bag_igens(ibag_idx);
            if !first_igens.iter().any(|g| g.oper == GEN_SAMPLE_ID) {
                for &gen in first_igens {
                    global_inst.apply(gen);
                }
                ibag_idx += 1;
            }

            for ibi in ibag_idx..inst.bag_end {
                let igens = self.bag_igens(ibi);
                let mut local_inst = global_inst.clone();
                let mut sample_id: Option<u16> = None;

                for &gen in igens {
                    if gen.oper == GEN_SAMPLE_ID {
                        sample_id = Some(gen.amount);
                        break;
                    }
                    local_inst.apply(gen);
                }

                let Some(sid) = sample_id else {
                    continue;
                };
                let Some(shdr) = self.shdrs.get(sid as usize) else {
                    continue;
                };
                if (shdr.sample_type & SAMPLE_TYPE_ROM_BIT) != 0 {
                    continue;
                }

                let Some((key_low, key_high)) = intersect_range(
                    local_preset.key_range.unwrap_or((0, 127)),
                    local_inst.key_range.unwrap_or((0, 127)),
                ) else {
                    continue;
                };
                let Some((vel_low, vel_high)) = intersect_range(
                    local_preset.vel_range.unwrap_or((0, 127)),
                    local_inst.vel_range.unwrap_or((0, 127)),
                ) else {
                    continue;
                };

                let abs_start_i64 =
                    (shdr.start as i64 + local_inst.effective_start_offset()).clamp(0, max_smpl_samples);
                let abs_end_i64 = (shdr.end as i64 + local_inst.effective_end_offset())
                    .clamp(abs_start_i64, max_smpl_samples);
                if abs_end_i64 <= abs_start_i64 {
                    continue;
                }

                let abs_start = abs_start_i64 as u32;
                let abs_end = abs_end_i64 as u32;
                let sample_len = (abs_end - abs_start) as usize;

                let abs_loop_start = shdr.start_loop as i64 + local_inst.effective_startloop_offset();
                let abs_loop_end = shdr.end_loop as i64 + local_inst.effective_endloop_offset();
                let rel_loop_end = (abs_loop_end - abs_start_i64).clamp(0, sample_len as i64) as usize;
                let rel_loop_start =
                    (abs_loop_start - abs_start_i64).clamp(0, rel_loop_end as i64) as usize;
                let loop_enabled =
                    (local_inst.sample_modes & 1) == 1 && rel_loop_end > rel_loop_start + 8;

                let root_key = if (0..=127).contains(&local_inst.overriding_root_key) {
                    local_inst.overriding_root_key as u8
                } else if shdr.original_pitch <= 127 {
                    shdr.original_pitch
                } else if shdr.original_pitch == 255 && key_low == key_high {
                    key_low
                } else {
                    60
                };

                let coarse_tune = local_inst.coarse_tune + local_preset.coarse_tune;
                let fine_tune = local_inst.fine_tune + local_preset.fine_tune;
                let fine_tune_cents =
                    coarse_tune * 100 + fine_tune + (shdr.pitch_correction as i32);

                let att = local_inst.initial_attenuation + local_preset.initial_attenuation;
                let gain = 10f32.powf(-(att.max(0) as f32) * 0.004).clamp(0.05, 1.5);
                let pan = ((local_inst.pan + local_preset.pan) / 5).clamp(-100, 100);
                let sample_rate = if shdr.sample_rate == 0 {
                    44_100
                } else {
                    shdr.sample_rate
                };

                pending.push(PendingZone {
                    pbag_idx: bi,
                    inst_idx: inst_id,
                    sample_id: sid,
                    sample_link: shdr.sample_link,
                    sample_type_low: shdr.sample_type & 0x0F,
                    root_key,
                    key_low,
                    key_high,
                    vel_low,
                    vel_high,
                    name: shdr.name.clone(),
                    sample_rate,
                    abs_start,
                    abs_end,
                    rel_loop_start,
                    rel_loop_end,
                    loop_enabled,
                    gain,
                    pan,
                    coarse_tune,
                    fine_tune_cents,
                });
            }
        }

        pending
    }

    #[inline]
    fn bag_pgens(&self, bag_idx: usize) -> &[RawGen] {
        let b = self.pbags[bag_idx];
        &self.pgens[b.gen_start..b.gen_end]
    }

    #[inline]
    fn bag_igens(&self, bag_idx: usize) -> &[RawGen] {
        let b = self.ibags[bag_idx];
        &self.igens[b.gen_start..b.gen_end]
    }

    fn load_required_samples(
        &self,
        pending: &[PendingZone],
    ) -> Option<HashMap<(u32, u32), Arc<Vec<f32>>>> {
        let mut resolved: HashMap<(u32, u32), Arc<Vec<f32>>> = HashMap::new();
        let mut missing: Vec<(u32, u32)> = Vec::new();

        {
            let cache = self.sample_cache.lock();
            for z in pending {
                let key = (z.abs_start, z.abs_end);
                if resolved.contains_key(&key) {
                    continue;
                }
                if let Some(pcm) = cache.get(&key) {
                    resolved.insert(key, Arc::clone(pcm));
                } else {
                    resolved.insert(key, Arc::new(Vec::new()));
                    missing.push(key);
                }
            }
        }

        if missing.is_empty() {
            return Some(resolved);
        }

        // Sort ascending so disk reads issue sequential forward seeks.
        missing.sort_unstable();

        let file = File::open(&self.path).ok()?;
        let mut reader = BufReader::new(file);
        let mut raw_buf = Vec::new();
        let mut newly_decoded = Vec::with_capacity(missing.len());

        for (start, end) in missing {
            let sample_count = (end - start) as usize;
            let byte_len = sample_count * 2;
            let byte_offset = self.smpl_file_offset + (start as u64) * 2;

            raw_buf.resize(byte_len, 0);
            if reader.seek(SeekFrom::Start(byte_offset)).is_err() {
                return None;
            }
            if reader.read_exact(&mut raw_buf[..byte_len]).is_err() {
                return None;
            }

            let mut pcm = Vec::with_capacity(sample_count);
            for pair in raw_buf[..byte_len].chunks_exact(2) {
                let s = i16::from_le_bytes([pair[0], pair[1]]);
                pcm.push((s as f32) / 32_768.0);
            }
            let arc = Arc::new(pcm);
            resolved.insert((start, end), Arc::clone(&arc));
            newly_decoded.push(((start, end), arc));
        }

        let mut cache = self.sample_cache.lock();
        for (key, arc) in newly_decoded {
            let entry = cache.entry(key).or_insert(arc);
            resolved.insert(key, Arc::clone(entry));
        }

        Some(resolved)
    }
}

fn assemble_zones(
    pending: &[PendingZone],
    pcm_map: &HashMap<(u32, u32), Arc<Vec<f32>>>,
) -> Vec<Sf2ZoneData> {
    let n = pending.len();
    let mut paired_right: Vec<Option<usize>> = vec![None; n];
    let mut consumed = vec![false; n];

    // Pass 1: Match each Left zone (`sample_type == 4`) with its best unpaired Right partner (`2`).
    for i in 0..n {
        let left = &pending[i];
        if consumed[i] || left.sample_type_low != SAMPLE_TYPE_LEFT {
            continue;
        }

        let mut best: Option<(usize, (u8, u8, usize))> = None;

        for j in 0..n {
            if j == i || consumed[j] {
                continue;
            }
            let right = &pending[j];
            if right.sample_type_low != SAMPLE_TYPE_RIGHT
                || right.pbag_idx != left.pbag_idx
                || right.inst_idx != left.inst_idx
                || right.key_low != left.key_low
                || right.key_high != left.key_high
                || right.vel_low != left.vel_low
                || right.vel_high != left.vel_high
            {
                continue;
            }

            if left.sample_link != 0 || right.sample_link != 0 {
                let left_ok = left.sample_link == 0 || left.sample_link == right.sample_id;
                let right_ok = right.sample_link == 0 || right.sample_link == left.sample_id;
                if !left_ok || !right_ok {
                    continue;
                }
            }

            // Rank candidates so multi-layer stereo zones (e.g. detuned oscillators) pair cleanly.
            let sample_adjacent = u8::from(
                !(right.sample_id == left.sample_id.saturating_add(1)
                    || left.sample_id == right.sample_id.saturating_add(1)
                    || strip_channel_suffix(&left.name) == strip_channel_suffix(&right.name)),
            );
            let tune_mismatch = u8::from(
                right.root_key != left.root_key || right.coarse_tune != left.coarse_tune,
            );
            let score = (sample_adjacent, tune_mismatch, i.abs_diff(j));

            if best.as_ref().is_none_or(|(_, bscore)| score < *bscore) {
                best = Some((j, score));
            }
        }

        if let Some((j, _)) = best {
            paired_right[i] = Some(j);
            consumed[i] = true;
            consumed[j] = true;
        }
    }

    // Pass 2: Emit merged stereo zones and remaining mono zones in order.
    let mut zones = Vec::with_capacity(n);
    for (i, z) in pending.iter().enumerate() {
        if let Some(j) = paired_right[i] {
            let r = &pending[j];
            let mut pcm_l = Arc::clone(&pcm_map[&(z.abs_start, z.abs_end)]);
            let mut pcm_r = Arc::clone(&pcm_map[&(r.abs_start, r.abs_end)]);

            let min_len = pcm_l.len().min(pcm_r.len());
            if pcm_l.len() != min_len {
                pcm_l = Arc::new(pcm_l[..min_len].to_vec());
            }
            if pcm_r.len() != min_len {
                pcm_r = Arc::new(pcm_r[..min_len].to_vec());
            }

            let loop_end = z.rel_loop_end.min(min_len);
            let loop_start = z.rel_loop_start.min(loop_end);
            let loop_enabled = z.loop_enabled && loop_end > loop_start + 8;

            zones.push(Sf2ZoneData {
                root_key: z.root_key,
                key_low: z.key_low,
                key_high: z.key_high,
                vel_low: z.vel_low,
                vel_high: z.vel_high,
                name: strip_channel_suffix(&z.name).to_string(),
                sample_rate: z.sample_rate,
                loop_start,
                loop_end,
                loop_enabled,
                gain: z.gain,
                pan: 0,
                fine_tune_cents: z.fine_tune_cents,
                pcm_l,
                pcm_r,
            });
        } else if !consumed[i] || z.sample_type_low == SAMPLE_TYPE_MONO {
            let pcm_l = Arc::clone(&pcm_map[&(z.abs_start, z.abs_end)]);
            let loop_end = z.rel_loop_end.min(pcm_l.len());
            let loop_start = z.rel_loop_start.min(loop_end);
            let loop_enabled = z.loop_enabled && loop_end > loop_start + 8;

            zones.push(Sf2ZoneData {
                root_key: z.root_key,
                key_low: z.key_low,
                key_high: z.key_high,
                vel_low: z.vel_low,
                vel_high: z.vel_high,
                name: z.name.clone(),
                sample_rate: z.sample_rate,
                loop_start,
                loop_end,
                loop_enabled,
                gain: z.gain,
                pan: z.pan,
                fine_tune_cents: z.fine_tune_cents,
                pcm_r: Arc::clone(&pcm_l),
                pcm_l,
            });
        }
    }

    zones
}

fn strip_channel_suffix(name: &str) -> &str {
    for suffix in ["(L)", "(R)", "(l)", "(r)"] {
        if let Some(stripped) = name.strip_suffix(suffix) {
            let trimmed = stripped.trim_end();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }
    name
}

#[inline]
fn intersect_range(a: (u8, u8), b: (u8, u8)) -> Option<(u8, u8)> {
    if a.0 > a.1 || b.0 > b.1 || a.0 > 127 || b.0 > 127 {
        return None;
    }
    let low = a.0.max(b.0);
    let high = a.1.min(b.1).min(127);
    if low <= high {
        Some((low, high))
    } else {
        None
    }
}

#[inline]
fn advance_riff_offset(payload_start: u64, size: u64, bound: u64) -> u64 {
    let padded = size.saturating_add(size & 1);
    payload_start.saturating_add(padded).min(bound)
}

fn read_chunk_header<R: Read>(reader: &mut R) -> Result<Option<([u8; 4], u64)>, String> {
    let mut buf = [0u8; 8];
    match reader.read_exact(&mut buf) {
        Ok(()) => {
            let id = [buf[0], buf[1], buf[2], buf[3]];
            let size = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]) as u64;
            Ok(Some((id, size)))
        }
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(None),
        Err(e) => Err(format!("failed reading RIFF chunk header: {e}")),
    }
}

fn parse_info_inam<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
) -> Result<Option<String>, String> {
    let mut pos = start;
    while pos.saturating_add(8) <= end {
        reader
            .seek(SeekFrom::Start(pos))
            .map_err(|e| format!("INFO seek error: {e}"))?;
        let Some((id, size)) = read_chunk_header(reader)? else {
            break;
        };
        let payload_start = pos + 8;
        let payload_end = payload_start.saturating_add(size).min(end);
        if &id == b"INAM" {
            let len = (payload_end - payload_start).min(4096) as usize;
            let mut bytes = vec![0u8; len];
            reader
                .read_exact(&mut bytes)
                .map_err(|e| format!("failed reading INFO/INAM: {e}"))?;
            let text = decode_sf2_str(&bytes);
            if !text.is_empty() {
                return Ok(Some(text));
            }
        }
        pos = advance_riff_offset(payload_start, size, end);
    }
    Ok(None)
}

fn locate_smpl_subchunk<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
) -> Result<Option<(u64, u64)>, String> {
    let mut pos = start;
    while pos.saturating_add(8) <= end {
        reader
            .seek(SeekFrom::Start(pos))
            .map_err(|e| format!("sdta seek error: {e}"))?;
        let Some((id, size)) = read_chunk_header(reader)? else {
            break;
        };
        let payload_start = pos + 8;
        let payload_end = payload_start.saturating_add(size).min(end);
        if &id == b"smpl" {
            return Ok(Some((payload_start, payload_end - payload_start)));
        }
        pos = advance_riff_offset(payload_start, size, end);
    }
    Ok(None)
}

#[allow(clippy::too_many_arguments)]
fn read_pdta_subchunks<R: Read + Seek>(
    reader: &mut R,
    start: u64,
    end: u64,
    phdr: &mut Vec<u8>,
    pbag: &mut Vec<u8>,
    pgen: &mut Vec<u8>,
    inst: &mut Vec<u8>,
    ibag: &mut Vec<u8>,
    igen: &mut Vec<u8>,
    shdr: &mut Vec<u8>,
) -> Result<(), String> {
    let mut pos = start;
    while pos.saturating_add(8) <= end {
        reader
            .seek(SeekFrom::Start(pos))
            .map_err(|e| format!("pdta seek error: {e}"))?;
        let Some((id, size)) = read_chunk_header(reader)? else {
            break;
        };
        let payload_start = pos + 8;
        let payload_end = payload_start.saturating_add(size).min(end);
        let read_len = (payload_end - payload_start) as usize;

        let target = match &id {
            b"phdr" => Some(&mut *phdr),
            b"pbag" => Some(&mut *pbag),
            b"pgen" => Some(&mut *pgen),
            b"inst" => Some(&mut *inst),
            b"ibag" => Some(&mut *ibag),
            b"igen" => Some(&mut *igen),
            b"shdr" => Some(&mut *shdr),
            _ => None,
        };
        if let Some(buf) = target {
            buf.resize(read_len, 0);
            reader
                .read_exact(buf)
                .map_err(|e| format!("failed reading pdta/{}: {e}", String::from_utf8_lossy(&id)))?;
        }

        pos = advance_riff_offset(payload_start, size, end);
    }
    Ok(())
}

fn decode_sf2_str(bytes: &[u8]) -> String {
    let nul = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..nul]).trim().to_string()
}

fn parse_gens(raw: &[u8]) -> Vec<RawGen> {
    raw.chunks_exact(4)
        .map(|c| RawGen {
            oper: u16::from_le_bytes([c[0], c[1]]),
            amount: u16::from_le_bytes([c[2], c[3]]),
        })
        .collect()
}

fn parse_bags(raw: &[u8], gen_count: usize) -> Vec<ParsedBag> {
    let gen_indices: Vec<usize> = raw
        .chunks_exact(4)
        .map(|c| (u16::from_le_bytes([c[0], c[1]]) as usize).min(gen_count))
        .collect();
    if gen_indices.len() < 2 {
        return Vec::new();
    }
    gen_indices
        .windows(2)
        .map(|w| ParsedBag {
            gen_start: w[0],
            gen_end: w[1].max(w[0]),
        })
        .collect()
}

fn parse_presets(raw: &[u8], pbag_count: usize) -> Result<Vec<ParsedPreset>, String> {
    let records: Vec<(Sf2PresetInfo, usize)> = raw
        .chunks_exact(38)
        .map(|c| {
            let name = decode_sf2_str(&c[0..20]);
            let program = u16::from_le_bytes([c[20], c[21]]);
            let bank = u16::from_le_bytes([c[22], c[23]]);
            let bag_ndx = (u16::from_le_bytes([c[24], c[25]]) as usize).min(pbag_count);
            (
                Sf2PresetInfo {
                    bank,
                    program,
                    name,
                    is_drumkit: bank == 128,
                },
                bag_ndx,
            )
        })
        .collect();
    if records.is_empty() {
        return Err("empty pdta/phdr table".to_string());
    }
    Ok(records
        .windows(2)
        .map(|w| ParsedPreset {
            info: w[0].0.clone(),
            bag_start: w[0].1,
            bag_end: w[1].1.max(w[0].1),
        })
        .collect())
}

fn parse_insts(raw: &[u8], ibag_count: usize) -> Result<Vec<ParsedInst>, String> {
    let bag_indices: Vec<usize> = raw
        .chunks_exact(22)
        .map(|c| (u16::from_le_bytes([c[20], c[21]]) as usize).min(ibag_count))
        .collect();
    if bag_indices.is_empty() {
        return Err("empty pdta/inst table".to_string());
    }
    Ok(bag_indices
        .windows(2)
        .map(|w| ParsedInst {
            bag_start: w[0],
            bag_end: w[1].max(w[0]),
        })
        .collect())
}

fn parse_shdrs(raw: &[u8]) -> Result<Vec<SampleHeader>, String> {
    let mut out: Vec<SampleHeader> = raw
        .chunks_exact(46)
        .map(|c| SampleHeader {
            name: decode_sf2_str(&c[0..20]),
            start: u32::from_le_bytes([c[20], c[21], c[22], c[23]]),
            end: u32::from_le_bytes([c[24], c[25], c[26], c[27]]),
            start_loop: u32::from_le_bytes([c[28], c[29], c[30], c[31]]),
            end_loop: u32::from_le_bytes([c[32], c[33], c[34], c[35]]),
            sample_rate: u32::from_le_bytes([c[36], c[37], c[38], c[39]]),
            original_pitch: c[40],
            pitch_correction: c[41] as i8,
            sample_link: u16::from_le_bytes([c[42], c[43]]),
            sample_type: u16::from_le_bytes([c[44], c[45]]),
        })
        .collect();
    if out.is_empty() {
        return Err("empty pdta/shdr table".to_string());
    }
    // Drop terminal EOS record.
    out.pop();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    const FLUID_PATH: &str = "/usr/share/sounds/sf2/FluidR3_GM.sf2";
    const TIM_PATH: &str = "/usr/share/sounds/sf2/TimGM6mb.sf2";

    fn rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
        (sum_sq / samples.len() as f64).sqrt() as f32
    }

    fn assert_valid_instrument(inst: &Sf2InstrumentData, expected_bank: u16, expected_program: u16) {
        assert_eq!(inst.bank, expected_bank);
        assert_eq!(inst.program, expected_program);
        assert_eq!(inst.is_drumkit, expected_bank == 128);
        assert!(!inst.name.is_empty(), "preset name must be non-empty");
        assert!(
            !inst.zones.is_empty(),
            "preset ({}, {}) '{}' must have non-empty zones",
            expected_bank,
            expected_program,
            inst.name
        );

        for (zi, z) in inst.zones.iter().enumerate() {
            assert!(
                z.key_low <= z.key_high && z.key_high <= 127,
                "zone {zi} invalid key range: {}..={}",
                z.key_low,
                z.key_high
            );
            assert!(
                z.vel_low <= z.vel_high && z.vel_high <= 127,
                "zone {zi} invalid vel range: {}..={}",
                z.vel_low,
                z.vel_high
            );
            assert!(z.root_key <= 127, "zone {zi} invalid root_key {}", z.root_key);
            assert!(z.sample_rate > 0, "zone {zi} sample_rate must be > 0");
            assert!(
                (-100..=100).contains(&z.pan),
                "zone {zi} pan {} out of range",
                z.pan
            );
            assert!(
                (0.05..=1.5).contains(&z.gain),
                "zone {zi} gain {} out of range",
                z.gain
            );
            assert!(!z.pcm_l.is_empty(), "zone {zi} pcm_l must be non-empty");
            assert_eq!(z.pcm_l.len(), z.pcm_r.len(), "zone {zi} L/R length mismatch");

            if z.loop_enabled {
                assert!(
                    z.loop_end <= z.pcm_l.len() && z.loop_end > z.loop_start + 8,
                    "zone {zi} invalid loop bounds: {}..{} (len {})",
                    z.loop_start,
                    z.loop_end,
                    z.pcm_l.len()
                );
            }

            for &s in z.pcm_l.iter().chain(z.pcm_r.iter()) {
                assert!(
                    s.is_finite() && (-1.0..=1.0).contains(&s),
                    "zone {zi} sample {s} out of [-1.0, 1.0]"
                );
            }
            assert!(
                rms(&z.pcm_l) > 1e-6 && rms(&z.pcm_r) > 1e-6,
                "zone {zi} '{}' has zero RMS energy",
                z.name
            );
        }
    }

    #[test]
    fn find_default_soundfont_finds_system_sf2() {
        let found = find_default_soundfont();
        assert!(found.is_some(), "expected a system default .sf2 file");
        assert!(found.unwrap().is_file());
    }

    #[test]
    fn fluid_r3_gm_open_fast_and_load_target_presets() {
        let path = Path::new(FLUID_PATH);
        if !path.is_file() {
            return;
        }

        let t0 = Instant::now();
        let bank = SoundFontBank::open(path).expect("FluidR3_GM.sf2 should open");
        let elapsed = t0.elapsed();
        assert!(
            elapsed.as_millis() < 250,
            "open() took {:?}, expected < 250ms",
            elapsed
        );
        assert_eq!(bank.path(), path);
        assert!(!bank.name().is_empty());

        let presets = bank.list_presets();
        assert!(
            presets.len() >= 128,
            "expected >= 128 presets, got {}",
            presets.len()
        );
        assert!(presets.windows(2).all(|w| (w[0].bank, w[0].program) <= (w[1].bank, w[1].program)));
        assert!(presets.iter().any(|p| p.is_drumkit && p.bank == 128));
        assert!(presets.iter().any(|p| !p.is_drumkit && p.bank == 0));

        let targets = [
            (0, 0),   // Grand Piano
            (0, 21),  // Accordion
            (0, 25),  // Steel Guitar
            (0, 33),  // Finger Bass
            (0, 48),  // Strings
            (0, 61),  // Brass
            (128, 0), // Standard Drums
            (128, 16), // Power Drums
        ];

        for &(b, p) in &targets {
            let inst = bank
                .load_preset(b, p)
                .unwrap_or_else(|| panic!("expected preset ({b}, {p}) to load"));
            assert_valid_instrument(&inst, b, p);
        }

        // Verify stereo linking merged (L)/(R) pairs for Grand Piano.
        let piano1 = bank.load_preset(0, 0).unwrap();
        let piano2 = bank.load_preset(0, 0).unwrap();
        assert!(Arc::ptr_eq(&piano1, &piano2), "preset cache must return same Arc");
        assert!(
            piano1
                .zones
                .iter()
                .any(|z| z.pan == 0 && !Arc::ptr_eq(&z.pcm_l, &z.pcm_r)),
            "expected merged stereo zones with distinct L/R buffers and pan=0"
        );

        // Verify fallback resolution for missing melodic bank and missing drumkit program.
        let fallback_guitar = bank.load_preset(77, 25).unwrap();
        assert_eq!((fallback_guitar.bank, fallback_guitar.program), (0, 25));
        let fallback_drums = bank.load_preset(128, 999).unwrap();
        assert_eq!((fallback_drums.bank, fallback_drums.program), (128, 0));
    }

    #[test]
    fn timgm_smoke_test() {
        let path = Path::new(TIM_PATH);
        if !path.is_file() {
            return;
        }
        let bank = SoundFontBank::open(path).expect("TimGM6mb.sf2 should open");
        assert!(bank.list_presets().len() >= 128);
        let piano = bank.load_preset(0, 0).expect("TimGM Piano 1");
        assert_valid_instrument(&piano, 0, 0);
        let drums = bank.load_preset(128, 0).expect("TimGM Standard Drums");
        assert_valid_instrument(&drums, 128, 0);
    }

    fn push_chunk(out: &mut Vec<u8>, id: &[u8; 4], payload: &[u8]) {
        out.extend_from_slice(id);
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(payload);
        if payload.len() % 2 == 1 {
            out.push(0);
        }
    }

    fn push_list(out: &mut Vec<u8>, list_type: &[u8; 4], body: &[u8]) {
        let mut payload = Vec::with_capacity(4 + body.len());
        payload.extend_from_slice(list_type);
        payload.extend_from_slice(body);
        push_chunk(out, b"LIST", &payload);
    }

    fn push_padded_name(out: &mut Vec<u8>, name: &str) {
        let mut buf = [0u8; 20];
        let bytes = name.as_bytes();
        let len = bytes.len().min(19);
        buf[..len].copy_from_slice(&bytes[..len]);
        out.extend_from_slice(&buf);
    }

    fn build_synthetic_sf2() -> Vec<u8> {
        let mut info = Vec::new();
        push_chunk(&mut info, b"INAM", b"Synthetic Test Bank\0");

        // Create 3 samples:
        // 0: Stereo L (len 100, loop 10..90)
        // 1: Stereo R (len 96, slightly shorter to test length equalization!)
        // 2: Mono unpitched (len 32, short loop 4..10 -> disabled because <= +8)
        let mut smpl = Vec::new();
        for i in 0..100i16 {
            smpl.extend_from_slice(&(i * 200 + 100).to_le_bytes());
        }
        for i in 0..96i16 {
            smpl.extend_from_slice(&(-(i * 200 + 100)).to_le_bytes());
        }
        for i in 0..32i16 {
            smpl.extend_from_slice(&((i + 1) * 500).to_le_bytes());
        }

        let mut phdr = Vec::new();
        push_padded_name(&mut phdr, "Synth Lead");
        phdr.extend_from_slice(&5u16.to_le_bytes()); // program
        phdr.extend_from_slice(&0u16.to_le_bytes()); // bank
        phdr.extend_from_slice(&0u16.to_le_bytes()); // bag 0
        phdr.extend_from_slice(&[0u8; 12]);
        push_padded_name(&mut phdr, "EOP");
        phdr.extend_from_slice(&0u16.to_le_bytes());
        phdr.extend_from_slice(&0u16.to_le_bytes());
        phdr.extend_from_slice(&2u16.to_le_bytes()); // 2 pbags (global + local)
        phdr.extend_from_slice(&[0u8; 12]);

        let mut pbag = Vec::new();
        // bag 0 (global): gen 0..2
        pbag.extend_from_slice(&0u16.to_le_bytes());
        pbag.extend_from_slice(&0u16.to_le_bytes());
        // bag 1 (local): gen 2..5
        pbag.extend_from_slice(&2u16.to_le_bytes());
        pbag.extend_from_slice(&0u16.to_le_bytes());
        // terminal
        pbag.extend_from_slice(&5u16.to_le_bytes());
        pbag.extend_from_slice(&0u16.to_le_bytes());

        let mut pgen = Vec::new();
        // Global: keyRange 20..90, coarseTune +1
        pgen.extend_from_slice(&GEN_KEY_RANGE.to_le_bytes());
        pgen.extend_from_slice(&[20u8, 90u8]);
        pgen.extend_from_slice(&GEN_COARSE_TUNE.to_le_bytes());
        pgen.extend_from_slice(&1i16.to_le_bytes());
        // Local: velRange 10..120, fineTune -15, inst 0
        pgen.extend_from_slice(&GEN_VEL_RANGE.to_le_bytes());
        pgen.extend_from_slice(&[10u8, 120u8]);
        pgen.extend_from_slice(&GEN_FINE_TUNE.to_le_bytes());
        pgen.extend_from_slice(&(-15i16).to_le_bytes());
        pgen.extend_from_slice(&GEN_INSTRUMENT.to_le_bytes());
        pgen.extend_from_slice(&0u16.to_le_bytes());
        // Terminal
        pgen.extend_from_slice(&0u32.to_le_bytes());

        let mut inst = Vec::new();
        push_padded_name(&mut inst, "DualInst");
        inst.extend_from_slice(&0u16.to_le_bytes());
        push_padded_name(&mut inst, "EOI");
        inst.extend_from_slice(&4u16.to_le_bytes()); // 4 ibags

        let mut ibag = Vec::new();
        // ibag 0 (global): igen 0..1
        // ibag 1 (stereo L): igen 1..5
        // ibag 2 (stereo R): igen 5..9
        // ibag 3 (mono unpitched): igen 9..13
        // ibag 4 (terminal): 13
        for idx in [0u16, 1, 5, 9, 13] {
            ibag.extend_from_slice(&idx.to_le_bytes());
            ibag.extend_from_slice(&0u16.to_le_bytes());
        }

        let mut igen = Vec::new();
        // ibag 0 (global): sampleModes = 1
        igen.extend_from_slice(&GEN_SAMPLE_MODES.to_le_bytes());
        igen.extend_from_slice(&1u16.to_le_bytes());
        // ibag 1 (stereo L): keyRange 40..100 (intersects 20..90 -> 40..90), pan -500, fineTune +5, sampleID 0
        igen.extend_from_slice(&GEN_KEY_RANGE.to_le_bytes());
        igen.extend_from_slice(&[40u8, 100u8]);
        igen.extend_from_slice(&GEN_PAN.to_le_bytes());
        igen.extend_from_slice(&(-500i16).to_le_bytes());
        igen.extend_from_slice(&GEN_FINE_TUNE.to_le_bytes());
        igen.extend_from_slice(&5i16.to_le_bytes());
        igen.extend_from_slice(&GEN_SAMPLE_ID.to_le_bytes());
        igen.extend_from_slice(&0u16.to_le_bytes());
        // ibag 2 (stereo R): keyRange 40..100, pan +500, fineTune +5, sampleID 1
        igen.extend_from_slice(&GEN_KEY_RANGE.to_le_bytes());
        igen.extend_from_slice(&[40u8, 100u8]);
        igen.extend_from_slice(&GEN_PAN.to_le_bytes());
        igen.extend_from_slice(&500i16.to_le_bytes());
        igen.extend_from_slice(&GEN_FINE_TUNE.to_le_bytes());
        igen.extend_from_slice(&5i16.to_le_bytes());
        igen.extend_from_slice(&GEN_SAMPLE_ID.to_le_bytes());
        igen.extend_from_slice(&1u16.to_le_bytes());
        // ibag 3 (mono unpitched): keyRange 36..36, startOffset +2, pan +250, sampleID 2
        igen.extend_from_slice(&GEN_KEY_RANGE.to_le_bytes());
        igen.extend_from_slice(&[36u8, 36u8]);
        igen.extend_from_slice(&GEN_START_ADDRS_OFFSET.to_le_bytes());
        igen.extend_from_slice(&2i16.to_le_bytes());
        igen.extend_from_slice(&GEN_PAN.to_le_bytes());
        igen.extend_from_slice(&250i16.to_le_bytes());
        igen.extend_from_slice(&GEN_SAMPLE_ID.to_le_bytes());
        igen.extend_from_slice(&2u16.to_le_bytes());
        // terminal
        igen.extend_from_slice(&0u32.to_le_bytes());

        let mut shdr = Vec::new();
        // sample 0 (L)
        push_padded_name(&mut shdr, "Wave(L)");
        shdr.extend_from_slice(&0u32.to_le_bytes());
        shdr.extend_from_slice(&100u32.to_le_bytes());
        shdr.extend_from_slice(&10u32.to_le_bytes());
        shdr.extend_from_slice(&98u32.to_le_bytes());
        shdr.extend_from_slice(&48_000u32.to_le_bytes());
        shdr.push(60); // pitch
        shdr.push((-3i8) as u8); // corr
        shdr.extend_from_slice(&1u16.to_le_bytes()); // link -> 1
        shdr.extend_from_slice(&SAMPLE_TYPE_LEFT.to_le_bytes());
        // sample 1 (R)
        push_padded_name(&mut shdr, "Wave(R)");
        shdr.extend_from_slice(&100u32.to_le_bytes());
        shdr.extend_from_slice(&196u32.to_le_bytes());
        shdr.extend_from_slice(&110u32.to_le_bytes());
        shdr.extend_from_slice(&194u32.to_le_bytes());
        shdr.extend_from_slice(&48_000u32.to_le_bytes());
        shdr.push(60);
        shdr.push((-3i8) as u8);
        shdr.extend_from_slice(&0u16.to_le_bytes()); // link -> 0
        shdr.extend_from_slice(&SAMPLE_TYPE_RIGHT.to_le_bytes());
        // sample 2 (mono unpitched, short loop 200..206)
        push_padded_name(&mut shdr, "Click");
        shdr.extend_from_slice(&196u32.to_le_bytes());
        shdr.extend_from_slice(&228u32.to_le_bytes());
        shdr.extend_from_slice(&200u32.to_le_bytes());
        shdr.extend_from_slice(&206u32.to_le_bytes());
        shdr.extend_from_slice(&44_100u32.to_le_bytes());
        shdr.push(255); // unpitched
        shdr.push(0);
        shdr.extend_from_slice(&0u16.to_le_bytes());
        shdr.extend_from_slice(&SAMPLE_TYPE_MONO.to_le_bytes());
        // EOS
        push_padded_name(&mut shdr, "EOS");
        shdr.extend_from_slice(&[0u8; 26]);

        let mut pdta = Vec::new();
        push_chunk(&mut pdta, b"phdr", &phdr);
        push_chunk(&mut pdta, b"pbag", &pbag);
        push_chunk(&mut pdta, b"pgen", &pgen);
        push_chunk(&mut pdta, b"inst", &inst);
        push_chunk(&mut pdta, b"ibag", &ibag);
        push_chunk(&mut pdta, b"igen", &igen);
        push_chunk(&mut pdta, b"shdr", &shdr);

        let mut sdta = Vec::new();
        push_chunk(&mut sdta, b"smpl", &smpl);

        let mut body = Vec::new();
        body.extend_from_slice(b"sfbk");
        push_list(&mut body, b"INFO", &info);
        push_list(&mut body, b"sdta", &sdta);
        push_list(&mut body, b"pdta", &pdta);

        let mut riff = Vec::with_capacity(8 + body.len());
        riff.extend_from_slice(b"RIFF");
        riff.extend_from_slice(&(body.len() as u32).to_le_bytes());
        riff.extend_from_slice(&body);
        riff
    }

    #[test]
    fn synthetic_sf2_handles_globals_stereo_merge_offsets_and_errors() {
        let bytes = build_synthetic_sf2();
        let tmp = std::env::temp_dir().join(format!(
            "piano_tect_sf2_test_{}.sf2",
            std::process::id()
        ));
        std::fs::write(&tmp, &bytes).unwrap();

        let bank = SoundFontBank::open(&tmp).expect("synthetic sf2 should parse");
        assert_eq!(bank.name(), "Synthetic Test Bank");

        let inst = bank.load_preset(0, 5).expect("preset (0, 5) should load");
        assert_eq!(inst.name, "Synth Lead");
        assert_eq!(inst.zones.len(), 2);

        // Zone 0: merged stereo pair (lengths equalized to 96, loop clamped to 96, pan = 0).
        let z0 = &inst.zones[0];
        assert_eq!(z0.name, "Wave");
        assert_eq!((z0.key_low, z0.key_high), (40, 90));
        assert_eq!((z0.vel_low, z0.vel_high), (10, 120));
        assert_eq!(z0.pcm_l.len(), 96);
        assert_eq!(z0.pcm_r.len(), 96);
        assert!(!Arc::ptr_eq(&z0.pcm_l, &z0.pcm_r));
        assert_eq!(z0.pan, 0);
        assert!(z0.loop_enabled);
        assert_eq!((z0.loop_start, z0.loop_end), (10, 96));
        // coarse +1*100 + preset fine(-15) + inst fine(+5) + corr(-3) = 87
        assert_eq!(z0.fine_tune_cents, 87);

        // Zone 1: mono unpitched single-key zone -> root_key defaults to key_low (36),
        // start offset +2 shifts length from 32 to 30, short loop (2..4) disabled!
        let z1 = &inst.zones[1];
        assert_eq!((z1.key_low, z1.key_high), (36, 36));
        assert_eq!(z1.root_key, 36);
        assert_eq!(z1.pcm_l.len(), 30);
        assert!(Arc::ptr_eq(&z1.pcm_l, &z1.pcm_r));
        assert!(!z1.loop_enabled);
        assert_eq!(z1.pan, 50);

        let _ = std::fs::remove_file(&tmp);

        // Malformed header should return Err without panicking.
        let bad = std::env::temp_dir().join(format!(
            "piano_tect_sf2_bad_{}.sf2",
            std::process::id()
        ));
        std::fs::write(&bad, b"NOT_RIFF_DATA").unwrap();
        assert!(SoundFontBank::open(&bad).is_err());
        let _ = std::fs::remove_file(&bad);
    }
}
