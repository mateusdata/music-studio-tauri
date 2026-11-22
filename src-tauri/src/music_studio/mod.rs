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
    pub gain: f32,
    pub pool_sid: u32,
    pub pcm_l: Arc<Vec<f32>>,
    pub pcm_r: Arc<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub struct LoadedInstrument {
    pub index: usize,
    pub name: String,
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
    pub octave_shift: i32,
    pub is_drum: bool,
    pub events: Vec<RhythmNoteEvent>,
}

#[derive(Debug, Clone)]
pub struct RhythmSection {
    pub id: u32,
    pub name: String,
    pub length_ticks: u32,
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
    matches!(
        sr,
        8000 | 11025 | 16000 | 22050 | 24000 | 31250 | 32000 | 44100 | 48000 | 88200 | 96000
    ) || (16000..=96000).contains(&sr)
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

fn parse_header_and_sample_pool(data: &[u8]) -> ParsedSamplePool {
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

    let header_bpm = read_u32_le(data, cur + 5)
        .filter(|&b| (40..=240).contains(&b))
        .map(|b| b as f32);

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
    let sr = u16::from_le_bytes([hdr[33], hdr[34]]) as u32;
    let bits = hdr[35];
    let ch = hdr[36];
    let sr2 = u32::from_le_bytes([hdr[37], hdr[38], hdr[39], hdr[40]]);
    let nfr = u32::from_le_bytes([hdr[45], hdr[46], hdr[47], hdr[48]]);
    root <= 127
        && nlen >= 1
        && nlen <= 32
        && is_valid_sample_rate(sr)
        && sr == sr2
        && (bits == 8 || bits == 16)
        && (ch == 1 || ch == 2)
        && nfr >= 8
        && nfr <= 15_000_000
}

fn looks_like_zone_96(data: &[u8], off: usize) -> bool {
    let Some(hdr) = data.get(off..off + 96) else {
        return false;
    };
    let sr = u16::from_le_bytes([hdr[34], hdr[35]]) as u32;
    let bits = hdr[36];
    let ch = hdr[37];
    let nfr = u32::from_le_bytes([hdr[46], hdr[47], hdr[48], hdr[49]]);
    let loop_end = u32::from_le_bytes([hdr[78], hdr[79], hdr[80], hdr[81]]);
    let printable = hdr[2..10].iter().any(|&b| b.is_ascii_alphanumeric());
    printable
        && is_valid_sample_rate(sr)
        && (bits == 8 || bits == 16)
        && (ch == 1 || ch == 2)
        && nfr >= 16
        && nfr <= 15_000_000
        && loop_end <= nfr + 16
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

fn parse_zone_at(
    data: &[u8],
    cur: usize,
    is_96: bool,
    inline_pcm: bool,
    pool: &HashMap<u32, (usize, usize)>,
    fallback_keys: &[u8],
    zone_idx: usize,
) -> Option<(SampleZone, usize)> {
    let hdr_len = if is_96 { 96 } else { 97 };
    let hdr = data.get(cur..cur + hdr_len)?;

    let (root_key, name, sr, bits, ch, pool_sid, nfr, loop_mode, loop_start, loop_end, gain_pct) =
        if is_96 {
            let rk = if hdr[1] <= 127 && hdr[1] >= 21 {
                hdr[1]
            } else {
                *fallback_keys.get(zone_idx).unwrap_or(&60)
            };
            let sname = decode_latin1(&hdr[2..34]);
            let sr = u16::from_le_bytes([hdr[34], hdr[35]]) as u32;
            let bits = hdr[36];
            let ch = hdr[37];
            let sid = u32::from_le_bytes([hdr[42], hdr[43], hdr[44], hdr[45]]);
            let nfr = u32::from_le_bytes([hdr[46], hdr[47], hdr[48], hdr[49]]) as usize;
            let gain = u32::from_le_bytes([hdr[54], hdr[55], hdr[56], hdr[57]]);
            let lmode = u32::from_le_bytes([hdr[70], hdr[71], hdr[72], hdr[73]]);
            let lstart = u32::from_le_bytes([hdr[74], hdr[75], hdr[76], hdr[77]]) as usize;
            let lend = u32::from_le_bytes([hdr[78], hdr[79], hdr[80], hdr[81]]) as usize;
            (rk, sname, sr, bits, ch, sid, nfr, lmode, lstart, lend, gain)
        } else {
            let rk = hdr[0];
            let sname = decode_latin1(&hdr[2..33]);
            let sr = u16::from_le_bytes([hdr[33], hdr[34]]) as u32;
            let bits = hdr[35];
            let ch = hdr[36];
            let sid = u32::from_le_bytes([hdr[41], hdr[42], hdr[43], hdr[44]]);
            let nfr = u32::from_le_bytes([hdr[45], hdr[46], hdr[47], hdr[48]]) as usize;
            let gain = u32::from_le_bytes([hdr[53], hdr[54], hdr[55], hdr[56]]);
            let lmode = u32::from_le_bytes([hdr[69], hdr[70], hdr[71], hdr[72]]);
            let lstart = u32::from_le_bytes([hdr[73], hdr[74], hdr[75], hdr[76]]) as usize;
            let lend = u32::from_le_bytes([hdr[77], hdr[78], hdr[79], hdr[80]]) as usize;
            (rk, sname, sr, bits, ch, sid, nfr, lmode, lstart, lend, gain)
        };

    let bytes_per_sample = (bits as usize / 8).max(1);
    let pcm_bytes = nfr.saturating_mul(ch as usize).saturating_mul(bytes_per_sample);
    let pcm_start = cur + hdr_len;

    let (pcm_l, pcm_r, next_cur) = if inline_pcm
        && pcm_bytes > 0
        && pcm_start + pcm_bytes <= data.len()
    {
        let (l, r) = decode_pcm_bytes(&data[pcm_start..pcm_start + pcm_bytes], bits, ch, nfr);
        (l, r, pcm_start + pcm_bytes)
    } else if let Some(&(pool_off, pool_len)) = pool_sid
        .checked_sub(1)
        .and_then(|id| pool.get(&id))
        .or_else(|| pool.get(&pool_sid))
    {
        let avail = pool_len.min(data.len().saturating_sub(pool_off));
        let (l, r) = decode_pcm_bytes(&data[pool_off..pool_off + avail], bits, ch, nfr);
        (l, r, pcm_start)
    } else {
        (Vec::new(), Vec::new(), pcm_start)
    };

    let gain_norm = if (1..=250).contains(&gain_pct) {
        gain_pct as f32 / 100.0
    } else {
        1.0
    };
    let valid_loop = loop_mode != 0 && loop_end > loop_start + 48 && loop_end <= pcm_l.len();

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
            gain: gain_norm,
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
    is_96: bool,
    fallback_keys: &[u8],
    pool: &HashMap<u32, (usize, usize)>,
) -> Vec<SampleZone> {
    let mut zones = Vec::new();
    let hdr_len = if is_96 { 96 } else { 97 };
    let mut cur = start;
    for z in 0..nzones.max(1).min(128) {
        if cur >= block_end {
            break;
        }
        let valid_hdr = if is_96 {
            looks_like_zone_96(data, cur)
        } else {
            looks_like_zone_97(data, cur)
        };
        if !valid_hdr {
            break;
        }
        let nfr = read_u32_le(data, cur + if is_96 { 46 } else { 45 }).unwrap_or(0) as usize;
        let bits = data[cur + if is_96 { 36 } else { 35 }] as usize;
        let ch = data[cur + if is_96 { 37 } else { 36 }] as usize;
        let pcm_bytes = nfr.saturating_mul(ch).saturating_mul((bits / 8).max(1));
        let next_is_hdr = if z + 1 < nzones {
            if is_96 {
                looks_like_zone_96(data, cur + hdr_len)
            } else {
                looks_like_zone_97(data, cur + hdr_len)
            }
        } else {
            cur + hdr_len + 16 >= block_end
        };
        let inline_pcm = pcm_bytes > 0 && !next_is_hdr && cur + hdr_len + pcm_bytes <= block_end;
        if let Some((zone, nxt)) =
            parse_zone_at(data, cur, is_96, inline_pcm, pool, fallback_keys, z)
        {
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
        let template_path = Path::new(DEFAULT_MUSIC_STUDIO_DIR).join("Default.Template");
        if template_path.exists() {
            if let Ok(pkg) = parse_music_file_internal(&template_path, false) {
                for inst in pkg.instruments {
                    if inst.zones.iter().any(|z| !z.pcm_l.is_empty()) {
                        map.insert(inst.name.to_lowercase(), inst.zones);
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
    let fallback_keys: Vec<u8> = data
        .get(hdr_end + 5..hdr_end + 133)
        .map(|s| s.iter().copied().filter(|&k| k >= 12 && k <= 127).collect())
        .unwrap_or_default();

    let mut zones = Vec::new();
    let prim_rel = data
        .get(hdr_end..block_end.min(hdr_end + 3000))
        .and_then(|slice| {
            slice
                .windows(11)
                .position(|w| w == b"\x07\x00\x00\x00Primary")
        });

    if let Some(rel) = prim_rel {
        let after_prim = hdr_end + rel + 11;
        if let Some(&nz) = data.get(after_prim + 6) {
            zones = parse_zones_sequence(
                data,
                after_prim + 7,
                nz as usize,
                block_end,
                false,
                &fallback_keys,
                pool,
            );
        }
    }

    if zones.is_empty() {
        let scan_start = hdr_end + 128;
        let scan_end = block_end.min(hdr_end + 3000);
        let mut found_97 = None;
        let mut found_96 = None;
        for off in scan_start..scan_end {
            if looks_like_zone_97(data, off) {
                found_97 = Some(off);
                break;
            }
            if found_96.is_none() && looks_like_zone_96(data, off) {
                found_96 = Some(off);
            }
        }

        if let Some(z0) = found_97 {
            let nzones = data.get(z0 - 1).copied().unwrap_or(0) as usize;
            zones = parse_zones_sequence(
                data,
                z0,
                nzones,
                block_end,
                false,
                &fallback_keys,
                pool,
            );
        } else if let Some(z0) = found_96 {
            let nzones = read_u32_le(data, z0 - 4)
                .filter(|&v| (1..=128).contains(&v))
                .unwrap_or(1) as usize;
            zones = parse_zones_sequence(
                data,
                z0,
                nzones,
                block_end,
                true,
                &fallback_keys,
                pool,
            );
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
    let is_drumkit = up.contains("BTR")
        || up.contains("DRUM")
        || up.contains("BATERIA")
        || up.contains("KIT")
        || up.contains("PERC")
        || up.contains("VINHETA")
        || up.contains("HIT HAT")
        || up == "GENERAL"
        || name == "-"
        || (zones.len() >= 20
            && zones
                .iter()
                .filter(|z| (33..=75).contains(&z.root_key))
                .count()
                >= 18);

    assign_key_ranges(&mut zones, is_drumkit);

    let display_name = if name == "-" {
        "Metronome / Internal FX".to_string()
    } else {
        name
    };

    LoadedInstrument {
        index,
        name: display_name,
        is_drumkit,
        zones,
    }
}

#[derive(Debug, Clone)]
struct TailNode {
    id: u32,
    parent_id: u32,
    code: u8,
    name: String,
    order: usize,
    length_ticks: u32,
    bpm: Option<f32>,
    slot_code: u8,
}

fn canonical_section_name(raw: &str, slot_code: u8) -> (String, bool) {
    let trimmed = raw.trim();
    let lower = trimmed.to_lowercase();
    let is_cv_alt = lower.contains(" cv2") || lower.contains(" cv3");
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
        "ending 1" | "final 1" => "Ending 1",
        "ending 2" | "final 2" => "Ending 2",
        "ending 3" | "final 3" => "Ending 3",
        _ => match slot_code {
            1 => "Intro 1",
            2 => "Intro 2",
            3 => "Variation 1",
            4 => "Variation 2",
            5 => "Variation 3",
            6 => "Variation 4",
            7 => "Fill 1",
            8 => "Fill 2",
            9 => "Ending 1",
            10 => "Ending 2",
            11 => "Break",
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
        },
    };
    (canon.to_string(), is_cv_alt)
}

fn is_music_studio_magic(data: &[u8]) -> bool {
    data.starts_with(b"Music Studio Music\x00") || data.starts_with(b"Music Studio music\x00")
}

fn parse_tail_tree(data: &[u8]) -> (usize, Vec<TailNode>) {
    let search_start = data.len().saturating_sub(550_000);
    let tail = &data[search_start..];
    // Find Node 0: \x00\x00\x00\x00 \xff\xff\xff\xff <lenByte> Music \0
    let Some(rel) = tail.windows(15).position(|w| {
        &w[0..8] == b"\x00\x00\x00\x00\xff\xff\xff\xff" && &w[9..15] == b"Music\0"
    }) else {
        return (data.len(), Vec::new());
    };
    let n0_off = search_start + rel;
    let count = read_u32_le(data, n0_off.saturating_sub(4)).unwrap_or(0) as usize;
    if count == 0 || count > 2000 {
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
        let name = decode_latin1(&data[cur + 9..cur + 41]);
        let code = data[cur + 41];
        let length_ticks = read_u32_le(data, cur + 48)
            .filter(|&v| (500..=64_000).contains(&v))
            .unwrap_or(4000);
        let bpm = data
            .get(cur + 62)
            .copied()
            .filter(|&b| (40..=240).contains(&b))
            .map(|b| b as f32);
        let slot_code = data.get(cur + 68).copied().unwrap_or(0);
        Some(TailNode {
            id: nid,
            parent_id: pid,
            code,
            name,
            order,
            length_ticks,
            bpm,
            slot_code,
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
) -> HashMap<u32, Vec<RhythmTrack>> {
    let mut by_section: HashMap<u32, Vec<RhythmTrack>> = HashMap::new();
    // Skip Scales + Mixer channels ("Normal" -> scales -> "Master" -> "Up 1..3", "Low", "Rhythm", "Wave", Bass, Acc.1..5, Drum)
    // First advance to "Master" (handles v1.4.6+ files where 10 microtuning scale headers sit between "Normal" and "Master"),
    // then advance past the first "Drum" mixer entry inside the mixer block!
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
        let Some(mut slot) = track_name_to_slot(clean) else {
            continue;
        };
        let next_blk = headers
            .get(idx + 1)
            .map(|h| h.0.min(tail_start))
            .unwrap_or(tail_start);

        let Some(sec_id) = read_u32_le(data, hdr_end) else {
            continue;
        };
        let inst_idx = read_u32_le(data, hdr_end + 12).unwrap_or(0) as usize;
        let vol_raw = data.get(hdr_end + 159).copied().unwrap_or(100);
        let vol = (vol_raw as f32 / 100.0).clamp(0.15, 1.5);
        let octave_shift = read_i32_le(data, hdr_end + 179).unwrap_or(0).clamp(-4, 4);

        let b28 = data.get(hdr_end + 28).copied().unwrap_or(0);
        let b30 = data.get(hdr_end + 30).copied().unwrap_or(0);
        let no_transpose_flag = read_i32_le(data, hdr_end + 143) == Some(1);

        let mut is_drum = slot <= 1;
        if b28 == 0x80 && b30 == 0xC0 {
            slot = 2; // Authored as Bass role
            is_drum = false;
        } else if no_transpose_flag || (b28 == 0 && b30 == 0xC0) {
            is_drum = true;
        }

        // Locate event_count at +531 (v1.3/1.4) or +544 (v1.2)
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
                                duration_ticks: dur.max(60) as u32,
                            });
                        }
                    }
                    events.sort_by_key(|ev| ev.start_tick);
                    break;
                }
            }
        }

        if !events.is_empty() {
            by_section.entry(sec_id).or_default().push(RhythmTrack {
                track_type: clean.to_string(),
                track_slot: slot,
                instrument_index: inst_idx,
                volume: vol,
                octave_shift,
                is_drum,
                events,
            });
        }
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

    let sample_pool = parse_header_and_sample_pool(&data);
    let (tail_start, tail_nodes) = parse_tail_tree(&data);
    let headers = find_versioned_headers(&data[..tail_start], sample_pool.pool_end);

    // Locate Mixer/Scales boundary ("Normal" followed within 24 headers by "Master",
    // since v1.4.6+ stores 10 microtuning scale presets between "Normal" and "Master")
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

    // headers[0] is the Instrument List container header (`name == ""`, `hdr_end + 4 = inst_count`)!
    // Skip it so instrument indices match track `inst_idx` (`hdr_end + 12`) 1-to-1!
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

    let tracks_by_section = parse_rhythm_tracks(&data, &headers, mixer_idx, tail_start);

    // Also upgrade `is_drumkit` from authoritative track role flags
    let mut inst_has_drum_track: HashMap<usize, bool> = HashMap::new();
    for tracks in tracks_by_section.values() {
        for tr in tracks {
            inst_has_drum_track
                .entry(tr.instrument_index)
                .and_modify(|v| *v |= tr.is_drum)
                .or_insert(tr.is_drum);
        }
    }
    for (idx, inst) in instruments.iter_mut().enumerate() {
        if inst_has_drum_track.get(&idx).copied().unwrap_or(false) && !inst.is_drumkit {
            inst.is_drumkit = true;
            assign_key_ranges(&mut inst.zones, true);
        }
    }

    // Parse initial BPM from file header (cur+5 or 0x44)
    let default_bpm = sample_pool.header_bpm.unwrap_or_else(|| {
        let raw_bpm = read_u32_le(&data, 0x44).unwrap_or(120);
        if (40..=240).contains(&raw_bpm) {
            raw_bpm as f32
        } else {
            120.0
        }
    });

    // Build Styles tree from Tail Nodes
    let node_by_id: HashMap<u32, &TailNode> = tail_nodes.iter().map(|n| (n.id, n)).collect();
    let mut styles_map: BTreeMap<(usize, u32), RhythmStyle> = BTreeMap::new();

    for node in &tail_nodes {
        if node.code == 0x01 {
            let Some(tracks) = tracks_by_section.get(&node.id) else {
                continue;
            };
            if tracks.is_empty() {
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

            let (canon_name, is_cv_alt) = canonical_section_name(&node.name, node.slot_code);
            let section_bpm = node.bpm.unwrap_or(default_bpm);
            let max_start_tick = tracks
                .iter()
                .flat_map(|t| t.events.iter())
                .map(|ev| ev.start_tick + 1)
                .max()
                .unwrap_or(node.length_ticks);
            let mut length_ticks = node.length_ticks.max(1000);
            if max_start_tick > length_ticks {
                length_ticks = ((max_start_tick + 999) / 1000 * 1000).max(4000);
            }

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
                    existing.tracks = tracks.clone();
                    existing.length_ticks = length_ticks;
                }
            } else {
                entry.sections.push(RhythmSection {
                    id: node.id,
                    name: canon_name,
                    length_ticks,
                    tracks: tracks.clone(),
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
                length_ticks: ((max_end + 999) / 1000 * 1000).max(4000),
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
    gain: f32,
    env: f32,
    attack_pos: u16,
    releasing: bool,
    remaining_samples: Option<usize>,
    trigger_key: u8,
    from_rhythm: bool,
}

const MAX_SAMPLER_VOICES: usize = 128;

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
            if up.contains("BANDONEON") || up.contains("ACORD") || up.contains("SANFONA") {
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
            }
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
            // Queue Fill/Variation cleanly at bar/section boundary
            if self.auto_fill && section_name.starts_with("Variation") && !self.active_section.starts_with("Fill") {
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

    fn trigger_instrument_note(
        &mut self,
        inst_index: usize,
        midi_note: u8,
        trigger_key: u8,
        velocity: u8,
        gain_scale: f32,
        from_rhythm: bool,
        duration_samples: Option<usize>,
    ) {
        let Some(ref pkg) = self.package else {
            return;
        };
        let Some(inst) = pkg.instruments.get(inst_index) else {
            return;
        };
        // Find matching zone across key ranges, or closest root_key
        let zone = inst
            .zones
            .iter()
            .find(|z| !z.pcm_l.is_empty() && midi_note >= z.key_low && midi_note <= z.key_high)
            .or_else(|| {
                inst.zones
                    .iter()
                    .filter(|z| !z.pcm_l.is_empty())
                    .min_by_key(|z| (z.root_key as i16 - midi_note as i16).abs())
            });
        let Some(z) = zone else {
            return;
        };

        let semitones = if inst.is_drumkit {
            0.0
        } else {
            midi_note as f64 - z.root_key as f64
        };
        let phase_inc = (z.sample_rate as f64 / self.sr as f64) * 2.0_f64.powf(semitones / 12.0);
        let vel_gain = (velocity as f32 / 127.0).powf(0.85);

        if self.voices.len() >= MAX_SAMPLER_VOICES {
            if let Some(pos) = self
                .voices
                .iter()
                .position(|v| v.releasing)
                .or_else(|| self.voices.iter().position(|v| v.from_rhythm))
                .or(Some(0))
            {
                self.voices.remove(pos);
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
            gain: vel_gain * z.gain * gain_scale,
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
                self.trigger_instrument_note(low_inst, played, note, velocity, 0.72, false, None);
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
        self.trigger_instrument_note(u1, played_upper, note, velocity, 1.0, false, None);
        if let Some(u2) = self.upper2_instrument {
            if u2 != u1 {
                self.trigger_instrument_note(u2, played_upper, note, velocity, 0.78, false, None);
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

    fn transpose_for_chord(&self, note: u8) -> u8 {
        // Tracks inside `.Music` are authored against a C Major reference (pitch class 0 = C)
        let mut root_shift = self.detected_chord_root as i16;
        if root_shift > 6 {
            root_shift -= 12; // Keep accompaniment & bass in natural range
        }
        let pc = (note % 12) as i16;
        let chord_iv = match self.detected_chord_quality {
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

    pub fn process_block(&mut self, out_l: &mut [f32], out_r: &mut [f32]) {
        out_l.fill(0.0);
        out_r.fill(0.0);
        let frames = out_l.len().min(out_r.len());
        if frames == 0 {
            return;
        }

        // Advance rhythm sequencer if active
        if self.rhythm_playing && self.package.is_some() {
            let ticks_per_block =
                (frames as f64) * (self.bpm as f64 / 60.0) * 1000.0 / (self.sr as f64);
            let prev_tick = self.current_tick;
            let next_tick = prev_tick + ticks_per_block;

            let mut to_trigger: Vec<(usize, u8, u8, f32, Option<usize>)> = Vec::new();
            let mut transition_next: Option<String> = None;
            let mut stop_at_end = false;

            if let Some(sec) = self.active_section_ref() {
                let sec_len = sec.length_ticks.max(1000) as f64;
                let wrapped = next_tick >= sec_len;
                let wrap_tick = if wrapped { next_tick - sec_len } else { 0.0 };
                let instruments = self
                    .package
                    .as_ref()
                    .map(|p| p.instruments.as_slice())
                    .unwrap_or(&[]);

                for track in &sec.tracks {
                    let inst_is_drum = instruments
                        .get(track.instrument_index)
                        .map(|i| i.is_drumkit)
                        .unwrap_or(false);
                    let is_drum = track.is_drum || inst_is_drum;
                    let vol = track.volume * self.track_volumes[track.track_slot.min(7)];
                    if vol <= 0.005 {
                        continue;
                    }
                    for ev in &track.events {
                        let t = ev.start_tick as f64;
                        let hit = if wrapped {
                            t >= prev_tick || t < wrap_tick
                        } else {
                            t >= prev_tick && t < next_tick
                        };
                        if hit {
                            let target_note = if is_drum {
                                ev.midi_note
                            } else {
                                let shifted = (ev.midi_note as i16
                                    + (track.octave_shift * 12) as i16)
                                    .clamp(12, 115) as u8;
                                self.transpose_for_chord(shifted)
                            };
                            let dur_samples = if is_drum {
                                None
                            } else {
                                let dur_sec = (ev.duration_ticks.max(60) as f64 / 1000.0)
                                    * (60.0 / self.bpm as f64);
                                Some(((dur_sec * self.sr as f64) as usize).max(384))
                            };
                            to_trigger.push((
                                track.instrument_index,
                                target_note,
                                ev.velocity,
                                vol,
                                dur_samples,
                            ));
                        }
                    }
                }

                if wrapped {
                    if let Some(ref pend) = self.pending_section {
                        transition_next = Some(pend.clone());
                    } else if sec.name.starts_with("Intro")
                        || sec.name.starts_with("Fill")
                        || sec.name == "Break"
                    {
                        transition_next = Some(self.last_main_variation.clone());
                    } else if sec.name.starts_with("Ending") {
                        stop_at_end = true;
                    }
                    self.current_tick = wrap_tick;
                } else {
                    self.current_tick = next_tick;
                }
            }

            if stop_at_end {
                self.rhythm_playing = false;
                self.current_tick = 0.0;
            } else if let Some(next_sec) = transition_next {
                self.active_section = next_sec;
                self.pending_section = None;
            }

            for (inst_idx, note, vel, vol, dur_samp) in to_trigger {
                self.trigger_instrument_note(
                    inst_idx,
                    note,
                    note,
                    vel,
                    vol * 0.62,
                    true,
                    dur_samp,
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

                if v.loop_enabled && v.pos >= v.loop_end as f64 && v.loop_end > v.loop_start + 8 {
                    let span = (v.loop_end - v.loop_start) as f64;
                    v.pos = v.loop_start as f64 + ((v.pos - v.loop_start as f64) % span);
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
                let l = s0_l + (s1_l - s0_l) * frac;
                let r = s0_r + (s1_r - s0_r) * frac;
                let g = v.gain * v.env * atk * master;
                out_l[f] += l * g;
                out_r[f] += r * g;

                v.pos += v.phase_inc;
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
}
