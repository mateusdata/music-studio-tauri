export type TemperamentId =
  | "Equal"
  | "Pythagorean"
  | "Zarlino"
  | "MeanTone"
  | "WellTempered"
  | "WerckmeisterIII"
  | "KirnbergerIII"
  | "Vallotti";

export type InstrumentCategory =
  | "GrandPiano"
  | "UprightPiano"
  | "ElectricPiano"
  | "Historical"
  | "Mallet"
  | "Plucked"
  | "Prepared"
  | "Synthesizer";

export interface PedalState {
  sustain: number;
  soft: number;
  sostenuto: number;
  harmonic: number;
  celeste: number;
  buff_stop: number;
  rattle: number;
  super_sostenuto: boolean;
  reversed_sustain: boolean;
}

export interface TuningParams {
  diapason_hz: number;
  temperament: TemperamentId;
  unison_width_cents: number;
  unison_balance: number;
  octave_stretching: number;
  direct_sound_duration: number;
  pitch_bend_semitones: number;
}

export interface VoicingParams {
  hammer_hardness_piano: number;
  hammer_hardness_mezzo: number;
  hammer_hardness_forte: number;
  spectrum_profile: number[];
  hammer_noise: number;
  hammer_tone: number;
  strike_point: number;
  soft_level: number;
  pickup_symmetry: number;
  pickup_distance: number;
}

export interface DesignParams {
  soundboard_impedance: number;
  impedance_cutoff: number;
  impedance_slope: number;
  string_length_m: number;
  sympathetic_resonance: number;
  duplex_scale_resonance: number;
  blooming_energy: number;
  blooming_inertia: number;
  quadratic_effect: number;
}

export interface ActionParams {
  damper_position: number;
  damper_noise: number;
  pedal_noise: number;
  key_release_noise: number;
  damping_duration: number;
  mute: number;
  keyboard_range_extended: boolean;
}

export interface MicConfig {
  enabled: boolean;
  name: string;
  x: number;
  y: number;
  z: number;
  azimuth: number;
  elevation: number;
  volume: number;
  pan: number;
}

export interface OutputStageParams {
  lid_position: number;
  stereo_width: number;
  mic_level_compensation: boolean;
  mic_delay_compensation: boolean;
  binaural_mode: boolean;
  head_x: number;
  head_y: number;
  head_z: number;
  head_diameter: number;
  head_angle: number;
  mics: MicConfig[];
}

export interface EqBand {
  freq_hz: number;
  gain_db: number;
  q: number;
}

export type EffectType =
  | "Delay"
  | "Chorus"
  | "Flanger"
  | "Phaser"
  | "Tremolo"
  | "Wah"
  | "Fuzz"
  | "Amp"
  | "Compressor"
  | "ParametricEq";

export interface EffectSlot {
  enabled: boolean;
  effect_type: EffectType;
  mix: number;
  param_1: number;
  param_2: number;
  param_3: number;
  param_4: number;
}

export interface ReverbParams {
  enabled: boolean;
  mix: number;
  duration_s: number;
  room_size_m: number;
  pre_delay_ms: number;
  early_reflections: number;
  tone: number;
  highpass_hz: number;
  preset_name: string;
}

export interface LimiterParams {
  enabled: boolean;
  threshold_db: number;
  sharpness: number;
  gain_db: number;
}

export interface NoteEditOffset {
  detune_cents: number;
  gain_db: number;
  hammer_hardness_offset: number;
  impedance_offset: number;
}

export interface BounceParams {
  enabled: boolean;
  delay_ms: number;
  sync_tempo: boolean;
  velocity_sensitivity: number;
  delay_loss: number;
  velocity_loss: number;
  humanization: number;
  max_bounces: number;
}

export interface NoteFxParams {
  lfo_shape: number;
  lfo_skew: number;
  lfo_rate_hz: number;
  lfo_phase_locked: boolean;
  onset_duration_ms: number;
  vibrato_cents: number;
  vibrato_offset_cents: number;
  tremolo_depth_db: number;
  tremolo_phase: number;
  attack_envelope: number;
  smear: number;
}

export type PlayMode = "Standard" | "DiatonicHarp" | "Guitar";

export interface PlayModeParams {
  mode: PlayMode;
  harp_pedals: number[];
  guitar_fret: number;
  guitar_legato: boolean;
  guitar_body: number;
}

export interface MidiTransportState {
  is_playing: boolean;
  is_recording: boolean;
  playback_speed: number;
  recorded_events_count: number;
  current_track_name: string | null;
}

export interface EngineParams {
  volume_db: number;
  dynamics_db: number;
  velocity_offset: number;
  condition: number;
  post_effect_gain_db: number;
  morphing: number;
  tuning: TuningParams;
  voicing: VoicingParams;
  design: DesignParams;
  action: ActionParams;
  output: OutputStageParams;
  eq_enabled: boolean;
  eq_bands: EqBand[];
  fx: EffectSlot[];
  reverb: ReverbParams;
  limiter: LimiterParams;
  note_edit: Record<number, NoteEditOffset>;
  bounce?: BounceParams;
  note_fx?: NoteFxParams;
  play_mode?: PlayModeParams;
}

export interface InstrumentModel {
  id: string;
  name: string;
  category: InstrumentCategory;
  exciter: string;
  description: string;
  inharmonicity_base: number;
  decay_scale: number;
  default_params: EngineParams;
}

export interface PresetData {
  id: string;
  name: string;
  instrument_id: string;
  category: string;
  description: string;
  params: EngineParams;
  is_factory: boolean;
}

export interface MidiPortInfo {
  index: number;
  name: string;
}

export interface MidiConfig {
  selected_port: string | null;
  channel: number;
  microtonal_multichannel: boolean;
  transpose: number;
  key_range_mode: "Full88" | "Compact61" | "Extended105";
  velocity_curve: [number, number][];
  cc_mappings: { cc: number; param_path: string; min: number; max: number }[];
}

export interface EngineTelemetry {
  sample_rate: number;
  buffer_size: number;
  latency_ms: number;
  active_voices: number;
  max_polyphony: number;
  cpu_load_pct: number;
  peak_l: number;
  peak_r: number;
  rms_l: number;
  rms_r: number;
  audio_backend: string;
  audio_active: boolean;
  midi_connected: boolean;
  midi_port_name: string | null;
  last_midi_note: number | null;
  last_midi_velocity: number | null;
  active_notes: number[];
  pedals: PedalState;
  midi_transport?: MidiTransportState;
  midi_learn_target?: string | null;
}

export interface EngineSnapshot {
  active_instrument_id: string;
  active_preset_id: string;
  preset_b_id: string | null;
  params: EngineParams;
  midi_config: MidiConfig;
  instruments: InstrumentModel[];
  presets: PresetData[];
}

export interface MusicPackageEntry {
  file_name: string;
  file_path: string;
  size_mb: number;
  version: string;
  format?: "music" | "sty" | string;
}

export interface MusicInstrumentSummary {
  index: number;
  name: string;
  zone_count: number;
  has_samples: boolean;
  is_drumkit: boolean;
}

export interface MusicStyleSummary {
  id: number;
  name: string;
  category: string;
  sections: string[];
}

export interface OtsPresetSummary {
  index: number;
  name: string;
  upper1_instrument: number;
  upper2_instrument: number | null;
  lower_instrument: number | null;
}

export interface MusicPackageSummary {
  loaded: boolean;
  file_name: string;
  file_path: string;
  version: string;
  source_format?: "music" | "sty" | "sty_pack" | string;
  soundfont_name?: string | null;
  instruments: MusicInstrumentSummary[];
  styles: MusicStyleSummary[];
  active_style_index: number;
  active_section: string;
  pending_section: string | null;
  upper1_instrument: number;
  upper2_instrument: number | null;
  lower_instrument: number | null;
  split_key: number;
  split_enabled: boolean;
  chord_scan_lower: boolean;
  chord_scan_upper: boolean;
  upper_octave: number;
  transpose: number;
  bpm: number;
  tempo_locked: boolean;
  rhythm_playing: boolean;
  sync_start: boolean;
  auto_fill: boolean;
  detected_chord_root: number;
  detected_chord_quality: string;
  detected_chord: string;
  chord_mode?: "fingered_3_7" | "single_finger" | string;
  track_volumes: number[];
  track_instruments?: (number | null)[];
  ots_presets?: OtsPresetSummary[];
  beat_position: number;
  sty_warnings?: string[];
  has_audio_style_chunks?: boolean;
  uses_expansion_kit?: boolean;
  sty_diagnostics_summary?: string | null;
}
