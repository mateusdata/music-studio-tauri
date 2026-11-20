use crate::contracts::{KeyRangeMode, MidiConfig, MidiPortInfo};
use midir::{Ignore, MidiInput, MidiInputConnection};
use std::sync::mpsc::Sender;

#[derive(Debug, Clone)]
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
    if curve.is_empty() {
        return raw_vel;
    }
    let x = (raw_vel as f32 / 127.0).clamp(0.0, 1.0);
    let mut pts = curve.to_vec();
    pts.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap_or(std::cmp::Ordering::Equal));

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

pub fn apply_transpose_and_range(note: u8, config: &MidiConfig) -> Option<u8> {
    let mut shifted = note as i16 + config.transpose as i16;
    if config.key_range_mode == KeyRangeMode::Compact61 {
        shifted = shifted.clamp(36, 96);
    }
    if (0..=127).contains(&shifted) {
        Some(shifted as u8)
    } else {
        None
    }
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
                Some(MidiEngineEvent::NoteOn { note, velocity: vel, channel })
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
    let mut selected = None;
    let mut matched_name = String::new();

    for p in &ports {
        if let Ok(name) = midi_in.port_name(p) {
            if name == port_name_or_query || name.contains(port_name_or_query) {
                selected = Some(p.clone());
                matched_name = name;
                break;
            }
        }
    }

    let port = selected.ok_or_else(|| format!("MIDI input port '{}' not found", port_name_or_query))?;
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
