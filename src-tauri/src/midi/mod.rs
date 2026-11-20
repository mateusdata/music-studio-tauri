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

/// Evaluates a piecewise-linear MIDI velocity curve mapping raw velocity [1..127] -> [1..127].
pub fn apply_velocity_curve(raw_vel: u8, curve: &[[f32; 2]]) -> u8 {
    if raw_vel == 0 {
        return 0;
    }
    let clamped_vel = raw_vel.min(127);
    if curve.is_empty() {
        return clamped_vel;
    }
    let mut pts: Vec<[f32; 2]> = curve
        .iter()
        .copied()
        .filter(|p| p[0].is_finite() && p[1].is_finite())
        .collect();
    if pts.is_empty() {
        return clamped_vel;
    }

    let x = (clamped_vel as f32 / 127.0).clamp(0.0, 1.0);
    pts.sort_by(|a, b| a[0].total_cmp(&b[0]));

    if x <= pts[0][0] {
        return (pts[0][1].clamp(0.01, 1.0) * 127.0).round() as u8;
    }
    for w in pts.windows(2) {
        let (x0, y0) = (w[0][0], w[0][1]);
        let (x1, y1) = (w[1][0], w[1][1]);
        if x <= x1 {
            let span = (x1 - x0).max(1e-5);
            let t = (x - x0) / span;
            let y = y0 + (y1 - y0) * t;
            return (y.clamp(0.01, 1.0) * 127.0).round() as u8;
        }
    }
    let last_y = pts.last().map(|p| p[1]).unwrap_or(1.0);
    (last_y.clamp(0.01, 1.0) * 127.0).round() as u8
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
}
