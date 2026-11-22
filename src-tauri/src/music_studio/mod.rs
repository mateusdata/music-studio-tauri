use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const DEFAULT_MUSIC_STUDIO_DIR: &str =
    "/home/data/.config/hydralauncher/wine-prefixes/1174180/drive_c/users/steamuser/AppData/Local/Programs/Music Studio";
pub const USER_DOCUMENTS_RHYTHMS_DIR: &str = "/home/data/Documents/Ritmos_Forro_Music_Studio";
pub const USER_DOCUMENTS_KEYBOARDS_DIR: &str = "/home/data/Documents/Ritmos de Teclados";
pub const USER_DOCUMENTS_KEYBOARD_DIR: &str = USER_DOCUMENTS_KEYBOARDS_DIR;
pub const USER_DOCUMENTS_INSTRUMENTS_DIR: &str = "/home/data/Documents/instrumentos pack";

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicPackageEntry {
    pub file_name: String,
    pub file_path: String,
    pub size_mb: f32,
    pub version: String,
    pub format: String,
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
pub struct OtsPresetSummary {
    pub index: usize,
    pub name: String,
    pub upper1_instrument: usize,
    pub upper2_instrument: Option<usize>,
    pub lower_instrument: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicPackageSummary {
    pub loaded: bool,
    pub file_name: String,
    pub file_path: String,
    pub version: String,
    pub source_format: String,
    pub soundfont_name: Option<String>,
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
    pub chord_mode: String,
    pub track_volumes: [f32; 8],
    pub track_instruments: [Option<usize>; 8],
    pub ots_presets: Vec<OtsPresetSummary>,
    pub beat_position: f32,
    pub sty_warnings: Vec<String>,
    pub has_audio_style_chunks: bool,
    pub uses_expansion_kit: bool,
    pub sty_diagnostics_summary: Option<String>,
}

impl Default for MusicPackageSummary {
    fn default() -> Self {
        Self {
            loaded: false,
            file_name: String::new(),
            file_path: String::new(),
            version: String::new(),
            source_format: "music".to_string(),
            soundfont_name: None,
            instruments: Vec::new(),
            styles: Vec::new(),
            active_style_index: 0,
            active_section: "Variation 1".to_string(),
            pending_section: None,
            upper1_instrument: 0,
            upper2_instrument: None,
            lower_instrument: None,
            split_key: 54,
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
            chord_mode: ChordMode::Fingered3To7.as_str().to_string(),
            track_volumes: [1.0; 8],
            track_instruments: [None; 8],
            ots_presets: Vec::new(),
            beat_position: 0.0,
            sty_warnings: Vec::new(),
            has_audio_style_chunks: false,
            uses_expansion_kit: false,
            sty_diagnostics_summary: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SampleZone {
    pub root_key: u8,
    pub key_low: u8,
    pub key_high: u8,
    pub vel_low: u8,
    pub vel_high: u8,
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
    pub casm_rule: Option<crate::yamaha_sty::CasmRule>,
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
    pub ots_presets: Vec<OtsPresetSummary>,
    pub sty_warnings: Vec<String>,
    pub has_audio_style_chunks: bool,
    pub uses_expansion_kit: bool,
    pub sty_diagnostics_summary: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedMusicPackage {
    pub file_name: String,
    pub file_path: String,
    pub version: String,
    pub source_format: String,
    pub soundfont_name: Option<String>,
    pub instruments: Vec<LoadedInstrument>,
    pub styles: Vec<RhythmStyle>,
    pub ots_presets: Vec<OtsPresetSummary>,
    pub source_paths: Vec<PathBuf>,
    pub sty_warnings: Vec<String>,
    pub has_audio_style_chunks: bool,
    pub uses_expansion_kit: bool,
    pub sty_diagnostics_summary: Option<String>,
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
    if ch == 2 {
        pcm_r.reserve(count);
    }
    if bits == 16 {
        let stride = ch_u * 2;
        if ch == 2 {
            for frame in raw.chunks_exact(stride).take(count) {
                let l = i16::from_le_bytes([frame[0], frame[1]]) as f32 / 32768.0;
                let r = i16::from_le_bytes([frame[2], frame[3]]) as f32 / 32768.0;
                pcm_l.push(l);
                pcm_r.push(r);
            }
        } else {
            for frame in raw.chunks_exact(stride).take(count) {
                let l = i16::from_le_bytes([frame[0], frame[1]]) as f32 / 32768.0;
                pcm_l.push(l);
            }
        }
    } else {
        let stride = ch_u;
        if ch == 2 {
            for frame in raw.chunks_exact(stride).take(count) {
                let l = (frame[0] as f32 - 128.0) / 128.0;
                let r = (frame[1] as f32 - 128.0) / 128.0;
                pcm_l.push(l);
                pcm_r.push(r);
            }
        } else {
            for frame in raw.chunks_exact(stride).take(count) {
                let l = (frame[0] as f32 - 128.0) / 128.0;
                pcm_l.push(l);
            }
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

    let pcm_l_arc = Arc::new(pcm_l);
    let pcm_r_arc = if ch == 1 || pcm_r.is_empty() {
        Arc::clone(&pcm_l_arc)
    } else {
        Arc::new(pcm_r)
    };

    let gain_norm = if (1..=250).contains(&vol_raw) {
        vol_raw as f32 / 100.0
    } else {
        1.0
    };
    let valid_loop = (loop_mode == 2 || loop_mode == 3)
        && loop_end > loop_start + 32
        && loop_end <= pcm_l_arc.len();

    Some((
        SampleZone {
            root_key,
            key_low: root_key,
            key_high: root_key,
            vel_low: 0,
            vel_high: 127,
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
            pcm_l: pcm_l_arc,
            pcm_r: pcm_r_arc,
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

fn skip_effect_records(data: &[u8], mut cur: usize, count: usize, limit: usize) -> usize {
    for _ in 0..count.min(32) {
        if cur + 12 > limit {
            break;
        }
        let tag = read_u32_le(data, cur).unwrap_or(0);
        let rec_len = match tag {
            0x07 => {
                let bands = read_u32_le(data, cur + 8).unwrap_or(8).min(32) as usize;
                12 + bands * 4
            }
            0x11 | 0x1f => 12,
            _ => 16,
        };
        if cur + rec_len > limit {
            break;
        }
        cur += rec_len;
    }
    cur
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

    let mut base_stride = None;
    if data.len().saturating_sub(n0_off) == count.saturating_mul(72) {
        base_stride = Some(72);
    }
    if base_stride.is_none() {
        if let Some(db) = db_off {
            let dp1_off = data[db..data.len().min(db + 512)]
                .windows(14)
                .position(|w| w == b"Default Pad 1\0")
                .map(|pos| db + pos - 9);
            if let Some(dp1) = dp1_off {
                let diff = dp1.saturating_sub(db);
                if matches!(diff, 72 | 124 | 140 | 148) {
                    base_stride = Some(diff);
                }
            }
            if base_stride.is_none() {
                let diff = db.saturating_sub(n0_off);
                if matches!(diff, 72 | 124 | 140 | 148) {
                    base_stride = Some(diff);
                }
            }
        }
    }
    if base_stride.is_none() {
        let dp1_off = data[n0_off..data.len().min(n0_off + 1024)]
            .windows(14)
            .position(|w| w == b"Default Pad 1\0")
            .map(|pos| n0_off + pos - 9);
        let dp2_off = data[n0_off..data.len().min(n0_off + 1024)]
            .windows(14)
            .position(|w| w == b"Default Pad 2\0")
            .map(|pos| n0_off + pos - 9);
        if let (Some(dp1), Some(dp2)) = (dp1_off, dp2_off) {
            let diff = dp2.saturating_sub(dp1);
            if matches!(diff, 72 | 124 | 140 | 148) {
                base_stride = Some(diff);
            }
        }
    }
    if base_stride.is_none() {
        for cand in [72_usize, 124, 140, 148] {
            if n0_off + cand > data.len() {
                continue;
            }
            let n1 = if cand == 72 {
                n0_off + 72
            } else {
                let ex = read_u32_le(data, n0_off + cand - 4).unwrap_or(0) as usize;
                skip_effect_records(data, n0_off + cand, ex, data.len())
            };
            if n1 + 44 <= data.len() {
                let pid1 = read_u32_le(data, n1 + 4).unwrap_or(1);
                let rl1 = data[n1 + 8];
                let code1 = data[n1 + 41];
                if (pid1 == 0 || pid1 == 0xffff_ffff)
                    && (1..=0x20).contains(&rl1)
                    && matches!(code1, 0 | 1 | 255)
                {
                    base_stride = Some(cand);
                    break;
                }
            }
        }
    }
    let base_stride = base_stride.unwrap_or(124);

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

    let mut cur = n0_off;
    for order in 0..count {
        if cur + base_stride.min(72) > data.len() {
            break;
        }
        if let Some(node) = parse_node(cur, order) {
            nodes.push(node);
        }
        if base_stride == 72 {
            cur += 72;
        } else {
            let extra = read_u32_le(data, cur + base_stride - 4).unwrap_or(0) as usize;
            cur = skip_effect_records(data, cur + base_stride, extra, data.len());
            if order == 0 {
                if let Some(db) = db_off {
                    if db >= n0_off + base_stride && cur != db {
                        cur = db;
                    }
                }
            }
        }
    }

    (n0_off.saturating_sub(4), nodes)
}

fn extract_track_events(
    data: &[u8],
    hdr_end: usize,
    next_blk: usize,
    hsz: usize,
    is_drum: bool,
    require_exact_end: bool,
) -> Option<Vec<RhythmNoteEvent>> {
    let evc_off = hdr_end + hsz;
    let evc = read_u32_le(data, evc_off)? as usize;
    if evc == 0 || evc > 16_000 {
        return None;
    }
    let arr_start = evc_off + 4;
    let arr_end = arr_start.saturating_add(evc.saturating_mul(24));
    if arr_end > next_blk {
        return None;
    }
    if require_exact_end && arr_end != next_blk {
        return None;
    }
    let mut events = Vec::new();
    for e in 0..evc {
        let eo = arr_start + e * 24;
        let etype = (read_u32_le(data, eo).unwrap_or(0) & 0xff) as u8;
        let note = read_i32_le(data, eo + 8).unwrap_or(-1);
        let vel = read_i32_le(data, eo + 12).unwrap_or(100);
        let tick = read_i32_le(data, eo + 16).unwrap_or(-1);
        let dur = read_i32_le(data, eo + 20).unwrap_or(120);
        if etype == 2 && (0..=127).contains(&note) && (0..=2_000_000).contains(&tick) && (dur > 0 || is_drum) {
            events.push(RhythmNoteEvent {
                midi_note: note as u8,
                velocity: vel.clamp(1, 127) as u8,
                start_tick: tick as u32,
                duration_ticks: dur.max(30) as u32,
            });
        }
    }
    if events.is_empty() {
        return None;
    }
    events.sort_by_key(|ev| ev.start_tick);
    Some(events)
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

        // Locate event_count dynamically (+531, +544, +635, or variable FX headers) while preserving TSoundSetting at +159/+172
        let mut detected_hsz = None;
        let mut events = Vec::new();
        for &hsz in &[531_usize, 544, 635, 648] {
            if let Some(evs) = extract_track_events(data, hdr_end, next_blk, hsz, is_drum, true) {
                detected_hsz = Some(hsz);
                events = evs;
                break;
            }
        }
        if detected_hsz.is_none() {
            let max_hsz = next_blk.saturating_sub(hdr_end + 4).min(2048);
            for hsz in 500..max_hsz {
                if let Some(evs) = extract_track_events(data, hdr_end, next_blk, hsz, is_drum, true)
                {
                    detected_hsz = Some(hsz);
                    events = evs;
                    break;
                }
            }
        }
        if detected_hsz.is_none() {
            for &hsz in &[531_usize, 544] {
                if let Some(evs) =
                    extract_track_events(data, hdr_end, next_blk, hsz, is_drum, false)
                {
                    detected_hsz = Some(hsz);
                    events = evs;
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
            casm_rule: None,
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
                    ots_presets: Vec::new(),
                    sty_warnings: Vec::new(),
                    has_audio_style_chunks: false,
                    uses_expansion_kit: false,
                    sty_diagnostics_summary: None,
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
                length_ticks: ((max_end + 499) / 500 * 500).max(1000),
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
            ots_presets: Vec::new(),
            sty_warnings: Vec::new(),
            has_audio_style_chunks: false,
            uses_expansion_kit: false,
            sty_diagnostics_summary: None,
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
        source_format: "music".to_string(),
        soundfont_name: None,
        instruments,
        styles,
        ots_presets: Vec::new(),
        source_paths: vec![path.to_path_buf()],
        sty_warnings: Vec::new(),
        has_audio_style_chunks: false,
        uses_expansion_kit: false,
        sty_diagnostics_summary: None,
    })
}

/// Parses a standalone Music Studio `.Instrument` binary file into a playable `LoadedInstrument`.
pub fn parse_instrument_file(path: &Path, index: usize) -> Result<LoadedInstrument, String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read '{}': {}", path.display(), e))?;
    let magic = b"Music Studio Instrument\x00";
    if data.len() < 0x40
        || !data[..magic.len()].eq_ignore_ascii_case(magic)
    {
        return Err(format!(
            "'{}' is not a valid Music Studio .Instrument file",
            path.display()
        ));
    }
    let slen = read_u32_le(&data, 0x20).unwrap_or(0) as usize;
    if slen == 0 || slen > 256 || 0x24 + slen > data.len() {
        return Err(format!(
            "Invalid .Instrument header length in '{}'",
            path.display()
        ));
    }
    let hdr_end = 0x24 + slen;
    let raw_tag = decode_latin1(&data[0x24..hdr_end]);
    let raw_name = raw_tag.split('\r').next().unwrap_or("").trim().to_string();
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported Instrument")
        .trim()
        .to_string();
    let display_name = if raw_name.is_empty() {
        file_stem
    } else if raw_name.eq_ignore_ascii_case(&file_stem.replace('_', " ")) {
        raw_name
    } else {
        format!("{} ({})", raw_name, file_stem)
    };

    let inst = parse_instrument_block(
        &data,
        index,
        display_name,
        hdr_end,
        data.len(),
        &HashMap::new(),
        true,
    );
    if !inst.zones.iter().any(|z| !z.pcm_l.is_empty()) {
        return Err(format!(
            "No playable sample zones found in '{}'",
            path.display()
        ));
    }
    Ok(inst)
}

fn is_sty_extension_str(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "sty" | "prs" | "sst" | "bcs"
    )
}

fn scan_dir_for_packages(dir: &Path, depth: usize, out: &mut Vec<MusicPackageEntry>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for item in rd.flatten() {
        if item.file_type().map(|kind| kind.is_symlink()).unwrap_or(true) {
            continue;
        }
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
        let is_music = ext.eq_ignore_ascii_case("music") || ext.eq_ignore_ascii_case("template");
        let is_sty = is_sty_extension_str(ext);
        if !is_music && !is_sty {
            continue;
        }
        if let Ok(meta) = std::fs::metadata(&p) {
            let mut hdr = [0_u8; 64];
            if let Ok(mut f) = std::fs::File::open(&p) {
                use std::io::Read;
                let n = f.read(&mut hdr).unwrap_or(0);
                let file_name = p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                let size_mb = (meta.len() as f32) / (1024.0 * 1024.0);
                if is_music && n >= 36 && is_music_studio_magic(&hdr) {
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
                        file_name,
                        file_path: p.to_string_lossy().to_string(),
                        size_mb,
                        version,
                        format: "music".to_string(),
                    });
                } else if is_sty && n >= 14 {
                    let version = if &hdr[0..4] == b"MThd" {
                        Some("SFF1/SFF2".to_string())
                    } else if &hdr[0..4] == b"STYL" {
                        Some("STYL".to_string())
                    } else {
                        None
                    };
                    if let Some(version) = version {
                        out.push(MusicPackageEntry {
                            file_name,
                            file_path: p.to_string_lossy().to_string(),
                            size_mb,
                            version,
                            format: "sty".to_string(),
                        });
                    }
                }
            }
        }
    }
}

pub fn list_music_packages_in_dir(dir: &Path) -> Vec<MusicPackageEntry> {
    let mut entries = Vec::new();
    scan_dir_for_packages(dir, 0, &mut entries);
    entries.sort_by(|a, b| {
        a.file_name
            .to_lowercase()
            .cmp(&b.file_name.to_lowercase())
            .then_with(|| a.file_path.to_lowercase().cmp(&b.file_path.to_lowercase()))
    });
    entries
}

pub fn list_all_music_packages() -> Vec<MusicPackageEntry> {
    let mut raw = Vec::new();
    scan_dir_for_packages(Path::new(USER_DOCUMENTS_RHYTHMS_DIR), 2, &mut raw);
    scan_dir_for_packages(Path::new(USER_DOCUMENTS_KEYBOARD_DIR), 5, &mut raw);
    scan_dir_for_packages(Path::new(DEFAULT_MUSIC_STUDIO_DIR), 1, &mut raw);

    let mut seen_paths = HashSet::new();
    let mut deduped = Vec::with_capacity(raw.len());
    for entry in raw {
        let canon = std::fs::canonicalize(&entry.file_path)
            .unwrap_or_else(|_| PathBuf::from(&entry.file_path));
        if seen_paths.insert(canon) {
            deduped.push(entry);
        }
    }
    deduped.sort_by(|a, b| {
        a.file_name
            .to_lowercase()
            .cmp(&b.file_name.to_lowercase())
            .then_with(|| a.file_path.to_lowercase().cmp(&b.file_path.to_lowercase()))
    });
    deduped
}

const CURATED_SF2_PALETTE: [(u16, u16); 13] = [
    (0, 0),    // Grand Piano
    (0, 21),   // Accordion
    (0, 24),   // Nylon Guitar
    (0, 25),   // Steel Guitar
    (0, 27),   // Clean Electric Guitar
    (0, 33),   // Finger Bass
    (0, 36),   // Slap Bass
    (0, 48),   // Strings
    (0, 61),   // Brass Section
    (0, 65),   // Alto Sax
    (0, 81),   // Lead Square
    (128, 0),  // Standard Drums
    (128, 16), // Power Drums
];

/// Converts a decoded SoundFont 2 preset into a playable `LoadedInstrument`.
pub fn sf2_to_loaded_instrument(
    inst_data: &crate::soundfont::Sf2InstrumentData,
    index: usize,
) -> LoadedInstrument {
    let zones = inst_data
        .zones
        .iter()
        .map(|z| SampleZone {
            root_key: z.root_key,
            key_low: z.key_low,
            key_high: z.key_high,
            vel_low: z.vel_low,
            vel_high: z.vel_high,
            name: z.name.clone(),
            sample_rate: z.sample_rate,
            channels: if Arc::ptr_eq(&z.pcm_l, &z.pcm_r) {
                1
            } else {
                2
            },
            loop_start: z.loop_start,
            loop_end: z.loop_end,
            loop_enabled: z.loop_enabled,
            loop_mode: if z.loop_enabled { 2 } else { 0 },
            gain: z.gain,
            pan: z.pan,
            fine_tune: z.fine_tune_cents * 10,
            pool_sid: 0,
            pcm_l: Arc::clone(&z.pcm_l),
            pcm_r: Arc::clone(&z.pcm_r),
        })
        .collect();

    let name = if inst_data.is_drumkit
        && !inst_data.name.to_ascii_lowercase().contains("drum")
        && !inst_data.name.to_ascii_lowercase().contains("kit")
    {
        format!("GM Drum: {}", inst_data.name)
    } else {
        inst_data.name.clone()
    };

    LoadedInstrument {
        index,
        name,
        volume: 1.0,
        pan: 0,
        inst_type: if inst_data.is_drumkit { 1 } else { 0 },
        is_drumkit: inst_data.is_drumkit,
        zones,
    }
}

fn sty_slot_name(slot: usize) -> &'static str {
    match slot {
        0 => "Drum",
        1 => "Percussion",
        2 => "Bass",
        3 => "Acc. 1",
        4 => "Acc. 2",
        5 => "Acc. 3",
        6 => "Acc. 4",
        _ => "Acc. 5",
    }
}

fn load_sf2_presets_once(
    bank: &crate::soundfont::SoundFontBank,
    requests: &[(u16, u16)],
) -> (Vec<LoadedInstrument>, HashMap<(u16, u16), usize>) {
    let mut instruments = Vec::new();
    let mut requested_to_idx: HashMap<(u16, u16), usize> = HashMap::new();
    let mut resolved_to_idx: HashMap<(u16, u16), usize> = HashMap::new();

    for &(req_bank, req_prog) in requests {
        if requested_to_idx.contains_key(&(req_bank, req_prog)) {
            continue;
        }
        let Some(sf2_data) = bank.load_preset(req_bank, req_prog) else {
            continue;
        };
        let actual_key = (sf2_data.bank, sf2_data.program);
        let idx = if let Some(&existing_idx) = resolved_to_idx.get(&actual_key) {
            existing_idx
        } else {
            let new_idx = instruments.len();
            instruments.push(sf2_to_loaded_instrument(&sf2_data, new_idx));
            resolved_to_idx.insert(actual_key, new_idx);
            new_idx
        };
        requested_to_idx.insert((req_bank, req_prog), idx);
    }

    (instruments, requested_to_idx)
}

/// Builds a playable `LoadedMusicPackage` from one or more Yamaha `.STY`/`.PRS`/`.SST`/`.BCS`
/// style files paired with a SoundFont 2 bank.
pub fn build_sty_package(
    paths: &[PathBuf],
    sf2_path: Option<&Path>,
) -> Result<LoadedMusicPackage, String> {
    if paths.is_empty() {
        return Err("Nenhum arquivo .STY especificado".to_string());
    }

    let resolved_sf2 = sf2_path
        .map(Path::to_path_buf)
        .or_else(crate::soundfont::find_default_soundfont)
        .ok_or_else(|| "Nenhum SoundFont (.sf2) encontrado no sistema".to_string())?;
    let bank = crate::soundfont::SoundFontBank::open(&resolved_sf2)?;

    let mut docs = Vec::with_capacity(paths.len());
    let mut errors = Vec::new();
    for p in paths {
        match crate::yamaha_sty::parse_sty_file(p) {
            Ok(doc) => docs.push(doc),
            Err(e) => errors.push(format!("{}: {}", p.display(), e)),
        }
    }
    if docs.is_empty() {
        return Err(errors.join("; "));
    }

    let mut requested_presets: Vec<(u16, u16)> = Vec::new();
    let mut seen_req: HashSet<(u16, u16)> = HashSet::new();
    let mut push_req = |bank_id: u16, prog_id: u16| {
        if seen_req.insert((bank_id, prog_id)) {
            requested_presets.push((bank_id, prog_id));
        }
    };

    for doc in &docs {
        for sec in &doc.sections {
            for tr in &sec.tracks {
                let kit_opt = crate::yamaha_sty::classify_drum_kit(
                    tr.dest_ch,
                    tr.voice.msb,
                    tr.voice.lsb,
                    tr.voice.program,
                );
                let (b, p, resolved_drum) = crate::yamaha_sty::resolve_yamaha_voice_to_gm(
                    tr.dest_ch,
                    tr.voice.msb,
                    tr.voice.lsb,
                    tr.voice.program,
                    &tr.voice.name_hint,
                );
                if (tr.is_drum || kit_opt.is_some() || tr.track_slot <= 1)
                    && (!resolved_drum || b != 128)
                {
                    push_req(128, 0);
                } else {
                    push_req(b, p);
                }
            }
        }
        for ots in &doc.ots_presets {
            let (b1, p1, _) = crate::yamaha_sty::resolve_yamaha_voice_to_gm(
                0,
                ots.right1_bank_msb,
                ots.right1_bank_lsb,
                ots.right1_program,
                &ots.name,
            );
            push_req(b1, p1);
            if let Some((m, l, pr)) = ots.right2_program {
                let (b2, p2, _) = crate::yamaha_sty::resolve_yamaha_voice_to_gm(1, m, l, pr, &ots.name);
                push_req(b2, p2);
            }
            if let Some((m, l, pr)) = ots.left_program {
                let (bl, pl, _) = crate::yamaha_sty::resolve_yamaha_voice_to_gm(3, m, l, pr, &ots.name);
                push_req(bl, pl);
            }
        }
    }

    for &(b, p) in &CURATED_SF2_PALETTE {
        push_req(b, p);
    }

    let (instruments, requested_to_idx) = load_sf2_presets_once(&bank, &requested_presets);
    if instruments.is_empty() {
        return Err(format!(
            "SoundFont '{}' não possui presets carregáveis",
            resolved_sf2.display()
        ));
    }
    let fallback_drum_idx = instruments.iter().position(|i| i.is_drumkit);

    let mut has_sff1 = false;
    let mut has_sff2 = false;
    let mut has_styl = false;
    let mut styles = Vec::with_capacity(docs.len());

    for (doc_idx, doc) in docs.iter().enumerate() {
        match doc.format {
            crate::yamaha_sty::StyFormat::Sff1 => has_sff1 = true,
            crate::yamaha_sty::StyFormat::Sff2 => has_sff2 = true,
            crate::yamaha_sty::StyFormat::LegacyStyl => has_styl = true,
        }

        let sections: Vec<RhythmSection> = doc
            .sections
            .iter()
            .map(|sec| {
                let tracks = sec
                    .tracks
                    .iter()
                    .map(|tr| {
                        let kit_opt = crate::yamaha_sty::classify_drum_kit(
                            tr.dest_ch,
                            tr.voice.msb,
                            tr.voice.lsb,
                            tr.voice.program,
                        );
                        let (req_b, req_p, resolved_drum) =
                            crate::yamaha_sty::resolve_yamaha_voice_to_gm(
                                tr.dest_ch,
                                tr.voice.msb,
                                tr.voice.lsb,
                                tr.voice.program,
                                &tr.voice.name_hint,
                            );
                        let is_drum_track = tr.is_drum
                            || resolved_drum
                            || kit_opt.is_some()
                            || tr.track_slot <= 1;
                        let (target_b, target_p) = if is_drum_track && req_b != 128 {
                            (128, 0)
                        } else {
                            (req_b, req_p)
                        };
                        let mut inst_idx = requested_to_idx
                            .get(&(target_b, target_p))
                            .copied()
                            .unwrap_or_else(|| {
                                if is_drum_track {
                                    fallback_drum_idx.unwrap_or(0)
                                } else {
                                    0
                                }
                            });
                        if is_drum_track {
                            let resolved_is_drumkit = instruments
                                .get(inst_idx)
                                .map(|i| i.is_drumkit)
                                .unwrap_or(false);
                            if !resolved_is_drumkit {
                                if let Some(drum_idx) = requested_to_idx
                                    .get(&(128, 0))
                                    .copied()
                                    .filter(|&idx| {
                                        instruments.get(idx).map(|i| i.is_drumkit).unwrap_or(false)
                                    })
                                    .or(fallback_drum_idx)
                                {
                                    inst_idx = drum_idx;
                                }
                            }
                        }
                        let inst_is_drum = instruments
                            .get(inst_idx)
                            .map(|i| i.is_drumkit)
                            .unwrap_or(false);
                        let effective_is_drum = is_drum_track || inst_is_drum;
                        let drum_kit = if effective_is_drum {
                            Some(kit_opt.unwrap_or(crate::yamaha_sty::DrumKitKind::XgStandard))
                        } else {
                            None
                        };
                        let events = tr
                            .events
                            .iter()
                            .map(|ev| RhythmNoteEvent {
                                midi_note: match drum_kit {
                                    Some(kit) => {
                                        crate::yamaha_sty::remap_yamaha_drum_note(ev.midi_note, kit)
                                    }
                                    None => ev.midi_note,
                                },
                                velocity: ev.velocity,
                                start_tick: ev.start_tick,
                                duration_ticks: ev.duration_ticks,
                            })
                            .collect();
                        RhythmTrack {
                            track_type: sty_slot_name(tr.track_slot).to_string(),
                            track_slot: tr.track_slot.min(7),
                            instrument_index: inst_idx,
                            volume: (tr.voice.volume as f32 / 100.0).clamp(0.05, 1.5),
                            pan: tr.voice.pan,
                            fine_tune: 0,
                            key_move: 0,
                            octave_shift: 0,
                            is_drum: effective_is_drum,
                            casm_rule: tr.casm_rule.clone(),
                            events,
                        }
                    })
                    .collect();
                RhythmSection {
                    id: sec.id,
                    name: sec.name.clone(),
                    length_ticks: sec.length_ticks,
                    is_loop: sec.is_loop,
                    jump_to: sec.jump_to,
                    tracks,
                }
            })
            .collect();

        let ots_presets: Vec<OtsPresetSummary> = doc
            .ots_presets
            .iter()
            .map(|ots| {
                let (b1, p1, _) = crate::yamaha_sty::resolve_yamaha_voice_to_gm(
                    0,
                    ots.right1_bank_msb,
                    ots.right1_bank_lsb,
                    ots.right1_program,
                    &ots.name,
                );
                let upper1_instrument = requested_to_idx.get(&(b1, p1)).copied().unwrap_or(0);
                let upper2_instrument = ots.right2_program.and_then(|(m, l, pr)| {
                    let (b2, p2, _) =
                        crate::yamaha_sty::resolve_yamaha_voice_to_gm(1, m, l, pr, &ots.name);
                    requested_to_idx.get(&(b2, p2)).copied()
                });
                let lower_instrument = ots.left_program.and_then(|(m, l, pr)| {
                    let (bl, pl, _) =
                        crate::yamaha_sty::resolve_yamaha_voice_to_gm(3, m, l, pr, &ots.name);
                    requested_to_idx.get(&(bl, pl)).copied()
                });
                OtsPresetSummary {
                    index: ots.index,
                    name: ots.name.clone(),
                    upper1_instrument,
                    upper2_instrument,
                    lower_instrument,
                }
            })
            .collect();

        let category = match doc.format {
            crate::yamaha_sty::StyFormat::Sff1 => "Yamaha SFF1",
            crate::yamaha_sty::StyFormat::Sff2 => "Yamaha SFF2",
            crate::yamaha_sty::StyFormat::LegacyStyl => "Yamaha STYL",
        }
        .to_string();

        let has_audio_style_chunks =
            doc.has_audio_style_chunks || doc.diagnostics.has_audio_style_chunks;
        let uses_expansion_kit = doc.diagnostics.uses_expansion_kit;
        let sty_warnings = doc.diagnostics.warnings.clone();
        let sty_diagnostics_summary = Some(doc.diagnostics.summary_text());

        styles.push(RhythmStyle {
            id: (doc_idx + 1) as u32,
            name: doc.title.clone(),
            category,
            bpm: doc.bpm.clamp(40.0, 240.0),
            sections,
            ots_presets,
            sty_warnings,
            has_audio_style_chunks,
            uses_expansion_kit,
            sty_diagnostics_summary,
        });
    }

    let version = match (has_sff1, has_sff2, has_styl) {
        (false, true, false) => "SFF2/SF2",
        (true, false, false) => "SFF1/SF2",
        (false, false, true) => "STYL/SF2",
        _ => "SFF1/SFF2/SF2",
    }
    .to_string();

    let is_pack = docs.len() > 1;
    let file_name = if is_pack {
        format!("{} Ritmos .STY", docs.len())
    } else {
        docs[0].file_name.clone()
    };
    let file_path = docs[0].file_path.clone();
    let ots_presets = styles
        .first()
        .map(|s| s.ots_presets.clone())
        .unwrap_or_default();
    let has_audio_style_chunks = styles.iter().any(|s| s.has_audio_style_chunks);
    let uses_expansion_kit = styles.iter().any(|s| s.uses_expansion_kit);
    let mut sty_warnings = Vec::new();
    for s in &styles {
        for w in &s.sty_warnings {
            if !sty_warnings.contains(w) {
                sty_warnings.push(w.clone());
            }
        }
    }
    let sty_diagnostics_summary = styles
        .first()
        .and_then(|s| s.sty_diagnostics_summary.clone());

    Ok(LoadedMusicPackage {
        file_name,
        file_path,
        version,
        source_format: if is_pack {
            "sty_pack".to_string()
        } else {
            "sty".to_string()
        },
        soundfont_name: Some(bank.name().to_string()),
        instruments,
        styles,
        ots_presets,
        source_paths: docs.iter().map(|d| PathBuf::from(&d.file_path)).collect(),
        sty_warnings,
        has_audio_style_chunks,
        uses_expansion_kit,
        sty_diagnostics_summary,
    })
}

/// Rebuilds an active `.STY` package from its source style paths using a newly selected SoundFont,
/// or augments an active `.Music` package with curated SoundFont solo instruments.
pub fn rebuild_package_with_soundfont(
    pkg: &LoadedMusicPackage,
    sf2_path: &Path,
) -> Result<LoadedMusicPackage, String> {
    if (pkg.source_format == "sty" || pkg.source_format == "sty_pack")
        && !pkg.source_paths.is_empty()
    {
        return build_sty_package(&pkg.source_paths, Some(sf2_path));
    }

    let bank = crate::soundfont::SoundFontBank::open(sf2_path)?;
    let (sf2_insts, _) = load_sf2_presets_once(&bank, &CURATED_SF2_PALETTE);
    if sf2_insts.is_empty() {
        return Err(format!(
            "SoundFont '{}' não possui presets carregáveis",
            sf2_path.display()
        ));
    }

    let mut next = pkg.clone();
    next.soundfont_name = Some(bank.name().to_string());
    next.instruments.retain(|inst| !inst.name.ends_with(" (SF2)"));
    for (idx, inst) in next.instruments.iter_mut().enumerate() {
        inst.index = idx;
    }
    for mut inst in sf2_insts {
        let new_idx = next.instruments.len();
        inst.index = new_idx;
        inst.name = format!("{} (SF2)", inst.name);
        next.instruments.push(inst);
    }
    Ok(next)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordMode {
    Fingered3To7,
    SingleFinger,
}

impl ChordMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChordMode::Fingered3To7 => "fingered_3_7",
            ChordMode::SingleFinger => "single_finger",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "single_finger" | "single" | "1_tecla" | "1key" | "one_finger" => {
                ChordMode::SingleFinger
            }
            _ => ChordMode::Fingered3To7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordQuality {
    Major,
    Minor,
    Dom7,
    Min7,
    Maj7,
    MinMaj7,
    Dim,
    Dim7,
    Min7b5,
    Aug,
    Sus2,
    Sus4,
    Dom7Sus4,
    Six,
    Min6,
    SixNine,
    Add9,
    MinAdd9,
    Dom9,
    Maj9,
    Min9,
    Dom11,
    Min11,
    Dom13,
    Maj13,
    Min13,
}

impl ChordQuality {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChordQuality::Major => "Major",
            ChordQuality::Minor => "Minor",
            ChordQuality::Dom7 => "Dom7",
            ChordQuality::Min7 => "Min7",
            ChordQuality::Maj7 => "Maj7",
            ChordQuality::MinMaj7 => "MinMaj7",
            ChordQuality::Dim => "Dim",
            ChordQuality::Dim7 => "Dim7",
            ChordQuality::Min7b5 => "Min7b5",
            ChordQuality::Aug => "Aug",
            ChordQuality::Sus2 => "Sus2",
            ChordQuality::Sus4 => "Sus4",
            ChordQuality::Dom7Sus4 => "Dom7Sus4",
            ChordQuality::Six => "Six",
            ChordQuality::Min6 => "Min6",
            ChordQuality::SixNine => "SixNine",
            ChordQuality::Add9 => "Add9",
            ChordQuality::MinAdd9 => "MinAdd9",
            ChordQuality::Dom9 => "Dom9",
            ChordQuality::Maj9 => "Maj9",
            ChordQuality::Min9 => "Min9",
            ChordQuality::Dom11 => "Dom11",
            ChordQuality::Min11 => "Min11",
            ChordQuality::Dom13 => "Dom13",
            ChordQuality::Maj13 => "Maj13",
            ChordQuality::Min13 => "Min13",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "minor" | "m" | "min" => ChordQuality::Minor,
            "dom7" | "7" => ChordQuality::Dom7,
            "min7" | "m7" => ChordQuality::Min7,
            "maj7" => ChordQuality::Maj7,
            "minmaj7" | "mmaj7" => ChordQuality::MinMaj7,
            "dim" => ChordQuality::Dim,
            "dim7" => ChordQuality::Dim7,
            "min7b5" | "m7b5" => ChordQuality::Min7b5,
            "aug" => ChordQuality::Aug,
            "sus2" => ChordQuality::Sus2,
            "sus4" | "sus" => ChordQuality::Sus4,
            "dom7sus4" | "7sus4" => ChordQuality::Dom7Sus4,
            "six" | "6" => ChordQuality::Six,
            "min6" | "m6" => ChordQuality::Min6,
            "sixnine" | "6/9" | "69" => ChordQuality::SixNine,
            "add9" => ChordQuality::Add9,
            "minadd9" | "m(add9)" | "madd9" => ChordQuality::MinAdd9,
            "dom9" | "9" | "7(9)" => ChordQuality::Dom9,
            "maj9" => ChordQuality::Maj9,
            "min9" | "m9" => ChordQuality::Min9,
            "dom11" | "11" => ChordQuality::Dom11,
            "min11" | "m11" => ChordQuality::Min11,
            "dom13" | "13" => ChordQuality::Dom13,
            "maj13" => ChordQuality::Maj13,
            "min13" | "m13" => ChordQuality::Min13,
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
            ChordQuality::MinMaj7 => "mMaj7",
            ChordQuality::Dim => "dim",
            ChordQuality::Dim7 => "dim7",
            ChordQuality::Min7b5 => "m7b5",
            ChordQuality::Aug => "aug",
            ChordQuality::Sus2 => "sus2",
            ChordQuality::Sus4 => "sus4",
            ChordQuality::Dom7Sus4 => "7sus4",
            ChordQuality::Six => "6",
            ChordQuality::Min6 => "m6",
            ChordQuality::SixNine => "6/9",
            ChordQuality::Add9 => "add9",
            ChordQuality::MinAdd9 => "m(add9)",
            ChordQuality::Dom9 => "7(9)",
            ChordQuality::Maj9 => "maj9",
            ChordQuality::Min9 => "m9",
            ChordQuality::Dom11 => "11",
            ChordQuality::Min11 => "m11",
            ChordQuality::Dom13 => "13",
            ChordQuality::Maj13 => "maj13",
            ChordQuality::Min13 => "m13",
        }
    }
}

pub fn format_chord_label(root: u8, quality: ChordQuality) -> String {
    format!("{}{}", NOTE_NAMES[(root % 12) as usize], quality.suffix())
}

pub fn detect_chord_from_notes(notes: &[u8]) -> Option<(u8, ChordQuality, String)> {
    detect_chord_with_mode(notes, ChordMode::SingleFinger)
}

pub fn detect_chord_with_mode(
    notes: &[u8],
    mode: ChordMode,
) -> Option<(u8, ChordQuality, String)> {
    // MIDI keys and pitch classes are different: octave doublings count as
    // physical keys, but must not hide a high seventh/ninth in a lowest-7 slice.
    // Stack-only masks also avoid sorting/allocating on every MIDI event.
    let mut keys = [0_u64; 2];
    let mut pcs = 0_u16;
    let mut lowest = 128_u8;
    for &note in notes {
        if note < 128 {
            keys[note as usize / 64] |= 1_u64 << (note % 64);
            pcs |= 1_u16 << (note % 12);
            lowest = lowest.min(note);
        }
    }
    let key_count = keys[0].count_ones() + keys[1].count_ones();
    if key_count == 0 || (mode == ChordMode::Fingered3To7 && key_count < 3) {
        return None;
    }
    let lowest_pc = lowest % 12;
    let pc_count = pcs.count_ones() as usize;
    if pc_count == 1 {
        let q = ChordQuality::Major;
        return Some((lowest_pc, q, format_chord_label(lowest_pc, q)));
    }

    if pc_count == 2 {
        let other_pc = (pcs & !(1_u16 << lowest_pc)).trailing_zeros() as u8;
        let interval = (other_pc + 12 - lowest_pc) % 12;
        let q = match interval {
            3 => ChordQuality::Minor,
            10 => ChordQuality::Dom7,
            11 => ChordQuality::Maj7,
            5 => ChordQuality::Sus4,
            2 => ChordQuality::Sus2,
            _ => ChordQuality::Major,
        };
        return Some((lowest_pc, q, format_chord_label(lowest_pc, q)));
    }

    // Ordered harmonic templates for 3-to-7-note voicings: (quality, intervals_from_root, priority)
    let templates: &[(ChordQuality, &[u8], i32)] = &[
        // 7-note & 6-note extensions
        (ChordQuality::Dom13, &[0, 2, 4, 5, 7, 9, 10], 14),
        (ChordQuality::Maj13, &[0, 2, 4, 5, 7, 9, 11], 13),
        (ChordQuality::Min13, &[0, 2, 3, 5, 7, 9, 10], 13),
        (ChordQuality::Dom13, &[0, 2, 4, 7, 9, 10], 12),
        (ChordQuality::Maj13, &[0, 2, 4, 7, 9, 11], 11),
        (ChordQuality::Min13, &[0, 2, 3, 7, 9, 10], 11),
        (ChordQuality::Dom11, &[0, 2, 4, 5, 7, 10], 12),
        (ChordQuality::Min11, &[0, 2, 3, 5, 7, 10], 12),
        // 5-note extensions (7/9, maj9, m9, 6/9, 13, 11)
        (ChordQuality::Dom9, &[0, 2, 4, 7, 10], 13),
        (ChordQuality::Maj9, &[0, 2, 4, 7, 11], 12),
        (ChordQuality::Min9, &[0, 2, 3, 7, 10], 13),
        (ChordQuality::SixNine, &[0, 2, 4, 7, 9], 10),
        (ChordQuality::Dom13, &[0, 4, 7, 9, 10], 11),
        (ChordQuality::Maj13, &[0, 4, 7, 9, 11], 10),
        (ChordQuality::Min13, &[0, 3, 7, 9, 10], 10),
        (ChordQuality::Dom11, &[0, 4, 5, 7, 10], 10),
        (ChordQuality::Min11, &[0, 3, 5, 7, 10], 10),
        // 4-note voicings (including shell 7/9 without 5th and standard 7ths/add9/6)
        (ChordQuality::Dom9, &[0, 2, 4, 10], 11),
        (ChordQuality::Maj9, &[0, 2, 4, 11], 10),
        (ChordQuality::Min9, &[0, 2, 3, 10], 11),
        (ChordQuality::Dom7, &[0, 4, 7, 10], 12),
        (ChordQuality::Min7, &[0, 3, 7, 10], 12),
        (ChordQuality::Maj7, &[0, 4, 7, 11], 11),
        (ChordQuality::MinMaj7, &[0, 3, 7, 11], 10),
        (ChordQuality::Dom7Sus4, &[0, 5, 7, 10], 10),
        (ChordQuality::Dim7, &[0, 3, 6, 9], 9),
        (ChordQuality::Min7b5, &[0, 3, 6, 10], 9),
        (ChordQuality::Six, &[0, 4, 7, 9], 9),
        (ChordQuality::Min6, &[0, 3, 7, 9], 9),
        (ChordQuality::Add9, &[0, 2, 4, 7], 9),
        (ChordQuality::MinAdd9, &[0, 2, 3, 7], 9),
        (ChordQuality::Dom13, &[0, 4, 9, 10], 8),
        // 3-note triads & shell voicings
        (ChordQuality::Minor, &[0, 3, 7], 11),
        (ChordQuality::Major, &[0, 4, 7], 11),
        (ChordQuality::Dim, &[0, 3, 6], 9),
        (ChordQuality::Aug, &[0, 4, 8], 8),
        (ChordQuality::Sus4, &[0, 5, 7], 9),
        (ChordQuality::Sus2, &[0, 2, 7], 8),
        (ChordQuality::Dom7, &[0, 4, 10], 6),
        (ChordQuality::Maj7, &[0, 4, 11], 6),
        (ChordQuality::Min7, &[0, 3, 10], 6),
    ];

    let mut best: Option<(i32, u8, ChordQuality)> = None;
    for &(q, tpl, prio) in templates {
        if pc_count < tpl.len() {
            continue;
        }
        let template_mask = tpl.iter().fold(0_u16, |mask, &iv| mask | (1_u16 << iv));
        for cand_root in 0..12_u8 {
            if pcs & (1_u16 << cand_root) == 0 {
                continue;
            }
            let rotated = (((template_mask as u32) << cand_root)
                | ((template_mask as u32) >> (12 - cand_root))) as u16 & 0x0fff;
            if pcs & rotated == rotated {
                let extra = (pc_count - tpl.len()) as i32;
                let mut score = (tpl.len() as i32) * 20 - extra * 28 + prio;
                if extra == 0 {
                    score += 45;
                }
                if cand_root == lowest_pc {
                    // Extended voicings often put a third/seventh in the bass.
                    // Prefer their explicit harmonic template over relabelling
                    // an inverted m9 as the relative major's incomplete maj13.
                    score += if tpl.len() >= 5 { 2 } else { 18 };
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

    // Fallback: derive best quality relative to lowest pressed bass pitch class
    let has = |iv: u8| pcs & (1_u16 << ((lowest_pc + iv) % 12)) != 0;
    let q = if has(4) && has(10) && has(2) {
        ChordQuality::Dom9
    } else if has(3) && has(10) && has(2) {
        ChordQuality::Min9
    } else if has(4) && has(11) && has(2) {
        ChordQuality::Maj9
    } else if has(4) && has(2) {
        ChordQuality::Add9
    } else if has(3) && has(2) {
        ChordQuality::MinAdd9
    } else if has(3) && has(10) {
        ChordQuality::Min7
    } else if has(4) && has(10) {
        ChordQuality::Dom7
    } else if has(4) && has(11) {
        ChordQuality::Maj7
    } else if has(3) {
        ChordQuality::Minor
    } else if has(5) && !has(4) {
        ChordQuality::Sus4
    } else {
        ChordQuality::Major
    };
    Some((lowest_pc, q, format_chord_label(lowest_pc, q)))
}

#[inline]
fn drum_choke_group(midi_note: u8) -> u8 {
    match midi_note {
        42 | 44 | 46 => 1, // Hi-Hat (Closed / Pedal / Open)
        80 | 81 => 2,      // Triangle (Mute / Open)
        71 | 72 => 3,      // Whistle
        73 | 74 => 4,      // Guiro
        78 | 79 => 5,      // Cuica
        86 | 87 => 6,      // Surdo
        _ => 0,
    }
}

#[inline]
fn catmull_rom(y0: f32, y1: f32, y2: f32, y3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * y1)
        + (-y0 + y2) * t
        + (2.0 * y0 - 5.0 * y1 + 4.0 * y2 - y3) * t2
        + (-y0 + 3.0 * y1 - 3.0 * y2 + y3) * t3)
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
    choke_releasing: bool,
    is_drum: bool,
    choke_group: u8,
    inst_index: usize,
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
    is_bass: bool,
) -> u8 {
    // Tracks inside `.Music` are authored against a C Major reference (pitch class 0 = C)
    let mut root_shift = detected_chord_root as i16;
    if root_shift > 6 {
        root_shift -= 12;
    }
    let pc = (note % 12) as i16;
    let mapped_iv = if is_bass {
        match detected_chord_quality {
            ChordQuality::Minor
            | ChordQuality::Min7
            | ChordQuality::MinAdd9
            | ChordQuality::Min9
            | ChordQuality::Min11
            | ChordQuality::Min13 => match pc {
                4 => 3,
                11 => 10,
                _ => pc,
            },
            ChordQuality::MinMaj7 => match pc {
                4 => 3,
                _ => pc,
            },
            ChordQuality::Min6 => match pc {
                4 => 3,
                11 => 9,
                _ => pc,
            },
            ChordQuality::Dim => match pc {
                4 => 3,
                7 => 6,
                _ => pc,
            },
            ChordQuality::Dim7 => match pc {
                4 => 3,
                7 => 6,
                11 => 9,
                _ => pc,
            },
            ChordQuality::Min7b5 => match pc {
                4 => 3,
                7 => 6,
                11 => 10,
                _ => pc,
            },
            ChordQuality::Aug => match pc {
                7 => 8,
                _ => pc,
            },
            ChordQuality::Sus2 => match pc {
                4 => 2,
                _ => pc,
            },
            ChordQuality::Sus4 => match pc {
                4 => 5,
                _ => pc,
            },
            ChordQuality::Dom7Sus4 => match pc {
                4 => 5,
                11 => 10,
                _ => pc,
            },
            ChordQuality::Dom7
            | ChordQuality::Dom9
            | ChordQuality::Dom11
            | ChordQuality::Dom13 => match pc {
                11 => 10,
                _ => pc,
            },
            ChordQuality::Six | ChordQuality::SixNine => match pc {
                11 => 9,
                _ => pc,
            },
            ChordQuality::Major
            | ChordQuality::Maj7
            | ChordQuality::Add9
            | ChordQuality::Maj9
            | ChordQuality::Maj13 => pc,
        }
    } else {
        match detected_chord_quality {
            ChordQuality::Major => pc,
            ChordQuality::Minor => match pc {
                4 => 3,
                11 => 10,
                _ => pc,
            },
            ChordQuality::Dom7 => match pc {
                0 => -2,
                11 => 10,
                _ => pc,
            },
            ChordQuality::Min7 => match pc {
                0 => -2,
                4 => 3,
                11 => 10,
                _ => pc,
            },
            ChordQuality::Maj7 => match pc {
                0 => -1,
                _ => pc,
            },
            ChordQuality::MinMaj7 => match pc {
                0 => -1,
                4 => 3,
                _ => pc,
            },
            ChordQuality::Dim => match pc {
                4 => 3,
                7 => 6,
                _ => pc,
            },
            ChordQuality::Dim7 => match pc {
                0 => -3,
                4 => 3,
                7 => 6,
                11 => 9,
                _ => pc,
            },
            ChordQuality::Min7b5 => match pc {
                0 => -2,
                4 => 3,
                7 => 6,
                11 => 10,
                _ => pc,
            },
            ChordQuality::Aug => match pc {
                7 => 8,
                _ => pc,
            },
            ChordQuality::Sus2 => match pc {
                4 => 2,
                _ => pc,
            },
            ChordQuality::Sus4 => match pc {
                4 => 5,
                _ => pc,
            },
            ChordQuality::Dom7Sus4 => match pc {
                0 => -2,
                4 => 5,
                11 => 10,
                _ => pc,
            },
            ChordQuality::Six => match pc {
                0 => -3,
                11 => 9,
                _ => pc,
            },
            ChordQuality::Min6 => match pc {
                0 => -3,
                4 => 3,
                11 => 9,
                _ => pc,
            },
            ChordQuality::Add9 => match pc {
                0 => 2,
                _ => pc,
            },
            ChordQuality::MinAdd9 => match pc {
                0 => 2,
                4 => 3,
                _ => pc,
            },
            ChordQuality::SixNine => match pc {
                0 => 2,
                7 => 9,
                11 => 9,
                _ => pc,
            },
            ChordQuality::Dom9 => match pc {
                0 => 2,
                7 => 10,
                11 => 10,
                _ => pc,
            },
            ChordQuality::Maj9 => match pc {
                0 => 2,
                7 => 11,
                _ => pc,
            },
            ChordQuality::Min9 => match pc {
                0 => 2,
                4 => 3,
                7 => 10,
                11 => 10,
                _ => pc,
            },
            ChordQuality::Dom11 => match pc {
                0 => -2,
                4 => 5,
                11 => 10,
                _ => pc,
            },
            ChordQuality::Min11 => match pc {
                0 => -2,
                4 => 3,
                7 => 5,
                11 => 10,
                _ => pc,
            },
            ChordQuality::Dom13 => match pc {
                0 => -2,
                7 => 9,
                11 => 10,
                _ => pc,
            },
            ChordQuality::Maj13 => match pc {
                0 => -1,
                7 => 9,
                _ => pc,
            },
            ChordQuality::Min13 => match pc {
                0 => -2,
                4 => 3,
                7 => 9,
                11 => 10,
                _ => pc,
            },
        }
    };
    let adjusted = note as i16 + (mapped_iv - pc) + root_shift;
    adjusted.clamp(24, 108) as u8
}

pub fn chord_quality_to_target_spec(
    root: u8,
    quality: ChordQuality,
) -> crate::yamaha_sty::TargetChordSpec {
    let (yamaha_bit, third_iv, fifth_iv, seventh_iv, add9, is_major_family) = match quality {
        ChordQuality::Major => (0, 4, 7, 11, false, true),
        ChordQuality::Six => (1, 4, 7, 9, false, true),
        ChordQuality::Maj7 => (2, 4, 7, 11, false, true),
        ChordQuality::Add9 => (4, 4, 7, 11, true, true),
        ChordQuality::Maj9 => (5, 4, 7, 11, true, true),
        ChordQuality::SixNine => (6, 4, 7, 9, true, true),
        ChordQuality::Aug => (7, 4, 8, 11, false, true),
        ChordQuality::Minor => (8, 3, 7, 10, false, false),
        ChordQuality::Min6 => (9, 3, 7, 9, false, false),
        ChordQuality::MinAdd9 => (10, 3, 7, 10, true, false),
        ChordQuality::Min7b5 => (11, 3, 6, 10, false, false),
        ChordQuality::Min7 => (12, 3, 7, 10, false, false),
        ChordQuality::Min9 => (13, 3, 7, 10, true, false),
        ChordQuality::Min11 => (14, 3, 7, 10, true, false),
        ChordQuality::MinMaj7 => (15, 3, 7, 11, false, false),
        ChordQuality::Dim => (17, 3, 6, 10, false, false),
        ChordQuality::Dim7 => (18, 3, 6, 9, false, false),
        ChordQuality::Dom7 => (19, 4, 7, 10, false, true),
        ChordQuality::Dom7Sus4 => (20, 5, 7, 10, false, true),
        ChordQuality::Dom9 => (22, 4, 7, 10, true, true),
        ChordQuality::Dom11 => (23, 4, 7, 10, true, true),
        ChordQuality::Dom13 => (24, 4, 7, 10, true, true),
        ChordQuality::Maj13 => (5, 4, 7, 11, true, true),
        ChordQuality::Min13 => (13, 3, 7, 10, true, false),
        ChordQuality::Sus4 => (32, 5, 7, 10, false, true),
        ChordQuality::Sus2 => (33, 2, 7, 10, true, true),
    };
    crate::yamaha_sty::TargetChordSpec {
        root_pc: root % 12,
        yamaha_bit,
        third_iv,
        fifth_iv,
        seventh_iv,
        add9,
        is_major_family,
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_section_events(
    sec: &RhythmSection,
    instruments: &[LoadedInstrument],
    track_volumes: &[f32; 8],
    track_instrument_overrides: &[Option<usize>; 8],
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
    let target_spec = chord_quality_to_target_spec(detected_chord_root, detected_chord_quality);
    for track in &sec.tracks {
        let slot = track.track_slot.min(7);
        let effective_inst_idx = track_instrument_overrides[slot]
            .filter(|&idx| idx < instruments.len())
            .unwrap_or(track.instrument_index);
        let inst_is_drum = instruments
            .get(effective_inst_idx)
            .map(|i| i.is_drumkit)
            .unwrap_or(false);
        let effective_is_drum = track.is_drum || inst_is_drum;
        let is_bass = track.track_slot == 2;
        let vol = track.volume * track_volumes[slot];
        if vol <= 0.002 {
            continue;
        }
        if let Some(ref rule) = track.casm_rule {
            if !rule.is_active(target_spec) {
                continue;
            }
        }
        for ev in &track.events {
            let t = ev.start_tick as f64;
            if t >= win_start && t < win_end {
                let target_note = if let Some(ref rule) = track.casm_rule {
                    if effective_is_drum || slot <= 1 {
                        ev.midi_note
                    } else {
                        let shifted = (ev.midi_note as i16
                            + track.key_move as i16
                            + (track.octave_shift * 12) as i16)
                            .clamp(0, 127) as u8;
                        match rule.transpose_note(shifted, target_spec, effective_is_drum, is_bass)
                        {
                            Some(n) => n,
                            None => continue,
                        }
                    }
                } else if effective_is_drum || slot <= 1 {
                    ev.midi_note
                } else {
                    let shifted = (ev.midi_note as i16
                        + track.key_move as i16
                        + (track.octave_shift * 12) as i16)
                        .clamp(12, 115) as u8;
                    transpose_note_for_chord(
                        shifted,
                        detected_chord_root,
                        detected_chord_quality,
                        is_bass,
                    )
                };
                let dur_samples = if effective_is_drum {
                    None
                } else {
                    let dur_sec =
                        (ev.duration_ticks.max(30) as f64 / 500.0) * (60.0 / bpm.max(40.0) as f64);
                    Some(((dur_sec * sr as f64) as usize).max(256))
                };
                out.push(PendingTrigger {
                    inst_idx: effective_inst_idx,
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
    pub chord_mode: ChordMode,
    pub track_volumes: [f32; 8],
    pub track_instrument_overrides: [Option<usize>; 8],
    pub user_imported_instruments: Vec<LoadedInstrument>,
    pub master_gain: f32,
    current_tick: f64,
    default_upper1_instrument: usize,
    held_keys: [bool; 128],
    held_lower_notes: Vec<u8>,
    held_upper_notes: Vec<u8>,
    pending_triggers: Vec<PendingTrigger>,
    voices: Vec<SamplerVoice>,
}

fn active_style_diagnostics<'a>(
    pkg: &'a LoadedMusicPackage,
    active_style_index: usize,
) -> (&'a [String], bool, bool, Option<&'a str>) {
    if let Some(st) = pkg.styles.get(active_style_index) {
        (
            st.sty_warnings.as_slice(),
            st.has_audio_style_chunks,
            st.uses_expansion_kit,
            st.sty_diagnostics_summary.as_deref(),
        )
    } else {
        (
            pkg.sty_warnings.as_slice(),
            pkg.has_audio_style_chunks,
            pkg.uses_expansion_kit,
            pkg.sty_diagnostics_summary.as_deref(),
        )
    }
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
            split_key: 54,
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
            chord_mode: ChordMode::Fingered3To7,
            track_volumes: [1.0; 8],
            track_instrument_overrides: [None; 8],
            user_imported_instruments: Vec::new(),
            master_gain: 0.85,
            current_tick: 0.0,
            default_upper1_instrument: 0,
            held_keys: [false; 128],
            held_lower_notes: Vec::with_capacity(128),
            held_upper_notes: Vec::with_capacity(128),
            pending_triggers: Vec::with_capacity(64),
            voices: Vec::with_capacity(MAX_SAMPLER_VOICES),
        }
    }

    pub fn has_loaded_package(&self) -> bool {
        self.package.is_some()
    }

    pub fn load_package(&mut self, mut pkg: Arc<LoadedMusicPackage>) {
        self.all_notes_off();
        self.rhythm_playing = false;
        self.current_tick = 0.0;
        self.pending_section = None;
        self.tempo_locked = false;
        self.track_instrument_overrides = [None; 8];

        let prev_user_upper1_name = self.package.as_ref().and_then(|old_pkg| {
            old_pkg.instruments.get(self.upper1_instrument).and_then(|inst| {
                self.user_imported_instruments
                    .iter()
                    .find(|u| u.name == inst.name)
                    .map(|u| u.name.clone())
            })
        });

        let package_instrument_count = pkg.instruments.len();
        if !self.user_imported_instruments.is_empty() {
            let mut pkg_mut = (*pkg).clone();
            for user_inst in &self.user_imported_instruments {
                if !pkg_mut.instruments.iter().any(|i| i.name == user_inst.name) {
                    let mut cloned = user_inst.clone();
                    cloned.index = pkg_mut.instruments.len();
                    pkg_mut.instruments.push(cloned);
                }
            }
            pkg = Arc::new(pkg_mut);
        }

        // Auto-pick the best playable melodic lead instrument (prefer Accordion/Bandoneon/Sanfona/Lead/Guitar/Strings with samples)
        let mut playable: Vec<&LoadedInstrument> = pkg
            .instruments
            .iter()
            .take(package_instrument_count)
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
            let tier = if up.contains("BANDONEON")
                || up.contains("ACCORD")
                || up.contains("ACORD")
                || up.contains("SANFONA")
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

        let preserved_user_upper1 = prev_user_upper1_name.and_then(|uname| {
            pkg.instruments
                .iter()
                .find(|i| {
                    i.name == uname
                        && !i.is_drumkit
                        && i.zones.iter().any(|z| !z.pcm_l.is_empty())
                })
                .map(|i| i.index)
        });
        let sty_ots_upper1 = if pkg.source_format == "sty" || pkg.source_format == "sty_pack" {
            pkg.ots_presets.first().and_then(|ots| {
                pkg.instruments
                    .get(ots.upper1_instrument)
                    .filter(|i| !i.is_drumkit && i.zones.iter().any(|z| !z.pcm_l.is_empty()))
                    .map(|i| i.index)
            })
        } else {
            None
        };

        self.default_upper1_instrument = sty_ots_upper1
            .or_else(|| playable.first().map(|i| i.index))
            .unwrap_or(0);
        self.upper1_instrument = preserved_user_upper1.unwrap_or(self.default_upper1_instrument);
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
        let ots_presets = pkg
            .styles
            .get(self.active_style_index)
            .filter(|st| !st.ots_presets.is_empty())
            .map(|st| st.ots_presets.clone())
            .unwrap_or_else(|| pkg.ots_presets.clone());
        let (sty_warnings, has_audio_style_chunks, uses_expansion_kit, sty_diagnostics_summary) =
            active_style_diagnostics(pkg, self.active_style_index);
        MusicPackageSummary {
            loaded: true,
            file_name: pkg.file_name.clone(),
            file_path: pkg.file_path.clone(),
            version: pkg.version.clone(),
            source_format: pkg.source_format.clone(),
            soundfont_name: pkg.soundfont_name.clone(),
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
            chord_mode: self.chord_mode.as_str().to_string(),
            track_volumes: self.track_volumes,
            track_instruments: self.track_instrument_overrides,
            ots_presets,
            beat_position: (self.current_tick / sec_len).clamp(0.0, 1.0) as f32,
            sty_warnings: sty_warnings.to_vec(),
            has_audio_style_chunks,
            uses_expansion_kit,
            sty_diagnostics_summary: sty_diagnostics_summary.map(str::to_string),
        }
    }

    pub fn sync_dynamic_summary(&self, dst: &mut MusicPackageSummary) {
        let Some(ref pkg) = self.package else {
            return;
        };
        let sec_len = self
            .active_section_ref()
            .map(|s| s.length_ticks.max(1) as f64)
            .unwrap_or(4000.0);
        dst.loaded = true;
        if dst.source_format != pkg.source_format {
            dst.source_format.clear();
            dst.source_format.push_str(&pkg.source_format);
        }
        match (&mut dst.soundfont_name, &pkg.soundfont_name) {
            (Some(d), Some(s)) => {
                if d != s {
                    d.clear();
                    d.push_str(s);
                }
            }
            (None, None) => {}
            (slot, src) => *slot = src.clone(),
        }
        dst.active_style_index = self.active_style_index;
        let (src_warnings, src_audio_chunks, src_expansion, src_diag_summary) =
            active_style_diagnostics(pkg, self.active_style_index);
        if dst.sty_warnings.as_slice() != src_warnings {
            dst.sty_warnings.clear();
            dst.sty_warnings.extend(src_warnings.iter().cloned());
        }
        dst.has_audio_style_chunks = src_audio_chunks;
        dst.uses_expansion_kit = src_expansion;
        match (&mut dst.sty_diagnostics_summary, src_diag_summary) {
            (Some(d), Some(s)) => {
                if d != s {
                    d.clear();
                    d.push_str(s);
                }
            }
            (None, None) => {}
            (Some(_), None) => dst.sty_diagnostics_summary = None,
            (slot @ None, Some(s)) => *slot = Some(s.to_string()),
        }
        if dst.active_section != self.active_section {
            dst.active_section.clear();
            dst.active_section.push_str(&self.active_section);
        }
        match (&mut dst.pending_section, &self.pending_section) {
            (Some(d), Some(s)) => {
                if d != s {
                    d.clear();
                    d.push_str(s);
                }
            }
            (None, None) => {}
            (slot, src) => *slot = src.clone(),
        }
        dst.upper1_instrument = self.upper1_instrument;
        dst.upper2_instrument = self.upper2_instrument;
        dst.lower_instrument = self.lower_instrument;
        dst.split_key = self.split_key;
        dst.split_enabled = self.split_enabled;
        dst.chord_scan_lower = self.chord_scan_lower;
        dst.chord_scan_upper = self.chord_scan_upper;
        dst.upper_octave = self.upper_octave;
        dst.transpose = self.transpose;
        dst.bpm = self.bpm;
        dst.tempo_locked = self.tempo_locked;
        dst.rhythm_playing = self.rhythm_playing;
        dst.sync_start = self.sync_start;
        dst.auto_fill = self.auto_fill;
        dst.detected_chord_root = self.detected_chord_root;
        let q_str = self.detected_chord_quality.as_str();
        if dst.detected_chord_quality != q_str {
            dst.detected_chord_quality.clear();
            dst.detected_chord_quality.push_str(q_str);
        }
        if dst.detected_chord != self.detected_chord_label {
            dst.detected_chord.clear();
            dst.detected_chord.push_str(&self.detected_chord_label);
        }
        let cm_str = self.chord_mode.as_str();
        if dst.chord_mode != cm_str {
            dst.chord_mode.clear();
            dst.chord_mode.push_str(cm_str);
        }
        dst.track_volumes = self.track_volumes;
        dst.track_instruments = self.track_instrument_overrides;
        dst.beat_position = (self.current_tick / sec_len).clamp(0.0, 1.0) as f32;
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
            if section_name.starts_with("Intro") {
                // Explicit Intro selection is a play command even with sync-start armed.
                self.rhythm_playing = true;
            }
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
        if !playing {
            self.pending_section = None;
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

    pub fn add_imported_instrument(
        &mut self,
        mut inst: LoadedInstrument,
        assign_upper1: bool,
    ) -> usize {
        if let Some(existing) = self
            .user_imported_instruments
            .iter_mut()
            .find(|u| u.name == inst.name)
        {
            *existing = inst.clone();
        } else {
            self.user_imported_instruments.push(inst.clone());
        }

        let mut next_pkg = match self.package.as_ref() {
            Some(pkg) => (**pkg).clone(),
            None => LoadedMusicPackage {
                file_name: "Custom Instruments".to_string(),
                file_path: "custom://instruments".to_string(),
                version: "1.4".to_string(),
                source_format: "music".to_string(),
                soundfont_name: None,
                instruments: Vec::new(),
                styles: Vec::new(),
                ots_presets: Vec::new(),
                source_paths: Vec::new(),
                sty_warnings: Vec::new(),
                has_audio_style_chunks: false,
                uses_expansion_kit: false,
                sty_diagnostics_summary: None,
            },
        };
        let target_idx = if let Some(existing_idx) = next_pkg
            .instruments
            .iter()
            .position(|i| i.name == inst.name)
        {
            inst.index = existing_idx;
            next_pkg.instruments[existing_idx] = inst;
            existing_idx
        } else {
            let new_idx = next_pkg.instruments.len();
            inst.index = new_idx;
            next_pkg.instruments.push(inst);
            new_idx
        };
        self.package = Some(Arc::new(next_pkg));
        if assign_upper1 {
            self.upper1_instrument = target_idx;
            for v in &mut self.voices {
                if !v.from_rhythm {
                    v.releasing = true;
                }
            }
        }
        target_idx
    }

    pub fn set_track_instrument(&mut self, track_slot: usize, instrument_index: Option<usize>) {
        if track_slot >= 8 {
            return;
        }
        let max_idx = self
            .package
            .as_ref()
            .map(|p| p.instruments.len())
            .unwrap_or(0);
        self.track_instrument_overrides[track_slot] = instrument_index.filter(|&i| i < max_idx);
    }

    /// Restore the active style's first OTS (or the package's original lead)
    /// and authored track instruments, without resetting performance settings.
    pub fn reset_instruments(&mut self) {
        let Some(pkg) = self.package.as_ref() else {
            return;
        };
        let ots = pkg.styles.get(self.active_style_index)
            .filter(|style| !style.ots_presets.is_empty())
            .map(|style| style.ots_presets.as_slice())
            .unwrap_or(pkg.ots_presets.as_slice())
            .first();
        let playable = |index: usize| pkg.instruments.get(index)
            .map(|inst| !inst.is_drumkit && inst.zones.iter().any(|zone| !zone.pcm_l.is_empty()))
            .unwrap_or(false);
        let upper1 = ots.map(|preset| preset.upper1_instrument)
            .filter(|&index| playable(index))
            .unwrap_or(self.default_upper1_instrument);
        let upper2 = ots.and_then(|preset| preset.upper2_instrument)
            .filter(|&index| playable(index));
        let lower = ots.and_then(|preset| preset.lower_instrument)
            .filter(|&index| playable(index));
        let keyboard_changed = (self.upper1_instrument, self.upper2_instrument, self.lower_instrument)
            != (upper1, upper2, lower);
        for voice in &mut self.voices {
            if (!voice.from_rhythm && keyboard_changed)
                || (voice.from_rhythm && self.track_instrument_overrides.contains(&Some(voice.inst_index)))
            {
                voice.releasing = true;
            }
        }
        self.upper1_instrument = upper1;
        self.upper2_instrument = upper2;
        self.lower_instrument = lower;
        self.track_instrument_overrides = [None; 8];
    }

    pub fn apply_ots(&mut self, ots_index: usize) {
        let Some(ref pkg) = self.package else {
            return;
        };
        let ots_list = pkg
            .styles
            .get(self.active_style_index)
            .filter(|st| !st.ots_presets.is_empty())
            .map(|st| st.ots_presets.as_slice())
            .unwrap_or(pkg.ots_presets.as_slice());
        let Some(ots) = ots_list
            .iter()
            .find(|p| p.index == ots_index)
            .or_else(|| ots_list.get(ots_index))
        else {
            return;
        };
        let max_idx = pkg.instruments.len();
        if ots.upper1_instrument < max_idx {
            self.upper1_instrument = ots.upper1_instrument;
        }
        self.upper2_instrument = ots.upper2_instrument.filter(|&i| i < max_idx);
        self.lower_instrument = ots.lower_instrument.filter(|&i| i < max_idx);
        for v in &mut self.voices {
            if !v.from_rhythm {
                v.releasing = true;
            }
        }
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

    #[allow(clippy::too_many_arguments)]
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
        chord_mode: Option<&str>,
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
        if let Some(cm) = chord_mode {
            self.chord_mode = ChordMode::from_str(cm);
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
                .find(|z| {
                    !z.pcm_l.is_empty()
                        && z.root_key == midi_note
                        && velocity >= z.vel_low
                        && velocity <= z.vel_high
                })
                .or_else(|| {
                    inst.zones.iter().find(|z| {
                        !z.pcm_l.is_empty()
                            && midi_note >= z.key_low
                            && midi_note <= z.key_high
                            && velocity >= z.vel_low
                            && velocity <= z.vel_high
                    })
                })
                .or_else(|| {
                    inst.zones
                        .iter()
                        .find(|z| !z.pcm_l.is_empty() && z.root_key == midi_note)
                })
                .or_else(|| {
                    inst.zones.iter().find(|z| {
                        !z.pcm_l.is_empty() && midi_note >= z.key_low && midi_note <= z.key_high
                    })
                })
        } else {
            inst.zones
                .iter()
                .find(|z| {
                    !z.pcm_l.is_empty()
                        && z.root_key == midi_note
                        && velocity >= z.vel_low
                        && velocity <= z.vel_high
                })
                .or_else(|| {
                    inst.zones.iter().find(|z| {
                        !z.pcm_l.is_empty()
                            && midi_note >= z.key_low
                            && midi_note <= z.key_high
                            && velocity >= z.vel_low
                            && velocity <= z.vel_high
                    })
                })
                .or_else(|| {
                    inst.zones
                        .iter()
                        .find(|z| !z.pcm_l.is_empty() && z.root_key == midi_note)
                })
                .or_else(|| {
                    inst.zones.iter().find(|z| {
                        !z.pcm_l.is_empty() && midi_note >= z.key_low && midi_note <= z.key_high
                    })
                })
                .or_else(|| {
                    inst.zones
                        .iter()
                        .filter(|z| !z.pcm_l.is_empty())
                        .min_by_key(|z| z.root_key.abs_diff(midi_note))
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
        let vel_gain = (velocity as f32 / 127.0).powf(1.55);
        let total_gain = vel_gain * inst.volume * z.gain * gain_scale;
        if total_gain <= 0.0005 {
            return;
        }

        let is_drum = inst.is_drumkit;
        let choke_group = if is_drum {
            drum_choke_group(midi_note)
        } else {
            0
        };
        if is_drum {
            for v in &mut self.voices {
                if v.is_drum
                    && !v.choke_releasing
                    && ((choke_group != 0 && v.choke_group == choke_group)
                        || (v.inst_index == inst_index && v.trigger_key == trigger_key))
                {
                    v.choke_releasing = true;
                }
            }
        }

        let total_pan = (inst.pan + z.pan + pan_offset).clamp(-100, 100);
        let pan_l = ((100 - total_pan).clamp(0, 100) as f32) / 100.0;
        let pan_r = ((100 + total_pan).clamp(0, 100) as f32) / 100.0;

        if self.voices.len() >= MAX_SAMPLER_VOICES {
            if let Some(pos) = self
                .voices
                .iter()
                .enumerate()
                .filter(|(_, v)| v.releasing || v.choke_releasing)
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
            loop_enabled: z.loop_enabled && !is_drum,
            loop_mode: z.loop_mode,
            loop_dir: 1,
            gain: total_gain,
            pan_l,
            pan_r,
            env: 1.0,
            attack_pos: if is_drum { 48 } else { 0 },
            releasing: false,
            choke_releasing: false,
            is_drum,
            choke_group,
            inst_index,
            remaining_samples: if is_drum { None } else { duration_samples },
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
                if let Some((root, q, label)) =
                    detect_chord_with_mode(&self.held_lower_notes, self.chord_mode)
                {
                    self.detected_chord_root = root;
                    self.detected_chord_quality = q;
                    self.detected_chord_label = label;
                    if self.sync_start && !self.rhythm_playing {
                        self.rhythm_playing = true;
                        self.current_tick = 0.0;
                    }
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
            if let Some((root, q, label)) =
                detect_chord_with_mode(&self.held_upper_notes, self.chord_mode)
            {
                self.detected_chord_root = root;
                self.detected_chord_quality = q;
                self.detected_chord_label = label;
                if self.sync_start && !self.rhythm_playing {
                    self.rhythm_playing = true;
                    self.current_tick = 0.0;
                }
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
        let was_lower = self.held_lower_notes.contains(&note);
        let was_upper = self.held_upper_notes.contains(&note);
        self.held_keys[note as usize] = false;
        self.held_lower_notes.retain(|&n| n != note);
        self.held_upper_notes.retain(|&n| n != note);

        let scanned = if was_lower && self.chord_scan_lower {
            Some(self.held_lower_notes.as_slice())
        } else if was_upper && self.chord_scan_upper
            && (self.held_upper_notes.len() >= 3 || !self.chord_scan_lower || !self.split_enabled)
        {
            Some(self.held_upper_notes.as_slice())
        } else {
            None
        };
        if let Some((root, quality, label)) = scanned
            .and_then(|notes| detect_chord_with_mode(notes, self.chord_mode))
        {
            self.detected_chord_root = root;
            self.detected_chord_quality = quality;
            self.detected_chord_label = label;
        }

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

    pub fn fill_active_notes(&self, out: &mut Vec<u8>) {
        out.clear();
        for (n, &on) in self.held_keys.iter().enumerate() {
            if on {
                out.push(n as u8);
            }
        }
    }

    pub fn active_notes(&self) -> Vec<u8> {
        let mut notes = Vec::with_capacity(8);
        self.fill_active_notes(&mut notes);
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

            let mut to_trigger = std::mem::take(&mut self.pending_triggers);
            to_trigger.clear();
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
                        // Intros are quantized to the next quarter note, not the
                        // end of a possibly multi-bar variation. Keep authored
                        // intro event positions (including leading rests) intact.
                        let boundary = if self.pending_section.as_deref()
                            .map(|name| name.starts_with("Intro")).unwrap_or(false)
                        {
                            (((prev_tick / 500.0).floor() + 1.0) * 500.0).min(sec_len)
                        } else {
                            sec_len
                        };
                        wrapped = next_tick >= boundary;
                        wrap_tick = if wrapped { next_tick - boundary } else { 0.0 };

                        if wrapped {
                            collect_section_events(
                                sec,
                                &pkg.instruments,
                                &self.track_volumes,
                                &self.track_instrument_overrides,
                                self.detected_chord_root,
                                self.detected_chord_quality,
                                self.bpm,
                                self.sr,
                                prev_tick,
                                boundary,
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
                                    &self.track_instrument_overrides,
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
                                &self.track_instrument_overrides,
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

            for trig in to_trigger.drain(..) {
                self.trigger_instrument_note(
                    trig.inst_idx,
                    trig.midi_note,
                    trig.midi_note,
                    trig.velocity,
                    trig.vol * 0.64,
                    trig.pan,
                    trig.fine_tune,
                    true,
                    trig.dur_samples,
                );
            }
            self.pending_triggers = to_trigger;
        }

        // Render all active polyphonic sampler voices
        let release_step = 1.0 / (0.08 * self.sr).max(64.0);
        let choke_step = 1.0 / (0.010 * self.sr).max(32.0);
        let master = self.master_gain;
        let mut i = 0;
        while i < self.voices.len() {
            let v = &mut self.voices[i];
            let pcm_len = v.pcm_l.len();
            if pcm_len < 2 {
                self.voices.swap_remove(i);
                continue;
            }
            let is_mono = Arc::ptr_eq(&v.pcm_l, &v.pcm_r) || v.pcm_r.len() < pcm_len;
            let pcm_l = v.pcm_l.as_slice();
            let pcm_r = v.pcm_r.as_slice();
            let gain_l = v.gain * master * v.pan_l;
            let gain_r = v.gain * master * v.pan_r;
            let has_loop = v.loop_enabled && v.loop_end > v.loop_start + 8;
            let loop_start_f = v.loop_start as f64;
            let loop_end_f = v.loop_end as f64;
            let loop_span_f = v.loop_end.saturating_sub(v.loop_start) as f64;
            let mut alive = true;
            for f in 0..frames {
                if let Some(ref mut rem) = v.remaining_samples {
                    if *rem == 0 {
                        v.releasing = true;
                    } else {
                        *rem -= 1;
                    }
                }
                if v.choke_releasing {
                    v.env -= choke_step;
                    if v.env <= 0.0005 {
                        alive = false;
                        break;
                    }
                } else if v.releasing {
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

                if has_loop {
                    if v.loop_mode == 3 {
                        if v.loop_dir > 0 && v.pos >= loop_end_f {
                            v.pos = (loop_end_f - (v.pos - loop_end_f))
                                .clamp(loop_start_f, (pcm_len - 2) as f64);
                            v.loop_dir = -1;
                        } else if v.loop_dir < 0 && v.pos <= loop_start_f {
                            v.pos = (loop_start_f + (loop_start_f - v.pos))
                                .clamp(0.0, loop_end_f);
                            v.loop_dir = 1;
                        }
                    } else if v.pos >= loop_end_f {
                        v.pos = loop_start_f + ((v.pos - loop_start_f) % loop_span_f);
                    }
                }

                let idx1 = v.pos as usize;
                if idx1 + 1 >= pcm_len {
                    alive = false;
                    break;
                }
                let idx0 = idx1.saturating_sub(1);
                let idx2 = idx1 + 1;
                let idx3 = (idx1 + 2).min(pcm_len - 1);
                let frac = (v.pos - idx1 as f64) as f32;
                let mut l = catmull_rom(pcm_l[idx0], pcm_l[idx1], pcm_l[idx2], pcm_l[idx3], frac);
                let mut r = if is_mono {
                    l
                } else {
                    catmull_rom(pcm_r[idx0], pcm_r[idx1], pcm_r[idx2], pcm_r[idx3], frac)
                };

                // Subtle 24-sample micro-crossfade before forward loop wrap when pre-loop audio exists
                if v.loop_enabled && v.loop_mode == 2 && v.loop_end > v.loop_start + 32 {
                    let xfade_len = 24_usize.min((v.loop_end - v.loop_start) / 4);
                    if xfade_len >= 4
                        && v.loop_start >= xfade_len
                        && v.pos + (xfade_len as f64) >= loop_end_f
                    {
                        let dist = v.pos + (xfade_len as f64) - loop_end_f;
                        let alpha = (dist / xfade_len as f64).clamp(0.0, 1.0) as f32;
                        let pre_pos = (v.loop_start - xfade_len) as f64 + dist;
                        let pidx = pre_pos as usize;
                        if pidx + 1 < pcm_len {
                            let pfrac = (pre_pos - pidx as f64) as f32;
                            let pl = pcm_l[pidx] + (pcm_l[pidx + 1] - pcm_l[pidx]) * pfrac;
                            l = l * (1.0 - alpha) + pl * alpha;
                            r = if is_mono {
                                l
                            } else {
                                let pr = pcm_r[pidx] + (pcm_r[pidx + 1] - pcm_r[pidx]) * pfrac;
                                r * (1.0 - alpha) + pr * alpha
                            };
                        }
                    }
                }

                let env_atk = v.env * atk;
                out_l[f] += l * env_atk * gain_l;
                out_r[f] += r * env_atk * gain_r;

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
    fn fingered_chords_all_roots_inversions_and_open_voicings() {
        let cases: &[(ChordQuality, &[u8])] = &[
            (ChordQuality::Major, &[0, 4, 7]),
            (ChordQuality::Minor, &[0, 3, 7]),
            (ChordQuality::Dom7, &[0, 4, 7, 10]),
            (ChordQuality::Maj7, &[0, 4, 7, 11]),
            (ChordQuality::Add9, &[0, 4, 7, 14]),
            (ChordQuality::Dom9, &[0, 4, 10, 14]),
            (ChordQuality::Maj9, &[0, 4, 7, 11, 14]),
            (ChordQuality::Min9, &[0, 3, 7, 10, 14]),
        ];
        for root in 0..12_u8 {
            for &(quality, intervals) in cases {
                for inversion in 0..intervals.len() {
                    let notes: Vec<u8> = intervals.iter().enumerate()
                        .map(|(idx, &iv)| 24 + root + iv + if idx < inversion { 24 } else { 0 })
                        .collect();
                    let (actual_root, actual_quality, _) =
                        detect_chord_with_mode(&notes, ChordMode::Fingered3To7).unwrap();
                    assert_eq!((actual_root, actual_quality), (root, quality),
                        "notes={notes:?} inversion={inversion}");
                }
            }
        }
        assert!(detect_chord_with_mode(&[36, 36, 40], ChordMode::Fingered3To7).is_none());
        let (_, quality, _) = detect_chord_with_mode(&[36, 40, 48], ChordMode::Fingered3To7).unwrap();
        assert_eq!(quality, ChordQuality::Major);
        // Doublings must not truncate a defining high seventh/ninth.
        let (root, quality, _) = detect_chord_with_mode(
            &[24, 28, 31, 36, 40, 43, 48, 58, 62], ChordMode::Fingered3To7).unwrap();
        assert_eq!((root, quality), (0, ChordQuality::Dom9));
    }

    #[test]
    fn chord_release_recomputes_while_sustained_and_latches_below_three() {
        let mut engine = MusicStudioArrangerEngine::new(48000.0);
        engine.sync_start = false;
        for note in [36, 40, 43, 46, 50] { engine.note_on(note, 90); }
        assert_eq!(engine.detected_chord_quality, ChordQuality::Dom9);
        engine.note_off(50, true);
        assert_eq!(engine.detected_chord_quality, ChordQuality::Dom7);
        engine.note_off(46, true);
        assert_eq!(engine.detected_chord_quality, ChordQuality::Major);
        engine.note_off(40, true);
        assert_eq!(engine.detected_chord_quality, ChordQuality::Major);
        assert_eq!(engine.held_lower_notes.len(), 2);
        engine.note_on(54, 90);
        assert!(engine.held_lower_notes.contains(&54));
        engine.note_on(55, 90);
        assert!(engine.held_upper_notes.contains(&55));
        assert!(!engine.held_lower_notes.contains(&55));
    }

    fn performance_fixture() -> Arc<LoadedMusicPackage> {
        let pcm = Arc::new(vec![0.15; 48000]);
        let lead = LoadedInstrument {
            index: 0, name: "Accordion".into(), volume: 1.0, pan: 0,
            inst_type: 0, is_drumkit: false,
            zones: vec![SampleZone {
                root_key: 60, key_low: 0, key_high: 127, vel_low: 0, vel_high: 127,
                name: "test".into(), sample_rate: 48000, channels: 1,
                loop_start: 0, loop_end: 0, loop_enabled: false, loop_mode: 0,
                gain: 1.0, pan: 0, fine_tune: 0, pool_sid: 0,
                pcm_l: pcm.clone(), pcm_r: pcm,
            }],
        };
        let mut second = lead.clone(); second.index = 1; second.name = "Piano".into();
        let track = RhythmTrack {
            track_type: "Acc1".into(), track_slot: 3, instrument_index: 0,
            volume: 1.0, pan: 0, fine_tune: 0, key_move: 0, octave_shift: 0,
            is_drum: false, casm_rule: None,
            events: vec![RhythmNoteEvent { midi_note: 60, velocity: 90, start_tick: 0, duration_ticks: 200 }],
        };
        let section = |id, name: &str, length, looped| RhythmSection {
            id, name: name.into(), length_ticks: length, is_loop: looped,
            jump_to: if looped { 0 } else { -2 }, tracks: vec![track.clone()],
        };
        Arc::new(LoadedMusicPackage {
            file_name: "test.sty".into(), file_path: "fixture://test".into(),
            version: "1".into(), source_format: "sty".into(), soundfont_name: None,
            instruments: vec![lead, second],
            styles: vec![RhythmStyle {
                id: 1, name: "test".into(), category: "test".into(), bpm: 120.0,
                sections: vec![section(1, "Variation 1", 16000, true), section(2, "Intro 1", 1000, false)],
                ots_presets: vec![OtsPresetSummary {index:0, name:"default".into(), upper1_instrument:0, upper2_instrument:None, lower_instrument:None}],
                sty_warnings: vec![], has_audio_style_chunks:false, uses_expansion_kit:false, sty_diagnostics_summary:None,
            }], ots_presets: vec![], source_paths: vec![], sty_warnings: vec![],
            has_audio_style_chunks: false, uses_expansion_kit: false, sty_diagnostics_summary: None,
        })
    }

    #[test]
    fn intro_starts_stopped_and_quantizes_to_beat_not_long_section() {
        for frames in [128, 150, 190, 210, 256, 512, 1024] {
            let mut engine = MusicStudioArrangerEngine::new(48000.0);
            engine.load_package(performance_fixture());
            engine.set_section("Intro 1", false);
            assert!(engine.rhythm_playing);
            let mut l = vec![0.0; frames]; let mut r = l.clone();
            engine.process_block(&mut l, &mut r);
            assert!(engine.active_voice_count() > 0, "Intro first note missing: {frames}");
            engine.set_section("Variation 1", true);
            engine.current_tick = 120.0;
            engine.set_section("Intro 1", false);
            let max_blocks = 24000 / frames + 2;
            let mut blocks = 0;
            while engine.active_section != "Intro 1" && blocks < max_blocks {
                engine.process_block(&mut l, &mut r); blocks += 1;
            }
            assert_eq!(engine.active_section, "Intro 1");
            assert!(blocks * frames < 24000 + frames);
            for _ in 0..(48000 / frames + 3) { engine.process_block(&mut l, &mut r); }
            assert_eq!(engine.active_section, "Variation 1");
            assert!(l.iter().chain(r.iter()).all(|x| x.is_finite()));
        }
    }

    #[test]
    fn instrument_reset_preserves_performance_settings() {
        let mut engine = MusicStudioArrangerEngine::new(48000.0);
        engine.load_package(performance_fixture());
        engine.upper1_instrument = 1; engine.upper2_instrument = Some(1); engine.lower_instrument = Some(1);
        engine.track_instrument_overrides = [Some(1);8];
        engine.track_volumes = [0.37;8]; engine.bpm = 97.0;
        engine.split_key = 54; engine.transpose = 2; engine.tempo_locked = true;
        engine.rhythm_playing = true; engine.current_tick = 243.0;
        engine.reset_instruments();
        assert_eq!((engine.upper1_instrument,engine.upper2_instrument,engine.lower_instrument),(0,None,None));
        assert_eq!(engine.track_instrument_overrides,[None;8]);
        assert_eq!(engine.track_volumes,[0.37;8]);
        assert_eq!((engine.bpm,engine.current_tick,engine.split_key,engine.transpose),(97.0,243.0,54,2));
        assert!(engine.rhythm_playing && engine.tempo_locked);
        assert_eq!(engine.package.as_ref().unwrap().instruments.len(),2);
    }

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
    fn test_chord_modes_3_to_7_extended_chords_and_transposition() {
        // Fingered3To7 rejects 1 or 2 held keys
        assert!(detect_chord_with_mode(&[48], ChordMode::Fingered3To7).is_none());
        assert!(detect_chord_with_mode(&[48, 52], ChordMode::Fingered3To7).is_none());

        // SingleFinger accepts 1 key as Major immediately
        let (r1, q1, l1) =
            detect_chord_with_mode(&[48], ChordMode::SingleFinger).expect("1-key C major");
        assert_eq!((r1, q1, l1.as_str()), (0, ChordQuality::Major, "C"));

        // 3..7 note extended chords in Fingered3To7
        let (_, q_add9, l_add9) =
            detect_chord_with_mode(&[48, 50, 52, 55], ChordMode::Fingered3To7).expect("Cadd9");
        assert_eq!((q_add9, l_add9.as_str()), (ChordQuality::Add9, "Cadd9"));

        let (_, q_dom9_shell, l_dom9_shell) =
            detect_chord_with_mode(&[48, 50, 52, 58], ChordMode::Fingered3To7)
                .expect("C7(9) 4-note");
        assert_eq!(
            (q_dom9_shell, l_dom9_shell.as_str()),
            (ChordQuality::Dom9, "C7(9)")
        );

        let (_, q_dom9_full, l_dom9_full) =
            detect_chord_with_mode(&[48, 50, 52, 55, 58], ChordMode::Fingered3To7)
                .expect("C7(9) 5-note");
        assert_eq!(
            (q_dom9_full, l_dom9_full.as_str()),
            (ChordQuality::Dom9, "C7(9)")
        );

        let (_, q_m9, l_m9) =
            detect_chord_with_mode(&[48, 50, 51, 55, 58], ChordMode::Fingered3To7).expect("Cm9");
        assert_eq!((q_m9, l_m9.as_str()), (ChordQuality::Min9, "Cm9"));

        let (_, q_maj9, l_maj9) =
            detect_chord_with_mode(&[48, 50, 52, 55, 59], ChordMode::Fingered3To7)
                .expect("Cmaj9");
        assert_eq!((q_maj9, l_maj9.as_str()), (ChordQuality::Maj9, "Cmaj9"));

        // Bass note breaks C6 vs Am7 tie
        let (_, q_c6, l_c6) =
            detect_chord_with_mode(&[48, 52, 55, 57], ChordMode::Fingered3To7).expect("C6");
        assert_eq!((q_c6, l_c6.as_str()), (ChordQuality::Six, "C6"));

        let (_, q_am7, l_am7) =
            detect_chord_with_mode(&[45, 48, 52, 55], ChordMode::Fingered3To7).expect("Am7");
        assert_eq!((q_am7, l_am7.as_str()), (ChordQuality::Min7, "Am7"));

        // 7-note C13 chord
        let (_, q_13, l_13) =
            detect_chord_with_mode(&[48, 50, 52, 53, 55, 57, 58], ChordMode::Fingered3To7)
                .expect("C13");
        assert_eq!((q_13, l_13.as_str()), (ChordQuality::Dom13, "C13"));

        // Bass preserves root while accompaniment adds 9th/7th color
        assert_eq!(transpose_note_for_chord(36, 0, ChordQuality::Dom9, true), 36);
        assert_eq!(transpose_note_for_chord(60, 0, ChordQuality::Dom9, false), 62); // D4 (9th)
        assert_eq!(transpose_note_for_chord(67, 0, ChordQuality::Dom9, false), 70); // Bb4 (b7)
        assert_eq!(transpose_note_for_chord(60, 0, ChordQuality::Add9, false), 62); // D4 (9th)
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
        assert_eq!(engine.split_key, 54);
        assert_eq!(engine.chord_mode, ChordMode::Fingered3To7);
        assert!(engine.lower_instrument.is_none());

        // In default Fingered3To7 mode, 1 or 2 keys must NOT change chord or sync-start rhythm
        engine.note_on(45, 110); // A2 (1 key)
        assert!(!engine.rhythm_playing);
        assert_eq!(engine.detected_chord_label, "C");
        engine.note_on(48, 108); // C3 (2 keys)
        assert!(!engine.rhythm_playing);
        assert_eq!(engine.detected_chord_label, "C");

        // 3rd key forms Am chord below/at split_key=54 -> changes chord + sync-starts rhythm, NO Lower voice triggered
        engine.note_on(52, 105); // E3 -> Am triad
        assert!(engine.rhythm_playing);
        assert_eq!(engine.detected_chord_label, "Am");
        assert_eq!(engine.active_voice_count(), 0);

        // Pressing F#3 (54) stays in the left-hand chord/lower zone (no Upper1 solo voice triggered)
        engine.note_on(54, 105);
        assert_eq!(engine.active_voice_count(), 0);
        engine.note_off(54, false);

        // Pressing G3 (55) starts Upper1 solo voice
        engine.note_on(55, 105);
        assert!(engine.active_voice_count() > 0);
        engine.note_off(55, false);
        engine.voices.clear();

        // Release held left-hand keys and switch to single_finger mode -> 1 key changes chord immediately
        engine.note_off(45, false);
        engine.note_off(48, false);
        engine.note_off(52, false);
        engine.set_keyboard_voice(
            "split",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("single_finger"),
        );
        engine.note_on(41, 108); // F2 single key -> F
        assert_eq!(engine.detected_chord_label, "F");
        engine.note_off(41, false);
        engine.set_keyboard_voice(
            "split",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("fingered_3_7"),
        );

        // Toggle SCAN UP and play upper triad (F4, A4, C5) -> should update chord to F
        engine.set_keyboard_voice(
            "upper1",
            None,
            None,
            None,
            None,
            Some(true),
            Some(1),
            None,
            None,
        );
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
            source_format: "music".to_string(),
            soundfont_name: None,
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
                    vel_low: 0,
                    vel_high: 127,
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
                        casm_rule: None,
                        events: vec![RhythmNoteEvent {
                            midi_note: 36,
                            velocity: 100,
                            start_tick: 0,
                            duration_ticks: 100,
                        }],
                    }],
                }],
                ots_presets: Vec::new(),
                sty_warnings: Vec::new(),
                has_audio_style_chunks: false,
                uses_expansion_kit: false,
                sty_diagnostics_summary: None,
            }],
            ots_presets: Vec::new(),
            source_paths: Vec::new(),
            sty_warnings: Vec::new(),
            has_audio_style_chunks: false,
            uses_expansion_kit: false,
            sty_diagnostics_summary: None,
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

    #[test]
    fn test_tail_tree_parses_v12_and_v14_without_desync_or_silence() {
        let dir = Path::new(USER_DOCUMENTS_RHYTHMS_DIR);
        if !dir.exists() {
            return;
        }
        for (fname, expected_var1_ticks) in [
            ("Forro_Boys_V1_VT.Music", 2000_u32),
            ("Forro_Boys_V2_VT.Music", 6000_u32),
            ("Natanzinho_Lima_GR.Music", 16000_u32),
            ("O_Som_do_Povo.Music", 8000_u32),
        ] {
            let p = dir.join(fname);
            if !p.exists() {
                continue;
            }
            let pkg = parse_music_file(&p).expect("parse v12/v14 rhythm");
            let var1 = pkg
                .styles
                .iter()
                .flat_map(|s| s.sections.iter())
                .find(|sec| sec.name == "Variation 1")
                .unwrap_or_else(|| panic!("{} missing Variation 1", fname));
            assert!(
                var1.is_loop,
                "{} Variation 1 must have is_loop=true",
                fname
            );
            assert_eq!(
                var1.length_ticks, expected_var1_ticks,
                "{} Variation 1 length_ticks mismatch",
                fname
            );
        }
    }

    #[test]
    fn test_variable_ttrack_headers_and_etype_mask_restore_drums() {
        let dir = Path::new(USER_DOCUMENTS_RHYTHMS_DIR);
        if !dir.exists() {
            return;
        }
        let mega = dir.join("MEGA_PACK_70_RITMOS_JUNTOS_TC.Music");
        if mega.exists() {
            let pkg = parse_music_file(&mega).expect("parse MEGA_PACK");
            for (style_key, require_perc) in [
                ("FORRO D ANTIGAS", true),
                ("WASHINGTON BR PISEIRO", true),
                ("FORRO BOYS", true),
                ("FORRO PERFEITO", true),
            ] {
                let st = pkg
                    .styles
                    .iter()
                    .find(|s| s.name.eq_ignore_ascii_case(style_key))
                    .unwrap_or_else(|| panic!("missing style {}", style_key));
                let var1 = st
                    .sections
                    .iter()
                    .find(|sec| sec.name == "Variation 1")
                    .expect("Variation 1");
                let drum_events: usize = var1
                    .tracks
                    .iter()
                    .filter(|t| t.track_slot == 0)
                    .map(|t| t.events.len())
                    .sum();
                assert!(
                    drum_events >= 10,
                    "{} must extract Drum events (got {})",
                    style_key,
                    drum_events
                );
                if require_perc {
                    let perc_events: usize = var1
                        .tracks
                        .iter()
                        .filter(|t| t.track_slot <= 1)
                        .map(|t| t.events.len())
                        .sum();
                    assert!(
                        perc_events > drum_events,
                        "{} must also extract Percussion events",
                        style_key
                    );
                }
            }
        }

        let wb_muel = dir.join("Washington_Brasileiro_Muel_Teclas.Music");
        if wb_muel.exists() {
            let pkg = parse_music_file(&wb_muel).expect("parse WB Muel Teclas");
            let var1 = pkg.styles[0]
                .sections
                .iter()
                .find(|s| s.name == "Variation 1")
                .expect("Variation 1");
            assert!(
                var1.tracks.iter().any(|t| t.track_type == "Acc. 1" && !t.events.is_empty()),
                "Acc. 1 with etype=12 preamble must still extract note events"
            );
            assert!(
                var1.tracks.iter().any(|t| t.track_slot == 0 && !t.events.is_empty()),
                "Kit WB Drum track must be present"
            );
        }
    }

    #[test]
    fn test_drum_choke_groups_mute_open_triangle_and_hihat() {
        let mut engine = MusicStudioArrangerEngine::new(48000.0);
        let long_pcm = Arc::new(vec![0.4_f32; 48000]);
        let short_pcm = Arc::new(vec![0.3_f32; 2400]);
        let kit = LoadedInstrument {
            index: 0,
            name: "Test Kit".to_string(),
            inst_type: 1,
            is_drumkit: true,
            volume: 1.0,
            pan: 0,
            zones: vec![
                SampleZone {
                    root_key: 42,
                    key_low: 42,
                    key_high: 42,
                    vel_low: 0,
                    vel_high: 127,
                    name: "CHH".to_string(),
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
                    pcm_l: Arc::clone(&short_pcm),
                    pcm_r: Arc::clone(&short_pcm),
                },
                SampleZone {
                    root_key: 46,
                    key_low: 46,
                    key_high: 46,
                    vel_low: 0,
                    vel_high: 127,
                    name: "OHH".to_string(),
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
                    pcm_l: Arc::clone(&long_pcm),
                    pcm_r: Arc::clone(&long_pcm),
                },
                SampleZone {
                    root_key: 80,
                    key_low: 80,
                    key_high: 80,
                    vel_low: 0,
                    vel_high: 127,
                    name: "MuteTri".to_string(),
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
                    pcm_l: Arc::clone(&short_pcm),
                    pcm_r: Arc::clone(&short_pcm),
                },
                SampleZone {
                    root_key: 81,
                    key_low: 81,
                    key_high: 81,
                    vel_low: 0,
                    vel_high: 127,
                    name: "OpenTri".to_string(),
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
                    pcm_l: Arc::clone(&long_pcm),
                    pcm_r: Arc::clone(&long_pcm),
                },
            ],
        };
        engine.package = Some(Arc::new(LoadedMusicPackage {
            file_name: "choke.Music".to_string(),
            file_path: "test://choke".to_string(),
            version: "1.4.2".to_string(),
            source_format: "music".to_string(),
            soundfont_name: None,
            instruments: vec![kit],
            styles: Vec::new(),
            ots_presets: Vec::new(),
            source_paths: Vec::new(),
            sty_warnings: Vec::new(),
            has_audio_style_chunks: false,
            uses_expansion_kit: false,
            sty_diagnostics_summary: None,
        }));

        // Open triangle 81 followed by Mute triangle 80 chokes 81 cleanly
        engine.trigger_instrument_note(0, 81, 81, 100, 1.0, 0, 0, true, None);
        assert_eq!(engine.active_voice_count(), 1);
        engine.trigger_instrument_note(0, 80, 80, 100, 1.0, 0, 0, true, None);
        let mut l = vec![0.0_f32; 4096];
        let mut r = vec![0.0_f32; 4096];
        engine.process_block(&mut l, &mut r);
        assert_eq!(engine.active_voice_count(), 0);

        // Open hi-hat 46 followed by Closed hi-hat 42 chokes 46 cleanly
        engine.trigger_instrument_note(0, 46, 46, 100, 1.0, 0, 0, true, None);
        engine.trigger_instrument_note(0, 42, 42, 100, 1.0, 0, 0, true, None);
        engine.process_block(&mut l, &mut r);
        assert_eq!(engine.active_voice_count(), 0);
    }

    #[test]
    fn test_parse_and_import_external_instrument_files() {
        let dir = Path::new(USER_DOCUMENTS_INSTRUMENTS_DIR).join("Sanfonas/Instrument_AddMore");
        if !dir.exists() {
            return;
        }
        let mut tested = 0_usize;
        for entry in std::fs::read_dir(&dir).expect("read Instrument_AddMore").flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()).map(|s| s.eq_ignore_ascii_case("instrument")).unwrap_or(false) {
                let inst = parse_instrument_file(&p, 0)
                    .unwrap_or_else(|e| panic!("parse {}: {}", p.display(), e));
                assert!(
                    inst.zones.iter().any(|z| !z.pcm_l.is_empty()),
                    "imported {} must contain PCM zones",
                    p.display()
                );
                let mut engine = MusicStudioArrangerEngine::new(48000.0);
                let new_idx = engine.add_imported_instrument(inst, true);
                assert_eq!(new_idx, 0);
                assert_eq!(engine.upper1_instrument, 0);
                engine.note_on(64, 110);
                let mut l = vec![0.0_f32; 4096];
                let mut r = vec![0.0_f32; 4096];
                engine.process_block(&mut l, &mut r);
                let peak = l.iter().chain(r.iter()).fold(0.0_f32, |m, &s| m.max(s.abs()));
                assert!(peak > 0.005, "imported {} must play audio (peak={})", p.display(), peak);
                tested += 1;
            }
        }
        assert!(tested > 0, "expected .Instrument files in {}", dir.display());
    }

    #[test]
    fn test_chord_quality_to_target_spec_and_velocity_zone_selection() {
        let all_qualities = [
            ChordQuality::Major,
            ChordQuality::Minor,
            ChordQuality::Dom7,
            ChordQuality::Min7,
            ChordQuality::Maj7,
            ChordQuality::MinMaj7,
            ChordQuality::Dim,
            ChordQuality::Dim7,
            ChordQuality::Min7b5,
            ChordQuality::Aug,
            ChordQuality::Sus2,
            ChordQuality::Sus4,
            ChordQuality::Dom7Sus4,
            ChordQuality::Six,
            ChordQuality::Min6,
            ChordQuality::SixNine,
            ChordQuality::Add9,
            ChordQuality::MinAdd9,
            ChordQuality::Dom9,
            ChordQuality::Maj9,
            ChordQuality::Min9,
            ChordQuality::Dom11,
            ChordQuality::Min11,
            ChordQuality::Dom13,
            ChordQuality::Maj13,
            ChordQuality::Min13,
        ];
        for (idx, q) in all_qualities.iter().enumerate() {
            let spec = chord_quality_to_target_spec((idx as u8) + 12, *q);
            assert_eq!(spec.root_pc, (idx as u8) % 12);
            assert!(spec.yamaha_bit <= 33, "{:?} bit out of range", q);
        }

        // Verify velocity layer routing in trigger_instrument_note
        let soft_pcm = Arc::new(vec![0.15_f32; 2048]);
        let loud_pcm = Arc::new(vec![0.80_f32; 2048]);
        let vel_inst = LoadedInstrument {
            index: 0,
            name: "VelSplit".to_string(),
            volume: 1.0,
            pan: 0,
            inst_type: 0,
            is_drumkit: false,
            zones: vec![
                SampleZone {
                    root_key: 60,
                    key_low: 48,
                    key_high: 72,
                    vel_low: 0,
                    vel_high: 63,
                    name: "Soft".to_string(),
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
                    pcm_l: Arc::clone(&soft_pcm),
                    pcm_r: Arc::clone(&soft_pcm),
                },
                SampleZone {
                    root_key: 60,
                    key_low: 48,
                    key_high: 72,
                    vel_low: 64,
                    vel_high: 127,
                    name: "Loud".to_string(),
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
                    pcm_l: Arc::clone(&loud_pcm),
                    pcm_r: Arc::clone(&loud_pcm),
                },
            ],
        };
        let mut engine = MusicStudioArrangerEngine::new(48000.0);
        engine.package = Some(Arc::new(LoadedMusicPackage {
            file_name: "vel.Music".to_string(),
            file_path: "test://vel".to_string(),
            version: "1.4".to_string(),
            source_format: "music".to_string(),
            soundfont_name: None,
            instruments: vec![vel_inst],
            styles: Vec::new(),
            ots_presets: Vec::new(),
            source_paths: Vec::new(),
            sty_warnings: Vec::new(),
            has_audio_style_chunks: false,
            uses_expansion_kit: false,
            sty_diagnostics_summary: None,
        }));
        engine.trigger_instrument_note(0, 62, 62, 40, 1.0, 0, 0, false, None);
        assert!(Arc::ptr_eq(&engine.voices[0].pcm_l, &soft_pcm));
        engine.voices.clear();
        engine.trigger_instrument_note(0, 62, 62, 110, 1.0, 0, 0, false, None);
        assert!(Arc::ptr_eq(&engine.voices[0].pcm_l, &loud_pcm));
    }

    #[test]
    fn test_sty_six_real_styles_parse_load_and_render_clean_audio() {
        if crate::soundfont::find_default_soundfont().is_none() {
            return;
        }
        let kb = Path::new(USER_DOCUMENTS_KEYBOARD_DIR);
        if !kb.exists() {
            return;
        }
        let sty_paths = [
            kb.join("Ritmos PSR/O Som do Povo/O Som do Povo.S837.STY"),
            kb.join("Washington Brasileiro e Forro Boys/Forro Boys/(35) FORRÓ BOYS.sty"),
            kb.join("Ritmos PSR/Calcinha Preta/CALCINHAPRETA GB.sty"),
            kb.join("Ritmos PSR/Zé Vaqueiro/01-ZE VAQUEIRO.sty"),
            kb.join("Forro_novos_2026-11-08/Mastruz/Mastruz.STY"),
            kb.join("Forro_novos_2026-11-08/Forro_bandas/CALPRETA.STY"),
        ];

        let mut l = vec![0.0_f32; 4096];
        let mut r = vec![0.0_f32; 4096];

        for (idx, p) in sty_paths.iter().enumerate() {
            if !p.exists() {
                continue;
            }
            let pkg = Arc::new(
                build_sty_package(std::slice::from_ref(p), None)
                    .unwrap_or_else(|e| panic!("build_sty_package {}: {}", p.display(), e)),
            );
            assert_eq!(pkg.source_format, "sty");
            assert!(pkg.soundfont_name.is_some());
            assert!(!pkg.instruments.is_empty());
            assert_eq!(pkg.styles.len(), 1);

            if idx == 2 {
                // Verify CALCINHAPRETA GB.sty CASM chord_mute: Major activates src=11 & mutes src=7,
                // Minor activates src=7 & mutes src=11.
                let var1 = pkg.styles[0]
                    .sections
                    .iter()
                    .find(|s| s.name == "Variation 1")
                    .expect("CALCINHAPRETA Variation 1");
                let slot3_tracks: Vec<&RhythmTrack> = var1
                    .tracks
                    .iter()
                    .filter(|t| t.track_slot == 3 && t.casm_rule.is_some())
                    .collect();
                let mut maj_track = (*slot3_tracks
                    .iter()
                    .find(|t| t.casm_rule.as_ref().unwrap().src_ch == 11)
                    .expect("src=11 track"))
                .clone();
                let mut min_track = (*slot3_tracks
                    .iter()
                    .find(|t| t.casm_rule.as_ref().unwrap().src_ch == 7)
                    .expect("src=7 track"))
                .clone();
                maj_track.instrument_index = 0;
                min_track.instrument_index = 1;
                let probe_sec = RhythmSection {
                    id: var1.id,
                    name: var1.name.clone(),
                    length_ticks: var1.length_ticks,
                    is_loop: true,
                    jump_to: 0,
                    tracks: vec![maj_track, min_track],
                };
                let mut maj_triggers = Vec::new();
                collect_section_events(
                    &probe_sec,
                    &pkg.instruments,
                    &[1.0; 8],
                    &[None; 8],
                    0,
                    ChordQuality::Major,
                    pkg.styles[0].bpm,
                    48000.0,
                    0.0,
                    var1.length_ticks as f64,
                    &mut maj_triggers,
                );
                assert!(!maj_triggers.is_empty());
                assert!(maj_triggers.iter().all(|t| t.inst_idx == 0));

                let mut min_triggers = Vec::new();
                collect_section_events(
                    &probe_sec,
                    &pkg.instruments,
                    &[1.0; 8],
                    &[None; 8],
                    0,
                    ChordQuality::Minor,
                    pkg.styles[0].bpm,
                    48000.0,
                    0.0,
                    var1.length_ticks as f64,
                    &mut min_triggers,
                );
                assert!(!min_triggers.is_empty());
                assert!(min_triggers.iter().all(|t| t.inst_idx == 1));
            }

            let mut engine = MusicStudioArrangerEngine::new(48000.0);
            engine.load_package(Arc::clone(&pkg));
            engine.set_transport(true, None, None);

            let mut peak = 0.0_f32;
            for _ in 0..10 {
                engine.process_block(&mut l, &mut r);
                for (&sl, &sr) in l.iter().zip(r.iter()) {
                    assert!(sl.is_finite() && sr.is_finite());
                    peak = peak.max(sl.abs()).max(sr.abs());
                }
            }
            assert!(
                peak > 0.02 && peak <= 1.0,
                "{}: expected 0.02 < peak <= 1.0, got {}",
                p.display(),
                peak
            );
        }
    }

    #[test]
    fn test_sty_multi_file_pack_and_style_switching() {
        if crate::soundfont::find_default_soundfont().is_none() {
            return;
        }
        let kb = Path::new(USER_DOCUMENTS_KEYBOARD_DIR);
        if !kb.exists() {
            return;
        }
        let candidate_paths = vec![
            kb.join("Ritmos PSR/O Som do Povo/O Som do Povo.S837.STY"),
            kb.join("Washington Brasileiro e Forro Boys/Forro Boys/(35) FORRÓ BOYS.sty"),
            kb.join("Ritmos PSR/Calcinha Preta/CALCINHAPRETA GB.sty"),
            kb.join("Ritmos PSR/Zé Vaqueiro/01-ZE VAQUEIRO.sty"),
            kb.join("Forro_novos_2026-11-08/Forro_bandas/CALPRETA.STY"),
        ];
        if !candidate_paths.iter().all(|p| p.exists()) {
            return;
        }

        let pkg = Arc::new(build_sty_package(&candidate_paths, None).expect("5-style pack"));
        assert_eq!(pkg.source_format, "sty_pack");
        assert_eq!(pkg.styles.len(), 5);
        assert!(pkg.source_paths.len() == 5);

        let mut engine = MusicStudioArrangerEngine::new(48000.0);
        engine.load_package(Arc::clone(&pkg));
        assert_eq!(engine.summary().source_format, "sty_pack");

        let mut l = vec![0.0_f32; 4096];
        let mut r = vec![0.0_f32; 4096];
        for style_idx in 0..5 {
            engine.set_style(style_idx);
            assert!((engine.bpm - pkg.styles[style_idx].bpm).abs() < 0.01);
            engine.set_transport(true, None, None);
            let mut peak = 0.0_f32;
            for _ in 0..8 {
                engine.process_block(&mut l, &mut r);
                peak = l.iter().chain(r.iter()).fold(peak, |m, &s| m.max(s.abs()));
            }
            assert!(
                peak > 0.02 && peak <= 1.0,
                "pack style {} failed clean audio (peak={})",
                style_idx,
                peak
            );
            engine.set_transport(false, None, None);
        }
    }

    #[test]
    fn test_sty_import_external_instrument_and_track_override() {
        if crate::soundfont::find_default_soundfont().is_none() {
            return;
        }
        let sty_a = Path::new(USER_DOCUMENTS_KEYBOARD_DIR)
            .join("Ritmos PSR/O Som do Povo/O Som do Povo.S837.STY");
        let sty_b = Path::new(USER_DOCUMENTS_KEYBOARD_DIR)
            .join("Ritmos PSR/Zé Vaqueiro/01-ZE VAQUEIRO.sty");
        let inst_dir = Path::new(USER_DOCUMENTS_INSTRUMENTS_DIR).join("Sanfonas/Instrument_AddMore");
        if !sty_a.exists() || !sty_b.exists() || !inst_dir.exists() {
            return;
        }
        let inst_path = std::fs::read_dir(&inst_dir)
            .ok()
            .and_then(|rd| {
                rd.flatten()
                    .map(|e| e.path())
                    .find(|p| {
                        p.extension()
                            .and_then(|e| e.to_str())
                            .map(|s| s.eq_ignore_ascii_case("instrument"))
                            .unwrap_or(false)
                    })
            });
        let Some(inst_path) = inst_path else {
            return;
        };

        let pkg_a = Arc::new(build_sty_package(&[sty_a], None).expect("build STY A"));
        let pkg_b = Arc::new(build_sty_package(&[sty_b], None).expect("build STY B"));
        let imported = parse_instrument_file(&inst_path, 0).expect("parse .Instrument");
        let imported_name = imported.name.clone();

        let mut engine = MusicStudioArrangerEngine::new(48000.0);
        engine.load_package(pkg_a);
        let imp_idx = engine.add_imported_instrument(imported, true);
        assert_eq!(engine.upper1_instrument, imp_idx);

        // Switching to STY B must retain user_imported_instruments and reset overrides
        engine.set_track_instrument(3, Some(imp_idx));
        assert_eq!(engine.track_instrument_overrides[3], Some(imp_idx));
        engine.load_package(pkg_b);
        assert_eq!(engine.track_instrument_overrides, [None; 8]);
        let retained_idx = engine
            .package
            .as_ref()
            .unwrap()
            .instruments
            .iter()
            .find(|i| i.name == imported_name)
            .map(|i| i.index)
            .expect("user imported instrument must persist across STY package switches");

        engine.set_track_instrument(3, Some(retained_idx));
        assert_eq!(engine.summary().track_instruments[3], Some(retained_idx));

        engine.set_transport(true, None, None);
        engine.note_on(64, 110);
        let mut l = vec![0.0_f32; 4096];
        let mut r = vec![0.0_f32; 4096];
        let mut peak = 0.0_f32;
        for _ in 0..8 {
            engine.process_block(&mut l, &mut r);
            peak = l.iter().chain(r.iter()).fold(peak, |m, &s| m.max(s.abs()));
        }
        assert!(peak > 0.02 && peak <= 1.0, "override+solo peak={}", peak);

        // Library scanner must find both .Music and .STY entries cleanly
        let all_entries = list_all_music_packages();
        assert!(all_entries.iter().any(|e| e.format == "music"));
        assert!(all_entries.iter().any(|e| e.format == "sty"));
    }

    fn find_target_sty_files(roots: &[&Path], targets: &[&str]) -> Vec<PathBuf> {
        fn walk(dir: &Path, targets: &[&str], out: &mut Vec<PathBuf>, seen: &mut HashSet<String>) {
            let Ok(rd) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in rd.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    walk(&p, targets, out, seen);
                } else if let Some(fname) = p.file_name().and_then(|s| s.to_str()) {
                    let lower = fname.to_ascii_lowercase();
                    if let Some(&canon) = targets
                        .iter()
                        .find(|t| t.to_ascii_lowercase() == lower)
                    {
                        if seen.insert(canon.to_ascii_lowercase()) {
                            out.push(p);
                        }
                    }
                }
            }
        }

        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for root in roots {
            if root.exists() {
                walk(root, targets, &mut out, &mut seen);
            }
        }
        out
    }

    #[test]
    fn test_sty_drums_zero_pitch_transposition_and_zero_melodic_routing() {
        // 1. Verify slot <= 1 anti-transposition guard in collect_section_events even when is_drum == false
        //    and instrument is_drumkit == false, both with and without a CASM rule.
        let dummy_pcm = Arc::new(vec![0.1_f32; 512]);
        let non_drum_inst = LoadedInstrument {
            index: 0,
            name: "MelodicLookalike".to_string(),
            volume: 1.0,
            pan: 0,
            inst_type: 0,
            is_drumkit: false,
            zones: vec![SampleZone {
                root_key: 60,
                key_low: 0,
                key_high: 127,
                vel_low: 0,
                vel_high: 127,
                name: "All".to_string(),
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
                pcm_l: Arc::clone(&dummy_pcm),
                pcm_r: Arc::clone(&dummy_pcm),
            }],
        };
        let casm_rule = crate::yamaha_sty::CasmRule {
            name: "PercGuard".to_string(),
            src_ch: 8,
            dest_ch: 8,
            editable: 1,
            note_mute: 0x0FFF,
            chord_mute: u64::MAX,
            src_root: 0,
            src_chord_type: 0,
            low_split: None,
            high_split: None,
            ranges: std::array::from_fn(|_| crate::yamaha_sty::CasmRangeRule {
                ntr: crate::yamaha_sty::NtrRule::RootTrans,
                ntt: crate::yamaha_sty::NttRule::Chord,
                bass_on: false,
                high_key: 11,
                note_low: 0,
                note_high: 127,
                rtr: crate::yamaha_sty::RtrRule::PitchShift,
            }),
        };
        let guard_sec = RhythmSection {
            id: 1,
            name: "Variation 1".to_string(),
            length_ticks: 2000,
            is_loop: true,
            jump_to: 0,
            tracks: vec![
                RhythmTrack {
                    track_type: "Drum".to_string(),
                    track_slot: 0,
                    instrument_index: 0,
                    volume: 1.0,
                    pan: 0,
                    fine_tune: 0,
                    key_move: 3,
                    octave_shift: 1,
                    is_drum: false,
                    casm_rule: None,
                    events: vec![RhythmNoteEvent {
                        midi_note: 36,
                        velocity: 110,
                        start_tick: 0,
                        duration_ticks: 100,
                    }],
                },
                RhythmTrack {
                    track_type: "Percussion".to_string(),
                    track_slot: 1,
                    instrument_index: 0,
                    volume: 1.0,
                    pan: 0,
                    fine_tune: 0,
                    key_move: -4,
                    octave_shift: -1,
                    is_drum: false,
                    casm_rule: Some(casm_rule),
                    events: vec![RhythmNoteEvent {
                        midi_note: 54,
                        velocity: 100,
                        start_tick: 250,
                        duration_ticks: 100,
                    }],
                },
            ],
        };
        let mut trig_c = Vec::new();
        let mut trig_fs = Vec::new();
        collect_section_events(
            &guard_sec,
            std::slice::from_ref(&non_drum_inst),
            &[1.0; 8],
            &[None; 8],
            0,
            ChordQuality::Major,
            120.0,
            48000.0,
            0.0,
            1000.0,
            &mut trig_c,
        );
        collect_section_events(
            &guard_sec,
            std::slice::from_ref(&non_drum_inst),
            &[1.0; 8],
            &[None; 8],
            6,
            ChordQuality::Minor,
            120.0,
            48000.0,
            0.0,
            1000.0,
            &mut trig_fs,
        );
        assert_eq!(trig_c.len(), 2);
        assert_eq!(trig_fs.len(), 2);
        assert_eq!(trig_c[0].midi_note, 36);
        assert_eq!(trig_fs[0].midi_note, 36);
        assert_eq!(trig_c[1].midi_note, 54);
        assert_eq!(trig_fs[1].midi_note, 54);

        // 2. Verify set_style + summary + steady-state sync_dynamic_summary refresh STY diagnostics.
        let mut sty_engine = MusicStudioArrangerEngine::new(48000.0);
        sty_engine.load_package(Arc::new(LoadedMusicPackage {
            file_name: "diag_switch.sty".to_string(),
            file_path: "test://diag_switch".to_string(),
            version: "SFF2/SF2".to_string(),
            source_format: "sty_pack".to_string(),
            soundfont_name: Some("TestSF2".to_string()),
            instruments: vec![non_drum_inst],
            styles: vec![
                RhythmStyle {
                    id: 1,
                    name: "CleanStyle".to_string(),
                    category: "Yamaha SFF1".to_string(),
                    bpm: 120.0,
                    sections: vec![guard_sec.clone()],
                    ots_presets: Vec::new(),
                    sty_warnings: Vec::new(),
                    has_audio_style_chunks: false,
                    uses_expansion_kit: false,
                    sty_diagnostics_summary: Some("CleanDiag".to_string()),
                },
                RhythmStyle {
                    id: 2,
                    name: "ExpansionAudioStyle".to_string(),
                    category: "Yamaha SFF2".to_string(),
                    bpm: 138.0,
                    sections: vec![guard_sec],
                    ots_presets: Vec::new(),
                    sty_warnings: vec!["Expansion warning".to_string()],
                    has_audio_style_chunks: true,
                    uses_expansion_kit: true,
                    sty_diagnostics_summary: Some("ExpDiag".to_string()),
                },
            ],
            ots_presets: Vec::new(),
            source_paths: Vec::new(),
            sty_warnings: vec!["Expansion warning".to_string()],
            has_audio_style_chunks: true,
            uses_expansion_kit: true,
            sty_diagnostics_summary: Some("CleanDiag".to_string()),
        }));
        let mut dyn_sum = sty_engine.summary();
        assert!(!dyn_sum.uses_expansion_kit);
        assert!(!dyn_sum.has_audio_style_chunks);
        assert!(dyn_sum.sty_warnings.is_empty());
        assert_eq!(dyn_sum.sty_diagnostics_summary.as_deref(), Some("CleanDiag"));

        sty_engine.set_style(1);
        let switched_sum = sty_engine.summary();
        assert!(switched_sum.uses_expansion_kit);
        assert!(switched_sum.has_audio_style_chunks);
        assert_eq!(switched_sum.sty_warnings, vec!["Expansion warning".to_string()]);
        assert_eq!(
            switched_sum.sty_diagnostics_summary.as_deref(),
            Some("ExpDiag")
        );
        sty_engine.sync_dynamic_summary(&mut dyn_sum);
        assert!(dyn_sum.uses_expansion_kit);
        assert!(dyn_sum.has_audio_style_chunks);
        assert_eq!(dyn_sum.sty_warnings, vec!["Expansion warning".to_string()]);
        assert_eq!(dyn_sum.sty_diagnostics_summary.as_deref(), Some("ExpDiag"));
        sty_engine.sync_dynamic_summary(&mut dyn_sum);
        assert_eq!(dyn_sum.sty_warnings, vec!["Expansion warning".to_string()]);

        // 3. Locate real .STY files and validate drum routing, note remapping, zero transposition, and diagnostics.
        if crate::soundfont::find_default_soundfont().is_none() {
            return;
        }
        let target_names = [
            "(35) FORRÓ BOYS.sty",
            "(40) GUSTAVA LIMA.sty",
            "(44) LAMBASAIA 1.sty",
            "O Som do Povo.S837.STY",
        ];
        let search_roots = [
            Path::new(USER_DOCUMENTS_KEYBOARD_DIR),
            Path::new(USER_DOCUMENTS_RHYTHMS_DIR),
            Path::new("/home/data/Downloads"),
        ];
        let found_paths = find_target_sty_files(&search_roots, &target_names);
        if Path::new(USER_DOCUMENTS_KEYBOARD_DIR).exists() {
            assert!(
                found_paths.iter().any(|p| p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|n| n.eq_ignore_ascii_case("(35) FORRÓ BOYS.sty"))
                    .unwrap_or(false)),
                "Expected (35) FORRÓ BOYS.sty under {}",
                USER_DOCUMENTS_KEYBOARD_DIR
            );
            assert!(
                found_paths.iter().any(|p| p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|n| n.eq_ignore_ascii_case("O Som do Povo.S837.STY"))
                    .unwrap_or(false)),
                "Expected O Som do Povo.S837.STY under {}",
                USER_DOCUMENTS_KEYBOARD_DIR
            );
        }
        if found_paths.is_empty() {
            return;
        }

        for p in &found_paths {
            let doc = crate::yamaha_sty::parse_sty_file(p)
                .unwrap_or_else(|e| panic!("parse_sty_file {}: {}", p.display(), e));
            let pkg = Arc::new(
                build_sty_package(std::slice::from_ref(p), None)
                    .unwrap_or_else(|e| panic!("build_sty_package {}: {}", p.display(), e)),
            );
            assert_eq!(pkg.styles.len(), 1);
            assert!(pkg.sty_diagnostics_summary.is_some());
            assert_eq!(
                pkg.styles[0].sty_diagnostics_summary,
                pkg.sty_diagnostics_summary
            );
            assert_eq!(pkg.styles[0].sections.len(), doc.sections.len());

            for (sec_idx, pkg_sec) in pkg.styles[0].sections.iter().enumerate() {
                let doc_sec = &doc.sections[sec_idx];
                assert_eq!(pkg_sec.tracks.len(), doc_sec.tracks.len());
                for (tr_idx, pkg_tr) in pkg_sec.tracks.iter().enumerate() {
                    let doc_tr = &doc_sec.tracks[tr_idx];
                    let kit_opt = crate::yamaha_sty::classify_drum_kit(
                        doc_tr.dest_ch,
                        doc_tr.voice.msb,
                        doc_tr.voice.lsb,
                        doc_tr.voice.program,
                    );
                    if pkg_tr.is_drum || pkg_tr.track_slot <= 1 {
                        let inst = pkg
                            .instruments
                            .get(pkg_tr.instrument_index)
                            .unwrap_or_else(|| {
                                panic!(
                                    "{}: invalid instrument_index {} on drum track",
                                    p.display(),
                                    pkg_tr.instrument_index
                                )
                            });
                        assert!(
                            inst.is_drumkit,
                            "{} [{} slot {}]: drum track routed to melodic instrument '{}' (idx={})",
                            p.display(),
                            pkg_sec.name,
                            pkg_tr.track_slot,
                            inst.name,
                            pkg_tr.instrument_index
                        );
                        for ev in &pkg_tr.events {
                            assert!(
                                (27..=87).contains(&ev.midi_note),
                                "{} [{} slot {}]: drum note {} out of 27..=87",
                                p.display(),
                                pkg_sec.name,
                                pkg_tr.track_slot,
                                ev.midi_note
                            );
                            assert!(
                                ev.midi_note != 33 && ev.midi_note != 34,
                                "{} [{} slot {}]: unremapped XG metronome note {} found",
                                p.display(),
                                pkg_sec.name,
                                pkg_tr.track_slot,
                                ev.midi_note
                            );
                            if kit_opt == Some(crate::yamaha_sty::DrumKitKind::XgLatinCuban) {
                                assert!(
                                    !matches!(ev.midi_note, 58 | 71 | 72),
                                    "{} [{} slot {}]: unremapped MSB=126 Latin/Cuban note {} found",
                                    p.display(),
                                    pkg_sec.name,
                                    pkg_tr.track_slot,
                                    ev.midi_note
                                );
                            }
                        }
                    }
                }
            }

            let render_isolated_drums = |root_pc: u8, quality: &str| -> (Vec<f32>, Vec<f32>, f32, MusicPackageSummary) {
                let mut eng = MusicStudioArrangerEngine::new(48000.0);
                eng.load_package(Arc::clone(&pkg));
                for slot in 2..8 {
                    eng.set_track_volume(slot, 0.0);
                }
                eng.set_track_volume(0, 1.0);
                eng.set_track_volume(1, 1.0);
                eng.set_chord(root_pc, quality);
                eng.set_transport(true, None, None);

                let mut all_l = Vec::with_capacity(24 * 4096);
                let mut all_r = Vec::with_capacity(24 * 4096);
                let mut l = vec![0.0_f32; 4096];
                let mut r = vec![0.0_f32; 4096];
                let mut peak = 0.0_f32;
                for _ in 0..24 {
                    eng.process_block(&mut l, &mut r);
                    for (&sl, &sr) in l.iter().zip(r.iter()) {
                        peak = peak.max(sl.abs()).max(sr.abs());
                    }
                    all_l.extend_from_slice(&l);
                    all_r.extend_from_slice(&r);
                }
                let sum = eng.summary();
                (all_l, all_r, peak, sum)
            };

            let (lc, rc, peak_c, sum_c) = render_isolated_drums(0, "Major");
            let (lfs, rfs, peak_fs, _sum_fs) = render_isolated_drums(6, "Minor");
            assert!(
                peak_c > 0.01 && peak_fs > 0.01,
                "{}: expected audible isolated drums, got peak_c={} peak_fs={}",
                p.display(),
                peak_c,
                peak_fs
            );
            let max_diff = lc
                .iter()
                .zip(lfs.iter())
                .chain(rc.iter().zip(rfs.iter()))
                .fold(0.0_f32, |m, (&a, &b)| m.max((a - b).abs()));
            assert!(
                max_diff < 1e-6,
                "{}: isolated Drum+Perc differed between C Major and F# Minor (max_diff={})",
                p.display(),
                max_diff
            );

            if p.file_name()
                .and_then(|s| s.to_str())
                .map(|n| n.eq_ignore_ascii_case("(35) FORRÓ BOYS.sty"))
                .unwrap_or(false)
            {
                assert!(pkg.uses_expansion_kit, "FORRÓ BOYS must set uses_expansion_kit");
                assert!(!pkg.sty_warnings.is_empty(), "FORRÓ BOYS must emit sty_warnings");
                assert!(sum_c.uses_expansion_kit);
                assert!(!sum_c.sty_warnings.is_empty());
                assert!(sum_c.sty_diagnostics_summary.is_some());
            }
        }
    }
}
