use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemperamentId {
    Equal,
    Pythagorean,
    Zarlino,
    MeanTone,
    WellTempered,
    WerckmeisterIII,
    KirnbergerIII,
    Vallotti,
}

impl Default for TemperamentId {
    fn default() -> Self {
        Self::Equal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstrumentCategory {
    GrandPiano,
    UprightPiano,
    ElectricPiano,
    Historical,
    Mallet,
    Plucked,
    Prepared,
    Synthesizer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExciterType {
    FeltHammer,
    TineHammer,
    ReedHammer,
    PluckPlectrum,
    MalletHard,
    MalletSoft,
    PreparedBolt,
    SynthHybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedalState {
    pub sustain: f32,
    pub soft: f32,
    pub sostenuto: f32,
    pub harmonic: f32,
    pub celeste: f32,
    pub buff_stop: f32,
    pub rattle: f32,
    pub super_sostenuto: bool,
    pub reversed_sustain: bool,
}

impl Default for PedalState {
    fn default() -> Self {
        Self {
            sustain: 0.0,
            soft: 0.0,
            sostenuto: 0.0,
            harmonic: 0.0,
            celeste: 0.0,
            buff_stop: 0.0,
            rattle: 0.0,
            super_sostenuto: false,
            reversed_sustain: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningParams {
    pub diapason_hz: f32,
    pub temperament: TemperamentId,
    pub unison_width_cents: f32,
    pub unison_balance: f32,
    pub octave_stretching: f32,
    pub direct_sound_duration: f32,
    pub pitch_bend_semitones: f32,
}

impl Default for TuningParams {
    fn default() -> Self {
        Self {
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.7,
            unison_balance: 0.5,
            octave_stretching: 1.0,
            direct_sound_duration: 0.5,
            pitch_bend_semitones: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicingParams {
    pub hammer_hardness_piano: f32,
    pub hammer_hardness_mezzo: f32,
    pub hammer_hardness_forte: f32,
    pub spectrum_profile: [f32; 8],
    pub hammer_noise: f32,
    pub hammer_tone: f32,
    pub strike_point: f32,
    pub soft_level: f32,
    pub pickup_symmetry: f32,
    pub pickup_distance: f32,
}

impl Default for VoicingParams {
    fn default() -> Self {
        Self {
            hammer_hardness_piano: 0.38,
            hammer_hardness_mezzo: 0.58,
            hammer_hardness_forte: 0.82,
            spectrum_profile: [1.0, 0.92, 0.84, 0.72, 0.60, 0.48, 0.34, 0.28],
            hammer_noise: 0.35,
            hammer_tone: 0.5,
            strike_point: 0.125,
            soft_level: 0.65,
            pickup_symmetry: 0.5,
            pickup_distance: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignParams {
    pub soundboard_impedance: f32,
    pub impedance_cutoff: f32,
    pub impedance_slope: f32,
    pub string_length_m: f32,
    pub sympathetic_resonance: f32,
    pub duplex_scale_resonance: f32,
    pub blooming_energy: f32,
    pub blooming_inertia: f32,
    pub quadratic_effect: f32,
}

impl Default for DesignParams {
    fn default() -> Self {
        Self {
            soundboard_impedance: 0.52,
            impedance_cutoff: 0.5,
            impedance_slope: 0.5,
            string_length_m: 2.74,
            sympathetic_resonance: 0.45,
            duplex_scale_resonance: 0.35,
            blooming_energy: 0.28,
            blooming_inertia: 0.5,
            quadratic_effect: 0.22,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionParams {
    pub damper_position: f32,
    pub damper_noise: f32,
    pub pedal_noise: f32,
    pub key_release_noise: f32,
    pub damping_duration: f32,
    pub mute: f32,
    pub keyboard_range_extended: bool,
}

impl Default for ActionParams {
    fn default() -> Self {
        Self {
            damper_position: 0.5,
            damper_noise: 0.28,
            pedal_noise: 0.25,
            key_release_noise: 0.22,
            damping_duration: 0.5,
            mute: 0.0,
            keyboard_range_extended: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicConfig {
    pub enabled: bool,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub azimuth: f32,
    pub elevation: f32,
    pub volume: f32,
    pub pan: f32,
}

impl Default for MicConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            name: "Condenser U87".to_string(),
            x: 0.45,
            y: 0.35,
            z: 0.65,
            azimuth: 0.0,
            elevation: -15.0,
            volume: 0.85,
            pan: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStageParams {
    pub lid_position: f32,
    pub stereo_width: f32,
    pub mic_level_compensation: bool,
    pub mic_delay_compensation: bool,
    pub binaural_mode: bool,
    pub head_x: f32,
    pub head_y: f32,
    pub head_z: f32,
    pub head_diameter: f32,
    pub head_angle: f32,
    pub mics: [MicConfig; 3],
}

impl Default for OutputStageParams {
    fn default() -> Self {
        let mut mic_l = MicConfig::default();
        mic_l.name = "Left Close".to_string();
        mic_l.x = 0.28;
        mic_l.y = 0.42;
        mic_l.pan = -0.58;

        let mut mic_r = MicConfig::default();
        mic_r.name = "Right Close".to_string();
        mic_r.x = 0.72;
        mic_r.y = 0.42;
        mic_r.pan = 0.58;

        let mut mic_room = MicConfig::default();
        mic_room.enabled = false;
        mic_room.name = "Room Ribbon".to_string();
        mic_room.x = 0.5;
        mic_room.y = 0.85;
        mic_room.z = 0.80;
        mic_room.volume = 0.55;
        mic_room.pan = 0.0;

        Self {
            lid_position: 1.0,
            stereo_width: 0.62,
            mic_level_compensation: true,
            mic_delay_compensation: false,
            binaural_mode: false,
            head_x: 0.5,
            head_y: 0.22,
            head_z: 0.65,
            head_diameter: 0.18,
            head_angle: 0.0,
            mics: [mic_l, mic_r, mic_room],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqBand {
    pub freq_hz: f32,
    pub gain_db: f32,
    pub q: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectType {
    Delay,
    ParametricEq,
    Chorus,
    Tremolo,
    Compressor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSlot {
    pub enabled: bool,
    pub effect_type: EffectType,
    pub mix: f32,
    pub param_1: f32,
    pub param_2: f32,
    pub param_3: f32,
    pub param_4: f32,
}

impl Default for EffectSlot {
    fn default() -> Self {
        Self {
            enabled: false,
            effect_type: EffectType::Delay,
            mix: 0.25,
            param_1: 0.35,
            param_2: 0.40,
            param_3: 0.60,
            param_4: 0.50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverbParams {
    pub enabled: bool,
    pub mix: f32,
    pub duration_s: f32,
    pub room_size_m: f32,
    pub pre_delay_ms: f32,
    pub early_reflections: f32,
    pub tone: f32,
    pub highpass_hz: f32,
    pub preset_name: String,
}

impl Default for ReverbParams {
    fn default() -> Self {
        Self {
            enabled: true,
            mix: 0.22,
            duration_s: 1.8,
            room_size_m: 18.0,
            pre_delay_ms: 14.0,
            early_reflections: 0.45,
            tone: 0.52,
            highpass_hz: 110.0,
            preset_name: "Concert Hall".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimiterParams {
    pub enabled: bool,
    pub threshold_db: f32,
    pub sharpness: f32,
    pub gain_db: f32,
}

impl Default for LimiterParams {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold_db: -0.8,
            sharpness: 0.65,
            gain_db: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoteEditOffset {
    pub detune_cents: f32,
    pub gain_db: f32,
    pub hammer_hardness_offset: f32,
    pub impedance_offset: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BounceParams {
    pub enabled: bool,
    pub delay_ms: f32,
    pub sync_tempo: bool,
    pub velocity_sensitivity: f32,
    pub delay_loss: f32,
    pub velocity_loss: f32,
    pub humanization: f32,
    pub max_bounces: u8,
}

impl Default for BounceParams {
    fn default() -> Self {
        Self {
            enabled: false,
            delay_ms: 120.0,
            sync_tempo: false,
            velocity_sensitivity: 0.5,
            delay_loss: 0.15,
            velocity_loss: 0.35,
            humanization: 0.1,
            max_bounces: 6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteFxParams {
    pub lfo_shape: f32,
    pub lfo_skew: f32,
    pub lfo_rate_hz: f32,
    pub lfo_phase_locked: bool,
    pub onset_duration_ms: f32,
    pub vibrato_cents: f32,
    pub vibrato_offset_cents: f32,
    pub tremolo_depth_db: f32,
    pub tremolo_phase: f32,
    pub attack_envelope: f32,
    pub smear: f32,
}

impl Default for NoteFxParams {
    fn default() -> Self {
        Self {
            lfo_shape: 0.5,
            lfo_skew: 0.5,
            lfo_rate_hz: 5.2,
            lfo_phase_locked: false,
            onset_duration_ms: 80.0,
            vibrato_cents: 0.0,
            vibrato_offset_cents: 0.0,
            tremolo_depth_db: 0.0,
            tremolo_phase: 0.0,
            attack_envelope: 0.5,
            smear: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayMode {
    Standard,
    DiatonicHarp,
    Guitar,
}

impl Default for PlayMode {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayModeParams {
    pub mode: PlayMode,
    /// 7 diatonic harp pedals [C, D, E, F, G, A, B] where -1 = flat, 0 = natural, +1 = sharp.
    pub harp_pedals: [i8; 7],
    pub guitar_fret: u8,
    pub guitar_legato: bool,
    pub guitar_body: f32,
}

impl Default for PlayModeParams {
    fn default() -> Self {
        Self {
            mode: PlayMode::Standard,
            harp_pedals: [0; 7],
            guitar_fret: 0,
            guitar_legato: false,
            guitar_body: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineParams {
    pub volume_db: f32,
    pub dynamics_db: f32,
    pub velocity_offset: f32,
    pub condition: f32,
    pub post_effect_gain_db: f32,
    pub morphing: f32,
    pub tuning: TuningParams,
    pub voicing: VoicingParams,
    pub design: DesignParams,
    pub action: ActionParams,
    pub output: OutputStageParams,
    pub eq_enabled: bool,
    pub eq_bands: [EqBand; 3],
    pub fx: [EffectSlot; 3],
    pub reverb: ReverbParams,
    pub limiter: LimiterParams,
    pub note_edit: HashMap<u8, NoteEditOffset>,
    #[serde(default)]
    pub bounce: BounceParams,
    #[serde(default)]
    pub note_fx: NoteFxParams,
    #[serde(default)]
    pub play_mode: PlayModeParams,
}

impl Default for EngineParams {
    fn default() -> Self {
        let mut fx_chorus = EffectSlot::default();
        fx_chorus.effect_type = EffectType::Chorus;
        fx_chorus.mix = 0.20;

        let mut fx_comp = EffectSlot::default();
        fx_comp.effect_type = EffectType::Compressor;
        fx_comp.mix = 0.50;

        Self {
            volume_db: -4.0,
            dynamics_db: 40.0,
            velocity_offset: 0.0,
            condition: 0.04,
            post_effect_gain_db: 0.0,
            morphing: 0.0,
            tuning: TuningParams::default(),
            voicing: VoicingParams::default(),
            design: DesignParams::default(),
            action: ActionParams::default(),
            output: OutputStageParams::default(),
            eq_enabled: true,
            eq_bands: [
                EqBand { freq_hz: 110.0, gain_db: 0.5, q: 0.8 },
                EqBand { freq_hz: 1100.0, gain_db: 0.0, q: 1.0 },
                EqBand { freq_hz: 5200.0, gain_db: 1.0, q: 0.9 },
            ],
            fx: [EffectSlot::default(), fx_chorus, fx_comp],
            reverb: ReverbParams::default(),
            limiter: LimiterParams::default(),
            note_edit: HashMap::new(),
            bounce: BounceParams::default(),
            note_fx: NoteFxParams::default(),
            play_mode: PlayModeParams::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentModel {
    pub id: String,
    pub name: String,
    pub category: InstrumentCategory,
    pub exciter: ExciterType,
    pub description: String,
    pub inharmonicity_base: f32,
    pub decay_scale: f32,
    pub default_params: EngineParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetData {
    pub id: String,
    pub name: String,
    pub instrument_id: String,
    pub category: String,
    pub description: String,
    pub params: EngineParams,
    pub is_factory: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyRangeMode {
    Full88,
    Compact61,
    Extended105,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiCcMapping {
    pub cc: u8,
    pub param_path: String,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiConfig {
    pub selected_port: Option<String>,
    pub channel: u8,
    pub microtonal_multichannel: bool,
    pub transpose: i8,
    pub key_range_mode: KeyRangeMode,
    pub velocity_curve: Vec<[f32; 2]>,
    pub cc_mappings: Vec<MidiCcMapping>,
}

impl Default for MidiConfig {
    fn default() -> Self {
        Self {
            selected_port: None,
            channel: 0,
            microtonal_multichannel: false,
            transpose: 0,
            key_range_mode: KeyRangeMode::Full88,
            velocity_curve: vec![
                [0.0, 0.0],
                [0.25, 0.22],
                [0.5, 0.5],
                [0.75, 0.78],
                [1.0, 1.0],
            ],
            cc_mappings: vec![
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
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiPortInfo {
    pub index: usize,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiTransportState {
    pub is_playing: bool,
    pub is_recording: bool,
    pub playback_speed: f32,
    pub recorded_events_count: usize,
    pub current_track_name: Option<String>,
}

impl Default for MidiTransportState {
    fn default() -> Self {
        Self {
            is_playing: false,
            is_recording: false,
            playback_speed: 1.0,
            recorded_events_count: 0,
            current_track_name: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineTelemetry {
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub latency_ms: f32,
    pub active_voices: usize,
    pub max_polyphony: usize,
    pub cpu_load_pct: f32,
    pub peak_l: f32,
    pub peak_r: f32,
    pub rms_l: f32,
    pub rms_r: f32,
    pub audio_backend: String,
    pub audio_active: bool,
    pub midi_connected: bool,
    pub midi_port_name: Option<String>,
    pub last_midi_note: Option<u8>,
    pub last_midi_velocity: Option<u8>,
    pub active_notes: Vec<u8>,
    pub pedals: PedalState,
    #[serde(default)]
    pub midi_transport: MidiTransportState,
    #[serde(default)]
    pub midi_learn_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSnapshot {
    pub active_instrument_id: String,
    pub active_preset_id: String,
    pub preset_b_id: Option<String>,
    pub params: EngineParams,
    pub midi_config: MidiConfig,
    pub instruments: Vec<InstrumentModel>,
    pub presets: Vec<PresetData>,
}
