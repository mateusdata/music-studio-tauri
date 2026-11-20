use crate::contracts::{
    EngineParams, EngineSnapshot, EngineTelemetry, InstrumentModel, MidiConfig, PedalState,
    PresetData,
};
use crate::effects::EffectsChain;
use crate::instruments::build_factory_instruments;
use crate::midi::{
    apply_transpose_and_range, apply_velocity_curve, connect_midi_port, list_midi_input_ports,
    MidiConnectionHandle, MidiEngineEvent,
};
use crate::presets::{build_factory_presets, load_user_presets, morph_engine_params, save_user_preset};
use crate::synthesis::{PianoSynthEngine, MAX_POLYPHONY};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use parking_lot::{Mutex, RwLock};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum AudioCommand {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
    AllNotesOff,
    SetPedal { pedal: String, value: f32 },
    UpdateParams(Box<EngineParams>),
    UpdateInstrument(Box<InstrumentModel>),
}

pub struct SharedAudioState {
    pub params: RwLock<EngineParams>,
    pub pedals: RwLock<PedalState>,
    pub midi_config: RwLock<MidiConfig>,
    pub active_instrument: RwLock<InstrumentModel>,
    pub active_preset_id: RwLock<String>,
    pub preset_b_id: RwLock<Option<String>>,
    pub instruments: RwLock<Vec<InstrumentModel>>,
    pub presets: RwLock<Vec<PresetData>>,
    pub telemetry: RwLock<EngineTelemetry>,
}

pub struct AudioEngineManager {
    pub shared: Arc<SharedAudioState>,
    cmd_tx: Mutex<Sender<AudioCommand>>,
    midi_handle: Mutex<Option<MidiConnectionHandle>>,
    midi_tx: Mutex<Sender<MidiEngineEvent>>,
    _keep_alive: Arc<AtomicBool>,
}

impl AudioEngineManager {
    pub fn start() -> Self {
        let instruments = build_factory_instruments();
        let mut presets = build_factory_presets(&instruments);
        presets.extend(load_user_presets());

        let first_inst = instruments[0].clone();
        let first_preset_id = presets[0].id.clone();
        let initial_params = presets[0].params.clone();

        let (cmd_tx, cmd_rx) = mpsc::channel::<AudioCommand>();
        let (midi_tx, midi_rx) = mpsc::channel::<MidiEngineEvent>();

        let shared = Arc::new(SharedAudioState {
            params: RwLock::new(initial_params.clone()),
            pedals: RwLock::new(PedalState::default()),
            midi_config: RwLock::new(MidiConfig::default()),
            active_instrument: RwLock::new(first_inst.clone()),
            active_preset_id: RwLock::new(first_preset_id),
            preset_b_id: RwLock::new(presets.get(1).map(|p| p.id.clone())),
            instruments: RwLock::new(instruments),
            presets: RwLock::new(presets),
            telemetry: RwLock::new(EngineTelemetry {
                sample_rate: 48000,
                buffer_size: 256,
                latency_ms: 5.33,
                active_voices: 0,
                max_polyphony: MAX_POLYPHONY,
                cpu_load_pct: 0.0,
                peak_l: 0.0,
                peak_r: 0.0,
                rms_l: 0.0,
                rms_r: 0.0,
                audio_backend: "Initializing...".to_string(),
                audio_active: false,
                midi_connected: false,
                midi_port_name: None,
                last_midi_note: None,
                last_midi_velocity: None,
                active_notes: Vec::new(),
                pedals: PedalState::default(),
            }),
        });

        let keep_alive = Arc::new(AtomicBool::new(true));

        // MIDI event bridge thread -> dispatches into AudioCommand & SharedAudioState
        let midi_shared = Arc::clone(&shared);
        let midi_cmd_tx = cmd_tx.clone();
        std::thread::spawn(move || {
            Self::midi_router_loop(midi_rx, midi_shared, midi_cmd_tx);
        });

        // Start real-time cpal audio output or fallback headless DSP thread on an audio supervisor thread
        let stream_shared = Arc::clone(&shared);
        let stream_keep_alive = Arc::clone(&keep_alive);
        std::thread::spawn(move || {
            let _stream = Self::spawn_audio_stream(stream_shared, cmd_rx, Arc::clone(&stream_keep_alive));
            while stream_keep_alive.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        });

        let mgr = Self {
            shared,
            cmd_tx: Mutex::new(cmd_tx),
            midi_handle: Mutex::new(None),
            midi_tx: Mutex::new(midi_tx),
            _keep_alive: keep_alive,
        };

        // Attempt auto-connect to first non-Through MIDI input port if present
        let ports = list_midi_input_ports();
        if let Some(p) = ports.iter().find(|p| !p.name.contains("Midi Through")) {
            let _ = mgr.connect_midi(&p.name);
        }

        mgr
    }

    fn midi_router_loop(
        midi_rx: Receiver<MidiEngineEvent>,
        shared: Arc<SharedAudioState>,
        cmd_tx: Sender<AudioCommand>,
    ) {
        while let Ok(ev) = midi_rx.recv() {
            let cfg = shared.midi_config.read().clone();
            match ev {
                MidiEngineEvent::NoteOn { note, velocity, channel } => {
                    if cfg.channel != 0 && cfg.channel != channel && !cfg.microtonal_multichannel {
                        continue;
                    }
                    if let Some(mapped_note) = apply_transpose_and_range(note, &cfg) {
                        let mapped_vel = apply_velocity_curve(velocity, &cfg.velocity_curve);
                        {
                            let mut t = shared.telemetry.write();
                            t.last_midi_note = Some(mapped_note);
                            t.last_midi_velocity = Some(mapped_vel);
                        }
                        let _ = cmd_tx.send(AudioCommand::NoteOn {
                            note: mapped_note,
                            velocity: mapped_vel,
                        });
                    }
                }
                MidiEngineEvent::NoteOff { note, channel } => {
                    if cfg.channel != 0 && cfg.channel != channel && !cfg.microtonal_multichannel {
                        continue;
                    }
                    if let Some(mapped_note) = apply_transpose_and_range(note, &cfg) {
                        let _ = cmd_tx.send(AudioCommand::NoteOff { note: mapped_note });
                    }
                }
                MidiEngineEvent::ControlChange { cc, value, .. } => {
                    let norm = value as f32 / 127.0;
                    match cc {
                        64 => {
                            let _ = cmd_tx.send(AudioCommand::SetPedal {
                                pedal: "sustain".to_string(),
                                value: norm,
                            });
                        }
                        66 => {
                            let _ = cmd_tx.send(AudioCommand::SetPedal {
                                pedal: "sostenuto".to_string(),
                                value: norm,
                            });
                        }
                        67 => {
                            let _ = cmd_tx.send(AudioCommand::SetPedal {
                                pedal: "soft".to_string(),
                                value: norm,
                            });
                        }
                        69 => {
                            let _ = cmd_tx.send(AudioCommand::SetPedal {
                                pedal: "harmonic".to_string(),
                                value: norm,
                            });
                        }
                        120 | 123 => {
                            let _ = cmd_tx.send(AudioCommand::AllNotesOff);
                        }
                        _ => {
                            // Check CC mappings
                            for m in &cfg.cc_mappings {
                                if m.cc == cc {
                                    let val = m.min + (m.max - m.min) * norm;
                                    let mut params = shared.params.write();
                                    if m.param_path == "volume_db" {
                                        params.volume_db = val;
                                    } else if m.param_path == "morphing" {
                                        params.morphing = val;
                                    }
                                    let _ = cmd_tx.send(AudioCommand::UpdateParams(Box::new(params.clone())));
                                }
                            }
                        }
                    }
                }
                MidiEngineEvent::PitchBend { value_14bit, .. } => {
                    let norm = (value_14bit as f32 - 8192.0) / 8192.0;
                    let mut params = shared.params.write();
                    params.tuning.pitch_bend_semitones = norm * 2.0;
                    let _ = cmd_tx.send(AudioCommand::UpdateParams(Box::new(params.clone())));
                }
                _ => {}
            }
        }
    }

    fn spawn_audio_stream(
        shared: Arc<SharedAudioState>,
        cmd_rx: Receiver<AudioCommand>,
        keep_alive: Arc<AtomicBool>,
    ) -> Option<Stream> {
        let host = cpal::default_host();
        if let Some(device) = host.default_output_device() {
            if let Ok(stream) = Self::build_cpal_stream(&device, Arc::clone(&shared), cmd_rx) {
                if stream.play().is_ok() {
                    return Some(stream);
                }
            }
        }

        // Fallback if headless / audio hardware unavailable
        {
            let mut t = shared.telemetry.write();
            t.audio_backend = "Headless DSP Fallback (48kHz)".to_string();
            t.audio_active = true;
        }
        std::thread::spawn(move || {
            let sr = 48000.0_f32;
            let mut synth = PianoSynthEngine::new(sr);
            let mut fx = EffectsChain::new(sr);
            let mut params = shared.params.read().clone();
            let mut pedals = PedalState::default();
            let mut inst = shared.active_instrument.read().clone();
            let mut buf_l = vec![0.0_f32; 256];
            let mut buf_r = vec![0.0_f32; 256];
            while keep_alive.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(5));
                Self::render_dsp_slice(256, &mut buf_l, &mut buf_r, &mut synth, &mut fx, &mut params, &mut pedals, &mut inst, &shared, None, sr);
            }
        });
        None
    }

    fn build_cpal_stream(
        device: &Device,
        shared: Arc<SharedAudioState>,
        cmd_rx: Receiver<AudioCommand>,
    ) -> Result<Stream, String> {
        let default_cfg = device.default_output_config().map_err(|e| e.to_string())?;
        let sr = default_cfg.sample_rate().0;
        let channels = default_cfg.channels().max(2) as usize;
        let dev_name = device.name().unwrap_or_else(|_| "Linux Audio".to_string());

        let config = StreamConfig {
            channels: channels as u16,
            sample_rate: cpal::SampleRate(sr),
            buffer_size: cpal::BufferSize::Default,
        };

        {
            let mut t = shared.telemetry.write();
            t.sample_rate = sr;
            t.buffer_size = 256;
            t.latency_ms = (256.0 / sr as f32) * 1000.0;
            t.audio_backend = format!("PipeWire/ALSA ({})", dev_name);
            t.audio_active = true;
        }

        let mut synth = PianoSynthEngine::new(sr as f32);
        let mut fx = EffectsChain::new(sr as f32);
        let mut params = shared.params.read().clone();
        let mut pedals = PedalState::default();
        let mut inst = shared.active_instrument.read().clone();
        let mut buf_l = vec![0.0_f32; 2048];
        let mut buf_r = vec![0.0_f32; 2048];
        let mut telemetry_tick: u32 = 0;

        let stream = device
            .build_output_stream(
                &config,
                move |output: &mut [f32], _| {
                    let start = Instant::now();
                    let frames = output.len() / channels;
                    if frames > buf_l.len() {
                        buf_l.resize(frames, 0.0);
                        buf_r.resize(frames, 0.0);
                    }

                    // Drain lock-free command queue
                    while let Ok(cmd) = cmd_rx.try_recv() {
                        match cmd {
                            AudioCommand::NoteOn { note, velocity } => {
                                synth.note_on(note, velocity, &params, &inst, &pedals);
                            }
                            AudioCommand::NoteOff { note } => {
                                synth.note_off(note, &params, &pedals);
                            }
                            AudioCommand::AllNotesOff => synth.all_notes_off(),
                            AudioCommand::SetPedal { pedal, value } => {
                                let prev = pedals.clone();
                                match pedal.as_str() {
                                    "sustain" => pedals.sustain = value.clamp(0.0, 1.0),
                                    "soft" => pedals.soft = value.clamp(0.0, 1.0),
                                    "sostenuto" => pedals.sostenuto = value.clamp(0.0, 1.0),
                                    "harmonic" => pedals.harmonic = value.clamp(0.0, 1.0),
                                    "celeste" => pedals.celeste = value.clamp(0.0, 1.0),
                                    "buff_stop" => pedals.buff_stop = value.clamp(0.0, 1.0),
                                    "rattle" => pedals.rattle = value.clamp(0.0, 1.0),
                                    _ => {}
                                }
                                synth.update_pedals(&pedals, &prev, &params);
                            }
                            AudioCommand::UpdateParams(new_p) => params = *new_p,
                            AudioCommand::UpdateInstrument(new_i) => inst = *new_i,
                        }
                    }

                    let slice_l = &mut buf_l[..frames];
                    let slice_r = &mut buf_r[..frames];
                    synth.process_block(slice_l, slice_r, &params, &pedals);
                    fx.process_block(slice_l, slice_r, &params);

                    let mut peak_l = 0.0_f32;
                    let mut peak_r = 0.0_f32;
                    let mut sq_l = 0.0_f32;
                    let mut sq_r = 0.0_f32;

                    for f in 0..frames {
                        let l = slice_l[f];
                        let r = slice_r[f];
                        peak_l = peak_l.max(l.abs());
                        peak_r = peak_r.max(r.abs());
                        sq_l += l * l;
                        sq_r += r * r;
                        let base = f * channels;
                        output[base] = l;
                        if channels > 1 {
                            output[base + 1] = r;
                        }
                        for ch in 2..channels {
                            output[base + ch] = 0.0;
                        }
                    }

                    telemetry_tick = telemetry_tick.wrapping_add(1);
                    if telemetry_tick % 4 == 0 {
                        let elapsed_us = start.elapsed().as_secs_f32() * 1_000_000.0;
                        let budget_us = (frames as f32 / sr as f32) * 1_000_000.0;
                        let cpu = ((elapsed_us / budget_us.max(1.0)) * 100.0).clamp(0.0, 100.0);
                        if let Some(mut t) = shared.telemetry.try_write() {
                            t.buffer_size = frames;
                            t.latency_ms = (frames as f32 / sr as f32) * 1000.0;
                            t.active_voices = synth.active_voice_count();
                            t.cpu_load_pct = cpu;
                            t.peak_l = peak_l;
                            t.peak_r = peak_r;
                            t.rms_l = (sq_l / frames.max(1) as f32).sqrt();
                            t.rms_r = (sq_r / frames.max(1) as f32).sqrt();
                            t.active_notes = synth.active_notes();
                            t.pedals = pedals.clone();
                        }
                    }
                },
                |_err| {},
                None,
            )
            .map_err(|e| e.to_string())?;

        Ok(stream)
    }

    #[allow(clippy::too_many_arguments)]
    fn render_dsp_slice(
        frames: usize,
        buf_l: &mut [f32],
        buf_r: &mut [f32],
        synth: &mut PianoSynthEngine,
        fx: &mut EffectsChain,
        params: &mut EngineParams,
        pedals: &mut PedalState,
        inst: &mut InstrumentModel,
        shared: &Arc<SharedAudioState>,
        _cmd_rx: Option<&Receiver<AudioCommand>>,
        sr: f32,
    ) {
        *params = shared.params.read().clone();
        *pedals = shared.pedals.read().clone();
        *inst = shared.active_instrument.read().clone();
        synth.process_block(&mut buf_l[..frames], &mut buf_r[..frames], params, pedals);
        fx.process_block(&mut buf_l[..frames], &mut buf_r[..frames], params);
        if let Some(mut t) = shared.telemetry.try_write() {
            t.sample_rate = sr as u32;
            t.active_voices = synth.active_voice_count();
            t.active_notes = synth.active_notes();
            t.pedals = pedals.clone();
        }
    }

    pub fn note_on(&self, note: u8, velocity: u8) {
        let _ = self.cmd_tx.lock().send(AudioCommand::NoteOn { note, velocity });
    }

    pub fn note_off(&self, note: u8) {
        let _ = self.cmd_tx.lock().send(AudioCommand::NoteOff { note });
    }

    pub fn all_notes_off(&self) {
        let _ = self.cmd_tx.lock().send(AudioCommand::AllNotesOff);
    }

    pub fn set_pedal(&self, pedal: &str, value: f32) {
        {
            let mut p = self.shared.pedals.write();
            match pedal {
                "sustain" => p.sustain = value.clamp(0.0, 1.0),
                "soft" => p.soft = value.clamp(0.0, 1.0),
                "sostenuto" => p.sostenuto = value.clamp(0.0, 1.0),
                "harmonic" => p.harmonic = value.clamp(0.0, 1.0),
                "celeste" => p.celeste = value.clamp(0.0, 1.0),
                "buff_stop" => p.buff_stop = value.clamp(0.0, 1.0),
                "rattle" => p.rattle = value.clamp(0.0, 1.0),
                "super_sostenuto" => p.super_sostenuto = value >= 0.5,
                "reversed_sustain" => p.reversed_sustain = value >= 0.5,
                _ => {}
            }
        }
        let _ = self.cmd_tx.lock().send(AudioCommand::SetPedal {
            pedal: pedal.to_string(),
            value,
        });
    }

    pub fn set_params(&self, params: EngineParams) {
        *self.shared.params.write() = params.clone();
        let _ = self.cmd_tx.lock().send(AudioCommand::UpdateParams(Box::new(params)));
    }

    pub fn select_instrument(&self, instrument_id: &str) -> Result<EngineSnapshot, String> {
        let inst = self
            .shared
            .instruments
            .read()
            .iter()
            .find(|i| i.id == instrument_id)
            .cloned()
            .ok_or_else(|| format!("Unknown instrument '{}'", instrument_id))?;

        let matching_preset = self
            .shared
            .presets
            .read()
            .iter()
            .find(|p| p.instrument_id == instrument_id)
            .cloned();

        *self.shared.active_instrument.write() = inst.clone();
        let new_params = matching_preset
            .as_ref()
            .map(|p| p.params.clone())
            .unwrap_or_else(|| inst.default_params.clone());
        *self.shared.params.write() = new_params.clone();
        if let Some(p) = matching_preset {
            *self.shared.active_preset_id.write() = p.id;
        }
        let tx = self.cmd_tx.lock();
        let _ = tx.send(AudioCommand::UpdateInstrument(Box::new(inst)));
        let _ = tx.send(AudioCommand::UpdateParams(Box::new(new_params)));
        drop(tx);
        Ok(self.snapshot())
    }

    pub fn select_preset(&self, preset_id: &str) -> Result<EngineSnapshot, String> {
        let preset = self
            .shared
            .presets
            .read()
            .iter()
            .find(|p| p.id == preset_id)
            .cloned()
            .ok_or_else(|| format!("Unknown preset '{}'", preset_id))?;

        if let Some(inst) = self
            .shared
            .instruments
            .read()
            .iter()
            .find(|i| i.id == preset.instrument_id)
            .cloned()
        {
            *self.shared.active_instrument.write() = inst.clone();
            let _ = self.cmd_tx.lock().send(AudioCommand::UpdateInstrument(Box::new(inst)));
        }

        *self.shared.active_preset_id.write() = preset.id.clone();
        *self.shared.params.write() = preset.params.clone();
        let _ = self.cmd_tx.lock().send(AudioCommand::UpdateParams(Box::new(preset.params)));
        Ok(self.snapshot())
    }

    pub fn morph_ab(&self, preset_a_id: &str, preset_b_id: &str, t: f32) -> Result<EngineParams, String> {
        let presets = self.shared.presets.read();
        let a = presets
            .iter()
            .find(|p| p.id == preset_a_id)
            .ok_or_else(|| format!("Unknown preset A '{}'", preset_a_id))?;
        let b = presets
            .iter()
            .find(|p| p.id == preset_b_id)
            .ok_or_else(|| format!("Unknown preset B '{}'", preset_b_id))?;
        let morphed = morph_engine_params(&a.params, &b.params, t);
        *self.shared.preset_b_id.write() = Some(preset_b_id.to_string());
        drop(presets);
        self.set_params(morphed.clone());
        Ok(morphed)
    }

    pub fn save_preset(&self, name: &str, category: &str) -> Result<PresetData, String> {
        let inst_id = self.shared.active_instrument.read().id.clone();
        let id = format!(
            "user-{}",
            name.to_lowercase()
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                .collect::<String>()
        );
        let preset = PresetData {
            id: id.clone(),
            name: name.to_string(),
            instrument_id: inst_id,
            category: category.to_string(),
            description: "User saved physical modeling preset".to_string(),
            params: self.shared.params.read().clone(),
            is_factory: false,
        };
        save_user_preset(&preset)?;
        let mut list = self.shared.presets.write();
        list.retain(|p| p.id != id);
        list.push(preset.clone());
        *self.shared.active_preset_id.write() = id;
        Ok(preset)
    }

    pub fn connect_midi(&self, port_name: &str) -> Result<String, String> {
        let tx = self.midi_tx.lock().clone();
        let handle = connect_midi_port(port_name, tx)?;
        let connected_name = handle.port_name.clone();
        *self.midi_handle.lock() = Some(handle);
        self.shared.midi_config.write().selected_port = Some(connected_name.clone());
        let mut t = self.shared.telemetry.write();
        t.midi_connected = true;
        t.midi_port_name = Some(connected_name.clone());
        Ok(connected_name)
    }

    pub fn set_midi_config(&self, cfg: MidiConfig) {
        *self.shared.midi_config.write() = cfg;
    }

    pub fn snapshot(&self) -> EngineSnapshot {
        EngineSnapshot {
            active_instrument_id: self.shared.active_instrument.read().id.clone(),
            active_preset_id: self.shared.active_preset_id.read().clone(),
            preset_b_id: self.shared.preset_b_id.read().clone(),
            params: self.shared.params.read().clone(),
            midi_config: self.shared.midi_config.read().clone(),
            instruments: self.shared.instruments.read().clone(),
            presets: self.shared.presets.read().clone(),
        }
    }

    /// Renders a short C-major arpeggio / chord offline to a stereo 32-bit float WAV file.
    pub fn export_demo_wav(&self, path: PathBuf) -> Result<String, String> {
        let sr = 48000_u32;
        let mut synth = PianoSynthEngine::new(sr as f32);
        let mut fx = EffectsChain::new(sr as f32);
        let params = self.shared.params.read().clone();
        let inst = self.shared.active_instrument.read().clone();
        let mut pedals = PedalState::default();
        pedals.sustain = 1.0;

        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: sr,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(&path, spec).map_err(|e| e.to_string())?;

        let notes = [48_u8, 55, 60, 64, 67, 72];
        let step_samples = (sr as f32 * 0.28) as usize;
        let total_samples = sr as usize * 3;
        let block = 256_usize;
        let mut buf_l = vec![0.0_f32; block];
        let mut buf_r = vec![0.0_f32; block];
        let mut pos = 0_usize;
        let mut next_note_idx = 0_usize;

        while pos < total_samples {
            if next_note_idx < notes.len() && pos >= next_note_idx * step_samples {
                synth.note_on(notes[next_note_idx], 86, &params, &inst, &pedals);
                next_note_idx += 1;
            }
            synth.process_block(&mut buf_l, &mut buf_r, &params, &pedals);
            fx.process_block(&mut buf_l, &mut buf_r, &params);
            for i in 0..block {
                writer.write_sample(buf_l[i]).map_err(|e| e.to_string())?;
                writer.write_sample(buf_r[i]).map_err(|e| e.to_string())?;
            }
            pos += block;
        }
        writer.finalize().map_err(|e| e.to_string())?;
        Ok(path.to_string_lossy().to_string())
    }
}
