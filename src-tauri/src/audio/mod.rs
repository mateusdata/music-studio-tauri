use crate::contracts::{
    EngineParams, EngineSnapshot, EngineTelemetry, InstrumentModel, MidiCcMapping, MidiConfig,
    MidiTransportState, NoteEditOffset, PedalState, PresetData,
};
use crate::effects::EffectsChain;
use crate::instruments::build_factory_instruments;
use crate::midi::{
    apply_transpose_and_range, apply_velocity_curve, compute_microtonal_channel_offset,
    connect_midi_port, evaluate_cc_mapping, generate_demo_midi_sequence, list_midi_input_ports,
    MidiConnectionHandle, MidiEngineEvent,
};
use crate::music_studio::{
    build_sty_package, list_all_music_packages, parse_instrument_file, parse_music_file,
    rebuild_package_with_soundfont, LoadedInstrument, LoadedMusicPackage, MusicPackageEntry,
    MusicPackageSummary, MusicStudioArrangerEngine, DEFAULT_MUSIC_STUDIO_DIR,
    USER_DOCUMENTS_RHYTHMS_DIR,
};
use crate::presets::{build_factory_presets, load_user_presets, morph_engine_params, save_user_preset};
use crate::soundfont::find_default_soundfont;
use crate::synthesis::{PianoSynthEngine, MAX_POLYPHONY};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use parking_lot::{Mutex, RwLock};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Instant;

fn is_sty_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "sty" | "prs" | "sst" | "bcs"
            )
        })
        .unwrap_or(false)
}

#[derive(Debug, Clone)]
pub enum AudioCommand {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
    AllNotesOff,
    SetPedal { pedal: String, value: f32 },
    UpdateParams(Box<EngineParams>),
    UpdateInstrument(Box<InstrumentModel>),
    LoadMusicPackage(Arc<LoadedMusicPackage>),
    ImportInstrument {
        instrument: Box<LoadedInstrument>,
        assign_upper1: bool,
    },
    SetArrangerSection { section_name: String, immediate: bool },
    SetArrangerTransport {
        playing: bool,
        sync_start: Option<bool>,
        bpm: Option<f32>,
        auto_fill: Option<bool>,
        tempo_locked: Option<bool>,
    },
    SetArrangerStyle(usize),
    SetArrangerVoice {
        slot: String,
        instrument_index: Option<usize>,
        split_key: Option<u8>,
        split_enabled: Option<bool>,
        chord_scan_lower: Option<bool>,
        chord_scan_upper: Option<bool>,
        upper_octave: Option<i8>,
        transpose: Option<i8>,
        chord_mode: Option<String>,
    },
    SetArrangerChord { root_pc: u8, quality: String },
    SetArrangerTrackVolume { track_slot: usize, volume: f32 },
    SetArrangerTrackInstrument {
        track_slot: usize,
        instrument_index: Option<usize>,
    },
    ApplyArrangerOts(usize),
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
    pub active_music_package: RwLock<Option<Arc<LoadedMusicPackage>>>,
    pub arranger_summary: RwLock<MusicPackageSummary>,
    pub active_soundfont_path: RwLock<Option<PathBuf>>,
    pub user_imported_instruments: RwLock<Vec<LoadedInstrument>>,
}

pub struct AudioEngineManager {
    pub shared: Arc<SharedAudioState>,
    cmd_tx: Mutex<Sender<AudioCommand>>,
    reconfigure_tx: Mutex<Sender<u32>>,
    requested_buffer_size: Arc<AtomicU32>,
    midi_handle: Mutex<Option<MidiConnectionHandle>>,
    midi_tx: Mutex<Sender<MidiEngineEvent>>,
    recorded_events: Arc<RwLock<Vec<(u64, MidiEngineEvent)>>>,
    record_start: Arc<Mutex<Option<Instant>>>,
    playback_cancel: Mutex<Option<Arc<AtomicBool>>>,
    _keep_alive: Arc<AtomicBool>,
}

struct AudioDspRuntime {
    synth: PianoSynthEngine,
    arranger: MusicStudioArrangerEngine,
    fx: EffectsChain,
    params: EngineParams,
    pedals: PedalState,
    inst: InstrumentModel,
    cmd_rx: Receiver<AudioCommand>,
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
}

struct StreamRuntimeGuard {
    runtime: Option<AudioDspRuntime>,
    return_slot: Arc<Mutex<Option<AudioDspRuntime>>>,
}

impl Drop for StreamRuntimeGuard {
    fn drop(&mut self) {
        if let Some(rt) = self.runtime.take() {
            *self.return_slot.lock() = Some(rt);
        }
    }
}

impl Drop for AudioEngineManager {
    fn drop(&mut self) {
        self._keep_alive.store(false, Ordering::Relaxed);
        if let Some(cancel) = self.playback_cancel.lock().take() {
            cancel.store(true, Ordering::Relaxed);
        }
    }
}

fn default_cc_range(param_path: &str) -> (f32, f32) {
    match param_path {
        "volume_db" => (-48.0, 6.0),
        "morphing" => (0.0, 1.0),
        "reverb.mix" => (0.0, 1.0),
        "dynamics_db" => (10.0, 80.0),
        "diapason_hz" => (415.0, 466.0),
        "stereo_width" => (0.0, 1.0),
        _ => (0.0, 1.0),
    }
}

fn apply_cc_param(params: &mut EngineParams, path: &str, val: f32) -> bool {
    if !val.is_finite() {
        return false;
    }
    match path {
        "volume_db" => {
            params.volume_db = val.clamp(-60.0, 12.0);
            true
        }
        "morphing" => {
            params.morphing = val.clamp(0.0, 1.0);
            true
        }
        "reverb.mix" => {
            params.reverb.mix = val.clamp(0.0, 1.0);
            true
        }
        "dynamics_db" => {
            params.dynamics_db = val.clamp(5.0, 100.0);
            true
        }
        "diapason_hz" => {
            params.tuning.diapason_hz = val.clamp(380.0, 500.0);
            true
        }
        "stereo_width" => {
            params.output.stereo_width = val.clamp(0.0, 1.0);
            true
        }
        _ => false,
    }
}

fn dispatch_midi_event_to_audio(
    ev: &MidiEngineEvent,
    shared: &Arc<SharedAudioState>,
    cmd_tx: &Sender<AudioCommand>,
    allow_cc_learn: bool,
) {
    let mut cfg = shared.midi_config.read().clone();
    match ev {
        MidiEngineEvent::NoteOn { note, velocity, channel } => {
            if cfg.channel != 0 && cfg.channel != *channel && !cfg.microtonal_multichannel {
                return;
            }
            let (raw_note, channel_delta) = if cfg.microtonal_multichannel {
                let offset = compute_microtonal_channel_offset(*note, *channel, cfg.channel);
                let delta = (offset - (*note & 0x7F) as i32) / 128;
                (offset.rem_euclid(128) as u8, delta)
            } else {
                (*note, 0)
            };
            if let Some(mapped_note) = apply_transpose_and_range(raw_note, &cfg) {
                if cfg.microtonal_multichannel && channel_delta != 0 {
                    let mut params = shared.params.write();
                    let entry = params
                        .note_edit
                        .entry(mapped_note)
                        .or_insert_with(NoteEditOffset::default);
                    entry.detune_cents = (channel_delta as f32) * (100.0 / 16.0);
                    let _ = cmd_tx.send(AudioCommand::UpdateParams(Box::new(params.clone())));
                }
                let mapped_vel = apply_velocity_curve(*velocity, &cfg.velocity_curve);
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
            if cfg.channel != 0 && cfg.channel != *channel && !cfg.microtonal_multichannel {
                return;
            }
            let raw_note = if cfg.microtonal_multichannel {
                compute_microtonal_channel_offset(*note, *channel, cfg.channel).rem_euclid(128) as u8
            } else {
                *note
            };
            if let Some(mapped_note) = apply_transpose_and_range(raw_note, &cfg) {
                let _ = cmd_tx.send(AudioCommand::NoteOff { note: mapped_note });
            }
        }
        MidiEngineEvent::ControlChange { cc, value, .. } => {
            if allow_cc_learn && shared.telemetry.read().midi_learn_target.is_some() {
                if let Some(param_path) = shared.telemetry.write().midi_learn_target.take() {
                    let (min, max) = default_cc_range(&param_path);
                    let mut cfg_w = shared.midi_config.write();
                    cfg_w.cc_mappings.retain(|m| m.param_path != param_path);
                    cfg_w.cc_mappings.push(MidiCcMapping {
                        cc: *cc,
                        param_path,
                        min,
                        max,
                    });
                    cfg = cfg_w.clone();
                }
            }

            let norm = (*value as f32 / 127.0).clamp(0.0, 1.0);
            match *cc {
                64 => {
                    apply_pedal_value(&mut shared.pedals.write(), "sustain", norm);
                    let _ = cmd_tx.send(AudioCommand::SetPedal {
                        pedal: "sustain".to_string(),
                        value: norm,
                    });
                }
                66 => {
                    apply_pedal_value(&mut shared.pedals.write(), "sostenuto", norm);
                    let _ = cmd_tx.send(AudioCommand::SetPedal {
                        pedal: "sostenuto".to_string(),
                        value: norm,
                    });
                }
                67 => {
                    apply_pedal_value(&mut shared.pedals.write(), "soft", norm);
                    let _ = cmd_tx.send(AudioCommand::SetPedal {
                        pedal: "soft".to_string(),
                        value: norm,
                    });
                }
                69 => {
                    apply_pedal_value(&mut shared.pedals.write(), "harmonic", norm);
                    let _ = cmd_tx.send(AudioCommand::SetPedal {
                        pedal: "harmonic".to_string(),
                        value: norm,
                    });
                }
                120 | 123 => {
                    let _ = cmd_tx.send(AudioCommand::AllNotesOff);
                }
                _ => {
                    let mapped = evaluate_cc_mapping(*cc, *value, &cfg.cc_mappings);
                    if !mapped.is_empty() {
                        let mut params = shared.params.write();
                        let mut changed = false;
                        for (path, val) in mapped {
                            if apply_cc_param(&mut params, &path, val) {
                                changed = true;
                            }
                        }
                        if changed {
                            let _ = cmd_tx.send(AudioCommand::UpdateParams(Box::new(params.clone())));
                        }
                    }
                }
            }
        }
        MidiEngineEvent::PitchBend { value_14bit, .. } => {
            let norm = (*value_14bit as f32 - 8192.0) / 8192.0;
            let mut params = shared.params.write();
            params.tuning.pitch_bend_semitones = norm * 2.0;
            let _ = cmd_tx.send(AudioCommand::UpdateParams(Box::new(params.clone())));
        }
        _ => {}
    }
}

fn apply_pedal_value(pedals: &mut PedalState, pedal: &str, value: f32) {
    let clamped = if value.is_finite() { value.clamp(0.0, 1.0) } else { 0.0 };
    match pedal {
        "sustain" => pedals.sustain = clamped,
        "soft" => pedals.soft = clamped,
        "sostenuto" => pedals.sostenuto = clamped,
        "harmonic" => pedals.harmonic = clamped,
        "celeste" => pedals.celeste = clamped,
        "buff_stop" => pedals.buff_stop = clamped,
        "rattle" => pedals.rattle = clamped,
        "super_sostenuto" => pedals.super_sostenuto = clamped >= 0.5,
        "reversed_sustain" => pedals.reversed_sustain = clamped >= 0.5,
        _ => {}
    }
}

fn process_audio_command(
    cmd: AudioCommand,
    synth: &mut PianoSynthEngine,
    arranger: &mut MusicStudioArrangerEngine,
    params: &mut EngineParams,
    pedals: &mut PedalState,
    inst: &mut InstrumentModel,
    shared: &Arc<SharedAudioState>,
) {
    let mut full_summary_dirty = false;
    let mut dynamic_summary_dirty = false;
    match cmd {
        AudioCommand::NoteOn { note, velocity } => {
            if arranger.has_loaded_package() {
                arranger.note_on(note, velocity);
                dynamic_summary_dirty = true;
            } else {
                synth.note_on(note, velocity, params, inst, pedals);
            }
        }
        AudioCommand::NoteOff { note } => {
            if arranger.has_loaded_package() {
                arranger.note_off(note, pedals.sustain >= 0.5 || pedals.reversed_sustain);
            } else {
                synth.note_off(note, params, pedals);
            }
        }
        AudioCommand::AllNotesOff => {
            synth.all_notes_off();
            arranger.all_notes_off();
        }
        AudioCommand::SetPedal { pedal, value } => {
            let prev = pedals.clone();
            apply_pedal_value(pedals, &pedal, value);
            if arranger.has_loaded_package() {
                if pedals.sustain < 0.5 && !pedals.reversed_sustain {
                    arranger.release_unsustained();
                }
            } else {
                synth.update_pedals(pedals, &prev, params);
            }
        }
        AudioCommand::UpdateParams(new_p) => {
            *params = *new_p;
            arranger.set_master_gain_db(params.volume_db);
        }
        AudioCommand::UpdateInstrument(new_i) => *inst = *new_i,
        AudioCommand::LoadMusicPackage(pkg) => {
            arranger.load_package(pkg);
            arranger.set_master_gain_db(params.volume_db);
            full_summary_dirty = true;
        }
        AudioCommand::ImportInstrument {
            instrument,
            assign_upper1,
        } => {
            arranger.add_imported_instrument(*instrument, assign_upper1);
            arranger.set_master_gain_db(params.volume_db);
            full_summary_dirty = true;
        }
        AudioCommand::SetArrangerSection {
            section_name,
            immediate,
        } => {
            arranger.set_section(&section_name, immediate);
            dynamic_summary_dirty = true;
        }
        AudioCommand::SetArrangerTransport {
            playing,
            sync_start,
            bpm,
            auto_fill,
            tempo_locked,
        } => {
            if let Some(af) = auto_fill {
                arranger.set_auto_fill(af);
            }
            if let Some(tl) = tempo_locked {
                arranger.set_tempo_locked(tl);
            }
            arranger.set_transport(playing, sync_start, bpm);
            dynamic_summary_dirty = true;
        }
        AudioCommand::SetArrangerStyle(idx) => {
            arranger.set_style(idx);
            full_summary_dirty = true;
        }
        AudioCommand::SetArrangerVoice {
            slot,
            instrument_index,
            split_key,
            split_enabled,
            chord_scan_lower,
            chord_scan_upper,
            upper_octave,
            transpose,
            chord_mode,
        } => {
            arranger.set_keyboard_voice(
                &slot,
                instrument_index,
                split_key,
                split_enabled,
                chord_scan_lower,
                chord_scan_upper,
                upper_octave,
                transpose,
                chord_mode.as_deref(),
            );
            dynamic_summary_dirty = true;
        }
        AudioCommand::SetArrangerChord { root_pc, quality } => {
            arranger.set_chord(root_pc, &quality);
            dynamic_summary_dirty = true;
        }
        AudioCommand::SetArrangerTrackVolume { track_slot, volume } => {
            arranger.set_track_volume(track_slot, volume);
            dynamic_summary_dirty = true;
        }
        AudioCommand::SetArrangerTrackInstrument {
            track_slot,
            instrument_index,
        } => {
            arranger.set_track_instrument(track_slot, instrument_index);
            dynamic_summary_dirty = true;
        }
        AudioCommand::ApplyArrangerOts(ots_idx) => {
            arranger.apply_ots(ots_idx);
            dynamic_summary_dirty = true;
        }
    }
    if full_summary_dirty {
        if let Some(mut s) = shared.arranger_summary.try_write() {
            *s = arranger.summary();
        }
    } else if dynamic_summary_dirty {
        if let Some(mut s) = shared.arranger_summary.try_write() {
            arranger.sync_dynamic_summary(&mut s);
        }
    }
}

impl AudioEngineManager {
    pub fn start() -> Self {
        let instruments = build_factory_instruments();
        let mut presets = build_factory_presets(&instruments);
        presets.extend(load_user_presets());

        let first_inst = instruments[0].clone();
        let first_preset_id = presets[0].id.clone();
        let initial_params = presets[0].params.clone();

        let default_sf2 = find_default_soundfont();
        let (initial_pkg, initial_summary) = if !cfg!(test) {
            let doc_pref = std::path::Path::new(USER_DOCUMENTS_RHYTHMS_DIR)
                .join("Josiran_Pisadinha_GR.Music");
            let wine_pref =
                std::path::Path::new(DEFAULT_MUSIC_STUDIO_DIR).join("Josiran_Pisadinha_GR.Music");
            let target_path = if doc_pref.exists() {
                Some(doc_pref)
            } else if wine_pref.exists() {
                Some(wine_pref)
            } else {
                list_all_music_packages()
                    .into_iter()
                    .next()
                    .map(|e| PathBuf::from(e.file_path))
            };
            if let Some(p) = target_path {
                let parsed_res = if is_sty_path(&p) {
                    build_sty_package(&[p], default_sf2.as_deref())
                } else {
                    parse_music_file(&p)
                };
                if let Ok(parsed) = parsed_res {
                    let pkg_arc = Arc::new(parsed);
                    let mut tmp = MusicStudioArrangerEngine::new(48000.0);
                    tmp.load_package(Arc::clone(&pkg_arc));
                    let summary = tmp.summary();
                    (Some(pkg_arc), summary)
                } else {
                    (None, MusicPackageSummary::default())
                }
            } else {
                (None, MusicPackageSummary::default())
            }
        } else {
            (None, MusicPackageSummary::default())
        };

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
                buffer_size: 1024,
                latency_ms: 21.33,
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
                midi_transport: MidiTransportState::default(),
                midi_learn_target: None,
            }),
            active_music_package: RwLock::new(initial_pkg),
            arranger_summary: RwLock::new(initial_summary),
            active_soundfont_path: RwLock::new(default_sf2),
            user_imported_instruments: RwLock::new(Vec::new()),
        });

        let keep_alive = Arc::new(AtomicBool::new(true));
        let recorded_events = Arc::new(RwLock::new(Vec::new()));
        let record_start = Arc::new(Mutex::new(None));

        // MIDI event bridge thread -> dispatches into AudioCommand & SharedAudioState
        let midi_shared = Arc::clone(&shared);
        let midi_cmd_tx = cmd_tx.clone();
        let midi_rec_events = Arc::clone(&recorded_events);
        let midi_rec_start = Arc::clone(&record_start);
        std::thread::spawn(move || {
            Self::midi_router_loop(midi_rx, midi_shared, midi_cmd_tx, midi_rec_events, midi_rec_start);
        });

        let (reconfigure_tx, reconfigure_rx) = mpsc::channel::<u32>();
        let requested_buffer_size = Arc::new(AtomicU32::new(1024));

        // Start real-time cpal audio output or fallback headless DSP thread on an audio supervisor thread
        let stream_shared = Arc::clone(&shared);
        let stream_keep_alive = Arc::clone(&keep_alive);
        let stream_req_buf = Arc::clone(&requested_buffer_size);
        std::thread::spawn(move || {
            Self::audio_supervisor_loop(
                stream_shared,
                cmd_rx,
                reconfigure_rx,
                stream_req_buf,
                stream_keep_alive,
            );
        });

        let mgr = Self {
            shared,
            cmd_tx: Mutex::new(cmd_tx),
            reconfigure_tx: Mutex::new(reconfigure_tx),
            requested_buffer_size,
            midi_handle: Mutex::new(None),
            midi_tx: Mutex::new(midi_tx),
            recorded_events,
            record_start,
            playback_cancel: Mutex::new(None),
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
        recorded_events: Arc<RwLock<Vec<(u64, MidiEngineEvent)>>>,
        record_start: Arc<Mutex<Option<Instant>>>,
    ) {
        while let Ok(ev) = midi_rx.recv() {
            let is_recording = shared.telemetry.read().midi_transport.is_recording;
            if is_recording
                && matches!(
                    ev,
                    MidiEngineEvent::NoteOn { .. }
                        | MidiEngineEvent::NoteOff { .. }
                        | MidiEngineEvent::ControlChange { .. }
                )
            {
                let ts_ms = {
                    let mut guard = record_start.lock();
                    let start = guard.get_or_insert_with(Instant::now);
                    start.elapsed().as_millis() as u64
                };
                let count = {
                    let mut list = recorded_events.write();
                    list.push((ts_ms, ev.clone()));
                    list.len()
                };
                shared.telemetry.write().midi_transport.recorded_events_count = count;
            }

            dispatch_midi_event_to_audio(&ev, &shared, &cmd_tx, true);
        }
    }

    fn make_runtime(
        shared: &Arc<SharedAudioState>,
        cmd_rx: Receiver<AudioCommand>,
        sr: f32,
    ) -> AudioDspRuntime {
        let synth = PianoSynthEngine::new(sr);
        let mut arranger = MusicStudioArrangerEngine::new(sr);
        if let Some(pkg) = shared.active_music_package.read().clone() {
            arranger.load_package(pkg);
        }
        let fx = EffectsChain::new(sr);
        let params = shared.params.read().clone();
        arranger.set_master_gain_db(params.volume_db);
        let pedals = shared.pedals.read().clone();
        let inst = shared.active_instrument.read().clone();
        AudioDspRuntime {
            synth,
            arranger,
            fx,
            params,
            pedals,
            inst,
            cmd_rx,
            buf_l: vec![0.0_f32; 2048],
            buf_r: vec![0.0_f32; 2048],
        }
    }

    fn audio_supervisor_loop(
        shared: Arc<SharedAudioState>,
        cmd_rx: Receiver<AudioCommand>,
        reconfigure_rx: Receiver<u32>,
        requested_buffer_size: Arc<AtomicU32>,
        keep_alive: Arc<AtomicBool>,
    ) {
        let host = cpal::default_host();
        if let Some(device) = host.default_output_device() {
            if let Ok(default_cfg) = device.default_output_config() {
                let sr = default_cfg.sample_rate().0;
                let runtime_slot = Arc::new(Mutex::new(Some(Self::make_runtime(
                    &shared,
                    cmd_rx,
                    sr as f32,
                ))));
                let target = requested_buffer_size.load(Ordering::Relaxed);
                let mut active_stream = Self::build_cpal_stream(
                    &device,
                    &default_cfg,
                    Arc::clone(&shared),
                    target,
                    Arc::clone(&runtime_slot),
                )
                .ok()
                .filter(|s| s.play().is_ok());

                if active_stream.is_none() {
                    let rt = runtime_slot.lock().take();
                    Self::spawn_headless_fallback(shared, rt, requested_buffer_size, keep_alive);
                    return;
                }

                while keep_alive.load(Ordering::Relaxed) {
                    match reconfigure_rx.recv_timeout(std::time::Duration::from_millis(200)) {
                        Ok(mut next_frames) => {
                            while let Ok(newer) = reconfigure_rx.try_recv() {
                                next_frames = newer;
                            }
                            drop(active_stream.take());
                            let rebuilt = Self::build_cpal_stream(
                                &device,
                                &default_cfg,
                                Arc::clone(&shared),
                                next_frames,
                                Arc::clone(&runtime_slot),
                            )
                            .ok()
                            .filter(|s| s.play().is_ok());
                            if rebuilt.is_some() {
                                active_stream = rebuilt;
                            } else if let Some(rt) = runtime_slot.lock().take() {
                                Self::spawn_headless_fallback(
                                    shared,
                                    Some(rt),
                                    requested_buffer_size,
                                    keep_alive,
                                );
                                return;
                            }
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
                drop(active_stream);
                return;
            }
        }

        let rt = Self::make_runtime(&shared, cmd_rx, 48000.0);
        Self::spawn_headless_fallback(shared, Some(rt), requested_buffer_size, keep_alive);
    }

    fn spawn_headless_fallback(
        shared: Arc<SharedAudioState>,
        mut runtime_opt: Option<AudioDspRuntime>,
        requested_buffer_size: Arc<AtomicU32>,
        keep_alive: Arc<AtomicBool>,
    ) {
        {
            let mut t = shared.telemetry.write();
            t.audio_backend = "Headless DSP Fallback (48kHz)".to_string();
            t.audio_active = true;
        }
        std::thread::spawn(move || {
            let sr = 48000.0_f32;
            let mut dummy_synth;
            let mut dummy_arranger;
            let mut dummy_fx;
            let mut dummy_params;
            let mut dummy_pedals;
            let mut dummy_inst;
            let mut dummy_l = vec![0.0_f32; 2048];
            let mut dummy_r = vec![0.0_f32; 2048];

            while keep_alive.load(Ordering::Relaxed) {
                let target_frames =
                    (requested_buffer_size.load(Ordering::Relaxed) as usize).clamp(64, 2048);
                let sleep_ms = ((target_frames as f32 / sr) * 1000.0).clamp(2.0, 20.0) as u64;
                std::thread::sleep(std::time::Duration::from_millis(sleep_ms));

                if let Some(ref mut rt) = runtime_opt {
                    if rt.buf_l.len() < target_frames {
                        rt.buf_l.resize(target_frames, 0.0);
                        rt.buf_r.resize(target_frames, 0.0);
                    }
                    Self::render_dsp_slice(
                        target_frames,
                        &mut rt.buf_l,
                        &mut rt.buf_r,
                        &mut rt.synth,
                        &mut rt.arranger,
                        &mut rt.fx,
                        &mut rt.params,
                        &mut rt.pedals,
                        &mut rt.inst,
                        &shared,
                        Some(&rt.cmd_rx),
                        sr,
                    );
                } else {
                    dummy_synth = PianoSynthEngine::new(sr);
                    dummy_arranger = MusicStudioArrangerEngine::new(sr);
                    dummy_fx = EffectsChain::new(sr);
                    dummy_params = shared.params.read().clone();
                    dummy_pedals = shared.pedals.read().clone();
                    dummy_inst = shared.active_instrument.read().clone();
                    Self::render_dsp_slice(
                        target_frames,
                        &mut dummy_l,
                        &mut dummy_r,
                        &mut dummy_synth,
                        &mut dummy_arranger,
                        &mut dummy_fx,
                        &mut dummy_params,
                        &mut dummy_pedals,
                        &mut dummy_inst,
                        &shared,
                        None,
                        sr,
                    );
                }
            }
        });
    }

    fn build_cpal_stream(
        device: &Device,
        default_cfg: &cpal::SupportedStreamConfig,
        shared: Arc<SharedAudioState>,
        target_buffer_frames: u32,
        runtime_slot: Arc<Mutex<Option<AudioDspRuntime>>>,
    ) -> Result<Stream, String> {
        let sr = default_cfg.sample_rate().0;
        let channels = default_cfg.channels().max(2) as usize;
        let dev_name = device.name().unwrap_or_else(|_| "Linux Audio".to_string());

        let ring_frames = target_buffer_frames.saturating_mul(2).clamp(256, 4096);
        let telemetry_interval_frames = (sr as usize / 25).max(512);

        for buf_mode in [
            cpal::BufferSize::Fixed(ring_frames),
            cpal::BufferSize::Default,
        ] {
            let Some(rt) = runtime_slot.lock().take() else {
                return Err("Audio DSP runtime unavailable".to_string());
            };
            let config = StreamConfig {
                channels: channels as u16,
                sample_rate: cpal::SampleRate(sr),
                buffer_size: buf_mode,
            };

            let mut guard = StreamRuntimeGuard {
                runtime: Some(rt),
                return_slot: Arc::clone(&runtime_slot),
            };
            let cb_shared = Arc::clone(&shared);
            let mut samples_since_telemetry: usize = 0;

            let attempt = device.build_output_stream(
                &config,
                move |output: &mut [f32], _| {
                    let Some(rt) = guard.runtime.as_mut() else {
                        output.fill(0.0);
                        return;
                    };
                    let start = Instant::now();
                    let frames = output.len() / channels;
                    if frames > rt.buf_l.len() {
                        rt.buf_l.resize(frames, 0.0);
                        rt.buf_r.resize(frames, 0.0);
                    }

                    while let Ok(cmd) = rt.cmd_rx.try_recv() {
                        process_audio_command(
                            cmd,
                            &mut rt.synth,
                            &mut rt.arranger,
                            &mut rt.params,
                            &mut rt.pedals,
                            &mut rt.inst,
                            &cb_shared,
                        );
                    }

                    let slice_l = &mut rt.buf_l[..frames];
                    let slice_r = &mut rt.buf_r[..frames];
                    if rt.arranger.has_loaded_package() {
                        rt.arranger.process_block(slice_l, slice_r);
                    } else {
                        rt.synth.process_block(slice_l, slice_r, &rt.params, &rt.pedals);
                        rt.fx.process_block(slice_l, slice_r, &rt.params);
                    }

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

                    samples_since_telemetry += frames;
                    if samples_since_telemetry >= telemetry_interval_frames {
                        samples_since_telemetry = 0;
                        let elapsed_us = start.elapsed().as_secs_f32() * 1_000_000.0;
                        let budget_us = (frames as f32 / sr as f32) * 1_000_000.0;
                        let cpu = ((elapsed_us / budget_us.max(1.0)) * 100.0).clamp(0.0, 100.0);
                        if let Some(mut t) = cb_shared.telemetry.try_write() {
                            t.buffer_size = frames;
                            t.latency_ms = (frames as f32 / sr as f32) * 1000.0;
                            if rt.arranger.has_loaded_package() {
                                t.active_voices = rt.arranger.active_voice_count();
                                rt.arranger.fill_active_notes(&mut t.active_notes);
                            } else {
                                t.active_voices = rt.synth.active_voice_count();
                                t.active_notes = rt.synth.active_notes();
                            }
                            t.cpu_load_pct = cpu;
                            t.peak_l = peak_l;
                            t.peak_r = peak_r;
                            t.rms_l = (sq_l / frames.max(1) as f32).sqrt();
                            t.rms_r = (sq_r / frames.max(1) as f32).sqrt();
                            t.pedals = rt.pedals.clone();
                        }
                        if rt.arranger.rhythm_playing {
                            if let Some(mut s) = cb_shared.arranger_summary.try_write() {
                                rt.arranger.sync_dynamic_summary(&mut s);
                            }
                        }
                    }
                },
                |_err| {},
                None,
            );

            if let Ok(stream) = attempt {
                let mut t = shared.telemetry.write();
                t.sample_rate = sr;
                t.buffer_size = target_buffer_frames.clamp(64, 2048) as usize;
                t.latency_ms = (t.buffer_size as f32 / sr as f32) * 1000.0;
                t.audio_backend = format!("PipeWire/ALSA ({})", dev_name);
                t.audio_active = true;
                return Ok(stream);
            }
        }

        Err("Failed to build cpal output stream".to_string())
    }

    #[allow(clippy::too_many_arguments)]
    fn render_dsp_slice(
        frames: usize,
        buf_l: &mut [f32],
        buf_r: &mut [f32],
        synth: &mut PianoSynthEngine,
        arranger: &mut MusicStudioArrangerEngine,
        fx: &mut EffectsChain,
        params: &mut EngineParams,
        pedals: &mut PedalState,
        inst: &mut InstrumentModel,
        shared: &Arc<SharedAudioState>,
        cmd_rx: Option<&Receiver<AudioCommand>>,
        sr: f32,
    ) {
        let count = frames.min(buf_l.len()).min(buf_r.len());
        if let Some(rx) = cmd_rx {
            while let Ok(cmd) = rx.try_recv() {
                process_audio_command(cmd, synth, arranger, params, pedals, inst, shared);
            }
        } else {
            let prev_pedals = pedals.clone();
            *params = shared.params.read().clone();
            *pedals = shared.pedals.read().clone();
            *inst = shared.active_instrument.read().clone();
            synth.update_pedals(pedals, &prev_pedals, params);
        }

        let slice_l = &mut buf_l[..count];
        let slice_r = &mut buf_r[..count];
        if arranger.has_loaded_package() {
            arranger.process_block(slice_l, slice_r);
        } else {
            synth.process_block(slice_l, slice_r, params, pedals);
            fx.process_block(slice_l, slice_r, params);
        }

        let mut peak_l = 0.0_f32;
        let mut peak_r = 0.0_f32;
        let mut sq_l = 0.0_f32;
        let mut sq_r = 0.0_f32;
        for f in 0..count {
            let l = slice_l[f];
            let r = slice_r[f];
            peak_l = peak_l.max(l.abs());
            peak_r = peak_r.max(r.abs());
            sq_l += l * l;
            sq_r += r * r;
        }

        if let Some(mut t) = shared.telemetry.try_write() {
            t.sample_rate = sr as u32;
            t.buffer_size = count;
            t.latency_ms = (count as f32 / sr.max(1.0)) * 1000.0;
            if arranger.has_loaded_package() {
                t.active_voices = arranger.active_voice_count();
                t.active_notes = arranger.active_notes();
            } else {
                t.active_voices = synth.active_voice_count();
                t.active_notes = synth.active_notes();
            }
            t.peak_l = peak_l;
            t.peak_r = peak_r;
            t.rms_l = (sq_l / count.max(1) as f32).sqrt();
            t.rms_r = (sq_r / count.max(1) as f32).sqrt();
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
            apply_pedal_value(&mut p, pedal, value);
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

    fn start_midi_playback(
        &self,
        track_name: &str,
        mut events: Vec<(u64, MidiEngineEvent)>,
    ) -> MidiTransportState {
        if let Some(prev_cancel) = self.playback_cancel.lock().take() {
            prev_cancel.store(true, Ordering::Relaxed);
        }
        let _ = self.cmd_tx.lock().send(AudioCommand::AllNotesOff);

        if events.is_empty() {
            let mut t = self.shared.telemetry.write();
            t.midi_transport.is_playing = false;
            t.midi_transport.current_track_name = None;
            return t.midi_transport.clone();
        }

        events.sort_by_key(|(ts, _)| *ts);
        let cancel = Arc::new(AtomicBool::new(false));
        *self.playback_cancel.lock() = Some(Arc::clone(&cancel));

        let state_snapshot = {
            let mut t = self.shared.telemetry.write();
            t.midi_transport.is_playing = true;
            t.midi_transport.current_track_name = Some(track_name.to_string());
            t.midi_transport.clone()
        };

        let shared = Arc::clone(&self.shared);
        let cmd_tx = self.cmd_tx.lock().clone();
        let keep_alive = Arc::clone(&self._keep_alive);

        std::thread::spawn(move || {
            let mut prev_ts = 0_u64;
            for (ts_ms, ev) in events {
                let delta_ms = ts_ms.saturating_sub(prev_ts);
                prev_ts = ts_ms;

                if delta_ms > 0 {
                    let mut remaining_track_ms = delta_ms as f64;
                    while remaining_track_ms > 0.25 {
                        if cancel.load(Ordering::Relaxed) || !keep_alive.load(Ordering::Relaxed) {
                            let _ = cmd_tx.send(AudioCommand::AllNotesOff);
                            return;
                        }
                        let speed = shared
                            .telemetry
                            .read()
                            .midi_transport
                            .playback_speed
                            .clamp(0.25, 2.0) as f64;
                        let real_wait_ms = (remaining_track_ms / speed).min(10.0);
                        std::thread::sleep(std::time::Duration::from_secs_f64(real_wait_ms * 0.001));
                        remaining_track_ms -= real_wait_ms * speed;
                    }
                }

                if cancel.load(Ordering::Relaxed) || !keep_alive.load(Ordering::Relaxed) {
                    let _ = cmd_tx.send(AudioCommand::AllNotesOff);
                    return;
                }
                dispatch_midi_event_to_audio(&ev, &shared, &cmd_tx, false);
            }

            if !cancel.load(Ordering::Relaxed) && keep_alive.load(Ordering::Relaxed) {
                let mut t = shared.telemetry.write();
                t.midi_transport.is_playing = false;
                t.midi_transport.current_track_name = None;
            }
        });

        state_snapshot
    }

    pub fn play_demo_midi(&self) -> MidiTransportState {
        let events = generate_demo_midi_sequence();
        self.start_midi_playback("Demo Prelude", events)
    }

    pub fn stop_midi_transport(&self) -> MidiTransportState {
        if let Some(cancel) = self.playback_cancel.lock().take() {
            cancel.store(true, Ordering::Relaxed);
        }
        *self.record_start.lock() = None;
        let _ = self.cmd_tx.lock().send(AudioCommand::AllNotesOff);
        let mut t = self.shared.telemetry.write();
        t.midi_transport.is_playing = false;
        t.midi_transport.is_recording = false;
        t.midi_transport.current_track_name = None;
        t.midi_transport.clone()
    }

    pub fn toggle_midi_record(&self) -> MidiTransportState {
        let mut t = self.shared.telemetry.write();
        let next_rec = !t.midi_transport.is_recording;
        t.midi_transport.is_recording = next_rec;
        if next_rec {
            self.recorded_events.write().clear();
            *self.record_start.lock() = Some(Instant::now());
            t.midi_transport.recorded_events_count = 0;
        } else {
            *self.record_start.lock() = None;
        }
        t.midi_transport.clone()
    }

    pub fn play_recorded_midi(&self) -> MidiTransportState {
        let events = self.recorded_events.read().clone();
        self.start_midi_playback("Recorded Take", events)
    }

    pub fn set_midi_playback_speed(&self, speed: f32) -> MidiTransportState {
        let mut t = self.shared.telemetry.write();
        t.midi_transport.playback_speed = if speed.is_finite() {
            speed.clamp(0.25, 2.0)
        } else {
            1.0
        };
        t.midi_transport.clone()
    }

    pub fn arm_midi_cc_learn(&self, param_path: Option<String>) -> Option<String> {
        let mut t = self.shared.telemetry.write();
        t.midi_learn_target = param_path.clone();
        param_path
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

    pub fn list_music_packages(&self) -> Vec<MusicPackageEntry> {
        list_all_music_packages()
    }

    pub fn arranger_state(&self) -> MusicPackageSummary {
        self.shared.arranger_summary.read().clone()
    }

    fn commit_loaded_package(&self, parsed: LoadedMusicPackage) -> MusicPackageSummary {
        let mut tmp = MusicStudioArrangerEngine::new(48000.0);
        tmp.user_imported_instruments = self.shared.user_imported_instruments.read().clone();
        if let Some(old_pkg) = self.shared.active_music_package.read().clone() {
            tmp.package = Some(old_pkg);
            tmp.upper1_instrument = self.shared.arranger_summary.read().upper1_instrument;
        }
        tmp.load_package(Arc::new(parsed));
        let pkg_arc = tmp
            .package
            .clone()
            .expect("tmp.load_package always sets package");
        let summary = tmp.summary();
        *self.shared.active_music_package.write() = Some(Arc::clone(&pkg_arc));
        *self.shared.arranger_summary.write() = summary.clone();
        let _ = self.cmd_tx.lock().send(AudioCommand::LoadMusicPackage(pkg_arc));
        summary
    }

    pub fn load_music_package(&self, path: &PathBuf) -> Result<MusicPackageSummary, String> {
        if is_sty_path(path) {
            return self.load_sty_files(std::slice::from_ref(path));
        }
        let parsed = parse_music_file(path)?;
        Ok(self.commit_loaded_package(parsed))
    }

    pub fn load_sty_files(&self, paths: &[PathBuf]) -> Result<MusicPackageSummary, String> {
        let sf2 = {
            let existing = self.shared.active_soundfont_path.read().clone();
            if existing.is_some() {
                existing
            } else {
                let found = find_default_soundfont();
                if found.is_some() {
                    *self.shared.active_soundfont_path.write() = found.clone();
                }
                found
            }
        };
        let parsed = build_sty_package(paths, sf2.as_deref())?;
        Ok(self.commit_loaded_package(parsed))
    }

    pub fn load_soundfont_file(&self, path: &PathBuf) -> Result<MusicPackageSummary, String> {
        let bank = crate::soundfont::SoundFontBank::open(path)?;
        let bank_name = bank.name().to_string();
        drop(bank);
        *self.shared.active_soundfont_path.write() = Some(path.clone());

        let base_pkg = self
            .shared
            .active_music_package
            .read()
            .clone()
            .map(|p| (*p).clone())
            .unwrap_or_else(|| LoadedMusicPackage {
                file_name: bank_name,
                file_path: path.to_string_lossy().to_string(),
                version: "SF2".to_string(),
                source_format: "music".to_string(),
                soundfont_name: None,
                instruments: Vec::new(),
                styles: Vec::new(),
                ots_presets: Vec::new(),
                source_paths: Vec::new(),
            });
        let rebuilt = rebuild_package_with_soundfont(&base_pkg, path)?;
        Ok(self.commit_loaded_package(rebuilt))
    }

    pub fn import_instrument_file(
        &self,
        path: &PathBuf,
        assign_upper1: bool,
    ) -> Result<MusicPackageSummary, String> {
        let next_idx = self
            .shared
            .active_music_package
            .read()
            .as_ref()
            .map(|p| p.instruments.len())
            .unwrap_or(0);
        let parsed = parse_instrument_file(path, next_idx)?;

        {
            let mut user_list = self.shared.user_imported_instruments.write();
            if let Some(existing) = user_list.iter_mut().find(|u| u.name == parsed.name) {
                *existing = parsed.clone();
            } else {
                user_list.push(parsed.clone());
            }
        }

        {
            let mut pkg_guard = self.shared.active_music_package.write();
            let mut pkg = match pkg_guard.as_ref() {
                Some(existing) => (**existing).clone(),
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
                },
            };
            let mut inst = parsed.clone();
            let (actual_idx, replaced) = if let Some(existing_idx) =
                pkg.instruments.iter().position(|i| i.name == inst.name)
            {
                inst.index = existing_idx;
                pkg.instruments[existing_idx] = inst.clone();
                (existing_idx, true)
            } else {
                let idx = pkg.instruments.len();
                inst.index = idx;
                pkg.instruments.push(inst.clone());
                (idx, false)
            };
            let inst_summary = crate::music_studio::MusicInstrumentSummary {
                index: actual_idx,
                name: inst.name.clone(),
                zone_count: inst.zones.len(),
                has_samples: inst.zones.iter().any(|z| !z.pcm_l.is_empty()),
                is_drumkit: inst.is_drumkit,
            };
            *pkg_guard = Some(Arc::new(pkg));

            let mut sum = self.shared.arranger_summary.write();
            if !sum.loaded {
                sum.loaded = true;
                sum.file_name = "Custom Instruments".to_string();
                sum.file_path = "custom://instruments".to_string();
                sum.version = "1.4".to_string();
                sum.source_format = "music".to_string();
            }
            if replaced && actual_idx < sum.instruments.len() {
                sum.instruments[actual_idx] = inst_summary;
            } else {
                sum.instruments.push(inst_summary);
            }
            if assign_upper1 {
                sum.upper1_instrument = actual_idx;
            }
        }

        let _ = self.cmd_tx.lock().send(AudioCommand::ImportInstrument {
            instrument: Box::new(parsed),
            assign_upper1,
        });
        Ok(self.arranger_state())
    }

    pub fn set_arranger_section(&self, section_name: &str, immediate: bool) -> MusicPackageSummary {
        {
            let mut s = self.shared.arranger_summary.write();
            if !s.rhythm_playing || immediate {
                s.active_section = section_name.to_string();
                s.pending_section = None;
            } else {
                s.pending_section = Some(section_name.to_string());
            }
        }
        let _ = self.cmd_tx.lock().send(AudioCommand::SetArrangerSection {
            section_name: section_name.to_string(),
            immediate,
        });
        self.arranger_state()
    }

    pub fn set_audio_buffer_size(&self, buffer_size: u32) -> Result<EngineTelemetry, String> {
        if !matches!(buffer_size, 128 | 256 | 512 | 1024) {
            return Err(format!(
                "Tamanho de buffer inválido: {} (use 128, 256, 512 ou 1024)",
                buffer_size
            ));
        }
        self.requested_buffer_size.store(buffer_size, Ordering::Relaxed);
        {
            let mut t = self.shared.telemetry.write();
            let sr = t.sample_rate.max(8000) as f32;
            t.buffer_size = buffer_size as usize;
            t.latency_ms = (buffer_size as f32 / sr) * 1000.0;
        }
        let _ = self.reconfigure_tx.lock().send(buffer_size);
        Ok(self.shared.telemetry.read().clone())
    }

    pub fn set_arranger_transport(
        &self,
        playing: bool,
        sync_start: Option<bool>,
        bpm: Option<f32>,
        auto_fill: Option<bool>,
        tempo_locked: Option<bool>,
    ) -> MusicPackageSummary {
        {
            let mut s = self.shared.arranger_summary.write();
            s.rhythm_playing = playing;
            if let Some(ss) = sync_start {
                s.sync_start = ss;
            }
            if let Some(b) = bpm {
                if b.is_finite() {
                    s.bpm = b.clamp(40.0, 240.0);
                }
            }
            if let Some(af) = auto_fill {
                s.auto_fill = af;
            }
            if let Some(tl) = tempo_locked {
                s.tempo_locked = tl;
            }
        }
        let _ = self.cmd_tx.lock().send(AudioCommand::SetArrangerTransport {
            playing,
            sync_start,
            bpm,
            auto_fill,
            tempo_locked,
        });
        self.arranger_state()
    }

    pub fn set_arranger_style(&self, style_index: usize) -> MusicPackageSummary {
        {
            let (pkg_bpm, pkg_ots) = self
                .shared
                .active_music_package
                .read()
                .as_ref()
                .and_then(|pkg| pkg.styles.get(style_index))
                .map(|st| (Some(st.bpm), (!st.ots_presets.is_empty()).then(|| st.ots_presets.clone())))
                .unwrap_or((None, None));
            let mut s = self.shared.arranger_summary.write();
            let next_sec = s.styles.get(style_index).and_then(|st| {
                st.sections
                    .iter()
                    .find(|x| *x == "Variation 1")
                    .or_else(|| st.sections.first())
                    .cloned()
            });
            if style_index < s.styles.len() {
                s.active_style_index = style_index;
                if !s.tempo_locked {
                    if let Some(b) = pkg_bpm {
                        s.bpm = b;
                    }
                }
                if let Some(first_sec) = next_sec {
                    s.active_section = first_sec;
                }
                if let Some(ots) = pkg_ots {
                    s.ots_presets = ots;
                }
                s.pending_section = None;
            }
        }
        let _ = self.cmd_tx.lock().send(AudioCommand::SetArrangerStyle(style_index));
        self.arranger_state()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_arranger_voice(
        &self,
        slot: &str,
        instrument_index: Option<usize>,
        split_key: Option<u8>,
        split_enabled: Option<bool>,
        chord_scan_lower: Option<bool>,
        chord_scan_upper: Option<bool>,
        upper_octave: Option<i8>,
        transpose: Option<i8>,
        chord_mode: Option<&str>,
    ) -> MusicPackageSummary {
        {
            let mut s = self.shared.arranger_summary.write();
            if let Some(sk) = split_key {
                s.split_key = sk.clamp(24, 96);
            }
            if let Some(se) = split_enabled {
                s.split_enabled = se;
            }
            if let Some(csl) = chord_scan_lower {
                s.chord_scan_lower = csl;
            }
            if let Some(csu) = chord_scan_upper {
                s.chord_scan_upper = csu;
            }
            if let Some(uo) = upper_octave {
                s.upper_octave = uo.clamp(-2, 2);
            }
            if let Some(tr) = transpose {
                s.transpose = tr.clamp(-12, 12);
            }
            if let Some(cm) = chord_mode {
                let normalized = crate::music_studio::ChordMode::from_str(cm).as_str();
                s.chord_mode.clear();
                s.chord_mode.push_str(normalized);
            }
            let max_idx = s.instruments.len();
            let valid = instrument_index.filter(|&i| i < max_idx);
            match slot {
                "upper1" => {
                    if let Some(i) = valid {
                        s.upper1_instrument = i;
                    }
                }
                "upper2" => s.upper2_instrument = valid,
                "lower" => s.lower_instrument = valid,
                _ => {}
            }
        }
        let _ = self.cmd_tx.lock().send(AudioCommand::SetArrangerVoice {
            slot: slot.to_string(),
            instrument_index,
            split_key,
            split_enabled,
            chord_scan_lower,
            chord_scan_upper,
            upper_octave,
            transpose,
            chord_mode: chord_mode.map(str::to_string),
        });
        self.arranger_state()
    }

    pub fn set_arranger_chord(&self, root_pc: u8, quality: &str) -> MusicPackageSummary {
        {
            let q = crate::music_studio::ChordQuality::from_str(quality);
            let mut s = self.shared.arranger_summary.write();
            s.detected_chord_root = root_pc % 12;
            s.detected_chord_quality = q.as_str().to_string();
            s.detected_chord = crate::music_studio::format_chord_label(root_pc % 12, q);
        }
        let _ = self.cmd_tx.lock().send(AudioCommand::SetArrangerChord {
            root_pc,
            quality: quality.to_string(),
        });
        self.arranger_state()
    }

    pub fn set_arranger_track_volume(&self, track_slot: usize, volume: f32) -> MusicPackageSummary {
        {
            let mut s = self.shared.arranger_summary.write();
            if track_slot < 8 && volume.is_finite() {
                s.track_volumes[track_slot] = volume.clamp(0.0, 1.5);
            }
        }
        let _ = self.cmd_tx.lock().send(AudioCommand::SetArrangerTrackVolume {
            track_slot,
            volume,
        });
        self.arranger_state()
    }

    pub fn set_arranger_track_instrument(
        &self,
        track_index: usize,
        instrument_index: Option<usize>,
    ) -> MusicPackageSummary {
        {
            let mut s = self.shared.arranger_summary.write();
            if track_index < 8 {
                let max_idx = s.instruments.len();
                s.track_instruments[track_index] = instrument_index.filter(|&i| i < max_idx);
            }
        }
        let _ = self
            .cmd_tx
            .lock()
            .send(AudioCommand::SetArrangerTrackInstrument {
                track_slot: track_index,
                instrument_index,
            });
        self.arranger_state()
    }

    pub fn apply_arranger_ots(&self, ots_index: usize) -> MusicPackageSummary {
        {
            let mut s = self.shared.arranger_summary.write();
            let max_idx = s.instruments.len();
            if let Some(ots) = s
                .ots_presets
                .iter()
                .find(|p| p.index == ots_index)
                .or_else(|| s.ots_presets.get(ots_index))
                .cloned()
            {
                if ots.upper1_instrument < max_idx {
                    s.upper1_instrument = ots.upper1_instrument;
                }
                s.upper2_instrument = ots.upper2_instrument.filter(|&i| i < max_idx);
                s.lower_instrument = ots.lower_instrument.filter(|&i| i < max_idx);
            }
        }
        let _ = self.cmd_tx.lock().send(AudioCommand::ApplyArrangerOts(ots_index));
        self.arranger_state()
    }

    /// Renders a short C-major arpeggio / chord offline to a stereo 32-bit float WAV file.
    pub fn export_demo_wav(&self, path: PathBuf) -> Result<String, String> {
        let sr = 48000_u32;
        let mut synth = PianoSynthEngine::new(sr as f32);
        let mut arranger = MusicStudioArrangerEngine::new(sr as f32);
        if let Some(pkg) = self.shared.active_music_package.read().clone() {
            arranger.load_package(pkg);
            let s = self.shared.arranger_summary.read().clone();
            arranger.set_style(s.active_style_index);
            arranger.set_section(&s.active_section, true);
            arranger.set_keyboard_voice(
                "upper1",
                Some(s.upper1_instrument),
                Some(s.split_key),
                Some(s.split_enabled),
                Some(s.chord_scan_lower),
                Some(s.chord_scan_upper),
                Some(s.upper_octave),
                Some(s.transpose),
                Some(&s.chord_mode),
            );
            arranger.set_keyboard_voice("upper2", s.upper2_instrument, None, None, None, None, None, None, None);
            arranger.set_keyboard_voice("lower", s.lower_instrument, None, None, None, None, None, None, None);
            arranger.set_chord(s.detected_chord_root, &s.detected_chord_quality);
            for (slot, &vol) in s.track_volumes.iter().enumerate() {
                arranger.set_track_volume(slot, vol);
            }
            for (slot, &inst_opt) in s.track_instruments.iter().enumerate() {
                arranger.set_track_instrument(slot, inst_opt);
            }
            arranger.set_tempo_locked(s.tempo_locked);
            arranger.set_transport(true, Some(s.sync_start), Some(s.bpm));
        }
        let mut fx = EffectsChain::new(sr as f32);
        let params = self.shared.params.read().clone();
        arranger.set_master_gain_db(params.volume_db);
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
                if arranger.has_loaded_package() {
                    arranger.note_on(notes[next_note_idx], 96);
                } else {
                    synth.note_on(notes[next_note_idx], 86, &params, &inst, &pedals);
                }
                next_note_idx += 1;
            }
            let frames = block.min(total_samples - pos);
            if arranger.has_loaded_package() {
                arranger.process_block(&mut buf_l[..frames], &mut buf_r[..frames]);
            } else {
                synth.process_block(&mut buf_l[..frames], &mut buf_r[..frames], &params, &pedals);
                fx.process_block(&mut buf_l[..frames], &mut buf_r[..frames], &params);
            }
            for i in 0..frames {
                writer.write_sample(buf_l[i]).map_err(|e| e.to_string())?;
                writer.write_sample(buf_r[i]).map_err(|e| e.to_string())?;
            }
            pos += frames;
        }
        writer.finalize().map_err(|e| e.to_string())?;
        Ok(path.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::TemperamentId;

    fn make_test_shared() -> Arc<SharedAudioState> {
        let instruments = build_factory_instruments();
        let presets = build_factory_presets(&instruments);
        let first_inst = instruments[0].clone();
        let first_preset_id = presets[0].id.clone();
        let initial_params = presets[0].params.clone();

        Arc::new(SharedAudioState {
            params: RwLock::new(initial_params),
            pedals: RwLock::new(PedalState::default()),
            midi_config: RwLock::new(MidiConfig::default()),
            active_instrument: RwLock::new(first_inst),
            active_preset_id: RwLock::new(first_preset_id),
            preset_b_id: RwLock::new(presets.get(1).map(|p| p.id.clone())),
            instruments: RwLock::new(instruments),
            presets: RwLock::new(presets),
            telemetry: RwLock::new(EngineTelemetry {
                sample_rate: 48000,
                buffer_size: 1024,
                latency_ms: 21.33,
                active_voices: 0,
                max_polyphony: MAX_POLYPHONY,
                cpu_load_pct: 0.0,
                peak_l: 0.0,
                peak_r: 0.0,
                rms_l: 0.0,
                rms_r: 0.0,
                audio_backend: "Test".to_string(),
                audio_active: true,
                midi_connected: false,
                midi_port_name: None,
                last_midi_note: None,
                last_midi_velocity: None,
                active_notes: Vec::new(),
                pedals: PedalState::default(),
                midi_transport: MidiTransportState::default(),
                midi_learn_target: None,
            }),
            active_music_package: RwLock::new(None),
            arranger_summary: RwLock::new(MusicPackageSummary::default()),
            active_soundfont_path: RwLock::new(None),
            user_imported_instruments: RwLock::new(Vec::new()),
        })
    }

    fn verify_wav_file(path: &PathBuf) {
        let meta = std::fs::metadata(path).expect("WAV file should exist");
        assert!(
            meta.len() > 44,
            "WAV file size {} must exceed 44-byte RIFF header",
            meta.len()
        );

        let mut reader = hound::WavReader::open(path).expect("hound must parse exported WAV");
        let spec = reader.spec();
        assert_eq!(spec.channels, 2);
        assert_eq!(spec.sample_rate, 48000);
        assert_eq!(spec.bits_per_sample, 32);
        assert_eq!(spec.sample_format, hound::SampleFormat::Float);

        let samples: Vec<f32> = reader
            .samples::<f32>()
            .map(|r| r.expect("valid f32 WAV sample"))
            .collect();
        assert_eq!(samples.len(), 48000 * 3 * 2);

        let mut sq_sum = 0.0_f64;
        for (idx, &s) in samples.iter().enumerate() {
            assert!(s.is_finite(), "non-finite sample at {}", idx);
            assert!(s.abs() <= 1.0, "sample {} exceeded 1.0 at {}", s, idx);
            sq_sum += (s as f64) * (s as f64);
        }
        let rms = (sq_sum / samples.len() as f64).sqrt();
        assert!(rms > 0.005, "exported WAV appears silent (rms={})", rms);
    }

    #[test]
    fn test_offline_wav_export_via_hound() {
        let mgr = AudioEngineManager::start();
        let path = std::env::temp_dir().join(format!(
            "piano-tect-export-test-{}-default.wav",
            std::process::id()
        ));
        let res = mgr.export_demo_wav(path.clone());
        assert!(res.is_ok(), "export_demo_wav failed: {:?}", res.err());
        verify_wav_file(&path);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_offline_wav_export_with_selected_instrument() {
        let mgr = AudioEngineManager::start();
        let snap = mgr
            .select_instrument("ruckers-harpsichord")
            .expect("select harpsichord");
        assert_eq!(snap.active_instrument_id, "ruckers-harpsichord");
        assert!((snap.params.tuning.diapason_hz - 415.0).abs() < 1e-3);
        assert_eq!(snap.params.tuning.temperament, TemperamentId::WerckmeisterIII);

        let path = std::env::temp_dir().join(format!(
            "piano-tect-export-test-{}-harpsichord.wav",
            std::process::id()
        ));
        let res = mgr.export_demo_wav(path.clone());
        assert!(res.is_ok(), "harpsichord WAV export failed: {:?}", res.err());
        verify_wav_file(&path);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_audio_manager_pedals_and_preset_morph() {
        let mgr = AudioEngineManager::start();
        let snap = mgr.snapshot();
        assert!(!snap.instruments.is_empty());
        assert!(snap.presets.len() >= 2);

        mgr.set_pedal("sustain", 1.0);
        mgr.set_pedal("harmonic", 0.8);
        mgr.set_pedal("rattle", 0.45);
        mgr.set_pedal("soft", 0.6);
        mgr.set_pedal("sostenuto", 0.9);
        mgr.set_pedal("super_sostenuto", 1.0);
        mgr.set_pedal("reversed_sustain", 1.0);
        mgr.set_pedal("celeste", f32::NAN);

        {
            let p = mgr.shared.pedals.read();
            assert!((p.sustain - 1.0).abs() < 1e-6);
            assert!((p.harmonic - 0.8).abs() < 1e-6);
            assert!((p.rattle - 0.45).abs() < 1e-6);
            assert!((p.soft - 0.6).abs() < 1e-6);
            assert!((p.sostenuto - 0.9).abs() < 1e-6);
            assert!(p.super_sostenuto);
            assert!(p.reversed_sustain);
            assert_eq!(p.celeste, 0.0);
        }

        let p_a = snap.presets[0].id.clone();
        let p_b = snap.presets[1].id.clone();
        let morphed = mgr.morph_ab(&p_a, &p_b, 0.5).expect("morph_ab");
        assert!((morphed.morphing - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_render_dsp_slice_64_voice_polyphony_and_dynamic_buffers() {
        let shared = make_test_shared();
        let sr = 48000.0_f32;
        let mut synth = PianoSynthEngine::new(sr);
        let mut arranger = MusicStudioArrangerEngine::new(sr);
        let mut fx = EffectsChain::new(sr);
        let mut params = shared.params.read().clone();
        params.volume_db = 0.0;
        let mut pedals = PedalState::default();
        let mut inst = shared.active_instrument.read().clone();

        let (tx, rx) = mpsc::channel::<AudioCommand>();
        tx.send(AudioCommand::SetPedal {
            pedal: "sustain".to_string(),
            value: 1.0,
        })
        .unwrap();
        tx.send(AudioCommand::SetPedal {
            pedal: "rattle".to_string(),
            value: 0.65,
        })
        .unwrap();
        for note in 24_u8..88_u8 {
            tx.send(AudioCommand::NoteOn {
                note,
                velocity: 127,
            })
            .unwrap();
        }

        let mut buf_l = vec![0.0_f32; 2048];
        let mut buf_r = vec![0.0_f32; 2048];

        for &frames in &[64_usize, 127, 256, 513, 1024] {
            AudioEngineManager::render_dsp_slice(
                frames,
                &mut buf_l,
                &mut buf_r,
                &mut synth,
                &mut arranger,
                &mut fx,
                &mut params,
                &mut pedals,
                &mut inst,
                &shared,
                Some(&rx),
                sr,
            );
            for i in 0..frames {
                assert!(buf_l[i].is_finite() && buf_l[i].abs() <= 1.0);
                assert!(buf_r[i].is_finite() && buf_r[i].abs() <= 1.0);
            }
        }

        {
            let t = shared.telemetry.read();
            assert_eq!(t.active_voices, 64);
            assert_eq!(t.active_notes.len(), 64);
            assert!(t.rms_l > 0.01 && t.rms_r > 0.01);
            assert!(t.peak_l <= 1.0 && t.peak_r <= 1.0);
        }

        tx.send(AudioCommand::AllNotesOff).unwrap();
        AudioEngineManager::render_dsp_slice(
            256,
            &mut buf_l,
            &mut buf_r,
            &mut synth,
            &mut arranger,
            &mut fx,
            &mut params,
            &mut pedals,
            &mut inst,
            &shared,
            Some(&rx),
            sr,
        );
        assert_eq!(shared.telemetry.read().active_voices, 0);
    }

    #[test]
    fn test_midi_transport_recorder_and_cc_learn() {
        let mgr = AudioEngineManager::start();

        // 1. Arm MIDI CC Learn and learn CC 74 -> "reverb.mix"
        let armed = mgr.arm_midi_cc_learn(Some("reverb.mix".to_string()));
        assert_eq!(armed.as_deref(), Some("reverb.mix"));
        {
            let midi_tx = mgr.midi_tx.lock().clone();
            midi_tx
                .send(MidiEngineEvent::ControlChange {
                    cc: 74,
                    value: 127,
                    channel: 1,
                })
                .unwrap();
        }
        for _ in 0..40 {
            if mgr.shared.telemetry.read().midi_learn_target.is_none() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(mgr.shared.telemetry.read().midi_learn_target.is_none());
        assert!(
            mgr.shared
                .midi_config
                .read()
                .cc_mappings
                .iter()
                .any(|m| m.cc == 74 && m.param_path == "reverb.mix"),
            "CC 74 should be bound to reverb.mix"
        );
        assert!((mgr.shared.params.read().reverb.mix - 1.0).abs() < 1e-4);

        // 2. MIDI Recording via toggle_midi_record
        let rec_on = mgr.toggle_midi_record();
        assert!(rec_on.is_recording);
        assert_eq!(rec_on.recorded_events_count, 0);
        {
            let midi_tx = mgr.midi_tx.lock().clone();
            midi_tx
                .send(MidiEngineEvent::NoteOn {
                    note: 60,
                    velocity: 100,
                    channel: 1,
                })
                .unwrap();
            midi_tx
                .send(MidiEngineEvent::ControlChange {
                    cc: 64,
                    value: 127,
                    channel: 1,
                })
                .unwrap();
            midi_tx
                .send(MidiEngineEvent::NoteOff {
                    note: 60,
                    channel: 1,
                })
                .unwrap();
        }
        for _ in 0..40 {
            if mgr.shared.telemetry.read().midi_transport.recorded_events_count == 3 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(
            mgr.shared.telemetry.read().midi_transport.recorded_events_count,
            3
        );
        let rec_off = mgr.toggle_midi_record();
        assert!(!rec_off.is_recording);

        // 3. Recorded take playback & stop
        let play_rec = mgr.play_recorded_midi();
        assert!(play_rec.is_playing);
        assert_eq!(play_rec.current_track_name.as_deref(), Some("Recorded Take"));
        let stopped_rec = mgr.stop_midi_transport();
        assert!(!stopped_rec.is_playing);

        // 4. Playback speed clamping & Demo MIDI transport start/stop
        assert!((mgr.set_midi_playback_speed(5.0).playback_speed - 2.0).abs() < 1e-6);
        assert!((mgr.set_midi_playback_speed(0.1).playback_speed - 0.25).abs() < 1e-6);
        assert!((mgr.set_midi_playback_speed(1.5).playback_speed - 1.5).abs() < 1e-6);

        let demo = mgr.play_demo_midi();
        assert!(demo.is_playing);
        assert_eq!(demo.current_track_name.as_deref(), Some("Demo Prelude"));

        // Wait briefly for initial demo events (CC 64 + NoteOn 36 at 20ms / 1.5)
        for _ in 0..30 {
            if mgr.shared.telemetry.read().last_midi_note == Some(36) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(mgr.shared.telemetry.read().last_midi_note, Some(36));

        let stopped = mgr.stop_midi_transport();
        assert!(!stopped.is_playing);
        assert!(!stopped.is_recording);
        assert!(stopped.current_track_name.is_none());
    }

    #[test]
    fn test_set_audio_buffer_size() {
        let mgr = AudioEngineManager::start();
        assert!(mgr.set_audio_buffer_size(99).is_err());

        for &sz in &[128_u32, 256, 512, 1024] {
            let t = mgr.set_audio_buffer_size(sz).expect("valid buffer size");
            assert_eq!(t.buffer_size, sz as usize);
            let expected_ms = (sz as f32 / t.sample_rate.max(8000) as f32) * 1000.0;
            assert!((t.latency_ms - expected_ms).abs() < 0.05);
        }
    }
}
