use crate::contracts::{KeyRangeMode, MidiCcMapping, MidiConfig, MidiPortInfo};
use midir::{Ignore, MidiInput, MidiInputConnection};
use std::collections::HashMap;
use std::sync::mpsc::Sender;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiEngineEvent {
    NoteOn { note: u8, velocity: u8, channel: u8 },
    NoteOff { note: u8, channel: u8 },
    ControlChange { cc: u8, value: u8, channel: u8 },
    PitchBend { value_14bit: u16, channel: u8 },
    Aftertouch { pressure: u8, channel: u8 },
    ProgramChange { program: u8, channel: u8 },
}

pub const MAX_VELOCITY_CURVE_GAIN: f32 = 1.30;

/// Evaluates a piecewise-linear MIDI velocity curve, returning `(midi_vel [1..127], gain [1.0..1.30])`.
/// NoteOff (`raw_vel == 0`) always returns `(0, 1.0)`. Values above 100% cap MIDI velocity at 127
/// and scale linear volume gain up to `MAX_VELOCITY_CURVE_GAIN`.
pub fn apply_velocity_curve_with_gain(raw_vel: u8, curve: &[[f32; 2]]) -> (u8, f32) {
    if raw_vel == 0 {
        return (0, 1.0);
    }
    let clamped_vel = raw_vel.min(127);
    if curve.is_empty() {
        return (clamped_vel, 1.0);
    }
    // Select the bracketing points in one pass, without allocating/sorting
    // on each note. Equal-x ties match the previous stable sorted evaluation.
    let x = clamped_vel as f32 / 127.0;
    let mut first: Option<[f32; 2]> = None;
    let mut last: Option<[f32; 2]> = None;
    let mut below: Option<[f32; 2]> = None;
    let mut above: Option<[f32; 2]> = None;
    for &point in curve {
        if !point[0].is_finite() || !point[1].is_finite() { continue; }
        if first.map(|p| point[0] < p[0]).unwrap_or(true) { first = Some(point); }
        if last.map(|p| point[0] >= p[0]).unwrap_or(true) { last = Some(point); }
        if point[0] < x {
            if below.map(|p| point[0] >= p[0]).unwrap_or(true) { below = Some(point); }
        } else if above.map(|p| point[0] < p[0]).unwrap_or(true) { above = Some(point); }
    }
    let Some(first) = first else { return (clamped_vel, 1.0); };
    let y = if x <= first[0] { first[1] }
        else if let (Some(lower), Some(upper)) = (below, above) {
            let t = (x - lower[0]) / (upper[0] - lower[0]).max(1e-5);
            lower[1] + (upper[1] - lower[1]) * t
        } else { last.unwrap_or(first)[1] };
    let y_eff = y.clamp(0.01, MAX_VELOCITY_CURVE_GAIN);
    let mapped_vel = (y_eff.min(1.0) * 127.0).round() as u8;
    let gain = y_eff.max(1.0);
    (mapped_vel, gain)
}

/// Evaluates a piecewise-linear MIDI velocity curve mapping raw velocity [1..127] -> [1..127].
pub fn apply_velocity_curve(raw_vel: u8, curve: &[[f32; 2]]) -> u8 {
    apply_velocity_curve_with_gain(raw_vel, curve).0
}

/// Applies semitone transposition (-36..=+36) and keyboard range clamping/filtering.
pub fn apply_transpose_and_range(note: u8, config: &MidiConfig) -> Option<u8> {
    let transpose = (config.transpose as i16).clamp(-36, 36);
    let mut shifted = note as i16 + transpose;
    if config.key_range_mode == KeyRangeMode::Compact61 {
        shifted = shifted.clamp(36, 96);
    }
    if (0..=127).contains(&shifted) {
        Some(shifted as u8)
    } else {
        None
    }
}

/// Computes the extended 16-channel microtonal note index per Pianoteq manual Sec. 6.3
/// (up to 16 * 128 = 2048 unique note slots centered around `main_channel`, with circular
/// wraparound so channel 16 precedes channel 1).
pub fn compute_microtonal_channel_offset(note: u8, channel: u8, main_channel: u8) -> i32 {
    let main = if main_channel == 0 {
        1
    } else {
        main_channel.clamp(1, 16)
    };
    let ch = if channel == 0 {
        main
    } else {
        channel.clamp(1, 16)
    };

    let rel = (ch as i32 - main as i32).rem_euclid(16);
    let channel_delta = if rel >= 8 { rel - 16 } else { rel };
    (note & 0x7F) as i32 + channel_delta * 128
}

/// Evaluates MIDI CC Learn parameter automation mappings for an incoming CC event.
pub fn evaluate_cc_mapping(cc: u8, value: u8, mappings: &[MidiCcMapping]) -> Vec<(String, f32)> {
    let norm = value.min(127) as f32 / 127.0;
    mappings
        .iter()
        .filter(|m| m.cc == cc && m.min.is_finite() && m.max.is_finite())
        .map(|m| {
            let scaled = m.min + (m.max - m.min) * norm;
            (m.param_path.clone(), scaled)
        })
        .collect()
}

/// Generates a deterministic built-in MIDI demo performance sequence `(timestamp_ms, event)`
/// for standalone auditioning and verification.
pub fn generate_demo_midi_sequence() -> Vec<(u64, MidiEngineEvent)> {
    let mut events = Vec::new();
    let ch = 1_u8;

    events.push((
        0,
        MidiEngineEvent::ProgramChange {
            program: 0,
            channel: ch,
        },
    ));
    events.push((
        0,
        MidiEngineEvent::ControlChange {
            cc: 64,
            value: 112,
            channel: ch,
        },
    ));

    // Expressive Cmaj9 -> Fmaj7 -> G13 -> Cmaj9 cadence across bass, tenor, and treble registers
    let steps: &[(u64, u8, u8, u64)] = &[
        (20, 36, 78, 920),
        (140, 48, 72, 940),
        (260, 60, 76, 960),
        (380, 64, 82, 980),
        (500, 67, 84, 1020),
        (620, 74, 88, 1060),
        (1100, 41, 80, 1980),
        (1220, 53, 74, 2000),
        (1340, 64, 80, 2020),
        (1460, 69, 86, 2040),
        (1580, 72, 90, 2060),
        (2100, 43, 84, 2860),
        (2220, 59, 78, 2880),
        (2340, 65, 84, 2900),
        (2460, 69, 88, 2920),
        (2580, 76, 94, 2940),
        (3000, 36, 92, 3900),
        (3000, 48, 86, 3900),
        (3000, 64, 90, 3900),
        (3000, 67, 92, 3900),
        (3000, 74, 96, 3900),
    ];

    let mut scheduled: Vec<(u64, u8, MidiEngineEvent)> = Vec::with_capacity(steps.len() * 2 + 2);
    for &(on_ms, note, velocity, off_ms) in steps {
        scheduled.push((
            on_ms,
            1,
            MidiEngineEvent::NoteOn {
                note,
                velocity,
                channel: ch,
            },
        ));
        scheduled.push((off_ms, 0, MidiEngineEvent::NoteOff { note, channel: ch }));
    }

    // Release sustain pedal cleanly before final note-off tail completes
    scheduled.push((
        3850,
        2,
        MidiEngineEvent::ControlChange {
            cc: 64,
            value: 0,
            channel: ch,
        },
    ));

    scheduled.sort_by_key(|(ts, order, _)| (*ts, *order));
    events.extend(scheduled.into_iter().map(|(ts, _, ev)| (ts, ev)));
    events
}

/// Detects MIDI input port hot-plug changes between `prev` and `current` port snapshots.
/// Returns `(added_ports, removed_ports)` matched by port name (preserving occurrence counts
/// so OS index renumbering on device unplug does not produce false additions/removals).
pub fn detect_midi_port_changes(
    prev: &[MidiPortInfo],
    current: &[MidiPortInfo],
) -> (Vec<MidiPortInfo>, Vec<MidiPortInfo>) {
    let mut prev_counts: HashMap<&str, usize> = HashMap::new();
    for p in prev {
        *prev_counts.entry(p.name.as_str()).or_insert(0) += 1;
    }

    let mut curr_counts: HashMap<&str, usize> = HashMap::new();
    for c in current {
        *curr_counts.entry(c.name.as_str()).or_insert(0) += 1;
    }

    let mut added = Vec::new();
    for c in current {
        if let Some(count) = prev_counts.get_mut(c.name.as_str()) {
            if *count > 0 {
                *count -= 1;
                continue;
            }
        }
        added.push(c.clone());
    }

    let mut removed = Vec::new();
    for p in prev {
        if let Some(count) = curr_counts.get_mut(p.name.as_str()) {
            if *count > 0 {
                *count -= 1;
                continue;
            }
        }
        removed.push(p.clone());
    }

    (added, removed)
}

pub fn parse_midi_message(bytes: &[u8]) -> Option<MidiEngineEvent> {
    if bytes.is_empty() {
        return None;
    }
    let status = bytes[0];
    let msg_type = status & 0xF0;
    let channel = (status & 0x0F) + 1;

    match msg_type {
        0x90 if bytes.len() >= 3 => {
            let note = bytes[1] & 0x7F;
            let vel = bytes[2] & 0x7F;
            if vel == 0 {
                Some(MidiEngineEvent::NoteOff { note, channel })
            } else {
                Some(MidiEngineEvent::NoteOn {
                    note,
                    velocity: vel,
                    channel,
                })
            }
        }
        0x80 if bytes.len() >= 3 => Some(MidiEngineEvent::NoteOff {
            note: bytes[1] & 0x7F,
            channel,
        }),
        0xB0 if bytes.len() >= 3 => Some(MidiEngineEvent::ControlChange {
            cc: bytes[1] & 0x7F,
            value: bytes[2] & 0x7F,
            channel,
        }),
        0xE0 if bytes.len() >= 3 => {
            let lsb = (bytes[1] & 0x7F) as u16;
            let msb = (bytes[2] & 0x7F) as u16;
            Some(MidiEngineEvent::PitchBend {
                value_14bit: (msb << 7) | lsb,
                channel,
            })
        }
        0xD0 if bytes.len() >= 2 => Some(MidiEngineEvent::Aftertouch {
            pressure: bytes[1] & 0x7F,
            channel,
        }),
        0xC0 if bytes.len() >= 2 => Some(MidiEngineEvent::ProgramChange {
            program: bytes[1] & 0x7F,
            channel,
        }),
        _ => None,
    }
}

pub fn list_midi_input_ports() -> Vec<MidiPortInfo> {
    let mut midi_in = match MidiInput::new("piano-tect-scanner") {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };
    midi_in.ignore(Ignore::None);
    midi_in
        .ports()
        .iter()
        .enumerate()
        .filter_map(|(index, port)| {
            midi_in
                .port_name(port)
                .ok()
                .map(|name| MidiPortInfo { index, name })
        })
        .collect()
}

pub struct MidiConnectionHandle {
    _conn: MidiInputConnection<()>,
    pub port_name: String,
}

pub fn connect_midi_port(
    port_name_or_query: &str,
    tx: Sender<MidiEngineEvent>,
) -> Result<MidiConnectionHandle, String> {
    let mut midi_in = MidiInput::new("piano-tect-input").map_err(|e| e.to_string())?;
    midi_in.ignore(Ignore::None);
    let ports = midi_in.ports();
    let mut exact = None;
    let mut partial = None;

    for p in &ports {
        if let Ok(name) = midi_in.port_name(p) {
            if name == port_name_or_query {
                exact = Some((p.clone(), name));
                break;
            } else if partial.is_none() && name.contains(port_name_or_query) {
                partial = Some((p.clone(), name));
            }
        }
    }

    let (port, matched_name) = exact
        .or(partial)
        .ok_or_else(|| format!("MIDI input port '{}' not found", port_name_or_query))?;

    let conn = midi_in
        .connect(
            &port,
            "piano-tect-live",
            move |_stamp, message, _| {
                if let Some(ev) = parse_midi_message(message) {
                    let _ = tx.send(ev);
                }
            },
            (),
        )
        .map_err(|e| format!("Failed to connect MIDI port: {:?}", e))?;

    Ok(MidiConnectionHandle {
        _conn: conn,
        port_name: matched_name,
    })
}

const DEFAULT_TEMPO_US_PER_QUARTER: u32 = 500_000;

fn encode_vlq(value: u32, out: &mut Vec<u8>) {
    let mut v = value.min(0x0FFF_FFFF);
    let mut buf = [0_u8; 4];
    let mut len = 0;
    buf[len] = (v & 0x7F) as u8;
    len += 1;
    v >>= 7;
    while v > 0 {
        buf[len] = ((v & 0x7F) as u8) | 0x80;
        len += 1;
        v >>= 7;
    }
    for i in (0..len).rev() {
        out.push(buf[i]);
    }
}

fn read_vlq(data: &[u8], cursor: &mut usize) -> Result<u32, String> {
    let mut value: u32 = 0;
    for i in 0..4 {
        if *cursor >= data.len() {
            return Err("Unexpected EOF while reading MIDI VLQ".to_string());
        }
        let b = data[*cursor];
        *cursor += 1;
        value = (value << 7) | (b & 0x7F) as u32;
        if (b & 0x80) == 0 {
            return Ok(value);
        }
        if i == 3 {
            return Err("MIDI VLQ exceeded 4 bytes".to_string());
        }
    }
    unreachable!()
}

/// Encodes `(timestamp_ms, MidiEngineEvent)` pairs into a Standard MIDI File (`.mid` Format 0)
/// assuming 120 BPM (500,000 us/quarter) when converting millisecond timestamps to ticks.
pub fn write_smf_bytes(events: &[(u64, MidiEngineEvent)], ppqn: u16) -> Vec<u8> {
    let ppqn = ppqn.clamp(1, 32_767);
    let mut indexed: Vec<(u64, usize, &MidiEngineEvent)> = events
        .iter()
        .enumerate()
        .map(|(idx, (ts_ms, ev))| (*ts_ms, idx, ev))
        .collect();
    indexed.sort_by_key(|&(ts_ms, idx, _)| (ts_ms, idx));

    let mut track = Vec::with_capacity(indexed.len() * 6 + 16);
    // Explicit 120 BPM Set Tempo meta event (500,000 us = 0x07A120) at tick 0
    track.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]);

    let mut prev_tick: u64 = 0;
    for (ts_ms, _, ev) in indexed {
        let tick = ((ts_ms as u128 * ppqn as u128 + 250) / 500) as u64;
        let delta_tick = tick.saturating_sub(prev_tick).min(0x0FFF_FFFF) as u32;
        prev_tick = tick;

        encode_vlq(delta_tick, &mut track);
        match *ev {
            MidiEngineEvent::NoteOn {
                note,
                velocity,
                channel,
            } => {
                let ch = channel.saturating_sub(1).min(15);
                track.extend_from_slice(&[0x90 | ch, note & 0x7F, velocity.min(127)]);
            }
            MidiEngineEvent::NoteOff { note, channel } => {
                let ch = channel.saturating_sub(1).min(15);
                track.extend_from_slice(&[0x80 | ch, note & 0x7F, 0x00]);
            }
            MidiEngineEvent::ControlChange { cc, value, channel } => {
                let ch = channel.saturating_sub(1).min(15);
                track.extend_from_slice(&[0xB0 | ch, cc & 0x7F, value & 0x7F]);
            }
            MidiEngineEvent::PitchBend {
                value_14bit,
                channel,
            } => {
                let ch = channel.saturating_sub(1).min(15);
                let clamped = value_14bit.min(16_383);
                let lsb = (clamped & 0x7F) as u8;
                let msb = ((clamped >> 7) & 0x7F) as u8;
                track.extend_from_slice(&[0xE0 | ch, lsb, msb]);
            }
            MidiEngineEvent::Aftertouch { pressure, channel } => {
                let ch = channel.saturating_sub(1).min(15);
                track.extend_from_slice(&[0xD0 | ch, pressure & 0x7F]);
            }
            MidiEngineEvent::ProgramChange { program, channel } => {
                let ch = channel.saturating_sub(1).min(15);
                track.extend_from_slice(&[0xC0 | ch, program & 0x7F]);
            }
        }
    }

    // End of Track meta event
    track.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    let mut out = Vec::with_capacity(14 + 8 + track.len());
    out.extend_from_slice(b"MThd");
    out.extend_from_slice(&6_u32.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes()); // Format 0
    out.extend_from_slice(&1_u16.to_be_bytes()); // 1 track
    out.extend_from_slice(&ppqn.to_be_bytes());

    out.extend_from_slice(b"MTrk");
    out.extend_from_slice(&(track.len() as u32).to_be_bytes());
    out.extend_from_slice(&track);
    out
}

fn parse_mtrk_chunk(
    track: &[u8],
    seq_counter: &mut usize,
    tempo_changes: &mut Vec<(u64, u32)>,
    raw_events: &mut Vec<(u64, usize, MidiEngineEvent)>,
) -> Result<(), String> {
    let mut cursor = 0_usize;
    let mut abs_tick = 0_u64;
    let mut running_status: Option<u8> = None;

    while cursor < track.len() {
        let delta = read_vlq(track, &mut cursor)?;
        abs_tick = abs_tick.saturating_add(delta as u64);

        if cursor >= track.len() {
            return Err("Unexpected end of MTrk chunk after delta-time".to_string());
        }

        let byte = track[cursor];
        if byte == 0xFF {
            cursor += 1;
            running_status = None;
            if cursor >= track.len() {
                return Err("Truncated meta event in MTrk".to_string());
            }
            let meta_type = track[cursor];
            cursor += 1;
            let meta_len = read_vlq(track, &mut cursor)? as usize;
            let end = cursor
                .checked_add(meta_len)
                .ok_or_else(|| "Meta event length overflow".to_string())?;
            if end > track.len() {
                return Err("Truncated meta event payload in MTrk".to_string());
            }
            let payload = &track[cursor..end];
            cursor = end;

            if meta_type == 0x2F {
                break;
            }
            if meta_type == 0x51 && meta_len == 3 {
                let us_per_q = ((payload[0] as u32) << 16)
                    | ((payload[1] as u32) << 8)
                    | (payload[2] as u32);
                if us_per_q > 0 {
                    tempo_changes.push((abs_tick, us_per_q));
                }
            }
            continue;
        }

        if byte == 0xF0 || byte == 0xF7 {
            cursor += 1;
            running_status = None;
            let sysex_len = read_vlq(track, &mut cursor)? as usize;
            let end = cursor
                .checked_add(sysex_len)
                .ok_or_else(|| "SysEx event length overflow".to_string())?;
            if end > track.len() {
                return Err("Truncated SysEx event payload in MTrk".to_string());
            }
            cursor = end;
            continue;
        }

        let status = if byte >= 0x80 {
            if byte >= 0xF0 {
                return Err(format!("Unsupported MIDI system status byte: 0x{byte:02X}"));
            }
            cursor += 1;
            running_status = Some(byte);
            byte
        } else {
            running_status.ok_or_else(|| {
                "MIDI data byte encountered without active running status".to_string()
            })?
        };

        let msg_type = status & 0xF0;
        let parsed = match msg_type {
            0xC0 | 0xD0 => {
                if cursor >= track.len() {
                    return Err("Truncated 2-byte MIDI channel message".to_string());
                }
                let d1 = track[cursor];
                cursor += 1;
                parse_midi_message(&[status, d1])
            }
            0x80 | 0x90 | 0xA0 | 0xB0 | 0xE0 => {
                if cursor + 1 >= track.len() {
                    return Err("Truncated 3-byte MIDI channel message".to_string());
                }
                let d1 = track[cursor];
                let d2 = track[cursor + 1];
                cursor += 2;
                parse_midi_message(&[status, d1, d2])
            }
            _ => None,
        };

        if let Some(ev) = parsed {
            let order = *seq_counter;
            *seq_counter += 1;
            raw_events.push((abs_tick, order, ev));
        }
    }

    Ok(())
}

/// Parses a Standard MIDI File (`.mid` Format 0 or Format 1) into timestamp-sorted
/// `(timestamp_ms, MidiEngineEvent)` pairs, supporting running status, `FF 51 03` tempo meta
/// events, and skipping meta/SysEx chunks.
pub fn parse_smf_bytes(data: &[u8]) -> Result<Vec<(u64, MidiEngineEvent)>, String> {
    if data.len() < 14 || &data[0..4] != b"MThd" {
        return Err("Invalid SMF header: missing MThd".to_string());
    }
    let header_len = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let header_end = 8_usize
        .checked_add(header_len)
        .ok_or_else(|| "SMF header length overflow".to_string())?;
    if header_len < 6 || header_end > data.len() {
        return Err("Truncated or invalid MThd header length".to_string());
    }

    let format = u16::from_be_bytes([data[8], data[9]]);
    if format > 1 {
        return Err(format!("Unsupported SMF format: {format}"));
    }
    let num_tracks = u16::from_be_bytes([data[10], data[11]]) as usize;
    let division = u16::from_be_bytes([data[12], data[13]]);
    if division == 0 {
        return Err("Invalid SMF division: 0".to_string());
    }
    if (division & 0x8000) != 0 {
        return Err("SMPTE timecode SMF division is not supported".to_string());
    }
    let ppqn = division as u128;

    let mut pos = header_end;
    let mut tracks_parsed = 0_usize;
    let mut seq_counter = 0_usize;
    let mut tempo_changes: Vec<(u64, u32)> = Vec::new();
    let mut raw_events: Vec<(u64, usize, MidiEngineEvent)> = Vec::new();

    while pos + 8 <= data.len() && tracks_parsed < num_tracks {
        let chunk_id = &data[pos..pos + 4];
        let chunk_len =
            u32::from_be_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
                as usize;
        let chunk_start = pos + 8;
        let chunk_end = chunk_start
            .checked_add(chunk_len)
            .ok_or_else(|| "SMF chunk length overflow".to_string())?;
        if chunk_end > data.len() {
            return Err("Truncated SMF chunk".to_string());
        }

        if chunk_id == b"MTrk" {
            parse_mtrk_chunk(
                &data[chunk_start..chunk_end],
                &mut seq_counter,
                &mut tempo_changes,
                &mut raw_events,
            )?;
            tracks_parsed += 1;
        }
        pos = chunk_end;
    }

    if tracks_parsed == 0 {
        return Err("SMF contains no MTrk chunks".to_string());
    }

    tempo_changes.sort_by_key(|&(tick, _)| tick);
    raw_events.sort_by_key(|&(tick, order, _)| (tick, order));

    let mut tempo_idx = 0_usize;
    let mut current_tick = 0_u64;
    let mut current_tempo_us = DEFAULT_TEMPO_US_PER_QUARTER as u128;
    let mut accum_tick_us = 0_u128;
    let denom = ppqn * 1000;

    let mut result = Vec::with_capacity(raw_events.len());
    for (abs_tick, _, ev) in raw_events {
        while tempo_idx < tempo_changes.len() && tempo_changes[tempo_idx].0 <= abs_tick {
            let (t_tick, new_tempo) = tempo_changes[tempo_idx];
            if t_tick > current_tick {
                accum_tick_us += (t_tick - current_tick) as u128 * current_tempo_us;
                current_tick = t_tick;
            }
            current_tempo_us = new_tempo as u128;
            tempo_idx += 1;
        }
        let total_tick_us =
            accum_tick_us + (abs_tick - current_tick) as u128 * current_tempo_us;
        let ms = ((total_tick_us + denom / 2) / denom) as u64;
        result.push((ms, ev));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap as StdHashMap, HashSet};

    #[test]
    fn parse_note_on_and_vel_zero_note_off() {
        assert_eq!(
            parse_midi_message(&[0x90, 60, 100]),
            Some(MidiEngineEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 1,
            })
        );
        assert_eq!(
            parse_midi_message(&[0x99, 48, 84]),
            Some(MidiEngineEvent::NoteOn {
                note: 48,
                velocity: 84,
                channel: 10,
            })
        );
        assert_eq!(
            parse_midi_message(&[0x92, 64, 0]),
            Some(MidiEngineEvent::NoteOff {
                note: 64,
                channel: 3,
            })
        );
        assert_eq!(
            parse_midi_message(&[0x8F, 108, 40]),
            Some(MidiEngineEvent::NoteOff {
                note: 108,
                channel: 16,
            })
        );
    }

    #[test]
    fn parse_control_change_pedals_and_channel_messages() {
        // CC64 Sustain, CC66 Sostenuto, CC67 Soft, CC69 Harmonic
        for (cc, val, ch_idx) in [(64, 127, 0), (66, 64, 1), (67, 90, 2), (69, 0, 15)] {
            assert_eq!(
                parse_midi_message(&[0xB0 | ch_idx, cc, val]),
                Some(MidiEngineEvent::ControlChange {
                    cc,
                    value: val,
                    channel: ch_idx + 1,
                })
            );
        }

        // PitchBend 14-bit center / min / max / arbitrary
        assert_eq!(
            parse_midi_message(&[0xE0, 0x00, 0x40]),
            Some(MidiEngineEvent::PitchBend {
                value_14bit: 8192,
                channel: 1,
            })
        );
        assert_eq!(
            parse_midi_message(&[0xE0, 0x00, 0x00]),
            Some(MidiEngineEvent::PitchBend {
                value_14bit: 0,
                channel: 1,
            })
        );
        assert_eq!(
            parse_midi_message(&[0xE0, 0x7F, 0x7F]),
            Some(MidiEngineEvent::PitchBend {
                value_14bit: 16383,
                channel: 1,
            })
        );
        assert_eq!(
            parse_midi_message(&[0xE5, 0x2A, 0x55]),
            Some(MidiEngineEvent::PitchBend {
                value_14bit: (0x55 << 7) | 0x2A,
                channel: 6,
            })
        );

        // Aftertouch & ProgramChange
        assert_eq!(
            parse_midi_message(&[0xD3, 95]),
            Some(MidiEngineEvent::Aftertouch {
                pressure: 95,
                channel: 4,
            })
        );
        assert_eq!(
            parse_midi_message(&[0xCF, 42]),
            Some(MidiEngineEvent::ProgramChange {
                program: 42,
                channel: 16,
            })
        );
    }

    #[test]
    fn parse_invalid_or_truncated_midi_returns_none() {
        assert_eq!(parse_midi_message(&[]), None);
        assert_eq!(parse_midi_message(&[0x40, 60, 100]), None);
        assert_eq!(parse_midi_message(&[0x90, 60]), None);
        assert_eq!(parse_midi_message(&[0x80, 60]), None);
        assert_eq!(parse_midi_message(&[0xB0, 64]), None);
        assert_eq!(parse_midi_message(&[0xE0, 0x00]), None);
        assert_eq!(parse_midi_message(&[0xD0]), None);
        assert_eq!(parse_midi_message(&[0xC0]), None);
        assert_eq!(parse_midi_message(&[0xF8]), None);
    }

    #[test]
    fn velocity_curve_interpolation_and_monotonicity() {
        let default_curve = MidiConfig::default().velocity_curve;
        assert_eq!(apply_velocity_curve(0, &default_curve), 0);
        assert_eq!(apply_velocity_curve(64, &[]), 64);

        // Unsorted multi-point curve still interpolates cleanly
        let custom = vec![[1.0, 1.0], [0.0, 0.0], [0.5, 0.25]];
        assert_eq!(apply_velocity_curve(127, &custom), 127);
        let mid = apply_velocity_curve(64, &custom);
        assert!((31..=33).contains(&mid), "unexpected midpoint velocity: {mid}");

        // Monotonicity across 1..=127 for default and expressive S/convex curves
        let soft_curve = vec![[0.0, 0.05], [0.4, 0.55], [0.8, 0.88], [1.0, 1.0]];
        let hard_curve = vec![[0.0, 0.01], [0.5, 0.2], [0.85, 0.65], [1.0, 1.0]];
        for curve in [&default_curve, &soft_curve, &hard_curve] {
            let mut prev = 1_u8;
            for v in 1..=127 {
                let mapped = apply_velocity_curve(v, curve);
                assert!(
                    (1..=127).contains(&mapped),
                    "velocity {v} mapped out of range: {mapped}"
                );
                assert!(
                    mapped >= prev,
                    "monotonicity violated at v={v}: {mapped} < {prev}"
                );
                prev = mapped;
            }
            assert_eq!(apply_velocity_curve(127, curve), 127);
        }

        // Resilient to non-finite points
        let bad_curve = vec![[f32::NAN, 0.5], [0.0, 0.0], [1.0, 1.0]];
        assert_eq!(apply_velocity_curve(127, &bad_curve), 127);
        assert!(apply_velocity_curve(1, &[[f32::NAN, f32::NAN]]) >= 1);
    }

    #[test]
    fn sensitive_default_boosts_midi_but_preserves_dynamics_and_noteoff() {
        let curve = MidiConfig::default().velocity_curve;
        assert_eq!(apply_velocity_curve(0, &curve), 0);
        assert!((95..=97).contains(&apply_velocity_curve(64, &curve)));
        assert!(apply_velocity_curve(32, &curve) > 50);
        assert!(apply_velocity_curve(32, &curve) < apply_velocity_curve(64, &curve));
        assert!(apply_velocity_curve(64, &curve) < apply_velocity_curve(96, &curve));
        assert_eq!(apply_velocity_curve(127, &curve), 127);
    }

    #[test]
    fn velocity_curve_boost_above_100_caps_midi_and_scales_gain() {
        assert_eq!(apply_velocity_curve_with_gain(0, &[[0.0, 1.3], [1.0, 1.3]]), (0, 1.0));
        let curve = [[0.0, 0.5], [0.5, 1.0], [1.0, 1.3]];
        let (v1, g1) = apply_velocity_curve_with_gain(1, &curve);
        assert_eq!(v1, 65, "v1={v1}");
        assert!((g1 - 1.0).abs() < 1e-5, "g1={g1}");
        let (v64, g64) = apply_velocity_curve_with_gain(64, &curve);
        assert_eq!(v64, 127);
        assert!(g64 <= 1.01, "g64={g64}");
        let (v96, g96) = apply_velocity_curve_with_gain(96, &curve);
        assert_eq!(v96, 127);
        assert!((1.14..=1.16).contains(&g96), "g96={g96}");
        let (v127, g127) = apply_velocity_curve_with_gain(127, &curve);
        assert_eq!(v127, 127);
        assert!((g127 - 1.30).abs() < 1e-5, "g127={g127}");
        let (vcapped, gcapped) = apply_velocity_curve_with_gain(64, &[[0.0, 2.0], [1.0, 2.0]]);
        assert_eq!(vcapped, 127);
        assert!((gcapped - 1.30).abs() < 1e-5);
        assert_eq!(apply_velocity_curve_with_gain(64, &[[f32::NAN, f32::NAN]]), (64, 1.0));
    }

    #[test]
    fn flat_velocity_and_three_node_shaping_preserve_note_off() {
        for percent in [0.0_f32, 0.25, 0.5, 0.713, 1.0] {
            let curve = [[0.0, percent], [0.5, percent], [1.0, percent]];
            assert_eq!(apply_velocity_curve(0, &curve), 0);
            let expected = (percent.clamp(0.01, 1.0) * 127.0).round() as u8;
            for input in 1..=127 { assert_eq!(apply_velocity_curve(input, &curve), expected); }
        }
        assert_eq!(apply_velocity_curve(1, &[[0.0, 0.8], [0.5, 0.5], [1.0, 0.2]]), 101);
        assert_eq!(apply_velocity_curve(127, &[[0.0, 0.8], [0.5, 0.5], [1.0, 0.2]]), 25);
    }

    #[test]
    fn allocation_free_velocity_matches_legacy_mapping() {
        let curves = [MidiConfig::default().velocity_curve,
            vec![[1.0,0.7],[0.0,0.2],[0.5,0.3],[0.5,0.8]],
            vec![[0.5,0.8],[f32::NAN,0.2],[1.0,0.1]],
            vec![[0.2,0.8],[0.2,0.4]], vec![]];
        for curve in curves {
            let mut pts:Vec<_> = curve.iter().copied().filter(|p|p[0].is_finite() && p[1].is_finite()).collect();
            pts.sort_by(|a,b|a[0].total_cmp(&b[0]));
            for v in 1..=127_u8 {
                let x=v as f32/127.0;
                let expected = if pts.is_empty() {v} else {
                    let mut y=pts.last().unwrap()[1];
                    if x<=pts[0][0] {y=pts[0][1];} else {
                        for w in pts.windows(2) {
                            if x<=w[1][0] {y=w[0][1]+(w[1][1]-w[0][1])*((x-w[0][0])/(w[1][0]-w[0][0]).max(1e-5));break;}
                        }
                    }
                    (y.clamp(0.01,1.0)*127.0).round() as u8
                };
                assert_eq!(apply_velocity_curve(v,&curve),expected,"v={v} curve={curve:?}");
            }
        }
    }

    #[test]
    fn transpose_and_key_range_clamping() {
        let mut cfg = MidiConfig::default();
        assert_eq!(cfg.key_range_mode, KeyRangeMode::Full88);

        // Full 88-key range preserves A0 (21) .. C8 (108)
        assert_eq!(apply_transpose_and_range(21, &cfg), Some(21));
        assert_eq!(apply_transpose_and_range(60, &cfg), Some(60));
        assert_eq!(apply_transpose_and_range(108, &cfg), Some(108));

        // Transpose -36..+36
        cfg.transpose = -36;
        assert_eq!(apply_transpose_and_range(60, &cfg), Some(24));
        assert_eq!(apply_transpose_and_range(21, &cfg), None);

        cfg.transpose = 36;
        assert_eq!(apply_transpose_and_range(60, &cfg), Some(96));
        assert_eq!(apply_transpose_and_range(108, &cfg), None);

        // Values beyond -36..+36 clamp safely to +-36
        cfg.transpose = -50;
        assert_eq!(apply_transpose_and_range(60, &cfg), Some(24));
        cfg.transpose = 50;
        assert_eq!(apply_transpose_and_range(60, &cfg), Some(96));

        // 61-key range clamps into [36..=96] (C2..C7) while 88-key does not
        cfg.transpose = 0;
        cfg.key_range_mode = KeyRangeMode::Compact61;
        assert_eq!(apply_transpose_and_range(21, &cfg), Some(36));
        assert_eq!(apply_transpose_and_range(60, &cfg), Some(60));
        assert_eq!(apply_transpose_and_range(108, &cfg), Some(96));

        cfg.transpose = -36;
        assert_eq!(apply_transpose_and_range(60, &cfg), Some(36));
        cfg.transpose = 36;
        assert_eq!(apply_transpose_and_range(70, &cfg), Some(96));

        // Extended105 mode preserves extended range [12..=116]
        cfg.transpose = 0;
        cfg.key_range_mode = KeyRangeMode::Extended105;
        assert_eq!(apply_transpose_and_range(12, &cfg), Some(12));
        assert_eq!(apply_transpose_and_range(116, &cfg), Some(116));
    }

    #[test]
    fn microtonal_16_channel_offset_and_bijection() {
        // Main channel plays notes at base offset (0..=127)
        assert_eq!(compute_microtonal_channel_offset(60, 1, 1), 60);
        // Omni main_channel=0 defaults to channel 1
        assert_eq!(compute_microtonal_channel_offset(60, 1, 0), 60);

        // Consecutive higher channels shift upward by +128 per channel
        assert_eq!(compute_microtonal_channel_offset(60, 2, 1), 188);
        assert_eq!(compute_microtonal_channel_offset(60, 3, 1), 316);

        // Previous channel wraps circularly (ch 16 precedes ch 1)
        assert_eq!(compute_microtonal_channel_offset(60, 16, 1), -68);
        assert_eq!(compute_microtonal_channel_offset(60, 15, 1), -196);

        // Wrapping forward from ch 16 reaches ch 1 (+1 block)
        assert_eq!(compute_microtonal_channel_offset(60, 1, 16), 188);
        assert_eq!(compute_microtonal_channel_offset(60, 15, 16), -68);

        // All 16 channels * 128 notes produce 2048 unique indices
        let mut seen = HashSet::new();
        for ch in 1..=16_u8 {
            for note in 0..=127_u8 {
                let idx = compute_microtonal_channel_offset(note, ch, 4);
                assert!(seen.insert(idx), "duplicate microtonal index {idx}");
            }
        }
        assert_eq!(seen.len(), 16 * 128);
    }

    #[test]
    fn cc_mapping_evaluation() {
        let mappings = vec![
            MidiCcMapping {
                cc: 1,
                param_path: "morphing".to_string(),
                min: 0.0,
                max: 1.0,
            },
            MidiCcMapping {
                cc: 7,
                param_path: "volume_db".to_string(),
                min: -48.0,
                max: 6.0,
            },
            MidiCcMapping {
                cc: 7,
                param_path: "post_effect_gain_db".to_string(),
                min: 6.0,
                max: -6.0,
            },
        ];

        let cc1_min = evaluate_cc_mapping(1, 0, &mappings);
        assert_eq!(cc1_min, vec![("morphing".to_string(), 0.0)]);

        let cc1_max = evaluate_cc_mapping(1, 127, &mappings);
        assert_eq!(cc1_max, vec![("morphing".to_string(), 1.0)]);

        let cc7_mid = evaluate_cc_mapping(7, 64, &mappings);
        assert_eq!(cc7_mid.len(), 2);
        assert_eq!(cc7_mid[0].0, "volume_db");
        assert!((cc7_mid[0].1 - (-20.7874)).abs() < 0.05);
        assert_eq!(cc7_mid[1].0, "post_effect_gain_db");
        assert!((cc7_mid[1].1 - (-0.0472)).abs() < 0.05);

        assert!(evaluate_cc_mapping(74, 100, &mappings).is_empty());
    }

    #[test]
    fn demo_midi_sequence_is_well_formed() {
        let seq = generate_demo_midi_sequence();
        assert!(!seq.is_empty());

        let mut last_ts = 0_u64;
        let mut active_notes: StdHashMap<(u8, u8), usize> = StdHashMap::new();
        let mut saw_sustain_on = false;
        let mut saw_sustain_off = false;

        for (ts, ev) in &seq {
            assert!(*ts >= last_ts, "timestamps must be monotonic");
            last_ts = *ts;
            match *ev {
                MidiEngineEvent::NoteOn {
                    note,
                    velocity,
                    channel,
                } => {
                    assert!((21..=108).contains(&note));
                    assert!((1..=127).contains(&velocity));
                    *active_notes.entry((note, channel)).or_insert(0) += 1;
                }
                MidiEngineEvent::NoteOff { note, channel } => {
                    let count = active_notes
                        .get_mut(&(note, channel))
                        .expect("NoteOff without matching NoteOn");
                    assert!(*count > 0);
                    *count -= 1;
                }
                MidiEngineEvent::ControlChange { cc: 64, value, .. } => {
                    if value > 0 {
                        saw_sustain_on = true;
                    } else {
                        saw_sustain_off = true;
                    }
                }
                _ => {}
            }
        }

        assert!(saw_sustain_on && saw_sustain_off);
        assert!(
            active_notes.values().all(|&c| c == 0),
            "demo sequence left hanging notes: {:?}",
            active_notes
        );
    }

    #[test]
    fn port_hotplug_diff_detection() {
        let p_a = MidiPortInfo {
            index: 0,
            name: "Keystation 88".to_string(),
        };
        let p_b = MidiPortInfo {
            index: 1,
            name: "Roland RD-2000".to_string(),
        };
        let p_b_renumbered = MidiPortInfo {
            index: 0,
            name: "Roland RD-2000".to_string(),
        };
        let p_c = MidiPortInfo {
            index: 1,
            name: "Nord Stage".to_string(),
        };

        // Unchanged
        let (added, removed) =
            detect_midi_port_changes(&[p_a.clone(), p_b.clone()], &[p_a.clone(), p_b.clone()]);
        assert!(added.is_empty());
        assert!(removed.is_empty());

        // Unplug A, B renumbered 1 -> 0, C plugged at 1
        let (added, removed) =
            detect_midi_port_changes(&[p_a.clone(), p_b.clone()], &[p_b_renumbered, p_c.clone()]);
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].name, "Nord Stage");
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].name, "Keystation 88");

        // Duplicate port names preserve multiplicity
        let usb0 = MidiPortInfo {
            index: 0,
            name: "USB MIDI".to_string(),
        };
        let usb1 = MidiPortInfo {
            index: 1,
            name: "USB MIDI".to_string(),
        };
        let (added, removed) = detect_midi_port_changes(&[usb0.clone()], &[usb0, usb1]);
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].index, 1);
        assert!(removed.is_empty());
    }

    #[test]
    fn midi_port_discovery_and_missing_port_connect_error() {
        let _ports = list_midi_input_ports();
        let (tx, _rx) = std::sync::mpsc::channel();
        let res = connect_midi_port("__nonexistent_midi_port_9999__", tx);
        assert!(res.is_err());
    }

    #[test]
    fn test_smf_midi_file_roundtrip() {
        let demo = generate_demo_midi_sequence();
        let smf_480 = write_smf_bytes(&demo, 480);

        assert_eq!(&smf_480[0..4], b"MThd");
        assert_eq!(&smf_480[14..18], b"MTrk");
        assert!(smf_480.ends_with(&[0x00, 0xFF, 0x2F, 0x00]));

        let parsed = parse_smf_bytes(&smf_480).expect("SMF roundtrip parse must succeed");
        assert_eq!(parsed.len(), demo.len());

        let mut prev_ms = 0_u64;
        let mut saw_cc64_on = false;
        let mut saw_cc64_off = false;
        let mut demo_notes = Vec::new();
        let mut parsed_notes = Vec::new();

        for ((orig_ms, orig_ev), (got_ms, got_ev)) in demo.iter().zip(parsed.iter()) {
            assert!(*got_ms >= prev_ms, "parsed SMF timestamps must be monotonic");
            prev_ms = *got_ms;
            assert!(
                orig_ms.abs_diff(*got_ms) <= 1,
                "timestamp drift >1ms at orig={orig_ms}, got={got_ms}"
            );
            assert_eq!(got_ev, orig_ev);

            match *got_ev {
                MidiEngineEvent::NoteOn { note, .. } => parsed_notes.push(note),
                MidiEngineEvent::ControlChange { cc: 64, value, .. } => {
                    if value > 0 {
                        saw_cc64_on = true;
                    } else {
                        saw_cc64_off = true;
                    }
                }
                _ => {}
            }
            if let MidiEngineEvent::NoteOn { note, .. } = *orig_ev {
                demo_notes.push(note);
            }
        }
        assert_eq!(parsed_notes, demo_notes);
        assert!(saw_cc64_on && saw_cc64_off);

        // Exact millisecond roundtrip across all 6 MidiEngineEvent variants at 500 PPQN (1 tick = 1 ms)
        let all_variants = vec![
            (
                0,
                MidiEngineEvent::ProgramChange {
                    program: 12,
                    channel: 2,
                },
            ),
            (
                50,
                MidiEngineEvent::NoteOn {
                    note: 60,
                    velocity: 96,
                    channel: 2,
                },
            ),
            (
                150,
                MidiEngineEvent::Aftertouch {
                    pressure: 77,
                    channel: 2,
                },
            ),
            (
                225,
                MidiEngineEvent::PitchBend {
                    value_14bit: 12_345,
                    channel: 2,
                },
            ),
            (
                400,
                MidiEngineEvent::ControlChange {
                    cc: 64,
                    value: 127,
                    channel: 16,
                },
            ),
            (
                800,
                MidiEngineEvent::NoteOff {
                    note: 60,
                    channel: 2,
                },
            ),
        ];
        let smf_500 = write_smf_bytes(&all_variants, 500);
        let parsed_500 = parse_smf_bytes(&smf_500).expect("500 PPQN SMF parse must succeed");
        assert_eq!(parsed_500, all_variants);
    }

    #[test]
    fn test_smf_format1_running_status_tempo_and_errors() {
        // Format 1, 2 tracks, 480 PPQN
        // Track 0: tempo 60 BPM (1_000_000 us = 0x0F4240) at tick 0,
        //          then tempo 120 BPM (500_000 us = 0x07A120) at tick 480 (vlq 0x83 0x60)
        let trk0: Vec<u8> = vec![
            0x00, 0xFF, 0x51, 0x03, 0x0F, 0x42, 0x40, // tick 0: 60 BPM
            0x00, 0xFF, 0x01, 0x04, b'T', b'e', b's', b't', // tick 0: text meta
            0x83, 0x60, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20, // tick 480: 120 BPM
            0x00, 0xFF, 0x2F, 0x00, // EOT
        ];
        // Track 1: SysEx + NoteOn + running status NoteOn + running status NoteOn(vel=0->NoteOff)
        //          + PolyPressure (skipped) + CC64 at tick 480 (1000ms) + NoteOff at tick 960 (1500ms)
        let trk1: Vec<u8> = vec![
            0x00, 0xF0, 0x04, 0x7E, 0x7F, 0x09, 0x01, // SysEx (cancels running status)
            0x00, 0x90, 60, 80, // tick 0: NoteOn 60, vel 80
            0x00, 64, 90, // tick 0 (running status 0x90): NoteOn 64, vel 90
            0x00, 60, 0, // tick 0 (running status 0x90, vel=0 -> NoteOff 60)
            0x00, 0xA0, 64, 50, // tick 0: Poly Key Pressure (skipped cleanly)
            0x83, 0x60, 0xB0, 64, 127, // tick 480 (= 1000 ms @ 60 BPM): CC64=127
            0x83, 0x60, 0x80, 64, 0, // tick 960 (= 1000 + 500 = 1500 ms @ 120 BPM): NoteOff 64
            0x00, 0xFF, 0x2F, 0x00, // EOT
        ];

        let mut smf = Vec::new();
        smf.extend_from_slice(b"MThd");
        smf.extend_from_slice(&6_u32.to_be_bytes());
        smf.extend_from_slice(&1_u16.to_be_bytes()); // Format 1
        smf.extend_from_slice(&2_u16.to_be_bytes()); // 2 tracks
        smf.extend_from_slice(&480_u16.to_be_bytes());
        smf.extend_from_slice(b"MTrk");
        smf.extend_from_slice(&(trk0.len() as u32).to_be_bytes());
        smf.extend_from_slice(&trk0);
        smf.extend_from_slice(b"MTrk");
        smf.extend_from_slice(&(trk1.len() as u32).to_be_bytes());
        smf.extend_from_slice(&trk1);

        let parsed = parse_smf_bytes(&smf).expect("Format 1 SMF parse must succeed");
        assert_eq!(
            parsed,
            vec![
                (
                    0,
                    MidiEngineEvent::NoteOn {
                        note: 60,
                        velocity: 80,
                        channel: 1,
                    }
                ),
                (
                    0,
                    MidiEngineEvent::NoteOn {
                        note: 64,
                        velocity: 90,
                        channel: 1,
                    }
                ),
                (
                    0,
                    MidiEngineEvent::NoteOff {
                        note: 60,
                        channel: 1,
                    }
                ),
                (
                    1000,
                    MidiEngineEvent::ControlChange {
                        cc: 64,
                        value: 127,
                        channel: 1,
                    }
                ),
                (
                    1500,
                    MidiEngineEvent::NoteOff {
                        note: 64,
                        channel: 1,
                    }
                ),
            ]
        );

        // Error checks
        assert!(parse_smf_bytes(b"BAD!").is_err());
        let mut smpte = smf.clone();
        smpte[12] = 0xE7; // SMPTE bit set
        assert!(parse_smf_bytes(&smpte).is_err());
        let mut fmt2 = smf.clone();
        fmt2[9] = 2; // Format 2 unsupported
        assert!(parse_smf_bytes(&fmt2).is_err());
    }

    #[test]
    fn test_live_or_loopback_midi_stream_dispatch() {
        let ports = list_midi_input_ports();
        let (tx, rx) = std::sync::mpsc::channel::<MidiEngineEvent>();

        // If any system ALSA/JACK port (including Midi Through) is present, verify clean connect & drop
        if let Some(target_port) = ports
            .iter()
            .find(|p| p.name.contains("Midi Through"))
            .or_else(|| ports.first())
        {
            let handle = connect_midi_port(&target_port.name, tx.clone())
                .expect("connecting to discovered MIDI port should succeed");
            assert_eq!(handle.port_name, target_port.name);
            drop(handle);
        }

        // End-to-end mpsc dispatch pipeline:
        // raw MIDI bytes -> parse_midi_message -> mpsc -> apply_transpose_and_range -> apply_velocity_curve -> evaluate_cc_mapping
        let raw_packets: &[&[u8]] = &[
            &[0x90, 60, 100], // NoteOn C4
            &[0xB0, 64, 127], // CC64 Sustain on
            &[0xB0, 1, 64],   // CC1 -> morphing
            &[0xB0, 7, 127],  // CC7 -> volume_db max
            &[0xE0, 0x00, 0x60], // PitchBend up
            &[0xD0, 88],      // Aftertouch
            &[0xC0, 3],       // ProgramChange
            &[0x90, 60, 0],   // NoteOn vel=0 -> NoteOff
        ];

        for pkt in raw_packets {
            if let Some(ev) = parse_midi_message(pkt) {
                tx.send(ev).expect("mpsc send must succeed");
            }
        }

        let mut cfg = MidiConfig::default();
        cfg.transpose = 12;

        let mut notes_on = Vec::new();
        let mut notes_off = Vec::new();
        let mut cc_updates = Vec::new();
        let mut sustain_values = Vec::new();
        let mut saw_pitch_bend = false;
        let mut saw_aftertouch = false;
        let mut saw_program_change = false;

        while let Ok(ev) = rx.try_recv() {
            match ev {
                MidiEngineEvent::NoteOn { note, velocity, .. } => {
                    let mapped_note = apply_transpose_and_range(note, &cfg)
                        .expect("transposed note should be in range");
                    let mapped_vel = apply_velocity_curve(velocity, &cfg.velocity_curve);
                    notes_on.push((mapped_note, mapped_vel));
                }
                MidiEngineEvent::NoteOff { note, .. } => {
                    let mapped_note = apply_transpose_and_range(note, &cfg)
                        .expect("transposed note-off should be in range");
                    notes_off.push(mapped_note);
                }
                MidiEngineEvent::ControlChange { cc, value, .. } => {
                    if cc == 64 {
                        sustain_values.push(value);
                    } else {
                        cc_updates.extend(evaluate_cc_mapping(cc, value, &cfg.cc_mappings));
                    }
                }
                MidiEngineEvent::PitchBend { value_14bit, .. } => {
                    assert!(value_14bit > 8192);
                    saw_pitch_bend = true;
                }
                MidiEngineEvent::Aftertouch { pressure, .. } => {
                    assert_eq!(pressure, 88);
                    saw_aftertouch = true;
                }
                MidiEngineEvent::ProgramChange { program, .. } => {
                    assert_eq!(program, 3);
                    saw_program_change = true;
                }
            }
        }

        assert_eq!(notes_on.len(), 1);
        assert_eq!(notes_on[0].0, 72); // C4 (60) + 12 = C5 (72)
        assert_eq!(notes_on[0].1, 118); // Reinforced factory curve: input velocity 100.
        assert_eq!(notes_off, vec![72]);
        assert_eq!(sustain_values, vec![127]);
        assert_eq!(cc_updates.len(), 2);
        assert_eq!(cc_updates[0].0, "morphing");
        assert!((cc_updates[0].1 - (64.0 / 127.0)).abs() < 1e-4);
        assert_eq!(cc_updates[1].0, "volume_db");
        assert!((cc_updates[1].1 - 6.0).abs() < 1e-4);
        assert!(saw_pitch_bend && saw_aftertouch && saw_program_change);
    }
}
