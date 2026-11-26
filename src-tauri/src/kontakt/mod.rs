//! Native Instruments Kontakt (`.nkm` Multi / `.nki` Instrument) & `.ncw` Lossless Wave
//! importer for Modo Kontakt arranger playback (paired with companion Yamaha `.STY` styles).

use crate::music_studio::{
    LoadedInstrument, LoadedMusicPackage, MusicPackageEntry, OtsPresetSummary, RhythmNoteEvent,
    RhythmSection, RhythmStyle, RhythmTrack, SampleZone,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const USER_DOWNLOADS_KONTAKT_DIR: &str = "/home/data/Downloads/ritmos kontakt 2026";
pub const USER_DOCUMENTS_KONTAKT_DIR: &str = "/home/data/Documents/Ritmos Kontakt";

const NCW_MAGIC: [u8; 4] = [0x01, 0xa8, 0x9e, 0xd6];
const NCW_BLOCK_MAGIC: u32 = 0x3e9a_0c16;

#[derive(Debug, Clone)]
pub struct KontaktGroupMeta {
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub tune: f32,
    pub key_tracking: bool,
    pub release_trigger: bool,
    pub voice_group: i32,
}

#[derive(Debug, Clone)]
pub struct KontaktZoneMeta {
    pub group_index: usize,
    pub sample_start: u32,
    pub sample_end: u32,
    pub vel_low: u8,
    pub vel_high: u8,
    pub key_low: u8,
    pub key_high: u8,
    pub root_key: u8,
    pub volume: f32,
    pub pan: f32,
    pub tune: f32,
    pub filename_idx: usize,
    pub sample_rate: u32,
    pub channels: u8,
    pub loop_enabled: bool,
    pub loop_start: usize,
    pub loop_end: usize,
}

#[derive(Debug, Clone)]
pub struct KontaktProgramMeta {
    pub slot_index: usize,
    pub midi_channel: u8, // 1-based Kontakt channel (1..16 Port A, 17..32 Port B), 0 = omni/unknown
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub tune: f32,
    pub groups: Vec<KontaktGroupMeta>,
    pub zones: Vec<KontaktZoneMeta>,
}

#[derive(Debug, Clone)]
pub struct ParsedKontaktMulti {
    pub name: String,
    pub filenames: Vec<String>,
    pub programs: Vec<KontaktProgramMeta>,
}

#[inline]
fn read_u16_le(data: &[u8], off: usize) -> Option<u16> {
    let b = data.get(off..off + 2)?;
    Some(u16::from_le_bytes([b[0], b[1]]))
}

#[inline]
fn read_i16_le(data: &[u8], off: usize) -> Option<i16> {
    let b = data.get(off..off + 2)?;
    Some(i16::from_le_bytes([b[0], b[1]]))
}

#[inline]
fn read_u32_le(data: &[u8], off: usize) -> Option<u32> {
    let b = data.get(off..off + 4)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

#[inline]
fn read_i32_le(data: &[u8], off: usize) -> Option<i32> {
    let b = data.get(off..off + 4)?;
    Some(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

#[inline]
fn read_f32_le(data: &[u8], off: usize) -> Option<f32> {
    let b = data.get(off..off + 4)?;
    Some(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn decode_utf16le(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16_lossy(&units)
        .trim_end_matches('\0')
        .to_string()
}

/// Pure-Rust FastLZ (level 1 & 2) decompressor used by Kontakt 4.2+ `DSIN 0x74` preset chunks.
pub fn fastlz_decompress(inp: &[u8], expected_len: usize) -> Result<Vec<u8>, String> {
    if inp.is_empty() {
        return Err("Empty FastLZ input".to_string());
    }
    let mut out = Vec::with_capacity(expected_len.min(64 * 1024 * 1024));
    let mut ip = 0_usize;
    let level = (inp[0] >> 5) + 1;
    let mut ctrl = (inp[ip] & 31) as usize;
    ip += 1;

    while ip < inp.len() {
        if ctrl >= 32 {
            let mut length = (ctrl >> 5) - 1;
            let mut ofs = (ctrl & 31) << 8;
            if level == 2 {
                if length == 6 {
                    loop {
                        let code = *inp.get(ip).ok_or("FastLZ truncated length")? as usize;
                        ip += 1;
                        length += code;
                        if code != 255 {
                            break;
                        }
                    }
                }
                let code = *inp.get(ip).ok_or("FastLZ truncated ofs")? as usize;
                ip += 1;
                ofs += code;
                if code == 255 && (ctrl & 31) == 31 {
                    let hi = *inp.get(ip).ok_or("FastLZ truncated ext ofs hi")? as usize;
                    let lo = *inp.get(ip + 1).ok_or("FastLZ truncated ext ofs lo")? as usize;
                    ip += 2;
                    ofs = (hi << 8) + lo + 8191;
                }
            } else {
                if length == 6 {
                    let code = *inp.get(ip).ok_or("FastLZ L1 truncated length")? as usize;
                    ip += 1;
                    length += code;
                }
                let code = *inp.get(ip).ok_or("FastLZ L1 truncated ofs")? as usize;
                ip += 1;
                ofs += code;
            }
            if ofs + 1 > out.len() {
                return Err(format!("FastLZ invalid back-reference {} > {}", ofs + 1, out.len()));
            }
            let mut r = out.len() - ofs - 1;
            for _ in 0..(length + 3) {
                let b = out[r];
                out.push(b);
                r += 1;
            }
        } else {
            let lit_len = ctrl + 1;
            let end = ip.checked_add(lit_len).ok_or("FastLZ literal overflow")?;
            let slice = inp.get(ip..end).ok_or("FastLZ truncated literal")?;
            out.extend_from_slice(slice);
            ip = end;
        }
        if ip < inp.len() {
            ctrl = inp[ip] as usize;
            ip += 1;
        }
    }
    Ok(out)
}

/// Decodes an unencrypted Native Instruments `.ncw` (Lossless Compressed Wave) buffer
/// into `(pcm_l, pcm_r, sample_rate)`.
pub fn decode_ncw_bytes(data: &[u8]) -> Result<(Vec<f32>, Vec<f32>, u32), String> {
    if data.len() < 36 || data[0..4] != NCW_MAGIC {
        return Err("Not a valid .ncw file".to_string());
    }
    let ch = read_u16_le(data, 8).ok_or("NCW header truncated")? as usize;
    let bits = read_u16_le(data, 10).ok_or("NCW header truncated")? as u32;
    let sr = read_u32_le(data, 12).ok_or("NCW header truncated")?;
    let nsamples = read_u32_le(data, 16).ok_or("NCW header truncated")? as usize;
    let blk_off = read_u32_le(data, 20).ok_or("NCW header truncated")? as usize;
    let data_off = read_u32_le(data, 24).ok_or("NCW header truncated")? as usize;

    if ch == 0 || ch > 2 || bits == 0 || bits > 32 || sr < 4000 || sr > 384_000 {
        return Err(format!("Unsupported NCW format: ch={ch} bits={bits} sr={sr}"));
    }
    if data_off <= blk_off + 4 || data_off > data.len() {
        return Err("Invalid NCW block table offsets".to_string());
    }

    let max_table_blocks = (data_off - blk_off) / 4;
    let num_blocks = nsamples.div_ceil(512).min(max_table_blocks);
    let scale = 1.0_f64 / ((1_i64 << (bits - 1)) as f64);

    let mut out_l = Vec::with_capacity(nsamples.min(num_blocks * 512));
    let mut out_r = if ch == 2 {
        Vec::with_capacity(nsamples.min(num_blocks * 512))
    } else {
        Vec::new()
    };

    let read_channel_block = |pos: usize, dst: &mut Vec<f32>| -> Result<(usize, u16), String> {
        let bmag = read_u32_le(data, pos).ok_or("NCW block truncated")?;
        if bmag != NCW_BLOCK_MAGIC {
            return Err(format!("Invalid NCW block magic 0x{bmag:08x}"));
        }
        let bbase_u32 = read_u32_le(data, pos + 4).ok_or("NCW block base truncated")?;
        let bbase = bbase_u32 as i32 as i64;
        let bbits = read_i16_le(data, pos + 8).ok_or("NCW block bits truncated")? as i32;
        let bflags = read_u16_le(data, pos + 10).ok_or("NCW block flags truncated")?;
        let is_float = (bflags & 2) != 0;
        let payload_start = pos + 16;

        let sanitize_f32 = |v: f32| -> f32 {
            if v.is_finite() {
                v.clamp(-4.0, 4.0)
            } else {
                0.0
            }
        };

        if bbits == 0 {
            let v = if is_float {
                sanitize_f32(f32::from_bits(bbase_u32))
            } else {
                (bbase as f64 * scale).clamp(-1.0, 1.0) as f32
            };
            dst.resize(dst.len() + 512, v);
            return Ok((payload_start, bflags));
        }

        let abits = bbits.unsigned_abs() as u32;
        if abits == 0 || abits > 32 {
            return Err(format!("Invalid NCW block bit width {bbits}"));
        }
        let nbytes = (512 * abits as usize).div_ceil(8);
        if payload_start + nbytes > data.len() {
            return Err("NCW block payload exceeds file length".to_string());
        }

        let mask: u64 = if abits == 32 { u32::MAX as u64 } else { (1_u64 << abits) - 1 };
        let sign_bit: u64 = 1_u64 << (abits - 1);
        let sign_ext: i64 = 1_i64 << abits;

        let mut acc: u64 = 0;
        let mut avail: u32 = 0;
        let mut bp = payload_start;
        let mut val = bbase;
        let mut val_u32 = bbase_u32;

        for _ in 0..512 {
            if bbits > 0 {
                if is_float {
                    dst.push(sanitize_f32(f32::from_bits(val_u32)));
                } else {
                    dst.push((val as f64 * scale).clamp(-1.0, 1.0) as f32);
                }
                while avail < abits {
                    acc |= (data[bp] as u64) << avail;
                    bp += 1;
                    avail += 8;
                }
                let raw = acc & mask;
                acc >>= abits;
                avail -= abits;
                let delta = if (raw & sign_bit) != 0 {
                    raw as i64 - sign_ext
                } else {
                    raw as i64
                };
                if is_float {
                    val_u32 = val_u32.wrapping_add(delta as i32 as u32);
                } else {
                    val = val.wrapping_add(delta);
                }
            } else {
                while avail < abits {
                    acc |= (data[bp] as u64) << avail;
                    bp += 1;
                    avail += 8;
                }
                let raw = acc & mask;
                acc >>= abits;
                avail -= abits;
                if is_float && abits == 32 {
                    dst.push(sanitize_f32(f32::from_bits(raw as u32)));
                } else {
                    let sample = if (raw & sign_bit) != 0 {
                        raw as i64 - sign_ext
                    } else {
                        raw as i64
                    };
                    dst.push((sample as f64 * scale).clamp(-1.0, 1.0) as f32);
                }
            }
        }
        Ok((payload_start + nbytes, bflags))
    };

    for bi in 0..num_blocks {
        let rel = read_u32_le(data, blk_off + bi * 4).ok_or("NCW block offset truncated")? as usize;
        let pos = data_off + rel;
        if ch == 1 {
            let _ = read_channel_block(pos, &mut out_l)?;
        } else {
            let blk_start = out_l.len();
            let (pos2, l_flags) = read_channel_block(pos, &mut out_l)?;
            let _ = read_channel_block(pos2, &mut out_r)?;
            // Kontakt NCW bit 0 (0x01) indicates Mid/Side stereo coding: L = M + S, R = M - S.
            if (l_flags & 1) != 0 {
                let blk_end = out_l.len().min(out_r.len());
                for i in blk_start..blk_end {
                    let m = out_l[i];
                    let s = out_r[i];
                    out_l[i] = m + s;
                    out_r[i] = m - s;
                }
            }
        }
    }

    out_l.truncate(nsamples);
    if ch == 2 {
        out_r.truncate(nsamples);
    }
    Ok((out_l, out_r, sr))
}

/// Resilient RIFF/WAVE decoder for WAV files with non-standard `fmt ` chunk sizes
/// (e.g. 18-byte PCM fmt chunks or Kontakt/DAW-exported WAVs rejected by `hound`).
fn decode_wav_bytes_fallback(bytes: &[u8]) -> Result<(Vec<f32>, Vec<f32>, u32), String> {
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("Not a valid RIFF/WAVE file".to_string());
    }
    let mut audio_format = 1_u16;
    let mut channels = 1_u16;
    let mut sample_rate = 44_100_u32;
    let mut bits_per_sample = 16_u16;
    let mut data_slice: Option<&[u8]> = None;

    let mut pos = 12_usize;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let sz = read_u32_le(bytes, pos + 4).unwrap_or(0) as usize;
        let body_start = pos + 8;
        let body_end = (body_start + sz).min(bytes.len());
        if id == b"fmt " && body_end >= body_start + 16 {
            let mut fmt = read_u16_le(bytes, body_start).unwrap_or(1);
            channels = read_u16_le(bytes, body_start + 2).unwrap_or(1).max(1);
            sample_rate = read_u32_le(bytes, body_start + 4).unwrap_or(44_100).max(8000);
            bits_per_sample = read_u16_le(bytes, body_start + 14).unwrap_or(16);
            if fmt == 0xFFFE && body_end >= body_start + 26 {
                fmt = read_u16_le(bytes, body_start + 24).unwrap_or(1);
            }
            audio_format = fmt;
        } else if id == b"data" {
            data_slice = Some(&bytes[body_start..body_end]);
            break;
        }
        // Chunks are word-aligned (padded to even byte boundary)
        pos = body_start + sz + (sz & 1);
    }

    let raw = data_slice.ok_or_else(|| "WAV missing data chunk".to_string())?;
    let ch = channels as usize;
    let bps = (bits_per_sample as usize).div_ceil(8);
    if bps == 0 || ch == 0 {
        return Err("Invalid WAV sample format".to_string());
    }
    let frame_bytes = bps * ch;
    let nframes = raw.len() / frame_bytes;
    let mut pcm_l = Vec::with_capacity(nframes);
    let mut pcm_r = if ch >= 2 {
        Vec::with_capacity(nframes)
    } else {
        Vec::new()
    };

    let decode_one = |chunk: &[u8]| -> f32 {
        match (audio_format, bits_per_sample) {
            (3, 32) => f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]).clamp(-1.0, 1.0),
            (3, 64) if chunk.len() >= 8 => {
                f64::from_le_bytes([
                    chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
                ])
                .clamp(-1.0, 1.0) as f32
            }
            (_, 8) => ((chunk[0] as f32) - 128.0) / 128.0,
            (_, 16) => (i16::from_le_bytes([chunk[0], chunk[1]]) as f32) / 32768.0,
            (_, 24) => {
                let b0 = chunk[0] as i32;
                let b1 = (chunk[1] as i32) << 8;
                let b2 = (chunk[2] as i8 as i32) << 16;
                ((b0 | b1 | b2) as f32) / 8_388_608.0
            }
            (_, 32) => {
                (i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f32)
                    / 2_147_483_648.0
            }
            _ => 0.0,
        }
    };

    for frame in raw.chunks_exact(frame_bytes) {
        pcm_l.push(decode_one(&frame[0..bps]));
        if ch >= 2 {
            pcm_r.push(decode_one(&frame[bps..bps * 2]));
        }
    }
    Ok((pcm_l, pcm_r, sample_rate))
}

/// Decodes either a `.ncw` or `.wav` sample file into `(pcm_l, pcm_r, sample_rate)`.
pub fn decode_sample_file(path: &Path) -> Result<(Vec<f32>, Vec<f32>, u32), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "ncw" {
        let bytes = std::fs::read(path)
            .map_err(|e| format!("Failed to read '{}': {e}", path.display()))?;
        return decode_ncw_bytes(&bytes);
    }
    if ext == "wav" {
        if let Ok(mut reader) = hound::WavReader::open(path) {
            let spec = reader.spec();
            let sr = spec.sample_rate.max(8000);
            let ch = spec.channels.max(1) as usize;
            let mut pcm_l = Vec::new();
            let mut pcm_r = Vec::new();
            match spec.sample_format {
                hound::SampleFormat::Float => {
                    let samples: Vec<f32> = reader.samples::<f32>().filter_map(Result::ok).collect();
                    for frame in samples.chunks(ch) {
                        pcm_l.push(frame[0]);
                        if ch >= 2 {
                            pcm_r.push(frame[1]);
                        }
                    }
                }
                hound::SampleFormat::Int => {
                    let scale = 1.0_f32
                        / ((1_i64 << (spec.bits_per_sample.saturating_sub(1).min(31))) as f32);
                    let samples: Vec<i32> = reader.samples::<i32>().filter_map(Result::ok).collect();
                    for frame in samples.chunks(ch) {
                        pcm_l.push((frame[0] as f32 * scale).clamp(-1.0, 1.0));
                        if ch >= 2 {
                            pcm_r.push((frame[1] as f32 * scale).clamp(-1.0, 1.0));
                        }
                    }
                }
            }
            if !pcm_l.is_empty() {
                return Ok((pcm_l, pcm_r, sr));
            }
        }
        let bytes = std::fs::read(path)
            .map_err(|e| format!("Failed to read WAV '{}': {e}", path.display()))?;
        return decode_wav_bytes_fallback(&bytes);
    }
    Err(format!("Unsupported sample extension '{}'", path.display()))
}

fn decompress_dsin_payload(data: &[u8]) -> Result<Vec<u8>, String> {
    let needle = b"DSIN\x74\x00\x00\x00";
    let p74 = data
        .windows(needle.len())
        .position(|w| w == needle)
        .ok_or_else(|| "Kontakt DSIN 0x74 preset chunk not found".to_string())?;
    let search_end = (p74 + 160).min(data.len().saturating_sub(9));
    for off in p74..search_end {
        if data[off] == 1 {
            let usz = read_u32_le(data, off + 1).unwrap_or(0) as usize;
            let csz = read_u32_le(data, off + 5).unwrap_or(0) as usize;
            if (512..=64 * 1024 * 1024).contains(&usz)
                && csz >= 32
                && off + 9 + csz <= data.len()
            {
                if let Ok(dec) = fastlz_decompress(&data[off + 9..off + 9 + csz], usz) {
                    if dec.len() == usz {
                        return Ok(dec);
                    }
                }
            }
        }
    }
    Err("Failed to decompress Kontakt FastLZ preset payload".to_string())
}

fn parse_filename_list_004b(buf: &[u8]) -> Vec<String> {
    // FilenameList2 (0x004b): each file entry contains a segment marker 0x04 followed by
    // u32 char_len and UTF-16LE filename (e.g. "Bumbo 1.ncw").
    let mut out = Vec::new();
    let mut i = 0_usize;
    while i + 5 < buf.len() {
        if buf[i] == 0x04 {
            let slen = read_u32_le(buf, i + 1).unwrap_or(0) as usize;
            if (4..=240).contains(&slen) && i + 5 + slen * 2 <= buf.len() {
                let s = decode_utf16le(&buf[i + 5..i + 5 + slen * 2]);
                let lower = s.to_ascii_lowercase();
                if lower.ends_with(".ncw") || lower.ends_with(".wav") || lower.ends_with(".aif") || lower.ends_with(".aiff") {
                    out.push(s);
                    i += 5 + slen * 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

fn parse_zone_loop_0039(zch: &[u8]) -> (bool, usize, usize) {
    let mut lc = 0_usize;
    while lc + 6 <= zch.len() {
        let lcid = read_u16_le(zch, lc).unwrap_or(0);
        let lcsz = read_u32_le(zch, lc + 2).unwrap_or(0) as usize;
        if lc + 6 + lcsz > zch.len() {
            break;
        }
        if lcid == 0x0039 && lcsz >= 16 {
            let lb = &zch[lc + 6..lc + 6 + lcsz];
            let count = lb[0];
            if count >= 1 {
                let lmode = read_u32_le(lb, 4).unwrap_or(0);
                let lstart = read_u32_le(lb, 8).unwrap_or(0) as usize;
                let llen = read_u32_le(lb, 12).unwrap_or(0) as usize;
                if lmode > 0 && llen > 64 {
                    return (true, lstart, lstart.saturating_add(llen));
                }
            }
        }
        lc += 6 + lcsz;
    }
    (false, 0, 0)
}

fn parse_groups_0033(gdata: &[u8]) -> Vec<KontaktGroupMeta> {
    let mut groups = Vec::new();
    if gdata.len() < 12 {
        return groups;
    }
    let count = read_u32_le(gdata, 0).unwrap_or(0) as usize;
    let mut cur = 4_usize;
    for gi in 0..count.min(512) {
        if cur + 11 > gdata.len() {
            break;
        }
        // +0: u8 is_struct, +1: u16 ver, +3: u32 pub_sz
        let pub_sz = read_u32_le(gdata, cur + 3).unwrap_or(0) as usize;
        let priv_off = cur + 7 + pub_sz;
        if priv_off + 4 > gdata.len() {
            break;
        }
        let priv_sz = read_u32_le(gdata, priv_off).unwrap_or(0) as usize;
        let priv_start = priv_off + 4;
        let priv_end = priv_start + priv_sz;
        if priv_end + 4 > gdata.len() {
            break;
        }
        let child_sz = read_u32_le(gdata, priv_end).unwrap_or(0) as usize;
        let child_end = (priv_end + 4 + child_sz).min(gdata.len());

        let gpriv = &gdata[priv_start..priv_end];
        let slen = read_u32_le(gpriv, 0).unwrap_or(0) as usize;
        let (name, post) = if slen <= 256 && 4 + slen * 2 <= gpriv.len() {
            let raw = decode_utf16le(&gpriv[4..4 + slen * 2]);
            let clean = if raw.trim().is_empty() {
                format!("Group {}", gi + 1)
            } else {
                raw.trim().to_string()
            };
            (clean, &gpriv[4 + slen * 2..])
        } else {
            (format!("Group {}", gi + 1), &[][..])
        };

        let volume = read_f32_le(post, 0)
            .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 8.0)
            .unwrap_or(1.0);
        let pan = read_f32_le(post, 4)
            .filter(|v| v.is_finite() && *v >= -1.0 && *v <= 1.0)
            .unwrap_or(0.0);
        let tune = read_f32_le(post, 8)
            .filter(|v| v.is_finite() && *v > 0.1 && *v < 16.0)
            .unwrap_or(1.0);
        let key_tracking = post.get(12).copied().unwrap_or(1) != 0;
        let release_trigger = post.get(14).copied().unwrap_or(0) != 0;
        let voice_group = read_i32_le(post, 22).unwrap_or(-1);

        groups.push(KontaktGroupMeta {
            name,
            volume,
            pan,
            tune,
            key_tracking,
            release_trigger,
            voice_group,
        });
        cur = child_end;
    }
    groups
}

fn parse_zones_0034(zdata: &[u8]) -> Vec<KontaktZoneMeta> {
    let mut zones = Vec::new();
    if zdata.len() < 12 {
        return zones;
    }
    let count = read_u32_le(zdata, 0).unwrap_or(0) as usize;
    let mut cur = 4_usize;
    for _ in 0..count.min(4096) {
        if cur + 15 > zdata.len() {
            break;
        }
        // +0: u32 group_index, +4: u8 is_struct(1), +5: u16 ver, +7: u32 pub_sz
        let group_index = read_u32_le(zdata, cur).unwrap_or(0) as usize;
        let pub_sz = read_u32_le(zdata, cur + 7).unwrap_or(0) as usize;
        let priv_off = cur + 11 + pub_sz;
        if priv_off + 4 > zdata.len() {
            break;
        }
        let priv_sz = read_u32_le(zdata, priv_off).unwrap_or(0) as usize;
        let priv_start = priv_off + 4;
        let priv_end = priv_start + priv_sz;
        if priv_end + 4 > zdata.len() {
            break;
        }
        let child_sz = read_u32_le(zdata, priv_end).unwrap_or(0) as usize;
        let child_start = priv_end + 4;
        let child_end = (child_start + child_sz).min(zdata.len());

        // BZone v0x9a has priv_sz >= 82 (filename_idx@48, sr@56, ch@60);
        // BZone v0x99 has priv_sz == 76 (filename_idx@42, sr@50, ch@54).
        if priv_sz >= 56 {
            let p = &zdata[priv_start..priv_end];
            let sample_start = read_u32_le(p, 0).unwrap_or(0);
            let sample_end = read_u32_le(p, 4).unwrap_or(0);
            let vel_low = read_u16_le(p, 12).unwrap_or(0).min(127) as u8;
            let vel_high = read_u16_le(p, 14).unwrap_or(127).min(127) as u8;
            let key_low = read_u16_le(p, 16).unwrap_or(0).min(127) as u8;
            let key_high = read_u16_le(p, 18).unwrap_or(127).min(127) as u8;
            let root_key = read_u16_le(p, 28).unwrap_or(60).min(127) as u8;
            let volume = read_f32_le(p, 30)
                .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 8.0)
                .unwrap_or(1.0);
            let pan = read_f32_le(p, 34)
                .filter(|v| v.is_finite() && *v >= -1.0 && *v <= 1.0)
                .unwrap_or(0.0);
            let tune = read_f32_le(p, 38)
                .filter(|v| v.is_finite() && *v > 0.1 && *v < 16.0)
                .unwrap_or(1.0);
            let (fidx_off, sr_off, ch_off) = if priv_sz >= 82 {
                (48_usize, 56_usize, 60_usize)
            } else {
                (42_usize, 50_usize, 54_usize)
            };
            let filename_idx = read_u32_le(p, fidx_off).unwrap_or(0) as usize;
            let sample_rate = read_u32_le(p, sr_off).unwrap_or(44_100);
            let channels = p.get(ch_off).copied().unwrap_or(1).clamp(1, 2);
            let (loop_enabled, loop_start, loop_end) =
                parse_zone_loop_0039(&zdata[child_start..child_end]);

            zones.push(KontaktZoneMeta {
                group_index,
                sample_start,
                sample_end,
                vel_low: vel_low.min(vel_high),
                vel_high: vel_high.max(vel_low),
                key_low: key_low.min(key_high),
                key_high: key_high.max(key_low),
                root_key,
                volume,
                pan,
                tune,
                filename_idx,
                sample_rate: if (8000..=384_000).contains(&sample_rate) {
                    sample_rate
                } else {
                    44_100
                },
                channels,
                loop_enabled,
                loop_start,
                loop_end,
            });
        }
        cur = child_end;
    }
    zones
}

fn parse_program_0036(
    buf: &[u8],
    slot_index: usize,
    midi_channel: u8,
) -> Option<KontaktProgramMeta> {
    if buf.len() < 16 {
        return None;
    }
    // +0: u32(1), +4: u8(1), +5: u16 ver, +7: u32 pub_sz
    let pub_sz = read_u32_le(buf, 7)? as usize;
    let priv_off = 11 + pub_sz;
    let priv_sz = read_u32_le(buf, priv_off)? as usize;
    let priv_slice = buf.get(priv_off + 4..priv_off + 4 + priv_sz)?;

    let slen = read_u32_le(priv_slice, 0)? as usize;
    let name = if slen > 0 && slen <= 128 && 4 + slen * 2 <= priv_slice.len() {
        let raw = decode_utf16le(&priv_slice[4..4 + slen * 2]);
        let first_line = raw.lines().next().unwrap_or(&raw).trim();
        if first_line.is_empty() {
            format!("Kontakt Slot {}", slot_index + 1)
        } else {
            first_line.to_string()
        }
    } else {
        format!("Kontakt Slot {}", slot_index + 1)
    };
    let after = priv_slice.get(4 + slen * 2..).unwrap_or(&[]);
    let volume = read_f32_le(after, 9)
        .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 4.0)
        .unwrap_or(1.0);
    let pan = read_f32_le(after, 13)
        .filter(|v| v.is_finite() && *v >= -1.0 && *v <= 1.0)
        .unwrap_or(0.0);
    let tune = read_f32_le(after, 17)
        .filter(|v| v.is_finite() && *v > 0.25 && *v < 4.0)
        .unwrap_or(1.0);

    let children_off = priv_off + 4 + priv_sz;
    let child_sz = read_u32_le(buf, children_off).unwrap_or(0) as usize;
    let mut cur = children_off + 4;
    let end = (cur + child_sz).min(buf.len());
    let mut groups = Vec::new();
    let mut zones = Vec::new();
    while cur + 6 <= end {
        let cid = read_u16_le(buf, cur).unwrap_or(0);
        let csz = read_u32_le(buf, cur + 2).unwrap_or(0) as usize;
        if cur + 6 + csz > end {
            break;
        }
        if cid == 0x0033 {
            groups = parse_groups_0033(&buf[cur + 6..cur + 6 + csz]);
        } else if cid == 0x0034 {
            zones = parse_zones_0034(&buf[cur + 6..cur + 6 + csz]);
        }
        cur += 6 + csz;
    }

    Some(KontaktProgramMeta {
        slot_index,
        midi_channel,
        name,
        volume,
        pan,
        tune,
        groups,
        zones,
    })
}

/// Parses an unencrypted Kontakt `.nkm` Multi or `.nki` Instrument file.
pub fn parse_kontakt_file(path: &Path) -> Result<ParsedKontaktMulti, String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("Falha ao ler '{}': {e}", path.display()))?;
    let dec = decompress_dsin_payload(&data)?;

    // Locate root BChunk (0x0003 BMulti or 0x0028/0x0036 BProgram) around 0x00a0..0x0100
    let mut root_off = None;
    for probe in 0x00a0..0x0100.min(dec.len().saturating_sub(6)) {
        let cid = read_u16_le(&dec, probe).unwrap_or(0);
        let csz = read_u32_le(&dec, probe + 2).unwrap_or(0) as usize;
        if matches!(cid, 0x0003 | 0x0028 | 0x0036) && csz > 64 && probe + 6 + csz <= dec.len() {
            root_off = Some((probe, cid, csz));
            break;
        }
    }
    let (r_off, r_cid, r_sz) =
        root_off.ok_or_else(|| "Estrutura BChunk Kontakt não encontrada".to_string())?;

    // Find top-level FilenameList (0x004b)
    let mut filenames = Vec::new();
    let mut top_cur = r_off;
    while top_cur + 6 <= dec.len() {
        let cid = read_u16_le(&dec, top_cur).unwrap_or(0);
        let csz = read_u32_le(&dec, top_cur + 2).unwrap_or(0) as usize;
        if csz == 0 || top_cur + 6 + csz > dec.len() {
            break;
        }
        if cid == 0x004b {
            filenames = parse_filename_list_004b(&dec[top_cur + 6..top_cur + 6 + csz]);
        }
        top_cur += 6 + csz;
    }

    let fallback_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Kontakt Multi")
        .to_string();

    if r_cid == 0x0036 {
        let prog = parse_program_0036(&dec[r_off + 6..r_off + 6 + r_sz], 0, 1)
            .ok_or_else(|| "Falha ao ler programa .nki".to_string())?;
        return Ok(ParsedKontaktMulti {
            name: prog.name.clone(),
            filenames,
            programs: vec![prog],
        });
    }

    // Parse BMulti (0x0003)
    let body = r_off + 6;
    let body_end = body + r_sz;
    let pub_sz = read_u32_le(&dec, body + 10).unwrap_or(0) as usize;
    let children_start = body + 14 + pub_sz + 4;

    // Pass 1: find 0x0048 (64 x 24-byte MIDI channel table)
    let mut slot_channels: Vec<u8> = Vec::new();
    let mut prog_list_slice: Option<&[u8]> = None;
    let mut cur = children_start;
    while cur + 6 <= body_end {
        let cid = read_u16_le(&dec, cur).unwrap_or(0);
        let csz = read_u32_le(&dec, cur + 2).unwrap_or(0) as usize;
        if cur + 6 + csz > body_end {
            break;
        }
        if cid == 0x0048 {
            let c48 = &dec[cur + 6..cur + 6 + csz];
            for rec in c48.chunks_exact(24) {
                let raw = u32::from_le_bytes([rec[0], rec[1], rec[2], rec[3]]);
                let ch = raw & 0xFFFF;
                slot_channels.push(if (1..=64).contains(&ch) { ch as u8 } else { 0 });
            }
        } else if cid == 0x0037 {
            prog_list_slice = Some(&dec[cur + 6..cur + 6 + csz]);
        }
        cur += 6 + csz;
    }

    // Pass 2: parse 0x0029 program containers inside 0x0037.
    // The first 8 bytes of 0x0037 are a little-endian u64 64-slot occupancy bitmask:
    // each set bit `b` (0..64) maps the corresponding 0x0029 program to `slot_channels[b]` in 0x0048.
    let mut programs = Vec::new();
    if let Some(pl) = prog_list_slice {
        let occupied_slots: Vec<usize> = if pl.len() >= 8 {
            let mask = u64::from_le_bytes([
                pl[0], pl[1], pl[2], pl[3], pl[4], pl[5], pl[6], pl[7],
            ]);
            (0..64).filter(|&b| ((mask >> b) & 1) != 0).collect()
        } else {
            Vec::new()
        };
        let mut ic = 8_usize;
        let mut prog_ord = 0_usize;
        while ic + 6 <= pl.len() {
            let icid = read_u16_le(pl, ic).unwrap_or(0);
            let icsz = read_u32_le(pl, ic + 2).unwrap_or(0) as usize;
            if icid != 0x0029 || ic + 6 + icsz > pl.len() {
                break;
            }
            let rack_slot = occupied_slots.get(prog_ord).copied().unwrap_or(prog_ord);
            let sub = &pl[ic + 6..ic + 6 + icsz];
            if let Some(p0_pub_sz) = read_u32_le(sub, 3).map(|v| v as usize) {
                let p0_priv_off = 7 + p0_pub_sz;
                if let Some(p0_priv_sz) = read_u32_le(sub, p0_priv_off).map(|v| v as usize) {
                    let c_off = p0_priv_off + 4 + p0_priv_sz;
                    let c_sz = read_u32_le(sub, c_off).unwrap_or(0) as usize;
                    let mut cc = c_off + 4;
                    let cend = (cc + c_sz).min(sub.len());
                    while cc + 6 <= cend {
                        let ccid = read_u16_le(sub, cc).unwrap_or(0);
                        let ccsz = read_u32_le(sub, cc + 2).unwrap_or(0) as usize;
                        if cc + 6 + ccsz > cend {
                            break;
                        }
                        if ccid == 0x0036 {
                            let ch = slot_channels.get(rack_slot).copied().unwrap_or(0);
                            if let Some(prog) =
                                parse_program_0036(&sub[cc + 6..cc + 6 + ccsz], rack_slot, ch)
                            {
                                programs.push(prog);
                            }
                        }
                        cc += 6 + ccsz;
                    }
                }
            }
            prog_ord += 1;
            ic += 6 + icsz;
        }
    }

    Ok(ParsedKontaktMulti {
        name: fallback_name,
        filenames,
        programs,
    })
}

fn index_sample_files_in_dir(dir: &Path, depth: usize, out: &mut HashMap<String, PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() && depth > 0 {
            index_sample_files_in_dir(&p, depth - 1, out);
        } else if p.is_file() {
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("ncw") || ext.eq_ignore_ascii_case("wav") {
                    if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                        out.entry(name.to_lowercase()).or_insert(p);
                    }
                }
            }
        }
    }
}

fn find_companion_sty_files(base_dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(base_dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_file() {
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if matches!(
                    ext.to_ascii_lowercase().as_str(),
                    "sty" | "prs" | "sst" | "bcs"
                ) {
                    out.push(p);
                }
            }
        }
    }
    out.sort();
    out
}

fn is_drum_program_name(name: &str) -> bool {
    let up = name.to_uppercase();
    up.contains("BUMBO")
        || up.contains("PEDAL")
        || up.contains("CAIXA")
        || up.contains("CX ")
        || up.contains("CHIMBAL")
        || up.contains("CIMBAL")
        || up.contains("TOM")
        || up.contains("PRATO")
        || up.contains("PERC")
        || up.contains("CLAP")
        || up.contains("ZABUMBA")
        || up.contains("TRIANG")
        || up.contains("TRIÂNG")
        || up.contains("AGOG")
        || up.contains("CONGA")
        || up.contains("GONGA")
        || up.contains("BONGO")
        || up.contains("BOMGO")
        || up.contains("REPIK")
        || up.contains("SURD")
        || up.contains("PANDEIR")
        || up.contains("GIRO")
        || up.contains("BACURINH")
        || up.contains("CLAVE")
        || up.contains("JAM BLOCK")
        || up.contains("RUNFO")
        || up.contains("CARILH")
        || up.contains("BATER")
        || up.contains("DRUM")
        || up.contains("KIT")
}

fn decode_program_zones(
    prog: &KontaktProgramMeta,
    is_drum: bool,
    filenames: &[String],
    sample_index: &HashMap<String, PathBuf>,
    pcm_cache: &mut HashMap<PathBuf, Arc<(Vec<f32>, Vec<f32>, u32)>>,
) -> Vec<SampleZone> {
    let mut zones = Vec::with_capacity(prog.zones.len());
    let prog_pan_i = (prog.pan * 100.0).round() as i32;
    let prog_tune_ratio = if prog.tune > 0.25 && prog.tune < 4.0 {
        prog.tune as f64
    } else {
        1.0
    };
    let mut seen_melodic_slots: HashSet<(u8, u8, u8, u8)> = HashSet::new();

    for z in &prog.zones {
        let grp = prog.groups.get(z.group_index);
        if grp.map(|g| g.release_trigger).unwrap_or(false) {
            // Release-trigger groups (e.g. guitar fret/string release noises) must not fire on Note-On.
            continue;
        }
        // Deduplicate multi-group identical (key_low, key_high, vel_low, vel_high) melodic overlaps
        // so alternating groups/octaves don't collide on adjacent keys.
        if !is_drum && !seen_melodic_slots.insert((z.key_low, z.key_high, z.vel_low, z.vel_high)) {
            continue;
        }

        let Some(fname) = filenames.get(z.filename_idx) else {
            continue;
        };
        let Some(path) = sample_index.get(&fname.to_lowercase()) else {
            continue;
        };
        let decoded = if let Some(cached) = pcm_cache.get(path) {
            Some(Arc::clone(cached))
        } else {
            decode_sample_file(path).ok().map(|tup| {
                let arc = Arc::new(tup);
                pcm_cache.insert(path.clone(), Arc::clone(&arc));
                arc
            })
        };

        let Some(pcm) = decoded else {
            continue;
        };
        let raw_len = pcm.0.len();
        if raw_len < 4 {
            continue;
        }

        // Honor Kontakt BZone sample_start / sample_end offsets (crucial for trimmed drum hits)
        let s_start = (z.sample_start as usize).min(raw_len.saturating_sub(4));
        let s_end = if z.sample_end > 0 && (z.sample_end as usize) > s_start + 32 {
            (z.sample_end as usize).min(raw_len)
        } else {
            raw_len
        };

        let mut sliced_l = pcm.0[s_start..s_end].to_vec();
        let mut sliced_r = if pcm.1.len() >= s_end {
            pcm.1[s_start..s_end].to_vec()
        } else {
            Vec::new()
        };
        // Short 32-sample de-click fade-in when trimmed mid-waveform
        if s_start > 0 {
            let fade = sliced_l.len().min(32);
            for i in 0..fade {
                let w = i as f32 / fade as f32;
                sliced_l[i] *= w;
                if i < sliced_r.len() {
                    sliced_r[i] *= w;
                }
            }
        }
        let sliced_len = sliced_l.len();
        let pcm_l = Arc::new(sliced_l);
        let pcm_r = if sliced_r.is_empty() {
            Arc::clone(&pcm_l)
        } else {
            Arc::new(sliced_r)
        };

        let sr = if pcm.2 > 0 { pcm.2 } else { z.sample_rate };
        let z_tune = if z.tune > 0.1 && z.tune < 16.0 {
            z.tune as f64
        } else {
            1.0
        };
        let g_vol = grp.map(|g| g.volume).unwrap_or(1.0);
        let g_pan = grp.map(|g| g.pan).unwrap_or(0.0);
        let g_tune = grp
            .map(|g| g.tune as f64)
            .filter(|t| *t > 0.1 && *t < 16.0)
            .unwrap_or(1.0);
        let g_key_tracking = grp.map(|g| g.key_tracking).unwrap_or(true);
        let g_exclusive_group = grp.map(|g| g.voice_group).unwrap_or(-1);

        let total_tune_semitones = 12.0 * (z_tune * g_tune * prog_tune_ratio).log2();
        let fine_tune = (total_tune_semitones * 1000.0)
            .round()
            .clamp(-12000.0, 12000.0) as i32;
        let pan = (prog_pan_i + ((z.pan + g_pan) * 100.0).round() as i32).clamp(-100, 100);

        // Fix known 1-octave-low high-key mapping in Brazilian Acd Master packs (e.g. Sao Joao / Biu)
        // where "Acd Master 2 mi" (E4) is mapped at E5/E6 (root_key 76/88) above "Acd Master 3 re sus".
        let fname_lower = fname.to_ascii_lowercase();
        let root_key = if !is_drum && fname_lower.contains("acd master 2 mi") && z.root_key >= 72 {
            z.root_key.saturating_sub(12)
        } else {
            z.root_key
        };

        // Honor authored Kontakt BProgram * BGroup * BZone volumes faithfully (without artificial
        // lower clamps that previously amplified quiet Triangle/Percussion groups 10x-33x).
        let gain = if is_drum {
            (prog.volume.clamp(0.0, 2.0) * g_vol * z.volume).clamp(0.0, 2.0)
        } else {
            (g_vol * z.volume).clamp(0.0, 2.0)
        };

        let (loop_enabled, loop_start, loop_end, loop_mode) = if !is_drum && z.loop_enabled {
            let ls = z.loop_start.saturating_sub(s_start);
            let le = z.loop_end.saturating_sub(s_start).min(sliced_len);
            if le > ls + 64 {
                (true, ls, le, 2_u8)
            } else {
                (false, 0, 0, 0_u8)
            }
        } else {
            (false, 0, 0, 0_u8)
        };

        zones.push(SampleZone {
            root_key,
            key_low: z.key_low,
            key_high: z.key_high,
            vel_low: z.vel_low,
            vel_high: z.vel_high,
            name: fname.clone(),
            sample_rate: sr,
            channels: if Arc::ptr_eq(&pcm_l, &pcm_r) { 1 } else { 2 },
            loop_start,
            loop_end,
            loop_enabled,
            loop_mode,
            gain,
            pan,
            fine_tune,
            key_tracking: g_key_tracking,
            exclusive_group: g_exclusive_group,
            pool_sid: 0,
            pcm_l,
            pcm_r,
        });
    }

    if !is_drum {
        // Sort narrow/exact-key zones before wide stretch zones so per-key samples always win.
        zones.sort_by_key(|z| {
            (
                z.key_high.saturating_sub(z.key_low),
                z.key_low,
                z.vel_low,
            )
        });
    }
    zones
}

fn slot_label(slot: usize) -> &'static str {
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

/// Builds a playable `LoadedMusicPackage` in **Modo Kontakt** from a `.nkm`/`.nki` file
/// paired with its companion Yamaha `.STY` rhythm files in the same folder.
pub fn build_kontakt_package(
    nkm_path: &Path,
    explicit_sty_paths: &[PathBuf],
) -> Result<LoadedMusicPackage, String> {
    let multi = parse_kontakt_file(nkm_path)?;
    let base_dir = nkm_path.parent().unwrap_or_else(|| Path::new("."));

    let mut sample_index = HashMap::new();
    index_sample_files_in_dir(base_dir, 3, &mut sample_index);

    let mut pcm_cache: HashMap<PathBuf, Arc<(Vec<f32>, Vec<f32>, u32)>> = HashMap::new();

    // Decode every program in the .nkm / .nki
    let mut decoded_progs: Vec<(KontaktProgramMeta, Vec<SampleZone>, bool)> = Vec::new();
    for prog in &multi.programs {
        // Every program assigned to Yamaha Rhythm 1/2 (MIDI Ch 9 / Ch 10) is a drum/percussion
        // instrument regardless of zone count (e.g. TONS with 80-192 zones or ATAQUE with 40 zones).
        let is_drum = prog.midi_channel == 9
            || prog.midi_channel == 10
            || is_drum_program_name(&prog.name);
        let zones = decode_program_zones(
            prog,
            is_drum,
            &multi.filenames,
            &sample_index,
            &mut pcm_cache,
        );
        if !zones.is_empty() {
            decoded_progs.push((prog.clone(), zones, is_drum));
        }
    }
    if decoded_progs.is_empty() {
        return Err(format!(
            "Nenhum sample (.ncw/.wav) carregável encontrado para '{}'",
            nkm_path.display()
        ));
    }

    // Preserve authored Kontakt program & group relative gains (no artificial boost that
    // could distort quiet percussion/triangle balance).
    let kit_norm = 1.0_f32;

    let mut instruments: Vec<LoadedInstrument> = Vec::new();
    let mut inst_mix_vol: Vec<f32> = Vec::new();
    let mut channel_to_inst: HashMap<u8, usize> = HashMap::new();
    let mut drum_by_channel: HashMap<u8, Vec<(String, Vec<SampleZone>)>> = HashMap::new();
    let mut melodic_progs: Vec<(KontaktProgramMeta, Vec<SampleZone>)> = Vec::new();

    for (prog, zones, is_drum) in decoded_progs {
        if is_drum {
            drum_by_channel
                .entry(prog.midi_channel)
                .or_default()
                .push((prog.name, zones));
        } else {
            melodic_progs.push((prog, zones));
        }
    }

    // Build strictly separated drumkits per MIDI channel (Ch 9 = Rhythm 1, Ch 10 = Rhythm 2).
    // If Ch 10 is missing core kit pieces (kick 36, snare 38/40, hi-hat 42/46) that exist on
    // a standalone drum channel (e.g. Ch 6 BUMBO in WB Forro or Ch 1 CIMBAL in Gusttavo Lima),
    // borrow only those uncovered keys into Ch 10 without disturbing Ch 9/Ch 10 separation.
    let extra_drum_zones: Vec<SampleZone> = drum_by_channel
        .iter()
        .filter(|(&ch, _)| ch != 9 && ch != 10)
        .flat_map(|(_, list)| list.iter())
        .filter(|(name, _)| {
            let up = name.to_uppercase();
            up.contains("BUMBO")
                || up.contains("CHIMBAL")
                || up.contains("CIMBAL")
                || up.contains("CAIXA")
                || up.contains("TON")
        })
        .flat_map(|(_, zs)| zs.iter().cloned())
        .collect();

    let mut drum_channels: Vec<u8> = drum_by_channel.keys().copied().collect();
    drum_channels.sort_unstable();
    let mut drum_ch9: Option<usize> = None;
    let mut drum_ch10: Option<usize> = None;

    for ch in drum_channels {
        let entries = &drum_by_channel[&ch];
        let mut combined_zones = Vec::new();
        for (_, zs) in entries {
            for z in zs {
                let mut zc = z.clone();
                zc.gain = (zc.gain * kit_norm).clamp(0.0, 1.5);
                combined_zones.push(zc);
            }
        }
        if ch == 10 {
            for ez in &extra_drum_zones {
                let already_covered = combined_zones
                    .iter()
                    .any(|z| z.key_low <= ez.root_key && ez.root_key <= z.key_high);
                if !already_covered {
                    let mut zc = ez.clone();
                    zc.gain = (zc.gain * kit_norm).clamp(0.0, 1.5);
                    combined_zones.push(zc);
                }
            }
            let has_k36_now = combined_zones.iter().any(|z| z.key_low <= 36 && 36 <= z.key_high);
            if !has_k36_now {
                if let Some(k35) = combined_zones
                    .iter()
                    .find(|z| z.key_low <= 35 && 35 <= z.key_high)
                    .cloned()
                {
                    let mut k36 = k35;
                    k36.key_low = 36;
                    k36.key_high = 36;
                    k36.root_key = 36;
                    combined_zones.push(k36);
                }
            }
        }
        let combined_name = if entries.len() == 1 {
            entries[0].0.clone()
        } else {
            format!("Kontakt Drums Ch{} ({} peças)", ch, entries.len())
        };
        let idx = instruments.len();
        instruments.push(LoadedInstrument {
            index: idx,
            name: combined_name,
            volume: 1.0,
            pan: 0,
            inst_type: 1,
            is_drumkit: true,
            zones: combined_zones,
        });
        inst_mix_vol.push(1.0);
        if ch == 9 {
            drum_ch9 = Some(idx);
        } else if ch == 10 {
            drum_ch10 = Some(idx);
        }
        channel_to_inst.entry(ch).or_insert(idx);
    }

    // Emit each melodic program cleanly as its own selectable instrument. When multiple melodic
    // programs share a MIDI channel (e.g. muted VIOLAO 02 + active VIOLÃO 01 on Ch 12), choose
    // the active/loudest full-range program as the primary arranger instrument for that channel.
    let mut melodic_ch_to_inst: HashMap<u8, (usize, f32, usize)> = HashMap::new();
    melodic_progs.sort_by_key(|(p, _)| (p.midi_channel, p.slot_index));
    for (prog, zones) in melodic_progs {
        let zcount = zones.len();
        let idx = instruments.len();
        instruments.push(LoadedInstrument {
            index: idx,
            name: prog.name.clone(),
            volume: 1.0,
            pan: 0,
            inst_type: 0,
            is_drumkit: false,
            zones,
        });
        let scaled_vol = if prog.volume < 0.005 {
            0.0
        } else {
            (prog.volume * kit_norm).clamp(0.0, 1.5)
        };
        inst_mix_vol.push(scaled_vol);
        let replace = match melodic_ch_to_inst.get(&prog.midi_channel) {
            None => true,
            Some(&(_, prev_vol, prev_z)) => {
                // Prefer active (>= 0.012) and fuller-range (>= 8 zones) programs over muted/FX slots
                let cur_score = (
                    prog.volume >= 0.012,
                    zcount >= 8,
                    (prog.volume * 1000.0) as i32,
                );
                let prev_score = (prev_vol >= 0.012, prev_z >= 8, (prev_vol * 1000.0) as i32);
                cur_score > prev_score
            }
        };
        if replace {
            melodic_ch_to_inst.insert(prog.midi_channel, (idx, prog.volume, zcount));
        }
        if prog.midi_channel != 9 && prog.midi_channel != 10 {
            channel_to_inst.entry(prog.midi_channel).or_insert(idx);
        }
    }

    let get_melodic_ch = |ch: u8| -> Option<usize> { melodic_ch_to_inst.get(&ch).map(|&(i, _, _)| i) };

    // Detect whether this .nkm uses vArranger channel layout (Bass on ch 5, Chord1 on ch 6/7)
    // vs direct Yamaha .STY channel layout (Bass on ch 11, Chord1 on ch 12).
    let uses_varranger_layout = get_melodic_ch(5).is_some()
        && !get_melodic_ch(11)
            .map(|idx| {
                let up = instruments[idx].name.to_uppercase();
                up.contains("BASS") || up.contains("BAIXO")
            })
            .unwrap_or(false);

    let resolve_sty_channel = |dest_ch: u8, track_slot: usize, is_drum: bool| -> usize {
        if is_drum || track_slot <= 1 {
            // Strict 1-to-1 channel mapping: dest_ch 8 = MIDI Ch 9 (Rhythm 1), dest_ch 9 = MIDI Ch 10 (Rhythm 2)
            if dest_ch == 8 || (dest_ch != 9 && track_slot == 0) {
                return drum_ch9.or(drum_ch10).unwrap_or(0);
            }
            return drum_ch10.or(drum_ch9).unwrap_or(0);
        }
        let direct_ch = dest_ch + 1;
        if uses_varranger_layout {
            let mapped = match dest_ch {
                10 => &[5_u8, 11][..],     // Bass
                11 => &[6_u8, 7, 12][..],  // Chord 1
                12 => &[7_u8, 8, 13][..],  // Chord 2
                13 => &[12_u8, 8, 14][..], // Pad
                14 => &[8_u8, 14, 7][..],  // Phrase 1
                15 => &[15_u8, 7, 6][..],  // Phrase 2
                _ => &[direct_ch][..],
            };
            for &c in mapped {
                if let Some(idx) = get_melodic_ch(c) {
                    return idx;
                }
            }
        }
        if let Some(idx) = get_melodic_ch(direct_ch) {
            return idx;
        }
        // Fallback by slot
        let fallback_chs: &[u8] = match track_slot {
            2 => &[11, 5, 2],
            3 => &[12, 6, 3],
            4 => &[13, 7, 4],
            5 => &[14, 16, 8, 12],
            _ => &[7, 13, 14, 6, 12, 1],
        };
        for &c in fallback_chs {
            if let Some(idx) = get_melodic_ch(c) {
                return idx;
            }
        }
        instruments
            .iter()
            .position(|i| !i.is_drumkit)
            .unwrap_or(0)
    };

    // Parse companion .STY files
    let sty_paths = if !explicit_sty_paths.is_empty() {
        explicit_sty_paths.to_vec()
    } else {
        find_companion_sty_files(base_dir)
    };

    let mut styles = Vec::new();
    let mut source_paths = vec![nkm_path.to_path_buf()];
    let default_upper = instruments
        .iter()
        .position(|i| {
            !i.is_drumkit && {
                let up = i.name.to_uppercase();
                up.contains("ACORD") || up.contains("SANFONA") || up.contains("SCANDALLI") || up.contains("BANDONEON")
            }
        })
        .or_else(|| get_melodic_ch(13))
        .or_else(|| get_melodic_ch(1))
        .or_else(|| instruments.iter().position(|i| !i.is_drumkit))
        .unwrap_or(0);

    for (doc_idx, sty_path) in sty_paths.iter().enumerate() {
        let Ok(doc) = crate::yamaha_sty::parse_sty_file(sty_path) else {
            continue;
        };
        source_paths.push(sty_path.clone());
        let sections: Vec<RhythmSection> = doc
            .sections
            .iter()
            .map(|sec| {
                let tracks: Vec<RhythmTrack> = sec
                    .tracks
                    .iter()
                    .map(|tr| {
                        let is_drum = tr.is_drum || tr.track_slot <= 1;
                        let inst_idx = resolve_sty_channel(tr.dest_ch, tr.track_slot, is_drum);
                        let mix_vol = inst_mix_vol.get(inst_idx).copied().unwrap_or(1.0);
                        let sty_vol = (tr.voice.volume as f32 / 100.0).clamp(0.0, 1.6);
                        let track_vol = if is_drum {
                            (sty_vol * 0.85).clamp(0.0, 1.5)
                        } else if mix_vol < 0.005 {
                            // Keep Kontakt-muted background tracks (e.g. PAD vol=0.0) silent
                            0.0
                        } else {
                            (mix_vol * sty_vol * 0.85).clamp(0.0, 1.5)
                        };
                        let events: Vec<RhythmNoteEvent> = tr
                            .events
                            .iter()
                            .map(|ev| RhythmNoteEvent {
                                midi_note: ev.midi_note,
                                velocity: ev.velocity,
                                start_tick: ev.start_tick,
                                duration_ticks: ev.duration_ticks,
                            })
                            .collect();
                        RhythmTrack {
                            track_type: slot_label(tr.track_slot).to_string(),
                            track_slot: tr.track_slot,
                            instrument_index: inst_idx,
                            volume: track_vol,
                            pan: tr.voice.pan,
                            fine_tune: 0,
                            key_move: 0,
                            octave_shift: 0,
                            is_drum,
                            casm_rule: tr.casm_rule.clone(),
                            events,
                        }
                    })
                    .collect();
                RhythmSection {
                    id: (doc_idx as u32) * 100 + sec.id,
                    name: sec.name.clone(),
                    length_ticks: sec.length_ticks,
                    is_loop: sec.is_loop,
                    jump_to: sec.jump_to,
                    tracks,
                }
            })
            .collect();

        styles.push(RhythmStyle {
            id: (doc_idx + 1) as u32,
            name: doc.title.clone(),
            category: "Kontakt".to_string(),
            bpm: doc.bpm,
            sections,
            ots_presets: vec![OtsPresetSummary {
                index: 0,
                name: format!("Kontakt: {}", instruments[default_upper].name),
                upper1_instrument: default_upper,
                upper2_instrument: None,
                lower_instrument: None,
            }],
            sty_warnings: Vec::new(),
            has_audio_style_chunks: false,
            uses_expansion_kit: false,
            sty_diagnostics_summary: None,
        });
    }

    let file_name = nkm_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Kontakt.nkm")
        .to_string();

    let ots_presets = vec![OtsPresetSummary {
        index: 0,
        name: format!("Kontakt: {}", instruments[default_upper].name),
        upper1_instrument: default_upper,
        upper2_instrument: None,
        lower_instrument: None,
    }];

    Ok(LoadedMusicPackage {
        file_name,
        file_path: nkm_path.to_string_lossy().to_string(),
        version: format!("Kontakt Multi ({} inst)", instruments.len()),
        source_format: "kontakt".to_string(),
        soundfont_name: Some(multi.name),
        instruments,
        styles,
        ots_presets,
        source_paths,
        sty_warnings: Vec::new(),
        has_audio_style_chunks: false,
        uses_expansion_kit: false,
        sty_diagnostics_summary: None,
    })
}

fn scan_dir_for_kontakt(dir: &Path, depth: usize, out: &mut Vec<MusicPackageEntry>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        if entry.file_type().map(|t| t.is_symlink()).unwrap_or(true) {
            continue;
        }
        let p = entry.path();
        if p.is_dir() && depth > 0 {
            scan_dir_for_kontakt(&p, depth - 1, out);
            continue;
        }
        if !p.is_file() {
            continue;
        }
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        // Only list Kontakt Multi (.nkm) packs that have companion .STY rhythm files.
        if ext != "nkm" {
            continue;
        }
        let has_sty = p
            .parent()
            .map(|d| !find_companion_sty_files(d).is_empty())
            .unwrap_or(false);
        if !has_sty {
            continue;
        }
        if let Ok(meta) = std::fs::metadata(&p) {
            let file_name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            out.push(MusicPackageEntry {
                file_name,
                file_path: p.to_string_lossy().to_string(),
                size_mb: (meta.len() as f32) / (1024.0 * 1024.0),
                version: "Kontakt Multi + STY".to_string(),
                format: "kontakt".to_string(),
            });
        }
    }
}

/// Lists discovered Kontakt `.nkm` rhythm packs (and `.nki` instruments) in user directories.
pub fn list_kontakt_packages() -> Vec<MusicPackageEntry> {
    let mut raw = Vec::new();
    scan_dir_for_kontakt(Path::new(USER_DOWNLOADS_KONTAKT_DIR), 4, &mut raw);
    scan_dir_for_kontakt(Path::new(USER_DOCUMENTS_KONTAKT_DIR), 4, &mut raw);

    let mut seen = HashSet::new();
    let mut deduped = Vec::with_capacity(raw.len());
    for entry in raw {
        let canon = std::fs::canonicalize(&entry.file_path)
            .unwrap_or_else(|_| PathBuf::from(&entry.file_path));
        if seen.insert(canon) {
            deduped.push(entry);
        }
    }
    // Sort: .nkm with companion .STY first, then .nkm, then .nki
    deduped.sort_by(|a, b| {
        let rank = |e: &MusicPackageEntry| -> u8 {
            if e.version.contains("+ STY") {
                0
            } else if e.file_name.to_ascii_lowercase().ends_with(".nkm") {
                1
            } else {
                2
            }
        };
        rank(a)
            .cmp(&rank(b))
            .then_with(|| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()))
    });
    deduped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_kontakt_nkm_packs_and_renders_clean_audio() {
        let fb_nkm = Path::new(USER_DOWNLOADS_KONTAKT_DIR)
            .join("FORRO_BOYS_HB_SAMPLER/FORRO BOYS HB SAMPLER/FORRO BOYS.nkm");
        if !fb_nkm.exists() {
            return;
        }
        let pkg = build_kontakt_package(&fb_nkm, &[]).expect("build FORRO BOYS Kontakt package");
        assert_eq!(pkg.source_format, "kontakt");
        assert!(
            pkg.instruments.len() >= 5,
            "expected >= 5 Kontakt instruments, got {}",
            pkg.instruments.len()
        );
        assert!(
            !pkg.styles.is_empty(),
            "expected companion FORROBOYS .S948.STY to be loaded"
        );
        assert!(
            pkg.instruments.iter().any(|i| i.is_drumkit && !i.zones.is_empty()),
            "expected layered Kontakt drumkit"
        );

        let mut engine = crate::music_studio::MusicStudioArrangerEngine::new(48000.0);
        engine.load_package(Arc::new(pkg));
        engine.set_transport(true, None, None);
        let mut l = vec![0.0_f32; 4096];
        let mut r = vec![0.0_f32; 4096];
        let mut peak = 0.0_f32;
        for _ in 0..8 {
            engine.process_block(&mut l, &mut r);
            for (&a, &b) in l.iter().zip(r.iter()) {
                assert!(a.is_finite() && b.is_finite());
                peak = peak.max(a.abs()).max(b.abs());
            }
        }
        assert!(
            peak > 0.01,
            "Kontakt rhythm playback should produce audible non-zero audio, peak={peak}"
        );
    }

    #[test]
    fn parses_sao_joao_varranger_kontakt_pack_with_multiple_styles() {
        let sj_nkm = Path::new(USER_DOWNLOADS_KONTAKT_DIR)
            .join("Kit_S_o_Jo_o_2023/Kit São João 2023/KIT SAO JOAO 2023 VARRANGER_2.nkm");
        if !sj_nkm.exists() {
            return;
        }
        let pkg = build_kontakt_package(&sj_nkm, &[]).expect("build Sao Joao Kontakt package");
        assert_eq!(pkg.styles.len(), 3, "expected Arrasta Pe, Baiao, Xote styles");
        assert!(
            pkg.instruments.len() >= 7,
            "expected >= 7 instruments in Sao Joao pack, got {}",
            pkg.instruments.len()
        );
        // Verify Ch10 kick covers MIDI 36 (cloned from 35) and ACORDEON MASTER E5 octave fix
        let ch10 = pkg
            .instruments
            .iter()
            .find(|i| i.is_drumkit && i.name.contains("Ch10"))
            .expect("Ch10 drumkit");
        assert!(
            ch10.zones.iter().any(|z| z.key_low <= 36 && 36 <= z.key_high),
            "Ch10 drumkit should cover kick MIDI 36"
        );
        let acd = pkg
            .instruments
            .iter()
            .find(|i| i.name.contains("ACORDEON MASTER"))
            .expect("ACORDEON MASTER");
        let e5_zone = acd
            .zones
            .iter()
            .find(|z| z.key_low == 76)
            .expect("E5 zone");
        assert_eq!(e5_zone.root_key, 64, "Acd Master 2 mi at key 76 should have root_key adjusted to 64");
    }

    #[test]
    fn parses_wb_forro_zver_0x99_and_biu_piseiro_and_filters_listing_to_nkm_plus_sty() {
        let wb_nkm = Path::new(USER_DOWNLOADS_KONTAKT_DIR)
            .join("PIZEIRO_WB_/TUTORIAIS TECLADO WB FORRÓ 2019.nkm");
        if wb_nkm.exists() {
            let pkg = build_kontakt_package(&wb_nkm, &[]).expect("build WB Forro zver 0x99 pack");
            assert!(!pkg.styles.is_empty(), "expected PIZEIRO WB.STY");
            assert!(
                pkg.instruments.len() >= 8,
                "expected >= 8 instruments in WB Forro pack, got {}",
                pkg.instruments.len()
            );
            let mut engine = crate::music_studio::MusicStudioArrangerEngine::new(48000.0);
            engine.load_package(Arc::new(pkg));
            engine.set_transport(true, None, None);
            let mut l = vec![0.0_f32; 4096];
            let mut r = vec![0.0_f32; 4096];
            let mut peak = 0.0_f32;
            for _ in 0..8 {
                engine.process_block(&mut l, &mut r);
                for (&a, &b) in l.iter().zip(r.iter()) {
                    assert!(a.is_finite() && b.is_finite());
                    peak = peak.max(a.abs()).max(b.abs());
                }
            }
            assert!(peak > 0.01, "WB Forro should render audible audio, peak={peak}");
        }

        let biu_nkm = Path::new(USER_DOWNLOADS_KONTAKT_DIR)
            .join("KIT_BIU_DO_PISEIRO_2021_RS_SAMPLES/KIT  BIU DO PISEIRO 2021 RS  SAMPLES/KIT BIU DO PISEIRO 2021  RS SAMPLES.nkm");
        if biu_nkm.exists() {
            let pkg = build_kontakt_package(&biu_nkm, &[]).expect("build Biu Piseiro pack");
            assert!(!pkg.styles.is_empty(), "expected BIU RS.STY");
            assert!(pkg.instruments.len() >= 7);
        }

        // Verify list_kontakt_packages ONLY returns .nkm Multi + STY packs (no standalone .nki)
        let listed = list_kontakt_packages();
        for entry in &listed {
            assert!(
                entry.file_name.to_ascii_lowercase().ends_with(".nkm"),
                "unexpected non-nkm entry in Kontakt list: {}",
                entry.file_name
            );
            assert_eq!(entry.version, "Kontakt Multi + STY");
        }
    }

    #[test]
    fn switching_packages_while_playing_auto_continues_and_preserves_chord() {
        let fb_nkm = Path::new(USER_DOWNLOADS_KONTAKT_DIR)
            .join("FORRO_BOYS_HB_SAMPLER/FORRO BOYS HB SAMPLER/FORRO BOYS.nkm");
        let sj_nkm = Path::new(USER_DOWNLOADS_KONTAKT_DIR)
            .join("Kit_S_o_Jo_o_2023/Kit São João 2023/KIT SAO JOAO 2023 VARRANGER_2.nkm");
        if !fb_nkm.exists() || !sj_nkm.exists() {
            return;
        }
        let fb = Arc::new(build_kontakt_package(&fb_nkm, &[]).unwrap());
        let sj = Arc::new(build_kontakt_package(&sj_nkm, &[]).unwrap());
        let mut engine = crate::music_studio::MusicStudioArrangerEngine::new(48000.0);
        engine.load_package(fb);
        assert!(!engine.rhythm_playing, "initial load should not auto-start");
        engine.set_chord(7, "Minor");
        engine.set_transport(true, None, None);
        assert!(engine.rhythm_playing);

        // Switch package while playing -> must keep playing & keep Gm chord
        engine.load_package(sj);
        assert!(engine.rhythm_playing, "switching rhythm pack while playing should auto-play");
        assert_eq!(engine.detected_chord_root, 7);
        assert_eq!(engine.detected_chord_label, "Gm");
    }

    #[test]
    fn decodes_atitude_67_float_ncw_without_hiss_or_static() {
        let dir = Path::new(USER_DOWNLOADS_KONTAKT_DIR)
            .join("SM_ATITUDE_67_CERVEJA_DE_GARRAFA/Atitude 67/Atitude 67 Samples");
        if !dir.exists() {
            return;
        }
        for name in ["1.ncw", "2 CAIXA FORT.ncw", "3 CAIXA FORTe.ncw", "ARO.ncw"] {
            let p = dir.join(name);
            let (l, r, sr) = decode_sample_file(&p)
                .unwrap_or_else(|e| panic!("failed to decode {}: {e}", p.display()));
            assert!(sr >= 44100 && !l.is_empty() && l.len() == r.len());
            assert!(l.iter().all(|v| v.is_finite()) && r.iter().all(|v| v.is_finite()));
            let peak = l.iter().fold(0.0_f32, |m, v| m.max(v.abs()));
            assert!(peak > 0.05 && peak <= 1.2, "{name}: unexpected peak={peak}");
            let tail = &l[l.len().saturating_sub(4096)..];
            let tail_rms = (tail.iter().map(|v| v * v).sum::<f32>() / tail.len() as f32).sqrt();
            assert!(
                tail_rms < 0.01,
                "{name}: tail_rms={tail_rms} indicates float-bit static hiss"
            );
        }
    }

    #[test]
    fn parses_bgroup_volumes_and_seventh_chords_in_kontakt_mode() {
        let forro_nkm = Path::new(USER_DOWNLOADS_KONTAKT_DIR)
            .join("FORRO_BOYS_HB_SAMPLER/FORRO BOYS HB SAMPLER/FORRO BOYS.nkm");
        if forro_nkm.exists() {
            let multi = parse_kontakt_file(&forro_nkm).expect("parse FORRO BOYS");
            assert!(
                multi.programs.iter().any(|p| !p.groups.is_empty()),
                "expected BGroup metadata to be parsed"
            );
            let pkg = Arc::new(build_kontakt_package(&forro_nkm, &[]).expect("build FORRO BOYS"));
            let mut engine = crate::music_studio::MusicStudioArrangerEngine::new(48000.0);
            engine.load_package(pkg);
            // Play 4-key C7 below split_key=54 (C2=36, E2=40, G2=43, Bb2=46)
            for &k in &[36_u8, 40, 43, 46] {
                engine.note_on(k, 100);
            }
            assert_eq!(engine.detected_chord_label, "C7");
        }

        // Verify CMaj7-declared triad-only Kontakt track voices distinct 7th on C7 vs C
        let rule = crate::yamaha_sty::CasmRule {
            src_ch: 11,
            dest_ch: 11,
            name: "Chord1".to_string(),
            editable: 1,
            note_mute: 0x0FFF,
            chord_mute: 0,
            src_root: 0,
            src_chord_type: 2, // CMaj7
            low_split: None,
            high_split: None,
            ranges: [
                crate::yamaha_sty::CasmRangeRule {
                    ntr: crate::yamaha_sty::NtrRule::RootTrans,
                    ntt: crate::yamaha_sty::NttRule::Chord,
                    bass_on: false,
                    high_key: 11,
                    note_low: 0,
                    note_high: 127,
                    rtr: crate::yamaha_sty::RtrRule::Retrigger,
                },
                crate::yamaha_sty::CasmRangeRule {
                    ntr: crate::yamaha_sty::NtrRule::RootTrans,
                    ntt: crate::yamaha_sty::NttRule::Chord,
                    bass_on: false,
                    high_key: 11,
                    note_low: 0,
                    note_high: 127,
                    rtr: crate::yamaha_sty::RtrRule::Retrigger,
                },
                crate::yamaha_sty::CasmRangeRule {
                    ntr: crate::yamaha_sty::NtrRule::RootTrans,
                    ntt: crate::yamaha_sty::NttRule::Chord,
                    bass_on: false,
                    high_key: 11,
                    note_low: 0,
                    note_high: 127,
                    rtr: crate::yamaha_sty::RtrRule::Retrigger,
                },
            ],
        };
        let spec_c = crate::music_studio::chord_quality_to_target_spec(
            0,
            crate::music_studio::ChordQuality::Major,
        );
        let spec_c7 = crate::music_studio::chord_quality_to_target_spec(
            0,
            crate::music_studio::ChordQuality::Dom7,
        );
        let spec_cmaj7 = crate::music_studio::chord_quality_to_target_spec(
            0,
            crate::music_studio::ChordQuality::Maj7,
        );
        // Triad-only CMaj7 track: root C4(60) stays C4(60) on C, shifts to Bb3(58) on C7, B3(59) on Cmaj7
        assert_eq!(rule.transpose_note_kontakt(60, spec_c, false, false, true), Some(60));
        assert_eq!(rule.transpose_note_kontakt(60, spec_c7, false, false, true), Some(58));
        assert_eq!(rule.transpose_note_kontakt(60, spec_cmaj7, false, false, true), Some(59));
        // Real 4-note CMaj7 track: B4(71) folds to C4/C5 (pc=0) on plain C major instead of leaking B
        let n_b_on_c = rule.transpose_note_kontakt(71, spec_c, false, false, false).unwrap();
        assert_eq!(n_b_on_c % 12, 0, "plain C major must not leak B from CMaj7 source");
    }

    #[test]
    fn all_discovered_kontakt_packs_play_continuously_without_stopping_or_silence() {
        let listed = list_kontakt_packages();
        if listed.is_empty() {
            return;
        }
        let mut problems: Vec<String> = Vec::new();
        for entry in &listed {
            let p = Path::new(&entry.file_path);
            let pkg = match build_kontakt_package(p, &[]) {
                Ok(pkg) => pkg,
                Err(e) => {
                    problems.push(format!("LOAD_FAIL {}: {}", entry.file_name, e));
                    continue;
                }
            };
            if pkg.styles.is_empty() {
                problems.push(format!("NO_STYLES {}", entry.file_name));
                continue;
            }
            let mut engine = crate::music_studio::MusicStudioArrangerEngine::new(48000.0);
            engine.load_package(Arc::new(pkg));
            engine.set_chord(0, "Dom7");
            engine.set_transport(true, None, None);
            let mut l = vec![0.0_f32; 4096];
            let mut r = vec![0.0_f32; 4096];
            let mut peak = 0.0_f32;
            // Simulate ~5.1 seconds (60 blocks of 4096 @ 48kHz) — must NEVER stop by itself!
            for _ in 0..60 {
                engine.process_block(&mut l, &mut r);
                for (&a, &b) in l.iter().zip(r.iter()) {
                    if !a.is_finite() || !b.is_finite() {
                        problems.push(format!("NAN {}", entry.file_name));
                        break;
                    }
                    peak = peak.max(a.abs()).max(b.abs());
                }
            }
            if !engine.rhythm_playing {
                problems.push(format!(
                    "STOPPED_ITSELF {} (sec='{}')",
                    entry.file_name, engine.active_section
                ));
            }
            if peak < 0.005 {
                problems.push(format!(
                    "SILENT {} (peak={:.5}, sec='{}')",
                    entry.file_name, peak, engine.active_section
                ));
            }
        }
        assert!(
            problems.is_empty(),
            "Found {} problem(s) across {} Kontakt packs:\n{}",
            problems.len(),
            listed.len(),
            problems.join("\n")
        );
    }
}
