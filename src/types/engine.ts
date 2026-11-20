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

export interface EffectSlot {
  enabled: boolean;
  effect_type: "Delay" | "ParametricEq" | "Chorus" | "Tremolo" | "Compressor";
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
