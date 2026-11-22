use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::Arc;

pub const DEFAULT_MUSIC_STUDIO_DIR: &str =
    "/home/data/.config/hydralauncher/wine-prefixes/1174180/drive_c/users/steamuser/AppData/Local/Programs/Music Studio";
pub const USER_DOCUMENTS_RHYTHMS_DIR: &str = "/home/data/Documents/Ritmos_Forro_Music_Studio";
pub const USER_DOCUMENTS_KEYBOARDS_DIR: &str = "/home/data/Documents/Ritmos de Teclados";

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicPackageEntry {
    pub file_name: String,
    pub file_path: String,
    pub size_mb: f32,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicInstrumentSummary {
    pub index: usize,
    pub name: String,
    pub zone_count: usize,
    pub has_samples: bool,
    pub is_drumkit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicStyleSummary {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicPackageSummary {
    pub loaded: bool,
    pub file_name: String,
    pub file_path: String,
    pub version: String,
    pub instruments: Vec<MusicInstrumentSummary>,
    pub styles: Vec<MusicStyleSummary>,
    pub active_style_index: usize,
    pub active_section: String,
    pub pending_section: Option<String>,
    pub upper1_instrument: usize,
    pub upper2_instrument: Option<usize>,
    pub lower_instrument: Option<usize>,
    pub split_key: u8,
    pub split_enabled: bool,
    pub chord_scan_lower: bool,
    pub chord_scan_upper: bool,
    pub upper_octave: i8,
    pub transpose: i8,
    pub bpm: f32,
    pub tempo_locked: bool,
    pub rhythm_playing: bool,
    pub sync_start: bool,
    pub auto_fill: bool,
    pub detected_chord_root: u8,
    pub detected_chord_quality: String,
    pub detected_chord: String,
    pub track_volumes: [f32; 8],
    pub beat_position: f32,
}

impl Default for MusicPackageSummary {
    fn default() -> Self {
        Self {
            loaded: false,
            file_name: String::new(),
            file_path: String::new(),
            version: String::new(),
            instruments: Vec::new(),
            styles: Vec::new(),
            active_style_index: 0,
            active_section: "Variation 1".to_string(),
            pending_section: None,
            upper1_instrument: 0,
            upper2_instrument: None,
            lower_instrument: None,
            split_key: 53,
            split_enabled: true,
            chord_scan_lower: true,
            chord_scan_upper: false,
            upper_octave: 0,
            transpose: 0,
            bpm: 120.0,
            tempo_locked: false,
            rhythm_playing: false,
            sync_start: true,
            auto_fill: true,
            detected_chord_root: 0,
            detected_chord_quality: "Major".to_string(),
            detected_chord: "C".to_string(),
            track_volumes: [1.0; 8],
            beat_position: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SampleZone {
    pub root_key: u8,
    pub key_low: u8,
    pub key_high: u8,
    pub name: String,
    pub sample_rate: u32,
    pub channels: u8,
    pub loop_start: usize,
    pub loop_end: usize,
    pub loop_enabled: bool,
    pub loop_mode: u8,
    pub gain: f32,
    pub pan: i32,
    pub fine_tune: i32,
    pub pool_sid: u32,
    pub pcm_l: Arc<Vec<f32>>,
    pub pcm_r: Arc<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub struct LoadedInstrument {
    pub index: usize,
    pub name: String,
    pub volume: f32,
    pub pan: i32,
    pub inst_type: u8,
    pub is_drumkit: bool,
    pub zones: Vec<SampleZone>,
}

#[derive(Debug, Clone)]
pub struct RhythmNoteEvent {
    pub midi_note: u8,
    pub velocity: u8,
    pub start_tick: u32,
    pub duration_ticks: u32,
}

#[derive(Debug, Clone)]
pub struct RhythmTrack {
    pub track_type: String,
    pub track_slot: usize,
    pub instrument_index: usize,
    pub volume: f32,
    pub pan: i32,
    pub fine_tune: i32,
    pub key_move: i32,
    pub octave_shift: i32,
    pub is_drum: bool,
    pub events: Vec<RhythmNoteEvent>,
}

#[derive(Debug, Clone)]
pub struct RhythmSection {
    pub id: u32,
    pub name: String,
    pub length_ticks: u32,
    pub is_loop: bool,
    pub jump_to: i32,
    pub tracks: Vec<RhythmTrack>,
}

#[derive(Debug, Clone)]
pub struct RhythmStyle {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub bpm: f32,
    pub sections: Vec<RhythmSection>,
}

#[derive(Debug, Clone)]
pub struct LoadedMusicPackage {
    pub file_name: String,
    pub file_path: String,
    pub version: String,
    pub instruments: Vec<LoadedInstrument>,
    pub styles: Vec<RhythmStyle>,
}

#[inline]
fn read_u32_le(data: &[u8], off: usize) -> Option<u32> {
    let b = data.get(off..off.checked_add(4)?)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

#[inline]
fn read_i32_le(data: &[u8], off: usize) -> Option<i32> {
    let b = data.get(off..off.checked_add(4)?)?;
    Some(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn decode_latin1(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    bytes[..end].iter().map(|&b| b as char).collect()
}

fn is_valid_sample_rate(sr: u32) -> bool {
    (4_000..=192_000).contains(&sr)
}

fn version_is_below_1_3(version: &str) -> bool {
    let last = version.split('\r').last().unwrap_or(version).trim();
    let mut parts = last.split('.');
    let Ok(major) = parts.next().unwrap_or("").parse::<u32>() else {
        return false;
    };
    let Ok(minor) = parts.next().unwrap_or("").parse::<u32>() else {
        return false;
    };
    major == 1 && minor < 3
}

fn convert_stored_bpm(raw: u32, is_v12: bool) -> Option<f32> {
    if raw == 0 {
        return None;
    }
    let bpm = if is_v12 {
        ((raw as f32) * 120.0 / 100.0).round()
    } else {
        raw as f32
    };
    if (40.0..=240.0).contains(&bpm) {
        Some(bpm)
    } else {
        None
    }
}

fn track_name_to_slot(name: &str) -> Option<usize> {
    match name {
        "Drum" => Some(0),
        "Percussion" => Some(1),
        "Bass" => Some(2),
        "Acc. 1" => Some(3),
        "Acc. 2" => Some(4),
        "Acc. 3" => Some(5),
        "Acc. 4" => Some(6),
        "Acc. 5" => Some(7),
        _ => None,
    }
}

fn is_mixer_or_track_label(name: &str) -> bool {
    matches!(
        name,
        "Normal"
            | "Master"
            | "Up 1"
            | "Up 2"
            | "Up 3"
            | "Low"
            | "Rhythm"
            | "Wave"
            | "Bass"
            | "Acc. 1"
            | "Acc. 2"
            | "Acc. 3"
            | "Acc. 4"
            | "Acc. 5"
            | "Drum"
            | "Percussion"
    ) || name.starts_with("Acc. ")
        || name.contains(" - Copy")
}

#[derive(Debug, Clone)]
struct ParsedSamplePool {
    pool_end: usize,
    header_bpm: Option<f32>,
    entries: HashMap<u32, (usize, usize)>,
}

fn parse_header_and_sample_pool(data: &[u8], is_v12: bool) -> ParsedSamplePool {
    let vlen = read_u32_le(data, 0x20).unwrap_or(0) as usize;
    let mut cur = 0x24 + vlen;
    if data.get(cur..cur + 4) == Some(b"\xff\xff\x00\x00") {
        cur += 4;
        for _ in 0..3 {
            let Some(slen) = read_u32_le(data, cur) else {
                return ParsedSamplePool {
                    pool_end: 0x40,
                    header_bpm: None,
                    entries: HashMap::new(),
                };
            };
            if slen > 512 {
                return ParsedSamplePool {
                    pool_end: 0x40,
                    header_bpm: None,
                    entries: HashMap::new(),
                };
            }
            cur = cur + 4 + slen as usize;
        }
    }

    let raw_hdr_bpm = data
        .get(cur + 10..cur + 12)
        .map(|b| u16::from_le_bytes([b[0], b[1]]) as u32)
        .filter(|&v| v > 0)
        .or_else(|| read_u32_le(data, cur + 10));
    let header_bpm = raw_hdr_bpm.and_then(|raw| convert_stored_bpm(raw, is_v12));

    let has_pool = read_u32_le(data, 0x14) == Some(1);
    if !has_pool || cur + 142 >= data.len() {
        return ParsedSamplePool {
            pool_end: 0x40,
            header_bpm,
            entries: HashMap::new(),
        };
    }

    let pool_start = cur + 134;
    let Some(cnt) = read_u32_le(data, pool_start + 4) else {
        return ParsedSamplePool {
            pool_end: 0x40,
            header_bpm,
            entries: HashMap::new(),
        };
    };
    if cnt == 0 || cnt > 30_000 {
        return ParsedSamplePool {
            pool_end: 0x40,
            header_bpm,
            entries: HashMap::new(),
        };
    }

    let mut entries = HashMap::with_capacity(cnt as usize);
    let mut pos = pool_start + 8;
    for _ in 0..cnt {
        if pos + 8 > data.len() {
            break;
        }
        let sid = read_u32_le(data, pos).unwrap_or(0);
        let blen = read_u32_le(data, pos + 4).unwrap_or(0) as usize;
        if pos + 8 + blen > data.len() {
            break;
        }
        entries.insert(sid, (pos + 8, blen));
        pos += 8 + blen;
    }

    ParsedSamplePool {
        pool_end: pos.max(0x40),
        header_bpm,
        entries,
    }
}

/// Scans `start_off..data.len()` for all `<u32 nlen><text>` versioned tags containing `"\r1.<ver>"`.
/// Supports track/author tags such as `Name\rMUEL TECLAS\r1.4.2` and `Drum\r1.3.0\r120,064,053`.
fn find_versioned_headers(data: &[u8], start_off: usize) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let len = data.len();
    let mut i = start_off.max(0x40);
    while i + 5 < len {
        let Some(rel) = data[i..len.saturating_sub(4)].iter().position(|&b| b == b'\r') else {
            break;
        };
        i += rel;
        if data.get(i..i + 3) == Some(b"\r1.") {
            let mut v_end = i + 3;
            while v_end < len
                && v_end < i + 12
                && (data[v_end].is_ascii_digit() || data[v_end] == b'.')
            {
                v_end += 1;
            }
            if v_end > i + 3 {
                let mut matched = None;
                for back in 0..=120_usize {
                    if i < back + 4 {
                        break;
                    }
                    let str_start = i - back;
                    let u32_off = str_start - 4;
                    if let Some(nlen) = read_u32_le(data, u32_off) {
                        if (5..=160).contains(&nlen) {
                            let end_off = str_start + nlen as usize;
                            if end_off <= len
                                && end_off >= v_end
                                && (end_off == v_end || data.get(v_end) == Some(&b'\r'))
                            {
                                let all_bytes = &data[str_start..end_off];
                                if all_bytes
                                    .iter()
                                    .all(|&b| (0x20..=0xfe).contains(&b) || b == b'\r')
                                {
                                    let before_cr =
                                        all_bytes.split(|&b| b == b'\r').next().unwrap_or(&[]);
                                    let name = decode_latin1(before_cr);
                                    matched = Some((u32_off, end_off, name));
                                    break;
                                }
                            }
                        }
                    }
                }
                if let Some(hit) = matched {
                    let next_i = hit.1;
                    out.push(hit);
                    i = next_i;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

fn looks_like_zone_97(data: &[u8], off: usize) -> bool {
    let Some(hdr) = data.get(off..off + 97) else {
        return false;
    };
    let root = hdr[0];
    let nlen = hdr[1];
    let sr_u16 = u16::from_le_bytes([hdr[33], hdr[34]]) as u32;
    let bits = hdr[35];
    let ch = hdr[36];
    let sr_u32 = u32::from_le_bytes([hdr[37], hdr[38], hdr[39], hdr[40]]);
    let eff_sr = if sr_u32 != 0 { sr_u32 } else { sr_u16 };
    let sr_consistent = sr_u32 == 0 || sr_u32 == sr_u16 || sr_u16 == 0;
    let nfr = u32::from_le_bytes([hdr[45], hdr[46], hdr[47], hdr[48]]);
    root <= 127
        && nlen <= 32
        && sr_consistent
        && is_valid_sample_rate(eff_sr)
        && (bits == 8 || bits == 16)
        && (ch == 1 || ch == 2)
        && nfr >= 8
        && nfr <= 15_000_000
}

fn decode_pcm_bytes(raw: &[u8], bits: u8, ch: u8, nfr_hint: usize) -> (Vec<f32>, Vec<f32>) {
    let mut pcm_l = Vec::new();
    let mut pcm_r = Vec::new();
    if raw.is_empty() || ch == 0 {
        return (pcm_l, pcm_r);
    }
    let ch_u = ch as usize;
    let est_frames = if bits == 16 {
        raw.len() / (ch_u * 2)
    } else {
        raw.len() / ch_u
    };
    let count = if nfr_hint > 0 {
        nfr_hint.min(est_frames)
    } else {
        est_frames
    };
    pcm_l.reserve(count);
    pcm_r.reserve(count);
    if bits == 16 {
        let stride = ch_u * 2;
        for frame in raw.chunks_exact(stride).take(count) {
            let l = i16::from_le_bytes([frame[0], frame[1]]) as f32 / 32768.0;
            let r = if ch == 2 {
                i16::from_le_bytes([frame[2], frame[3]]) as f32 / 32768.0
            } else {
                l
            };
            pcm_l.push(l);
            pcm_r.push(r);
        }
    } else {
        let stride = ch_u;
        for frame in raw.chunks_exact(stride).take(count) {
            let l = (frame[0] as f32 - 128.0) / 128.0;
            let r = if ch == 2 {
                (frame[1] as f32 - 128.0) / 128.0
            } else {
                l
            };
            pcm_l.push(l);
            pcm_r.push(r);
        }
    }
    (pcm_l, pcm_r)
}

fn factory_sample_pool_index() -> &'static HashMap<u32, (std::path::PathBuf, u64, usize)> {
    static CACHE: std::sync::OnceLock<HashMap<u32, (std::path::PathBuf, u64, usize)>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| {
        let mut map = HashMap::new();
        for fname in ["User.Template", "Default.Template"] {
            let p = Path::new(DEFAULT_MUSIC_STUDIO_DIR).join(fname);
            if !p.exists() {
                continue;
            }
            if let Ok(bytes) = std::fs::read(&p) {
                let pool = parse_header_and_sample_pool(&bytes, false);
                for (sid, (off, len)) in pool.entries {
                    map.entry(sid).or_insert((p.clone(), off as u64, len));
                }
            }
        }
        map
    })
}

fn read_factory_pool_bytes(sid_key: u32) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let (path, off, len) = factory_sample_pool_index().get(&sid_key)?;
    if *len == 0 || *len > 32 * 1024 * 1024 {
        return None;
    }
    let mut file = std::fs::File::open(path).ok()?;
    file.seek(SeekFrom::Start(*off)).ok()?;
    let mut buf = vec![0_u8; *len];
    file.read_exact(&mut buf).ok()?;
    Some(buf)
}

fn parse_zone_at(
    data: &[u8],
    cur: usize,
    pool: &HashMap<u32, (usize, usize)>,
    fallback_keys: &[u8],
    zone_idx: usize,
) -> Option<(SampleZone, usize)> {
    let hdr = data.get(cur..cur + 97)?;

    let rk_raw = hdr[0];
    let root_key = if rk_raw <= 127 {
        rk_raw
    } else {
        *fallback_keys.get(zone_idx).unwrap_or(&60)
    };
    let nlen = (hdr[1] as usize).min(31);
    let name = decode_latin1(&hdr[2..2 + nlen]);
    let sr_u16 = u16::from_le_bytes([hdr[33], hdr[34]]) as u32;
    let bits = hdr[35];
    let ch = hdr[36];
    let sr_u32 = u32::from_le_bytes([hdr[37], hdr[38], hdr[39], hdr[40]]);
    let sr = if sr_u32 != 0 { sr_u32 } else { sr_u16 };
    let pool_sid = u32::from_le_bytes([hdr[41], hdr[42], hdr[43], hdr[44]]);
    let nfr = u32::from_le_bytes([hdr[45], hdr[46], hdr[47], hdr[48]]) as usize;
    let vol_raw = i32::from_le_bytes([hdr[53], hdr[54], hdr[55], hdr[56]]);
    let pan = i32::from_le_bytes([hdr[57], hdr[58], hdr[59], hdr[60]]).clamp(-100, 100);
    let fine_tune = i32::from_le_bytes([hdr[61], hdr[62], hdr[63], hdr[64]]).clamp(-12000, 12000);
    let loop_mode = hdr[69];
    let loop_start = u32::from_le_bytes([hdr[73], hdr[74], hdr[75], hdr[76]]) as usize;
    let loop_end = u32::from_le_bytes([hdr[77], hdr[78], hdr[79], hdr[80]]) as usize;

    let bytes_per_sample = (bits as usize / 8).max(1);
    let pcm_bytes = nfr.saturating_mul(ch as usize).saturating_mul(bytes_per_sample);
    let pcm_start = cur + 97;

    let (pcm_l, pcm_r, next_cur) = if pool_sid == 0 {
        if pcm_bytes > 0 && pcm_start + pcm_bytes <= data.len() {
            let (l, r) = decode_pcm_bytes(&data[pcm_start..pcm_start + pcm_bytes], bits, ch, nfr);
            (l, r, pcm_start + pcm_bytes)
        } else {
            (Vec::new(), Vec::new(), pcm_start)
        }
    } else {
        let pool_key = pool_sid - 1;
        if let Some(&(pool_off, pool_len)) = pool.get(&pool_key) {
            let avail = pool_len.min(data.len().saturating_sub(pool_off));
            let (l, r) = decode_pcm_bytes(&data[pool_off..pool_off + avail], bits, ch, nfr);
            (l, r, pcm_start)
        } else if let Some(ext_bytes) = read_factory_pool_bytes(pool_key) {
            let (l, r) = decode_pcm_bytes(&ext_bytes, bits, ch, nfr);
            (l, r, pcm_start)
        } else {
            (Vec::new(), Vec::new(), pcm_start)
        }
    };

    let gain_norm = if (1..=250).contains(&vol_raw) {
        vol_raw as f32 / 100.0
    } else {
        1.0
    };
    let valid_loop =
        (loop_mode == 2 || loop_mode == 3) && loop_end > loop_start + 32 && loop_end <= pcm_l.len();

    Some((
        SampleZone {
            root_key,
            key_low: root_key,
            key_high: root_key,
            name,
            sample_rate: sr,
            channels: ch,
            loop_start,
            loop_end,
            loop_enabled: valid_loop,
            loop_mode,
            gain: gain_norm,
            pan,
            fine_tune,
            pool_sid,
            pcm_l: Arc::new(pcm_l),
            pcm_r: Arc::new(pcm_r),
        },
        next_cur,
    ))
}

fn parse_zones_sequence(
    data: &[u8],
    start: usize,
    nzones: usize,
    block_end: usize,
    fallback_keys: &[u8],
    pool: &HashMap<u32, (usize, usize)>,
) -> Vec<SampleZone> {
    let mut zones = Vec::new();
    let mut cur = start;
    for z in 0..nzones.max(1).min(128) {
        if cur + 97 > block_end {
            break;
        }
        if !looks_like_zone_97(data, cur) {
            break;
        }
        if let Some((zone, nxt)) = parse_zone_at(data, cur, pool, fallback_keys, z) {
            zones.push(zone);
            cur = nxt;
        } else {
            break;
        }
    }
    zones
}

fn assign_key_ranges(zones: &mut [SampleZone], is_drumkit: bool) {
    if zones.is_empty() {
        return;
    }
    if is_drumkit {
        for z in zones.iter_mut() {
            z.key_low = z.root_key;
            z.key_high = z.root_key;
            z.loop_enabled = false;
        }
        return;
    }
    zones.sort_by_key(|z| z.root_key);
    let n = zones.len();
    for i in 0..n {
        let low = if i == 0 {
            0
        } else {
            ((zones[i - 1].root_key as u16 + zones[i].root_key as u16) / 2 + 1)
                .min(zones[i].root_key as u16) as u8
        };
        let high = if i + 1 == n {
            127
        } else {
            ((zones[i].root_key as u16 + zones[i + 1].root_key as u16) / 2)
                .max(zones[i].root_key as u16) as u8
        };
        zones[i].key_low = low;
        zones[i].key_high = high;
    }
}

fn factory_fallback_instruments() -> &'static HashMap<String, Vec<SampleZone>> {
    static CACHE: std::sync::OnceLock<HashMap<String, Vec<SampleZone>>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| {
        let mut map = HashMap::new();
        for fname in ["Default.Template", "User.Template"] {
            let template_path = Path::new(DEFAULT_MUSIC_STUDIO_DIR).join(fname);
            if template_path.exists() {
                if let Ok(pkg) = parse_music_file_internal(&template_path, false) {
                    for inst in pkg.instruments {
                        if inst.zones.iter().any(|z| !z.pcm_l.is_empty()) {
                            map.entry(inst.name.to_lowercase()).or_insert(inst.zones);
                        }
                    }
                }
            }
        }
        map
    })
}

fn parse_instrument_block(
    data: &[u8],
    index: usize,
    name: String,
    hdr_end: usize,
    block_end: usize,
    pool: &HashMap<u32, (usize, usize)>,
    use_factory_fallback: bool,
) -> LoadedInstrument {
    let inst_vol_raw = read_i32_le(data, hdr_end + 116).unwrap_or(100);
    let inst_volume = if (1..=250).contains(&inst_vol_raw) {
        inst_vol_raw as f32 / 100.0
    } else {
        1.0
    };
    let inst_pan = read_i32_le(data, hdr_end + 120)
        .unwrap_or(0)
        .clamp(-100, 100);
    let inst_type = data.get(hdr_end + 124).copied().unwrap_or(0);

    let fallback_keys: Vec<u8> = data
        .get(hdr_end + 5..hdr_end + 133)
        .map(|s| s.iter().copied().filter(|&k| k >= 12 && k <= 127).collect())
        .unwrap_or_default();

    let scan_limit = block_end.min(hdr_end + 65_536);
    let mut zones = Vec::new();
    let prim_rel = data
        .get(hdr_end..scan_limit)
        .and_then(|slice| {
            slice
                .windows(11)
                .position(|w| w == b"\x07\x00\x00\x00Primary")
        });

    if let Some(rel) = prim_rel {
        let after_prim = hdr_end + rel + 11;
        for delta in [7_usize, 4, 6, 5, 8] {
            let z0 = after_prim + delta;
            if looks_like_zone_97(data, z0) {
                let nz = data.get(z0 - 1).copied().unwrap_or(1).max(1) as usize;
                zones = parse_zones_sequence(data, z0, nz, block_end, &fallback_keys, pool);
                if !zones.is_empty() {
                    break;
                }
            }
        }
    }

    if zones.is_empty() {
        let scan_start = hdr_end + 125;
        let mut found_97 = None;
        for off in scan_start..scan_limit {
            if looks_like_zone_97(data, off) {
                found_97 = Some(off);
                break;
            }
        }

        if let Some(z0) = found_97 {
            let nz_u8 = data.get(z0.saturating_sub(1)).copied().unwrap_or(0) as usize;
            let nzones = if (1..=128).contains(&nz_u8) {
                nz_u8
            } else {
                read_u32_le(data, z0.saturating_sub(4))
                    .filter(|&v| (1..=128).contains(&v))
                    .unwrap_or(1) as usize
            };
            zones = parse_zones_sequence(data, z0, nzones, block_end, &fallback_keys, pool);
        }
    }

    if use_factory_fallback
        && (zones.is_empty() || zones.iter().all(|z| z.pcm_l.is_empty()))
    {
        if let Some(fallback_zones) = factory_fallback_instruments().get(&name.to_lowercase()) {
            zones = fallback_zones.clone();
        }
    }

    let up = name.to_uppercase();
    let is_drumkit = if inst_type <= 2 {
        inst_type != 0
    } else {
        up.contains("BTR")
            || up.contains("DRUM")
            || up.contains("BATERIA")
            || up.contains("KIT")
            || up.contains("PERC")
            || up.contains("VINHETA")
            || up.contains("HIT HAT")
            || up == "GENERAL"
            || name == "-"
    };

    assign_key_ranges(&mut zones, is_drumkit);

    let display_name = if name == "-" {
        "Metronome / Internal FX".to_string()
    } else {
        name
    };

    LoadedInstrument {
        index,
        name: display_name,
        volume: inst_volume,
        pan: inst_pan,
        inst_type,
        is_drumkit,
        zones,
    }
}

#[derive(Debug, Clone)]
struct TailNode {
    id: u32,
    parent_id: u32,
    code: u8,
    section_code: u8,
    name: String,
    order: usize,
    is_loop: bool,
    is_limited: bool,
    limit_start: u32,
    limit_end: u32,
    jump_to: i32,
    bpm: Option<f32>,
    vkey: u32,
}

fn canonical_section_name(raw: &str, section_code: u8) -> (String, bool) {
    let trimmed = raw.trim();
    let lower = trimmed.to_lowercase();
    let is_cv_alt = lower.contains(" cv2") || lower.contains(" cv3");

    let by_code = match section_code {
        0x10 => Some("Intro 1"),
        0x11 => Some("Intro 2"),
        0x12 | 0x13 => Some("Intro 3"),
        0x20 => Some("Variation 1"),
        0x21 => Some("Variation 2"),
        0x22 => Some("Variation 3"),
        0x23 => Some("Variation 4"),
        0x30 => Some("Fill 1"),
        0x31 => Some("Fill 2"),
        0x32 => Some("Fill 3"),
        0x33 => Some("Fill 4"),
        0x40..=0x43 => Some("Break"),
        0x50 => Some("Ending 1"),
        0x51 => Some("Ending 2"),
        0x52 | 0x53 => Some("Ending 3"),
        0x60 => Some("Pad 1"),
        0x61 => Some("Pad 2"),
        0x62 => Some("Pad 3"),
        0x63 => Some("Pad 4"),
        _ => None,
    };
    if let Some(canon) = by_code {
        return (canon.to_string(), is_cv_alt);
    }

    let base = lower
        .split(" cv")
        .next()
        .unwrap_or(lower.as_str())
        .trim();

    let canon = match base {
        "intro 1" | "introducao 1" | "introdução 1" => "Intro 1",
        "intro 2" | "introducao 2" | "introdução 2" => "Intro 2",
        "intro 3" | "introducao 3" | "introdução 3" | "intro 4" => "Intro 3",
        "variation 1" | "variacao 1" | "variação 1" | "varia??o 1" => "Variation 1",
        "variation 2" | "variacao 2" | "variação 2" | "varia??o 2" => "Variation 2",
        "variation 3" | "variacao 3" | "variação 3" | "varia??o 3" => "Variation 3",
        "variation 4" | "variacao 4" | "variação 4" | "varia??o 4" => "Variation 4",
        "fill 1" | "virada 1" => "Fill 1",
        "fill 2" | "virada 2" => "Fill 2",
        "fill 3" | "virada 3" => "Fill 3",
        "fill 4" | "virada 4" => "Fill 4",
        "break" | "default break" | "gta off/on" => "Break",
        "pad 1" | "default pad 1" => "Pad 1",
        "pad 2" | "default pad 2" => "Pad 2",
        "pad 3" | "default pad 3" => "Pad 3",
        "pad 4" | "default pad 4" => "Pad 4",
        "ending 1" | "final 1" => "Ending 1",
        "ending 2" | "final 2" => "Ending 2",
        "ending 3" | "final 3" => "Ending 3",
        _ => {
            if base.contains("varia") && base.ends_with('1') {
                "Variation 1"
            } else if base.contains("varia") && base.ends_with('2') {
                "Variation 2"
            } else if base.contains("varia") && base.ends_with('3') {
                "Variation 3"
            } else if base.contains("varia") && base.ends_with('4') {
                "Variation 4"
            } else if base.contains("virada") && base.ends_with('1') {
                "Fill 1"
            } else if base.contains("virada") && base.ends_with('2') {
                "Fill 2"
            } else if base.contains("virada") && base.ends_with('3') {
                "Fill 3"
            } else if base.contains("virada") && base.ends_with('4') {
                "Fill 4"
            } else if base.contains("intro") && base.ends_with('1') {
                "Intro 1"
            } else if base.contains("intro") && base.ends_with('2') {
                "Intro 2"
            } else if base.contains("intro") {
                "Intro 3"
            } else if base.contains("final") || base.contains("end") {
                "Ending 1"
            } else {
                return (trimmed.to_string(), is_cv_alt);
            }
        }
    };
    (canon.to_string(), is_cv_alt)
}

fn is_music_studio_magic(data: &[u8]) -> bool {
    data.starts_with(b"Music Studio Music\x00") || data.starts_with(b"Music Studio music\x00")
}

fn parse_tail_tree(data: &[u8], is_v12: bool) -> (usize, Vec<TailNode>) {
    let search_start = data.len().saturating_sub(2_000_000);
    let tail = &data[search_start..];
    // Find Node 0: \x00\x00\x00\x00 \xff\xff\xff\xff <lenByte> Music \0
    let Some(rel) = tail.windows(15).position(|w| {
        &w[0..8] == b"\x00\x00\x00\x00\xff\xff\xff\xff" && &w[9..15] == b"Music\0"
    }) else {
        return (data.len(), Vec::new());
    };
    let n0_off = search_start + rel;
    let count = read_u32_le(data, n0_off.saturating_sub(4)).unwrap_or(0) as usize;
    if count == 0 || count > 4000 {
        return (n0_off, Vec::new());
    }

    let db_off = data[n0_off..data.len().min(n0_off + 1024)]
        .windows(14)
        .position(|w| w == b"Default Break\0")
        .map(|pos| n0_off + pos - 9);

    let mut base_stride = 124_usize;
    if let Some(db) = db_off {
        let dp1_off = data[db..data.len().min(db + 512)]
            .windows(14)
            .position(|w| w == b"Default Pad 1\0")
            .map(|pos| db + pos - 9);
        if let Some(dp1) = dp1_off {
            let diff = dp1.saturating_sub(db);
            if diff == 72 || diff == 124 || diff == 140 {
                base_stride = diff;
            }
        } else {
            let diff = db.saturating_sub(n0_off);
            if diff == 72 || diff == 124 || diff == 140 {
                base_stride = diff;
            }
        }
    }

    let mut nodes = Vec::with_capacity(count);
    let parse_node = |cur: usize, order: usize| -> Option<TailNode> {
        if cur + base_stride.min(72) > data.len() {
            return None;
        }
        let nid = read_u32_le(data, cur).unwrap_or(0);
        let pid = read_u32_le(data, cur + 4).unwrap_or(0xffffffff);
        let raw_len = data[cur + 8] as usize;
        let name = if raw_len == 0x20 {
            decode_latin1(&data[cur + 9..cur + 40]).trim().to_string()
        } else {
            decode_latin1(&data[cur + 9..cur + 9 + raw_len.min(31)])
                .trim()
                .to_string()
        };
        let mut section_code = data[cur + 40];
        let code = data[cur + 41];
        let is_loop = data[cur + 42] != 0;
        let is_limited = data[cur + 43] != 0;
        let limit_start = read_u32_le(data, cur + 44).unwrap_or(0);
        let limit_end = read_u32_le(data, cur + 48).unwrap_or(4000);
        let jump_to = read_i32_le(data, cur + 52).unwrap_or(0);
        let tempo_raw = data
            .get(cur + 62..cur + 64)
            .map(|b| u16::from_le_bytes([b[0], b[1]]) as u32)
            .unwrap_or(0);
        let bpm = convert_stored_bpm(tempo_raw, is_v12);
        let vkey = read_u32_le(data, cur + 64).unwrap_or(0);

        if code == 1 && (section_code >> 4) == 0 {
            section_code = match vkey {
                0x71 => 0x10,
                0x72 => 0x11,
                0x73 => 0x20,
                0x74 => 0x21,
                0x75 => 0x22,
                0x76 => 0x23,
                0x77 => 0x30,
                0x78 => 0x31,
                0x79 => 0x50,
                0x7a => 0x51,
                0x7b => 0x40,
                _ => section_code,
            };
        }

        Some(TailNode {
            id: nid,
            parent_id: pid,
            code,
            section_code,
            name,
            order,
            is_loop,
            is_limited,
            limit_start,
            limit_end,
            jump_to,
            bpm,
            vkey,
        })
    };

    if let Some(n0) = parse_node(n0_off, 0) {
        nodes.push(n0);
    }

    let mut cur = db_off.unwrap_or(n0_off + base_stride);
    for order in 1..count {
        if cur + base_stride > data.len() {
            break;
        }
        if let Some(node) = parse_node(cur, order) {
            nodes.push(node);
        }
        if base_stride == 124 {
            let extra = read_u32_le(data, cur + 120).unwrap_or(0).min(32) as usize;
            cur += 124 + extra * 16;
        } else if base_stride == 140 {
            let extra = read_u32_le(data, cur + 136).unwrap_or(0).min(32) as usize;
            cur += 140 + extra * 16;
        } else {
            cur += base_stride;
        }
    }

    (n0_off.saturating_sub(4), nodes)
}

fn parse_rhythm_tracks(
    data: &[u8],
    headers: &[(usize, usize, String)],
    mixer_idx: usize,
    tail_start: usize,
    instruments: &[LoadedInstrument],
) -> HashMap<u32, Vec<RhythmTrack>> {
    let mut by_section: HashMap<u32, Vec<RhythmTrack>> = HashMap::new();
    let mut track_scan_start = mixer_idx + 1;
    for (idx, (_, _, nm)) in headers.iter().enumerate().skip(mixer_idx).take(28) {
        if nm == "Master" {
            track_scan_start = idx + 1;
            break;
        }
    }
    let mut track_start_idx = track_scan_start;
    for (idx, (_, _, nm)) in headers.iter().enumerate().skip(track_scan_start).take(24) {
        if nm == "Drum" {
            track_start_idx = idx + 1;
            break;
        }
    }

    for (idx, &(blk_start, hdr_end, ref full_name)) in headers.iter().enumerate().skip(track_start_idx)
    {
        if blk_start >= tail_start {
            break;
        }
        let clean = full_name.split(" - Copy").next().unwrap_or(full_name);
        let Some(slot) = track_name_to_slot(clean) else {
            continue;
        };
        let next_blk = headers
            .get(idx + 1)
            .map(|h| h.0.min(tail_start))
            .unwrap_or(tail_start);

        let Some(sec_id) = read_u32_le(data, hdr_end) else {
            continue;
        };
        if data.get(hdr_end + 4).copied().unwrap_or(1) == 0 {
            continue;
        }
        let inst_idx = read_u32_le(data, hdr_end + 12).unwrap_or(0) as usize;
        let is_drum = instruments
            .get(inst_idx)
            .map(|i| i.is_drumkit)
            .unwrap_or(slot <= 1);

        // Locate event_count at +531 (v1.3/1.4) or +544 (v1.2) FIRST so TSoundSetting offset shifts cleanly (+13)
        let mut detected_hsz = None;
        let mut events = Vec::new();
        for &hsz in &[531_usize, 544_usize] {
            let evc_off = hdr_end + hsz;
            let Some(evc) = read_u32_le(data, evc_off) else {
                continue;
            };
            let evc = evc as usize;
            let arr_start = evc_off + 4;
            let arr_end = arr_start.saturating_add(evc.saturating_mul(24));
            if evc > 0 && evc <= 8000 && arr_end <= next_blk {
                let t0 = read_i32_le(data, arr_start).unwrap_or(0);
                let n0 = read_i32_le(data, arr_start + 8).unwrap_or(-1);
                if (1..=4).contains(&t0) && (0..=127).contains(&n0) {
                    detected_hsz = Some(hsz);
                    events.reserve(evc);
                    for e in 0..evc {
                        let eo = arr_start + e * 24;
                        let etype = read_i32_le(data, eo).unwrap_or(0);
                        let note = read_i32_le(data, eo + 8).unwrap_or(-1);
                        let vel = read_i32_le(data, eo + 12).unwrap_or(100);
                        let tick = read_i32_le(data, eo + 16).unwrap_or(0);
                        let dur = read_i32_le(data, eo + 20).unwrap_or(120);
                        if etype == 2 && (0..=127).contains(&note) && tick >= 0 && (dur > 0 || is_drum) {
                            events.push(RhythmNoteEvent {
                                midi_note: note as u8,
                                velocity: vel.clamp(1, 127) as u8,
                                start_tick: tick as u32,
                                duration_ticks: dur.max(30) as u32,
                            });
                        }
                    }
                    events.sort_by_key(|ev| ev.start_tick);
                    break;
                }
            }
        }

        if events.is_empty() {
            continue;
        }

        let ss_base = if detected_hsz == Some(544) {
            172_usize
        } else {
            159_usize
        };
        let vol_raw = read_i32_le(data, hdr_end + ss_base).unwrap_or(100);
        let vol = (vol_raw.clamp(0, 250) as f32) / 100.0;
        let pan = read_i32_le(data, hdr_end + ss_base + 4)
            .unwrap_or(0)
            .clamp(-100, 100);
        let fine_tune = read_i32_le(data, hdr_end + ss_base + 8)
            .unwrap_or(0)
            .clamp(-12000, 12000);
        let key_move = read_i32_le(data, hdr_end + ss_base + 16)
            .unwrap_or(0)
            .clamp(-24, 24);
        let octave_shift = read_i32_le(data, hdr_end + ss_base + 20)
            .unwrap_or(0)
            .clamp(-4, 4);

        by_section.entry(sec_id).or_default().push(RhythmTrack {
            track_type: clean.to_string(),
            track_slot: slot,
            instrument_index: inst_idx,
            volume: vol,
            pan,
            fine_tune,
            key_move,
            octave_shift,
            is_drum,
            events,
        });
    }

    by_section
}

/// Parses a proprietary Music Studio `.Music` / `.Template` binary file completely cleanly.
pub fn parse_music_file(path: &Path) -> Result<LoadedMusicPackage, String> {
    parse_music_file_internal(path, true)
}

fn parse_music_file_internal(
    path: &Path,
    use_factory_fallback: bool,
) -> Result<LoadedMusicPackage, String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read '{}': {}", path.display(), e))?;
    if !is_music_studio_magic(&data) {
        return Err(format!(
            "'{}' is not a valid Music Studio .Music file",
            path.display()
        ));
    }

    let vlen = read_u32_le(&data, 0x20).unwrap_or(0) as usize;
    let version = if vlen > 0 && vlen <= 32 && 0x24 + vlen <= data.len() {
        decode_latin1(&data[0x24..0x24 + vlen])
            .trim_matches('\r')
            .to_string()
    } else {
        "unknown".to_string()
    };
    let is_v12 = version_is_below_1_3(&version);

    let sample_pool = parse_header_and_sample_pool(&data, is_v12);
    let (tail_start, tail_nodes) = parse_tail_tree(&data, is_v12);
    let headers = find_versioned_headers(&data[..tail_start], sample_pool.pool_end);

    let mixer_idx = headers
        .iter()
        .enumerate()
        .position(|(i, (_, _, name))| {
            name == "Normal"
                && headers
                    .get(i + 1..(i + 28).min(headers.len()))
                    .map(|slice| slice.iter().any(|(_, _, n)| n == "Master"))
                    .unwrap_or(false)
        })
        .or_else(|| {
            headers.iter().enumerate().position(|(i, (_, _, name))| {
                name == "Master"
                    && headers
                        .get(i + 1..(i + 16).min(headers.len()))
                        .map(|slice| slice.iter().any(|(_, _, n)| n == "Drum" || n == "Up 1"))
                        .unwrap_or(false)
            })
        })
        .unwrap_or(headers.len());

    let mut instruments = Vec::new();
    let (inst_start_idx, inst_end_idx) = if headers.first().map(|h| h.2.is_empty()).unwrap_or(false) {
        let count = read_u32_le(&data, headers[0].1 + 4).unwrap_or(0) as usize;
        let end = (1 + count).min(mixer_idx);
        (1_usize, end)
    } else {
        (0_usize, mixer_idx)
    };

    for i in inst_start_idx..inst_end_idx {
        let (_blk_start, hdr_end, ref name) = headers[i];
        if name.is_empty() || is_mixer_or_track_label(name) {
            continue;
        }
        let block_end = headers
            .get(i + 1)
            .map(|h| h.0)
            .unwrap_or(tail_start);
        let idx = instruments.len();
        instruments.push(parse_instrument_block(
            &data,
            idx,
            name.clone(),
            hdr_end,
            block_end,
            &sample_pool.entries,
            use_factory_fallback,
        ));
    }

    let tracks_by_section = parse_rhythm_tracks(&data, &headers, mixer_idx, tail_start, &instruments);

    let default_bpm = sample_pool.header_bpm.unwrap_or_else(|| {
        read_u32_le(&data, 0x44)
            .and_then(|raw| convert_stored_bpm(raw, is_v12))
            .unwrap_or(120.0)
    });

    // Build Styles tree from Tail Nodes
    let node_by_id: HashMap<u32, &TailNode> = tail_nodes.iter().map(|n| (n.id, n)).collect();
    let mut styles_map: BTreeMap<(usize, u32), RhythmStyle> = BTreeMap::new();

    for node in &tail_nodes {
        if node.code == 0x01 {
            if (node.section_code >> 4) == 6 {
                continue; // Skip Pad 1..4 sections from main rhythm transport
            }
            let Some(tracks_raw) = tracks_by_section.get(&node.id) else {
                continue;
            };
            if tracks_raw.is_empty() {
                continue;
            }
            let parent_style = node_by_id.get(&node.parent_id).copied();
            let style_id = parent_style.map(|p| p.id).unwrap_or(0);
            let style_order = parent_style.map(|p| p.order).unwrap_or(0);
            let style_name = parent_style
                .map(|p| {
                    if p.name.is_empty() {
                        "Main Style".to_string()
                    } else {
                        p.name.clone()
                    }
                })
                .unwrap_or_else(|| "Main Style".to_string());
            let category = parent_style
                .and_then(|p| node_by_id.get(&p.parent_id))
                .map(|cat| cat.name.clone())
                .filter(|c| !c.is_empty() && c != "Music")
                .unwrap_or_else(|| "Style".to_string());

            let (canon_name, is_cv_alt) = canonical_section_name(&node.name, node.section_code);
            if canon_name.starts_with("Pad ") {
                continue;
            }
            let _ = node.vkey;
            let section_bpm = node.bpm.unwrap_or(default_bpm);

            let (length_ticks, tracks) = if node.is_limited && node.limit_end > node.limit_start {
                let len_ticks = (node.limit_end - node.limit_start).max(500);
                let mut trimmed: Vec<RhythmTrack> = tracks_raw
                    .iter()
                    .filter_map(|t| {
                        let filtered: Vec<RhythmNoteEvent> = t
                            .events
                            .iter()
                            .filter(|ev| {
                                ev.start_tick >= node.limit_start && ev.start_tick < node.limit_end
                            })
                            .map(|ev| {
                                let mut cloned = ev.clone();
                                cloned.start_tick -= node.limit_start;
                                cloned
                            })
                            .collect();
                        if filtered.is_empty() {
                            None
                        } else {
                            let mut tr = t.clone();
                            tr.events = filtered;
                            Some(tr)
                        }
                    })
                    .collect();
                if trimmed.is_empty() {
                    trimmed = tracks_raw.clone();
                }
                (len_ticks, trimmed)
            } else {
                let fallback_len = node.limit_end.saturating_sub(node.limit_start).max(1000);
                let max_end_tick = tracks_raw
                    .iter()
                    .flat_map(|t| t.events.iter())
                    .map(|ev| ev.start_tick + ev.duration_ticks.max(125))
                    .max()
                    .unwrap_or(fallback_len);
                let len_ticks = ((max_end_tick + 499) / 500 * 500)
                    .max(fallback_len)
                    .max(1000);
                (len_ticks, tracks_raw.clone())
            };

            let entry = styles_map
                .entry((style_order, style_id))
                .or_insert_with(|| RhythmStyle {
                    id: style_id,
                    name: style_name,
                    category,
                    bpm: section_bpm,
                    sections: Vec::new(),
                });

            if let Some(b) = node.bpm {
                if canon_name == "Variation 1" || entry.sections.is_empty() {
                    entry.bpm = b;
                }
            }

            if is_cv_alt && entry.sections.iter().any(|s| s.name == canon_name) {
                continue;
            }
            if let Some(existing) = entry.sections.iter_mut().find(|s| s.name == canon_name) {
                if !is_cv_alt && existing.tracks.len() < tracks.len() {
                    existing.tracks = tracks;
                    existing.length_ticks = length_ticks;
                    existing.is_loop = node.is_loop;
                    existing.jump_to = node.jump_to;
                }
            } else {
                entry.sections.push(RhythmSection {
                    id: node.id,
                    name: canon_name,
                    length_ticks,
                    is_loop: node.is_loop,
                    jump_to: node.jump_to,
                    tracks,
                });
            }
        }
    }

    // Prefer real styles over `Live Style` (`id == 0x80000000`) when real styles exist
    let all_styles: Vec<RhythmStyle> = styles_map.into_values().collect();
    let real_styles: Vec<RhythmStyle> = all_styles
        .iter()
        .filter(|s| s.id != 0x8000_0000 && !s.name.eq_ignore_ascii_case("Live Style"))
        .cloned()
        .collect();

    let styles: Vec<RhythmStyle> = if !real_styles.is_empty() {
        real_styles
    } else if !all_styles.is_empty() {
        all_styles
    } else if !tracks_by_section.is_empty() {
        let mut fallback_sections = Vec::new();
        for (idx, (&sec_id, tracks)) in tracks_by_section.iter().enumerate() {
            let max_end = tracks
                .iter()
                .flat_map(|t| t.events.iter())
                .map(|e| e.start_tick + 1)
                .max()
                .unwrap_or(4000);
            fallback_sections.push(RhythmSection {
                id: sec_id,
                name: format!("Variation {}", idx + 1),
                length_ticks: ((max_end + 499) / 500 * 500).max(4000),
                is_loop: true,
                jump_to: 0,
                tracks: tracks.clone(),
            });
        }
        vec![RhythmStyle {
            id: 1,
            name: path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Style")
                .to_string(),
            category: "Music".to_string(),
            bpm: default_bpm,
            sections: fallback_sections,
        }]
    } else {
        Vec::new()
    };

    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown.Music")
        .to_string();

    Ok(LoadedMusicPackage {
        file_name,
        file_path: path.to_string_lossy().to_string(),
        version,
        instruments,
        styles,
    })
}

fn scan_dir_for_packages(dir: &Path, depth: usize, out: &mut Vec<MusicPackageEntry>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for item in rd.flatten() {
        let p = item.path();
        if p.is_dir() && depth > 0 {
            scan_dir_for_packages(&p, depth - 1, out);
            continue;
        }
        if !p.is_file() {
            continue;
        }
        let Some(ext) = p.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !ext.eq_ignore_ascii_case("music") && !ext.eq_ignore_ascii_case("template") {
            continue;
        }
        if let Ok(meta) = std::fs::metadata(&p) {
            let mut hdr = [0_u8; 64];
            if let Ok(mut f) = std::fs::File::open(&p) {
                use std::io::Read;
                if f.read(&mut hdr).unwrap_or(0) >= 36 && is_music_studio_magic(&hdr) {
                    let vlen = read_u32_le(&hdr, 0x20).unwrap_or(0) as usize;
                    let version = if vlen > 0 && vlen <= 32 && 0x24 + vlen <= 64 {
                        decode_latin1(&hdr[0x24..0x24 + vlen])
                            .split('\r')
                            .last()
                            .unwrap_or("1.x")
                            .trim_matches('\r')
                            .to_string()
                    } else {
                        "1.x".to_string()
                    };
                    out.push(MusicPackageEntry {
                        file_name: p
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or_default()
                            .to_string(),
                        file_path: p.to_string_lossy().to_string(),
                        size_mb: (meta.len() as f32) / (1024.0 * 1024.0),
                        version,
                    });
                }
            }
        }
    }
}

pub fn list_music_packages_in_dir(dir: &Path) -> Vec<MusicPackageEntry> {
    let mut entries = Vec::new();
    scan_dir_for_packages(dir, 0, &mut entries);
    entries.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    entries
}

pub fn list_all_music_packages() -> Vec<MusicPackageEntry> {
    let mut raw = Vec::new();
    scan_dir_for_packages(Path::new(USER_DOCUMENTS_RHYTHMS_DIR), 1, &mut raw);
    scan_dir_for_packages(Path::new(USER_DOCUMENTS_KEYBOARDS_DIR), 2, &mut raw);
    scan_dir_for_packages(Path::new(DEFAULT_MUSIC_STUDIO_DIR), 0, &mut raw);

    let mut seen_names = std::collections::HashSet::new();
    let mut deduped = Vec::with_capacity(raw.len());
    for entry in raw {
        if seen_names.insert(entry.file_name.to_lowercase()) {
            deduped.push(entry);
        }
    }
    deduped.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    deduped
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordQuality {
    Major,
    Minor,
    Dom7,
    Min7,
    Maj7,
    Dim,
    Sus4,
}

impl ChordQuality {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChordQuality::Major => "Major",
            ChordQuality::Minor => "Minor",
            ChordQuality::Dom7 => "Dom7",
            ChordQuality::Min7 => "Min7",
            ChordQuality::Maj7 => "Maj7",
            ChordQuality::Dim => "Dim",
            ChordQuality::Sus4 => "Sus4",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "minor" | "m" | "min" => ChordQuality::Minor,
            "dom7" | "7" => ChordQuality::Dom7,
            "min7" | "m7" => ChordQuality::Min7,
            "maj7" => ChordQuality::Maj7,
            "dim" => ChordQuality::Dim,
            "sus4" | "sus" => ChordQuality::Sus4,
            _ => ChordQuality::Major,
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            ChordQuality::Major => "",
            ChordQuality::Minor => "m",
            ChordQuality::Dom7 => "7",
            ChordQuality::Min7 => "m7",
            ChordQuality::Maj7 => "maj7",
            ChordQuality::Dim => "dim",
            ChordQuality::Sus4 => "sus4",
        }
    }
}

pub fn format_chord_label(root: u8, quality: ChordQuality) -> String {
    format!("{}{}", NOTE_NAMES[(root % 12) as usize], quality.suffix())
}

pub fn detect_chord_from_notes(notes: &[u8]) -> Option<(u8, ChordQuality, String)> {
    if notes.is_empty() {
        return None;
    }
    let mut sorted = notes.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let lowest_pc = sorted[0] % 12;

    if sorted.len() == 1 {
        let q = ChordQuality::Major;
        return Some((lowest_pc, q, format_chord_label(lowest_pc, q)));
    }

    let pcs: Vec<u8> = {
        let mut p: Vec<u8> = sorted.iter().map(|&n| n % 12).collect();
        p.sort_unstable();
        p.dedup();
        p
    };

    if pcs.len() == 2 {
        let interval = (pcs[1] + 12 - pcs[0]) % 12;
        let q = match interval {
            3 => ChordQuality::Minor,
            10 => ChordQuality::Dom7,
            5 => ChordQuality::Sus4,
            _ => ChordQuality::Major,
        };
        let root = pcs[0];
        return Some((root, q, format_chord_label(root, q)));
    }

    let templates: &[(ChordQuality, &[u8])] = &[
        (ChordQuality::Minor, &[0, 3, 7]),
        (ChordQuality::Major, &[0, 4, 7]),
        (ChordQuality::Dom7, &[0, 4, 7, 10]),
        (ChordQuality::Min7, &[0, 3, 7, 10]),
        (ChordQuality::Maj7, &[0, 4, 7, 11]),
        (ChordQuality::Dim, &[0, 3, 6]),
        (ChordQuality::Sus4, &[0, 5, 7]),
    ];

    let mut best: Option<(i32, u8, ChordQuality)> = None;
    for &(q, tpl) in templates {
        for &cand_root in &pcs {
            let mut matched = 0_i32;
            for &iv in tpl {
                if pcs.contains(&((cand_root + iv) % 12)) {
                    matched += 1;
                }
            }
            if matched == tpl.len() as i32 {
                let mut score = matched * 10;
                if cand_root == lowest_pc {
                    score += 5; // Prefer root position over inversions
                }
                if best.map(|b| score > b.0).unwrap_or(true) {
                    best = Some((score, cand_root, q));
                }
            }
        }
    }

    if let Some((_, root, q)) = best {
        return Some((root, q, format_chord_label(root, q)));
    }

    // Check root from lowest pitch class + 3rd interval
    let q = if pcs.contains(&((lowest_pc + 3) % 12)) {
        ChordQuality::Minor
    } else if pcs.contains(&((lowest_pc + 10) % 12)) {
        ChordQuality::Dom7
    } else {
        ChordQuality::Major
    };
    Some((lowest_pc, q, format_chord_label(lowest_pc, q)))
}

#[derive(Debug, Clone)]
struct SamplerVoice {
    pcm_l: Arc<Vec<f32>>,
    pcm_r: Arc<Vec<f32>>,
    pos: f64,
    phase_inc: f64,
    loop_start: usize,
    loop_end: usize,
    loop_enabled: bool,
    loop_mode: u8,
    loop_dir: i8,
    gain: f32,
    pan_l: f32,
    pan_r: f32,
    env: f32,
    attack_pos: u16,
    releasing: bool,
    remaining_samples: Option<usize>,
    trigger_key: u8,
    from_rhythm: bool,
}

const MAX_SAMPLER_VOICES: usize = 192;

#[derive(Debug, Clone)]
struct PendingTrigger {
    inst_idx: usize,
    midi_note: u8,
    velocity: u8,
    vol: f32,
    pan: i32,
    fine_tune: i32,
    dur_samples: Option<usize>,
}

#[inline]
fn clean_soft_limit(x: f32) -> f32 {
    if !x.is_finite() {
        return 0.0;
    }
    let a = x.abs();
    if a <= 0.88 {
        x
    } else {
        x.signum() * (0.88 + 0.115 * ((a - 0.88) / 0.12).tanh())
    }
}

fn transpose_note_for_chord(
    note: u8,
    detected_chord_root: u8,
    detected_chord_quality: ChordQuality,
) -> u8 {
    // Tracks inside `.Music` are authored against a C Major reference (pitch class 0 = C)
    let mut root_shift = detected_chord_root as i16;
    if root_shift > 6 {
        root_shift -= 12;
    }
    let pc = (note % 12) as i16;
    let chord_iv = match detected_chord_quality {
        ChordQuality::Minor | ChordQuality::Min7 => match pc {
            4 => 3,   // E -> Eb
            11 => 10, // B -> Bb
            _ => pc,
        },
        ChordQuality::Dom7 => match pc {
            11 => 10, // B -> Bb
            _ => pc,
        },
        ChordQuality::Dim => match pc {
            4 => 3,
            7 => 6,
            _ => pc,
        },
        ChordQuality::Sus4 => match pc {
            4 => 5,
            _ => pc,
        },
        ChordQuality::Major | ChordQuality::Maj7 => pc,
    };
    let adjusted = note as i16 + (chord_iv - pc) + root_shift;
    adjusted.clamp(24, 108) as u8
}

#[allow(clippy::too_many_arguments)]
fn collect_section_events(
    sec: &RhythmSection,
    instruments: &[LoadedInstrument],
    track_volumes: &[f32; 8],
    detected_chord_root: u8,
    detected_chord_quality: ChordQuality,
    bpm: f32,
    sr: f32,
    win_start: f64,
    win_end: f64,
    out: &mut Vec<PendingTrigger>,
) {
    if win_end <= win_start {
        return;
    }
    for track in &sec.tracks {
        let inst_is_drum = instruments
            .get(track.instrument_index)
            .map(|i| i.is_drumkit)
            .unwrap_or(false);
        let is_drum = track.is_drum || inst_is_drum;
        let vol = track.volume * track_volumes[track.track_slot.min(7)];
        if vol <= 0.002 {
            continue;
        }
        for ev in &track.events {
            let t = ev.start_tick as f64;
            if t >= win_start && t < win_end {
                let target_note = if is_drum {
                    ev.midi_note
                } else {
                    let shifted = (ev.midi_note as i16
                        + track.key_move as i16
                        + (track.octave_shift * 12) as i16)
                        .clamp(12, 115) as u8;
                    transpose_note_for_chord(shifted, detected_chord_root, detected_chord_quality)
                };
                let dur_samples = if is_drum {
                    None
                } else {
                    let dur_sec =
                        (ev.duration_ticks.max(30) as f64 / 500.0) * (60.0 / bpm.max(40.0) as f64);
                    Some(((dur_sec * sr as f64) as usize).max(256))
                };
                out.push(PendingTrigger {
                    inst_idx: track.instrument_index,
                    midi_note: target_note,
                    velocity: ev.velocity,
                    vol,
                    pan: track.pan,
                    fine_tune: track.fine_tune,
                    dur_samples,
                });
            }
        }
    }
}

pub struct MusicStudioArrangerEngine {
    sr: f32,
    pub package: Option<Arc<LoadedMusicPackage>>,
    pub active_style_index: usize,
    pub active_section: String,
    pub last_main_variation: String,
    pub pending_section: Option<String>,
    pub upper1_instrument: usize,
    pub upper2_instrument: Option<usize>,
    pub lower_instrument: Option<usize>,
    pub split_key: u8,
    pub split_enabled: bool,
    pub chord_scan_lower: bool,
    pub chord_scan_upper: bool,
    pub upper_octave: i8,
    pub transpose: i8,
    pub bpm: f32,
    pub tempo_locked: bool,
    pub rhythm_playing: bool,
    pub sync_start: bool,
    pub auto_fill: bool,
    pub detected_chord_root: u8,
    pub detected_chord_quality: ChordQuality,
    pub detected_chord_label: String,
    pub track_volumes: [f32; 8],
    pub master_gain: f32,
    current_tick: f64,
    held_keys: [bool; 128],
    held_lower_notes: Vec<u8>,
    held_upper_notes: Vec<u8>,
    voices: Vec<SamplerVoice>,
}

impl MusicStudioArrangerEngine {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sr: sample_rate.max(8000.0),
            package: None,
            active_style_index: 0,
            active_section: "Variation 1".to_string(),
            last_main_variation: "Variation 1".to_string(),
            pending_section: None,
            upper1_instrument: 0,
            upper2_instrument: None,
            lower_instrument: None,
            split_key: 53,
            split_enabled: true,
            chord_scan_lower: true,
            chord_scan_upper: false,
            upper_octave: 0,
            transpose: 0,
            bpm: 120.0,
            tempo_locked: false,
            rhythm_playing: false,
            sync_start: true,
            auto_fill: true,
            detected_chord_root: 0,
            detected_chord_quality: ChordQuality::Major,
            detected_chord_label: "C".to_string(),
            track_volumes: [1.0; 8],
            master_gain: 0.85,
            current_tick: 0.0,
            held_keys: [false; 128],
            held_lower_notes: Vec::new(),
            held_upper_notes: Vec::new(),
            voices: Vec::with_capacity(MAX_SAMPLER_VOICES),
        }
    }

    pub fn has_loaded_package(&self) -> bool {
        self.package.is_some()
    }

    pub fn load_package(&mut self, pkg: Arc<LoadedMusicPackage>) {
        self.all_notes_off();
        self.rhythm_playing = false;
        self.current_tick = 0.0;
        self.pending_section = None;
        self.tempo_locked = false;

        // Auto-pick the best playable melodic lead instrument (prefer Accordion/Bandoneon/Sanfona/Lead/Guitar/Strings with samples)
        let mut playable: Vec<&LoadedInstrument> = pkg
            .instruments
            .iter()
            .filter(|inst| {
                !inst.is_drumkit
                    && !inst.zones.is_empty()
                    && inst.zones.iter().any(|z| !z.pcm_l.is_empty())
            })
            .collect();

        playable.sort_by_key(|inst| {
            let up = inst.name.to_uppercase();
            let max_zone_gain = inst
                .zones
                .iter()
                .filter(|z| !z.pcm_l.is_empty())
                .fold(0.0_f32, |m, z| m.max(z.gain));
            let effective_gain = inst.volume * max_zone_gain;
            let is_very_quiet = effective_gain < 0.20;
            let is_base = up.contains("BASE");
            let tier = if up.contains("BANDONEON") || up.contains("ACORD") || up.contains("SANFONA")
            {
                0
            } else if up.contains("PIANO") || up.contains("LEAD") || up.contains("ORGAN") {
                1
            } else if up.contains("GTR") || up.contains("GUITAR") || up.contains("VIOL") {
                2
            } else if up.contains("STRING") || up.contains("PAD") {
                3
            } else if up.contains("BASS") || up.contains("BAIXO") {
                5
            } else {
                4
            };
            (is_very_quiet, is_base, tier)
        });

        self.upper1_instrument = playable.first().map(|i| i.index).unwrap_or(0);
        self.upper2_instrument = None;
        // Default Lower voice to None (Chord Scan only, silent left-hand keys unless user assigns a timbre)
        self.lower_instrument = None;

        let style_idx = 0;
        self.active_style_index = style_idx;
        if let Some(style) = pkg.styles.get(style_idx) {
            self.bpm = style.bpm;
            if let Some(sec) = style
                .sections
                .iter()
                .find(|s| s.name == "Variation 1")
                .or_else(|| style.sections.first())
            {
                self.active_section = sec.name.clone();
                if sec.name.starts_with("Variation") {
                    self.last_main_variation = sec.name.clone();
                }
            }
        }
        self.package = Some(pkg);
    }

    pub fn summary(&self) -> MusicPackageSummary {
        let Some(ref pkg) = self.package else {
            return MusicPackageSummary::default();
        };
        let sec_len = self
            .active_section_ref()
            .map(|s| s.length_ticks.max(1) as f64)
            .unwrap_or(4000.0);
        MusicPackageSummary {
            loaded: true,
            file_name: pkg.file_name.clone(),
            file_path: pkg.file_path.clone(),
            version: pkg.version.clone(),
            instruments: pkg
                .instruments
                .iter()
                .map(|i| MusicInstrumentSummary {
                    index: i.index,
                    name: i.name.clone(),
                    zone_count: i.zones.len(),
                    has_samples: i.zones.iter().any(|z| !z.pcm_l.is_empty()),
                    is_drumkit: i.is_drumkit,
                })
                .collect(),
            styles: pkg
                .styles
                .iter()
                .map(|s| MusicStyleSummary {
                    id: s.id,
                    name: s.name.clone(),
                    category: s.category.clone(),
                    sections: s.sections.iter().map(|sec| sec.name.clone()).collect(),
                })
                .collect(),
            active_style_index: self.active_style_index,
            active_section: self.active_section.clone(),
            pending_section: self.pending_section.clone(),
            upper1_instrument: self.upper1_instrument,
            upper2_instrument: self.upper2_instrument,
            lower_instrument: self.lower_instrument,
            split_key: self.split_key,
            split_enabled: self.split_enabled,
            chord_scan_lower: self.chord_scan_lower,
            chord_scan_upper: self.chord_scan_upper,
            upper_octave: self.upper_octave,
            transpose: self.transpose,
            bpm: self.bpm,
            tempo_locked: self.tempo_locked,
            rhythm_playing: self.rhythm_playing,
            sync_start: self.sync_start,
            auto_fill: self.auto_fill,
            detected_chord_root: self.detected_chord_root,
            detected_chord_quality: self.detected_chord_quality.as_str().to_string(),
            detected_chord: self.detected_chord_label.clone(),
            track_volumes: self.track_volumes,
            beat_position: (self.current_tick / sec_len).clamp(0.0, 1.0) as f32,
        }
    }

    fn active_section_ref(&self) -> Option<&RhythmSection> {
        let pkg = self.package.as_ref()?;
        let style = pkg.styles.get(self.active_style_index)?;
        style
            .sections
            .iter()
            .find(|s| s.name == self.active_section)
            .or_else(|| style.sections.first())
    }

    pub fn set_section(&mut self, section_name: &str, immediate: bool) {
        let Some(ref pkg) = self.package else {
            return;
        };
        let Some(style) = pkg.styles.get(self.active_style_index) else {
            return;
        };
        if !style.sections.iter().any(|s| s.name == section_name) {
            return;
        }
        if section_name.starts_with("Variation") {
            self.last_main_variation = section_name.to_string();
        }
        if !self.rhythm_playing || immediate {
            self.active_section = section_name.to_string();
            self.pending_section = None;
            self.current_tick = 0.0;
        } else {
            if section_name == self.active_section {
                self.pending_section = None;
                return;
            }
            if self.auto_fill
                && section_name.starts_with("Variation")
                && !self.active_section.starts_with("Fill")
            {
                if let Some(suffix) = section_name.strip_prefix("Variation ") {
                    let fill_name = format!("Fill {}", suffix);
                    if style.sections.iter().any(|s| s.name == fill_name) {
                        self.active_section = fill_name;
                        self.pending_section = Some(section_name.to_string());
                        self.current_tick = 0.0;
                        return;
                    }
                }
            }
            self.pending_section = Some(section_name.to_string());
        }
    }

    pub fn set_transport(&mut self, playing: bool, sync_start: Option<bool>, bpm: Option<f32>) {
        if let Some(ss) = sync_start {
            self.sync_start = ss;
        }
        if let Some(b) = bpm {
            if b.is_finite() {
                self.bpm = b.clamp(40.0, 240.0);
            }
        }
        if playing && !self.rhythm_playing {
            self.rhythm_playing = true;
            self.current_tick = 0.0;
        } else if !playing && self.rhythm_playing {
            self.rhythm_playing = false;
            self.voices.retain(|v| !v.from_rhythm);
        }
    }

    pub fn set_tempo_locked(&mut self, locked: bool) {
        self.tempo_locked = locked;
    }

    pub fn set_chord(&mut self, root_pc: u8, quality: &str) {
        let root = root_pc % 12;
        let q = ChordQuality::from_str(quality);
        self.detected_chord_root = root;
        self.detected_chord_quality = q;
        self.detected_chord_label = format_chord_label(root, q);
    }

    pub fn set_style(&mut self, style_index: usize) {
        let Some(ref pkg) = self.package else {
            return;
        };
        if let Some(style) = pkg.styles.get(style_index) {
            self.active_style_index = style_index;
            if !self.tempo_locked {
                self.bpm = style.bpm;
            }
            if let Some(sec) = style
                .sections
                .iter()
                .find(|s| s.name == "Variation 1")
                .or_else(|| style.sections.first())
            {
                self.active_section = sec.name.clone();
                if sec.name.starts_with("Variation") {
                    self.last_main_variation = sec.name.clone();
                }
            }
            self.pending_section = None;
            self.current_tick = 0.0;
        }
    }

    pub fn set_keyboard_voice(
        &mut self,
        slot: &str,
        instrument_index: Option<usize>,
        split_key: Option<u8>,
        split_enabled: Option<bool>,
        chord_scan_lower: Option<bool>,
        chord_scan_upper: Option<bool>,
        upper_octave: Option<i8>,
        transpose: Option<i8>,
    ) {
        if let Some(sk) = split_key {
            self.split_key = sk.clamp(24, 96);
        }
        if let Some(se) = split_enabled {
            self.split_enabled = se;
        }
        if let Some(csl) = chord_scan_lower {
            self.chord_scan_lower = csl;
        }
        if let Some(csu) = chord_scan_upper {
            self.chord_scan_upper = csu;
        }
        if let Some(uo) = upper_octave {
            self.upper_octave = uo.clamp(-2, 2);
        }
        if let Some(tr) = transpose {
            self.transpose = tr.clamp(-12, 12);
        }
        let max_idx = self
            .package
            .as_ref()
            .map(|p| p.instruments.len())
            .unwrap_or(0);
        let valid = instrument_index.filter(|&i| i < max_idx);
        match slot {
            "upper1" => {
                if let Some(i) = valid {
                    self.upper1_instrument = i;
                }
            }
            "upper2" => {
                self.upper2_instrument = valid;
            }
            "lower" => {
                self.lower_instrument = valid;
            }
            _ => {}
        }
    }

    pub fn set_track_volume(&mut self, track_slot: usize, volume: f32) {
        if track_slot < 8 && volume.is_finite() {
            self.track_volumes[track_slot] = volume.clamp(0.0, 1.5);
        }
    }

    pub fn set_auto_fill(&mut self, auto_fill: bool) {
        self.auto_fill = auto_fill;
    }

    pub fn set_master_gain_db(&mut self, volume_db: f32) {
        if volume_db.is_finite() {
            self.master_gain = 10.0_f32.powf(volume_db.clamp(-60.0, 12.0) / 20.0) * 0.85;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn trigger_instrument_note(
        &mut self,
        inst_index: usize,
        midi_note: u8,
        trigger_key: u8,
        velocity: u8,
        gain_scale: f32,
        pan_offset: i32,
        tune_offset: i32,
        from_rhythm: bool,
        duration_samples: Option<usize>,
    ) {
        let Some(ref pkg) = self.package else {
            return;
        };
        let Some(inst) = pkg.instruments.get(inst_index) else {
            return;
        };

        let zone = if inst.is_drumkit {
            inst.zones
                .iter()
                .find(|z| !z.pcm_l.is_empty() && z.root_key == midi_note)
        } else {
            inst.zones
                .iter()
                .find(|z| !z.pcm_l.is_empty() && midi_note >= z.key_low && midi_note <= z.key_high)
                .or_else(|| {
                    inst.zones
                        .iter()
                        .filter(|z| !z.pcm_l.is_empty())
                        .min_by_key(|z| (z.root_key as i16 - midi_note as i16).abs())
                })
        };
        let Some(z) = zone else {
            return;
        };

        let tune_semitones = (z.fine_tune + tune_offset) as f64 / 1000.0;
        let semitones = if inst.is_drumkit {
            tune_semitones
        } else {
            (midi_note as f64 - z.root_key as f64) + tune_semitones
        };
        let phase_inc = (z.sample_rate as f64 / self.sr as f64) * 2.0_f64.powf(semitones / 12.0);
        let vel_gain = (velocity as f32 / 127.0).powf(0.85);
        let total_gain = vel_gain * inst.volume * z.gain * gain_scale;
        if total_gain <= 0.0005 {
            return;
        }

        let total_pan = (inst.pan + z.pan + pan_offset).clamp(-100, 100);
        let pan_l = ((100 - total_pan).clamp(0, 100) as f32) / 100.0;
        let pan_r = ((100 + total_pan).clamp(0, 100) as f32) / 100.0;

        if self.voices.len() >= MAX_SAMPLER_VOICES {
            if let Some(pos) = self
                .voices
                .iter()
                .enumerate()
                .filter(|(_, v)| v.releasing)
                .min_by(|(_, a), (_, b)| (a.env * a.gain).total_cmp(&(b.env * b.gain)))
                .map(|(idx, _)| idx)
                .or_else(|| self.voices.iter().position(|v| v.from_rhythm))
                .or(Some(0))
            {
                self.voices.swap_remove(pos);
            }
        }

        self.voices.push(SamplerVoice {
            pcm_l: Arc::clone(&z.pcm_l),
            pcm_r: Arc::clone(&z.pcm_r),
            pos: 0.0,
            phase_inc,
            loop_start: z.loop_start,
            loop_end: z.loop_end,
            loop_enabled: z.loop_enabled && !inst.is_drumkit,
            loop_mode: z.loop_mode,
            loop_dir: 1,
            gain: total_gain,
            pan_l,
            pan_r,
            env: 1.0,
            attack_pos: if inst.is_drumkit { 48 } else { 0 },
            releasing: false,
            remaining_samples: if inst.is_drumkit { None } else { duration_samples },
            trigger_key,
            from_rhythm,
        });
    }

    pub fn note_on(&mut self, note: u8, velocity: u8) {
        if note >= 128 || velocity == 0 {
            self.note_off(note, false);
            return;
        }
        self.held_keys[note as usize] = true;

        if self.split_enabled && note <= self.split_key {
            if !self.held_lower_notes.contains(&note) {
                self.held_lower_notes.push(note);
            }
            if self.chord_scan_lower {
                if let Some((root, q, label)) = detect_chord_from_notes(&self.held_lower_notes) {
                    self.detected_chord_root = root;
                    self.detected_chord_quality = q;
                    self.detected_chord_label = label;
                }
                if self.sync_start && !self.rhythm_playing {
                    self.rhythm_playing = true;
                    self.current_tick = 0.0;
                }
            }
            if let Some(low_inst) = self.lower_instrument {
                let played = (note as i16 + self.transpose as i16).clamp(0, 127) as u8;
                self.trigger_instrument_note(low_inst, played, note, velocity, 0.65, 0, 0, false, None);
            }
            return;
        }

        if !self.held_upper_notes.contains(&note) {
            self.held_upper_notes.push(note);
        }
        if self.chord_scan_upper
            && (self.held_upper_notes.len() >= 3 || !self.chord_scan_lower || !self.split_enabled)
        {
            if let Some((root, q, label)) = detect_chord_from_notes(&self.held_upper_notes) {
                self.detected_chord_root = root;
                self.detected_chord_quality = q;
                self.detected_chord_label = label;
            }
            if self.sync_start && !self.rhythm_playing {
                self.rhythm_playing = true;
                self.current_tick = 0.0;
            }
        }

        let played_upper = (note as i16 + (self.upper_octave as i16 * 12) + self.transpose as i16)
            .clamp(0, 127) as u8;
        let u1 = self.upper1_instrument;
        self.trigger_instrument_note(u1, played_upper, note, velocity, 0.85, 0, 0, false, None);
        if let Some(u2) = self.upper2_instrument {
            if u2 != u1 {
                self.trigger_instrument_note(u2, played_upper, note, velocity, 0.68, 0, 0, false, None);
            }
        }
    }

    pub fn note_off(&mut self, note: u8, sustain_down: bool) {
        if note >= 128 {
            return;
        }
        self.held_keys[note as usize] = false;
        self.held_lower_notes.retain(|&n| n != note);
        self.held_upper_notes.retain(|&n| n != note);

        if sustain_down {
            return;
        }
        for v in &mut self.voices {
            if !v.from_rhythm && v.trigger_key == note {
                v.releasing = true;
            }
        }
    }

    pub fn release_unsustained(&mut self) {
        for v in &mut self.voices {
            if !v.from_rhythm && !self.held_keys[v.trigger_key as usize] {
                v.releasing = true;
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        self.held_keys = [false; 128];
        self.held_lower_notes.clear();
        self.held_upper_notes.clear();
        for v in &mut self.voices {
            v.releasing = true;
        }
    }

    pub fn active_notes(&self) -> Vec<u8> {
        let mut notes = Vec::new();
        for (n, &on) in self.held_keys.iter().enumerate() {
            if on {
                notes.push(n as u8);
            }
        }
        notes
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.len()
    }

    pub fn process_block(&mut self, out_l: &mut [f32], out_r: &mut [f32]) {
        out_l.fill(0.0);
        out_r.fill(0.0);
        let frames = out_l.len().min(out_r.len());
        if frames == 0 {
            return;
        }

        // Advance rhythm sequencer if active (Music Studio clock = 500 ticks per quarter-note beat)
        if self.rhythm_playing && self.package.is_some() {
            let ticks_per_block =
                (frames as f64) * (self.bpm as f64 / 60.0) * 500.0 / (self.sr as f64);
            let prev_tick = self.current_tick;
            let next_tick = prev_tick + ticks_per_block;

            let mut to_trigger: Vec<PendingTrigger> = Vec::new();
            let mut transition_next: Option<String> = None;
            let mut stop_at_end = false;
            let mut wrap_tick = 0.0_f64;
            let mut wrapped = false;

            if let Some(pkg) = self.package.as_ref() {
                if let Some(style) = pkg.styles.get(self.active_style_index) {
                    if let Some(sec) = style
                        .sections
                        .iter()
                        .find(|s| s.name == self.active_section)
                        .or_else(|| style.sections.first())
                    {
                        let sec_len = sec.length_ticks.max(500) as f64;
                        wrapped = next_tick >= sec_len;
                        wrap_tick = if wrapped { next_tick - sec_len } else { 0.0 };

                        if wrapped {
                            collect_section_events(
                                sec,
                                &pkg.instruments,
                                &self.track_volumes,
                                self.detected_chord_root,
                                self.detected_chord_quality,
                                self.bpm,
                                self.sr,
                                prev_tick,
                                sec_len,
                                &mut to_trigger,
                            );
                            if let Some(ref pend) = self.pending_section {
                                transition_next = Some(pend.clone());
                            } else if !sec.is_loop {
                                if sec.jump_to == -2 {
                                    transition_next = Some(self.last_main_variation.clone());
                                } else if sec.jump_to > 0 {
                                    let target_id = sec.jump_to as u32;
                                    let next_name = style
                                        .sections
                                        .iter()
                                        .find(|s| s.id == target_id)
                                        .map(|s| s.name.clone())
                                        .unwrap_or_else(|| self.last_main_variation.clone());
                                    transition_next = Some(next_name);
                                } else {
                                    stop_at_end = true;
                                }
                            } else if sec.name.starts_with("Intro")
                                || sec.name.starts_with("Fill")
                                || sec.name == "Break"
                            {
                                transition_next = Some(self.last_main_variation.clone());
                            } else if sec.name.starts_with("Ending") {
                                stop_at_end = true;
                            }

                            if !stop_at_end {
                                let next_sec_ref = if let Some(ref next_name) = transition_next {
                                    style
                                        .sections
                                        .iter()
                                        .find(|s| &s.name == next_name)
                                        .unwrap_or(sec)
                                } else {
                                    sec
                                };
                                collect_section_events(
                                    next_sec_ref,
                                    &pkg.instruments,
                                    &self.track_volumes,
                                    self.detected_chord_root,
                                    self.detected_chord_quality,
                                    self.bpm,
                                    self.sr,
                                    0.0,
                                    wrap_tick,
                                    &mut to_trigger,
                                );
                            }
                        } else {
                            collect_section_events(
                                sec,
                                &pkg.instruments,
                                &self.track_volumes,
                                self.detected_chord_root,
                                self.detected_chord_quality,
                                self.bpm,
                                self.sr,
                                prev_tick,
                                next_tick,
                                &mut to_trigger,
                            );
                        }
                    }
                }
            }

            if stop_at_end {
                self.rhythm_playing = false;
                self.pending_section = None;
                self.current_tick = 0.0;
            } else {
                if let Some(next_sec) = transition_next {
                    self.active_section = next_sec;
                    self.pending_section = None;
                }
                self.current_tick = if wrapped { wrap_tick } else { next_tick };
            }

            for trig in to_trigger {
                self.trigger_instrument_note(
                    trig.inst_idx,
                    trig.midi_note,
                    trig.midi_note,
                    trig.velocity,
                    trig.vol * 0.55,
                    trig.pan,
                    trig.fine_tune,
                    true,
                    trig.dur_samples,
                );
            }
        }

        // Render all active polyphonic sampler voices
        let release_step = 1.0 / (0.08 * self.sr).max(64.0);
        let master = self.master_gain;
        let mut i = 0;
        while i < self.voices.len() {
            let v = &mut self.voices[i];
            let pcm_len = v.pcm_l.len();
            if pcm_len < 2 {
                self.voices.swap_remove(i);
                continue;
            }
            let mut alive = true;
            for f in 0..frames {
                if let Some(ref mut rem) = v.remaining_samples {
                    if *rem == 0 {
                        v.releasing = true;
                    } else {
                        *rem -= 1;
                    }
                }
                if v.releasing {
                    v.env -= release_step;
                    if v.env <= 0.0005 {
                        alive = false;
                        break;
                    }
                }

                let atk = if v.attack_pos < 48 {
                    v.attack_pos += 1;
                    v.attack_pos as f32 / 48.0
                } else {
                    1.0
                };

                if v.loop_enabled && v.loop_end > v.loop_start + 8 {
                    if v.loop_mode == 3 {
                        if v.loop_dir > 0 && v.pos >= v.loop_end as f64 {
                            v.pos = ((v.loop_end as f64) - (v.pos - v.loop_end as f64))
                                .clamp(v.loop_start as f64, (pcm_len - 2) as f64);
                            v.loop_dir = -1;
                        } else if v.loop_dir < 0 && v.pos <= v.loop_start as f64 {
                            v.pos = ((v.loop_start as f64) + (v.loop_start as f64 - v.pos))
                                .clamp(0.0, v.loop_end as f64);
                            v.loop_dir = 1;
                        }
                    } else if v.pos >= v.loop_end as f64 {
                        let span = (v.loop_end - v.loop_start) as f64;
                        v.pos = v.loop_start as f64 + ((v.pos - v.loop_start as f64) % span);
                    }
                }

                let idx0 = v.pos as usize;
                if idx0 + 1 >= pcm_len {
                    alive = false;
                    break;
                }
                let frac = (v.pos - idx0 as f64) as f32;
                let s0_l = v.pcm_l[idx0];
                let s1_l = v.pcm_l[idx0 + 1];
                let s0_r = v.pcm_r[idx0];
                let s1_r = v.pcm_r[idx0 + 1];
                let mut l = s0_l + (s1_l - s0_l) * frac;
                let mut r = s0_r + (s1_r - s0_r) * frac;

                // Subtle 24-sample micro-crossfade before forward loop wrap when pre-loop audio exists
                if v.loop_enabled && v.loop_mode == 2 && v.loop_end > v.loop_start + 32 {
                    let xfade_len = 24_usize.min((v.loop_end - v.loop_start) / 4);
                    if xfade_len >= 4
                        && v.loop_start >= xfade_len
                        && v.pos + (xfade_len as f64) >= v.loop_end as f64
                    {
                        let dist = v.pos + (xfade_len as f64) - (v.loop_end as f64);
                        let alpha = (dist / xfade_len as f64).clamp(0.0, 1.0) as f32;
                        let pre_pos = (v.loop_start - xfade_len) as f64 + dist;
                        let pidx = pre_pos as usize;
                        if pidx + 1 < pcm_len {
                            let pfrac = (pre_pos - pidx as f64) as f32;
                            let pl = v.pcm_l[pidx] + (v.pcm_l[pidx + 1] - v.pcm_l[pidx]) * pfrac;
                            let pr = v.pcm_r[pidx] + (v.pcm_r[pidx + 1] - v.pcm_r[pidx]) * pfrac;
                            l = l * (1.0 - alpha) + pl * alpha;
                            r = r * (1.0 - alpha) + pr * alpha;
                        }
                    }
                }

                let g = v.gain * v.env * atk * master;
                out_l[f] += l * g * v.pan_l;
                out_r[f] += r * g * v.pan_r;

                v.pos += (v.loop_dir as f64) * v.phase_inc;
            }
            if !alive {
                self.voices.swap_remove(i);
            } else {
                i += 1;
            }
        }

        for f in 0..frames {
            out_l[f] = clean_soft_limit(out_l[f]);
            out_r[f] = clean_soft_limit(out_r[f]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chord_detection_single_and_triad() {
        let (_, q1, l1) = detect_chord_from_notes(&[48]).expect("C major 1-finger");
        assert_eq!(q1, ChordQuality::Major);
        assert_eq!(l1, "C");

        let (r2, q2, l2) = detect_chord_from_notes(&[45, 48, 52]).expect("Am triad");
        assert_eq!(r2, 9);
        assert_eq!(q2, ChordQuality::Minor);
        assert_eq!(l2, "Am");

        let (_, q3, l3) = detect_chord_from_notes(&[43, 47, 50, 53]).expect("G7");
        assert_eq!(q3, ChordQuality::Dom7);
        assert_eq!(l3, "G7");
    }

    #[test]
    fn test_parse_real_music_packages_across_versions() {
        let dir = Path::new(DEFAULT_MUSIC_STUDIO_DIR);
        if !dir.exists() {
            return;
        }
        let packs = list_music_packages_in_dir(dir);
        assert!(!packs.is_empty(), "expected .Music packages in Wine dir");

        for name in [
            "Josiran_Pisadinha_GR.Music",
            "Arrasta_Pe_Forro_GR.Music",
            "Bonde_do_Brasil_TM.Music",
            "Calcinha_Preta_ORG2024.Music",
            "Avioes_do_Forro_Xand_Aviao_Inquilina.Music",
            "Manu_Batidao_TM.Music",
            "Washington_Brasileiro_Muel_Teclas.Music",
        ] {
            let path = dir.join(name);
            if !path.exists() {
                continue;
            }
            let pkg = parse_music_file(&path).unwrap_or_else(|e| panic!("{}: {}", name, e));
            assert!(
                !pkg.instruments.is_empty(),
                "{} should contain embedded instruments",
                name
            );
            let total_pcm_samples: usize = pkg
                .instruments
                .iter()
                .flat_map(|i| i.zones.iter())
                .map(|z| z.pcm_l.len())
                .sum();
            assert!(
                total_pcm_samples > 10_000,
                "{} should decode PCM audio samples (got {})",
                name,
                total_pcm_samples
            );
            assert!(
                !pkg.styles.is_empty(),
                "{} should extract at least one rhythm style",
                name
            );
            let total_events: usize = pkg
                .styles
                .iter()
                .flat_map(|s| s.sections.iter())
                .flat_map(|sec| sec.tracks.iter())
                .map(|t| t.events.len())
                .sum();
            assert!(
                total_events > 20,
                "{} should extract rhythm note events (got {})",
                name,
                total_events
            );
        }
    }

    #[test]
    fn test_arranger_playback_split_chord_scan_and_transposition_audio() {
        let path = Path::new(DEFAULT_MUSIC_STUDIO_DIR).join("Josiran_Pisadinha_GR.Music");
        if !path.exists() {
            return;
        }
        let pkg = Arc::new(parse_music_file(&path).expect("parse Josiran"));
        // Verify instrument table alignment (Drum=2 -> BTR JOSIRAN PISADINHA, Bass=3 -> BAIXO SLAP)
        assert!(pkg.instruments[2].name.contains("BTR"), "inst 2 should be BTR drumkit, got {}", pkg.instruments[2].name);
        assert!(pkg.instruments[3].name.contains("BAIXO"), "inst 3 should be BAIXO, got {}", pkg.instruments[3].name);

        let mut engine = MusicStudioArrangerEngine::new(48000.0);
        engine.load_package(pkg);
        assert!(engine.has_loaded_package());
        assert_eq!(engine.split_key, 53);
        assert!(engine.lower_instrument.is_none());

        // Trigger left-hand Am chord below split_key=53 -> changes chord + sync-starts rhythm, NO Lower voice triggered
        engine.note_on(45, 110); // A2
        engine.note_on(48, 108); // C3
        engine.note_on(52, 105); // E3 -> Am triad
        assert!(engine.rhythm_playing);
        assert_eq!(engine.detected_chord_label, "Am");
        assert_eq!(engine.active_voice_count(), 0);

        // Toggle SCAN UP and play upper triad (F4, A4, C5) -> should update chord to F
        engine.set_keyboard_voice("upper1", None, None, None, None, Some(true), Some(1), None);
        engine.note_on(65, 110); // F4
        engine.note_on(69, 112); // A4
        engine.note_on(72, 110); // C5
        assert_eq!(engine.detected_chord_label, "F");

        let mut l = vec![0.0_f32; 4096];
        let mut r = vec![0.0_f32; 4096];
        engine.process_block(&mut l, &mut r);
        let peak = l.iter().chain(r.iter()).fold(0.0_f32, |m, &s| m.max(s.abs()));
        assert!(peak > 0.02 && peak <= 1.0, "expected clean non-clipped peak, got {}", peak);
    }

    #[test]
    fn test_all_user_documents_forro_music_files_play_clean_audio() {
        let dir = Path::new(USER_DOCUMENTS_RHYTHMS_DIR);
        if !dir.exists() {
            return;
        }
        let packs = list_music_packages_in_dir(dir);
        assert!(!packs.is_empty(), "expected .Music files in {}", USER_DOCUMENTS_RHYTHMS_DIR);

        let mut failures = Vec::new();
        let mut l = vec![0.0_f32; 4096];
        let mut r = vec![0.0_f32; 4096];

        for entry in &packs {
            let pkg = match parse_music_file(Path::new(&entry.file_path)) {
                Ok(p) => Arc::new(p),
                Err(e) => {
                    failures.push(format!("{}: parse error {}", entry.file_name, e));
                    continue;
                }
            };
            if pkg.styles.is_empty() {
                failures.push(format!("{}: 0 styles extracted", entry.file_name));
                continue;
            }
            let mut engine = MusicStudioArrangerEngine::new(48000.0);
            engine.load_package(Arc::clone(&pkg));
            engine.set_transport(true, None, None);

            // Render ~0.68s (8 blocks x 4096 frames @ 48kHz) to cover downbeat & accompaniment
            let mut max_rhythm_peak = 0.0_f32;
            let mut non_finite = false;
            for _ in 0..8 {
                engine.process_block(&mut l, &mut r);
                for (&sl, &sr) in l.iter().zip(r.iter()) {
                    if !sl.is_finite() || !sr.is_finite() {
                        non_finite = true;
                    }
                    max_rhythm_peak = max_rhythm_peak.max(sl.abs()).max(sr.abs());
                }
            }

            // Also verify Upper1 solo key (E4) produces sound
            engine.set_transport(false, None, None);
            engine.all_notes_off();
            engine.note_on(64, 110);
            engine.process_block(&mut l, &mut r);
            let solo_peak = l.iter().chain(r.iter()).fold(0.0_f32, |m, &s| m.max(s.abs()));

            if non_finite || max_rhythm_peak <= 0.01 || max_rhythm_peak > 1.0 || solo_peak <= 0.005 {
                failures.push(format!(
                    "{}: rhythm_peak={:.4}, solo_peak={:.4}, non_finite={}",
                    entry.file_name, max_rhythm_peak, solo_peak, non_finite
                ));
            }
        }

        assert!(
            failures.is_empty(),
            "Some .Music files failed audio playback ({}/{}):\n{}",
            failures.len(),
            packs.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn test_clock_is_500_ticks_per_beat() {
        let pcm = Arc::new(vec![0.25_f32; 4000]);
        let pkg = Arc::new(LoadedMusicPackage {
            file_name: "clock_test.Music".to_string(),
            file_path: "test://clock".to_string(),
            version: "1.4.4".to_string(),
            instruments: vec![LoadedInstrument {
                index: 0,
                name: "Click".to_string(),
                volume: 1.0,
                pan: 0,
                inst_type: 1,
                is_drumkit: true,
                zones: vec![SampleZone {
                    root_key: 36,
                    key_low: 36,
                    key_high: 36,
                    name: "Kick".to_string(),
                    sample_rate: 48000,
                    channels: 1,
                    loop_start: 0,
                    loop_end: 0,
                    loop_enabled: false,
                    loop_mode: 0,
                    gain: 1.0,
                    pan: 0,
                    fine_tune: 0,
                    pool_sid: 0,
                    pcm_l: Arc::clone(&pcm),
                    pcm_r: Arc::clone(&pcm),
                }],
            }],
            styles: vec![RhythmStyle {
                id: 1,
                name: "Test 4/4".to_string(),
                category: "Test".to_string(),
                bpm: 120.0,
                sections: vec![RhythmSection {
                    id: 2,
                    name: "Variation 1".to_string(),
                    length_ticks: 2000, // 4 beats at 500 ticks/beat = exactly 2.0s @ 120 BPM
                    is_loop: true,
                    jump_to: 0,
                    tracks: vec![RhythmTrack {
                        track_type: "Drum".to_string(),
                        track_slot: 0,
                        instrument_index: 0,
                        volume: 1.0,
                        pan: 0,
                        fine_tune: 0,
                        key_move: 0,
                        octave_shift: 0,
                        is_drum: true,
                        events: vec![RhythmNoteEvent {
                            midi_note: 36,
                            velocity: 100,
                            start_tick: 0,
                            duration_ticks: 100,
                        }],
                    }],
                }],
            }],
        });

        let mut engine = MusicStudioArrangerEngine::new(48000.0);
        engine.load_package(pkg);
        engine.set_transport(true, None, Some(120.0));

        // After 1.0s (48000 frames), 1000 ticks elapsed = beat_position ~0.5 (halfway through 2s bar!)
        let mut l = vec![0.0_f32; 480];
        let mut r = vec![0.0_f32; 480];
        for _ in 0..100 {
            engine.process_block(&mut l, &mut r);
        }
        assert!(
            (engine.summary().beat_position - 0.5).abs() < 0.02,
            "At 120 BPM, 1.0s must be 0.5 of a 2000-tick bar (500 ticks/beat), got {}",
            engine.summary().beat_position
        );

        // After another 1.0s (total 2.0s = 96000 frames), must have wrapped cleanly back near 0.0
        for _ in 0..100 {
            engine.process_block(&mut l, &mut r);
        }
        let bp = engine.summary().beat_position;
        assert!(
            bp < 0.03 || bp > 0.97,
            "At 120 BPM, 2.0s must complete 1 full 2000-tick bar, got beat_position={}",
            bp
        );
    }

    #[test]
    fn test_bpm_parsing_and_tempo_lock() {
        let dir = Path::new(USER_DOCUMENTS_RHYTHMS_DIR);
        if !dir.exists() {
            return;
        }
        let josiran = dir.join("Josiran_Pisadinha_GR.Music");
        if josiran.exists() {
            let pkg = parse_music_file(&josiran).expect("parse Josiran");
            assert!((pkg.styles[0].bpm - 85.0).abs() < 0.5, "Josiran BPM expected 85, got {}", pkg.styles[0].bpm);
        }

        let calcinha = dir.join("Calcinha_Preta_ORG2024.Music");
        if calcinha.exists() {
            let pkg = parse_music_file(&calcinha).expect("parse Calcinha_Preta v1.2.5");
            assert!(
                (pkg.styles[0].bpm - 97.0).abs() < 1.5,
                "Calcinha Preta v1.2.5 raw=81 should scale *120/100 to ~97 BPM, got {}",
                pkg.styles[0].bpm
            );
        }

        let mega = dir.join("MEGA_PACK_70_RITMOS_JUNTOS_TC.Music");
        if mega.exists() {
            let pkg = Arc::new(parse_music_file(&mega).expect("parse MEGA_PACK_70"));
            assert!(pkg.styles.len() >= 10, "expected many styles in MEGA_PACK_70");
            assert!(
                (pkg.styles[0].bpm - 90.0).abs() < 0.5,
                "MEGA_PACK_70 style[0] expected 90 BPM, got {}",
                pkg.styles[0].bpm
            );

            let mut engine = MusicStudioArrangerEngine::new(48000.0);
            engine.load_package(Arc::clone(&pkg));
            assert!((engine.bpm - 90.0).abs() < 0.5);

            // Switch to style 1 without Tempo Lock -> adopts style 1's BPM (96)
            engine.set_style(1);
            assert!(
                (engine.bpm - pkg.styles[1].bpm).abs() < 0.5,
                "unlocked style change should update BPM to {}, got {}",
                pkg.styles[1].bpm,
                engine.bpm
            );

            // Lock Tempo at 90 BPM and switch styles -> preserves 90 BPM
            engine.set_transport(false, None, Some(90.0));
            engine.set_tempo_locked(true);
            engine.set_style(0);
            engine.set_style(1);
            assert!(
                (engine.bpm - 90.0).abs() < 0.5,
                "locked tempo must stay 90 BPM across style changes, got {}",
                engine.bpm
            );
        }
    }

    #[test]
    fn test_loop_continuity_no_silence_gap() {
        let dir = Path::new(USER_DOCUMENTS_RHYTHMS_DIR);
        if !dir.exists() {
            return;
        }
        for (fname, expected_var1_ticks) in [
            ("Toque_Dez_GR.Music", 16000_u32),
            ("Tarcisio_do_Acordeon_Forro_Romantico_TM.Music", 8000_u32),
        ] {
            let p = dir.join(fname);
            if !p.exists() {
                continue;
            }
            let pkg = Arc::new(parse_music_file(&p).expect("parse rhythm"));
            let var1 = pkg.styles[0]
                .sections
                .iter()
                .find(|s| s.name == "Variation 1")
                .expect("Variation 1");
            assert_eq!(
                var1.length_ticks, expected_var1_ticks,
                "{} Variation 1 length_ticks must equal LimitEnd={}",
                fname, expected_var1_ticks
            );

            let mut engine = MusicStudioArrangerEngine::new(48000.0);
            engine.load_package(pkg);
            engine.set_section("Variation 1", true);
            engine.set_transport(true, None, None);

            // Render across loop boundary (2.25 loops worth of 4096-frame blocks)
            let secs_per_loop = (expected_var1_ticks as f64 / 500.0) * (60.0 / engine.bpm as f64);
            let total_blocks = ((secs_per_loop * 2.25 * 48000.0) / 4096.0).ceil() as usize;
            let mut l = vec![0.0_f32; 4096];
            let mut r = vec![0.0_f32; 4096];
            let mut silent_blocks = 0_usize;
            for _ in 0..total_blocks {
                engine.process_block(&mut l, &mut r);
                let rms: f32 = (l.iter().map(|s| s * s).sum::<f32>() / l.len() as f32).sqrt();
                if rms < 0.002 {
                    silent_blocks += 1;
                }
            }
            assert_eq!(
                silent_blocks, 0,
                "{}: found {} silent 85ms blocks across 2.25 loop cycles",
                fname, silent_blocks
            );
        }
    }

    #[test]
    fn test_melodic_instruments_not_drumkits_and_all_sample_rates() {
        let dir = Path::new(USER_DOCUMENTS_RHYTHMS_DIR);
        if !dir.exists() {
            return;
        }
        let toque = dir.join("Toque_Dez_GR.Music");
        if toque.exists() {
            let pkg = parse_music_file(&toque).expect("parse Toque_Dez");
            let piano_base = pkg
                .instruments
                .iter()
                .find(|i| i.name.contains("PIANO BASE"))
                .expect("PIANO BASE (GR)");
            assert!(
                !piano_base.is_drumkit,
                "PIANO BASE (GR) must not be marked as drumkit"
            );
        }

        let arrasta = dir.join("Arrasta_Pe_Forro_GR.Music");
        if arrasta.exists() {
            let pkg = parse_music_file(&arrasta).expect("parse Arrasta_Pe");
            for kw in ["Finger Bass", "Safona base", "Distortion GtrGM"] {
                let inst = pkg
                    .instruments
                    .iter()
                    .find(|i| i.name.contains(kw))
                    .unwrap_or_else(|| panic!("missing {}", kw));
                assert!(!inst.is_drumkit, "{} must be melodic", kw);
                assert!(
                    inst.zones.iter().any(|z| !z.pcm_l.is_empty()),
                    "{} must have decoded PCM samples",
                    kw
                );
            }
        }

        let turma = dir.join("A_Turma_da_Pisadinha_TM.Music");
        if turma.exists() {
            let pkg = parse_music_file(&turma).expect("parse A_Turma");
            let btr = pkg
                .instruments
                .iter()
                .find(|i| i.name.contains("BATERIA 1"))
                .expect("BATERIA 1");
            assert!(
                btr.zones.iter().any(|z| !z.pcm_l.is_empty()),
                "BATERIA 1 (52kHz) must decode zones with samples"
            );
        }
    }
}
