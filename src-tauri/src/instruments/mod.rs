use crate::contracts::{
    EffectSlot, EffectType, EngineParams, ExciterType, InstrumentCategory, InstrumentModel,
    TemperamentId,
};

struct ReverbSpec {
    preset_name: &'static str,
    mix: f32,
    duration_s: f32,
    room_size_m: f32,
    pre_delay_ms: f32,
    early_reflections: f32,
    tone: f32,
    highpass_hz: f32,
}

impl ReverbSpec {
    const fn new(
        preset_name: &'static str,
        mix: f32,
        duration_s: f32,
        room_size_m: f32,
        pre_delay_ms: f32,
        early_reflections: f32,
        tone: f32,
        highpass_hz: f32,
    ) -> Self {
        Self {
            preset_name,
            mix,
            duration_s,
            room_size_m,
            pre_delay_ms,
            early_reflections,
            tone,
            highpass_hz,
        }
    }
}

struct FxSpec {
    delay_enabled: bool,
    delay_mix: f32,
    delay_time: f32,
    delay_feedback: f32,
    mod_enabled: bool,
    mod_type: EffectType,
    mod_mix: f32,
    mod_rate: f32,
    comp_enabled: bool,
    comp_mix: f32,
    comp_thresh: f32,
    comp_ratio: f32,
}

impl FxSpec {
    const fn acoustic_clean() -> Self {
        Self {
            delay_enabled: false,
            delay_mix: 0.20,
            delay_time: 0.35,
            delay_feedback: 0.36,
            mod_enabled: false,
            mod_type: EffectType::Chorus,
            mod_mix: 0.18,
            mod_rate: 0.32,
            comp_enabled: false,
            comp_mix: 0.40,
            comp_thresh: 0.42,
            comp_ratio: 0.38,
        }
    }

    const fn with_chorus(mix: f32, rate: f32) -> Self {
        let mut s = Self::acoustic_clean();
        s.mod_enabled = true;
        s.mod_type = EffectType::Chorus;
        s.mod_mix = mix;
        s.mod_rate = rate;
        s
    }

    const fn with_tremolo(mix: f32, rate: f32) -> Self {
        let mut s = Self::acoustic_clean();
        s.mod_enabled = true;
        s.mod_type = EffectType::Tremolo;
        s.mod_mix = mix;
        s.mod_rate = rate;
        s
    }

    const fn with_compressor(mix: f32, thresh: f32, ratio: f32) -> Self {
        let mut s = Self::acoustic_clean();
        s.comp_enabled = true;
        s.comp_mix = mix;
        s.comp_thresh = thresh;
        s.comp_ratio = ratio;
        s
    }

    const fn with_delay(mix: f32, time: f32, feedback: f32) -> Self {
        let mut s = Self::acoustic_clean();
        s.delay_enabled = true;
        s.delay_mix = mix;
        s.delay_time = time;
        s.delay_feedback = feedback;
        s
    }

    const fn with_delay_and_chorus(
        delay_mix: f32,
        delay_time: f32,
        delay_fb: f32,
        chorus_mix: f32,
        chorus_rate: f32,
    ) -> Self {
        let mut s = Self::acoustic_clean();
        s.delay_enabled = true;
        s.delay_mix = delay_mix;
        s.delay_time = delay_time;
        s.delay_feedback = delay_fb;
        s.mod_enabled = true;
        s.mod_type = EffectType::Chorus;
        s.mod_mix = chorus_mix;
        s.mod_rate = chorus_rate;
        s
    }
}

struct AcousticSpec {
    id: &'static str,
    name: &'static str,
    category: InstrumentCategory,
    exciter: ExciterType,
    description: &'static str,
    inharmonicity_base: f32,
    decay_scale: f32,
    string_length_m: f32,
    soundboard_impedance: f32,
    sympathetic_resonance: f32,
    duplex_scale_resonance: f32,
    blooming_energy: f32,
    quadratic_effect: f32,
    hammer_hardness: [f32; 3],
    hammer_noise: f32,
    hammer_tone: f32,
    strike_point: f32,
    spectrum_profile: [f32; 8],
    diapason_hz: f32,
    temperament: TemperamentId,
    unison_width_cents: f32,
    octave_stretching: f32,
    dynamics_db: f32,
    condition: f32,
    damper_noise: f32,
    key_release_noise: f32,
    damping_duration: f32,
    keyboard_range_extended: bool,
    pickup_symmetry: f32,
    pickup_distance: f32,
    lid_position: f32,
    stereo_width: f32,
    eq_gains_db: [f32; 3],
    reverb: ReverbSpec,
    fx: FxSpec,
}

fn build_instrument(spec: AcousticSpec) -> InstrumentModel {
    let mut params = EngineParams::default();

    params.dynamics_db = spec.dynamics_db;
    params.condition = spec.condition;

    params.tuning.diapason_hz = spec.diapason_hz;
    params.tuning.temperament = spec.temperament;
    params.tuning.unison_width_cents = spec.unison_width_cents;
    params.tuning.octave_stretching = spec.octave_stretching;

    params.voicing.hammer_hardness_piano = spec.hammer_hardness[0];
    params.voicing.hammer_hardness_mezzo = spec.hammer_hardness[1];
    params.voicing.hammer_hardness_forte = spec.hammer_hardness[2];
    params.voicing.spectrum_profile = spec.spectrum_profile;
    params.voicing.hammer_noise = spec.hammer_noise;
    params.voicing.hammer_tone = spec.hammer_tone;
    params.voicing.strike_point = spec.strike_point;
    params.voicing.pickup_symmetry = spec.pickup_symmetry;
    params.voicing.pickup_distance = spec.pickup_distance;

    params.design.string_length_m = spec.string_length_m;
    params.design.soundboard_impedance = spec.soundboard_impedance;
    params.design.sympathetic_resonance = spec.sympathetic_resonance;
    params.design.duplex_scale_resonance = spec.duplex_scale_resonance;
    params.design.blooming_energy = spec.blooming_energy;
    params.design.quadratic_effect = spec.quadratic_effect;

    params.action.damper_noise = spec.damper_noise;
    params.action.key_release_noise = spec.key_release_noise;
    params.action.damping_duration = spec.damping_duration;
    params.action.keyboard_range_extended = spec.keyboard_range_extended;

    params.output.lid_position = spec.lid_position;
    params.output.stereo_width = spec.stereo_width;

    for (band, gain) in params.eq_bands.iter_mut().zip(spec.eq_gains_db.iter()) {
        band.gain_db = *gain;
    }

    params.fx = [
        EffectSlot {
            enabled: spec.fx.delay_enabled,
            effect_type: EffectType::Delay,
            mix: spec.fx.delay_mix,
            param_1: spec.fx.delay_time,
            param_2: spec.fx.delay_feedback,
            param_3: 0.60,
            param_4: 0.50,
        },
        EffectSlot {
            enabled: spec.fx.mod_enabled,
            effect_type: spec.fx.mod_type,
            mix: spec.fx.mod_mix,
            param_1: spec.fx.mod_rate,
            param_2: 0.45,
            param_3: 0.60,
            param_4: 0.50,
        },
        EffectSlot {
            enabled: spec.fx.comp_enabled,
            effect_type: EffectType::Compressor,
            mix: spec.fx.comp_mix,
            param_1: spec.fx.comp_thresh,
            param_2: spec.fx.comp_ratio,
            param_3: 0.55,
            param_4: 0.50,
        },
    ];

    params.reverb.enabled = true;
    params.reverb.preset_name = spec.reverb.preset_name.to_string();
    params.reverb.mix = spec.reverb.mix;
    params.reverb.duration_s = spec.reverb.duration_s;
    params.reverb.room_size_m = spec.reverb.room_size_m;
    params.reverb.pre_delay_ms = spec.reverb.pre_delay_ms;
    params.reverb.early_reflections = spec.reverb.early_reflections;
    params.reverb.tone = spec.reverb.tone;
    params.reverb.highpass_hz = spec.reverb.highpass_hz;

    InstrumentModel {
        id: spec.id.to_string(),
        name: spec.name.to_string(),
        category: spec.category,
        exciter: spec.exciter,
        description: spec.description.to_string(),
        inharmonicity_base: spec.inharmonicity_base,
        decay_scale: spec.decay_scale,
        default_params: params,
    }
}

/// Builds the factory library of acoustically calibrated instrument models
/// spanning all 8 instrument categories and Pianoteq-inspired acoustic families.
pub fn build_factory_instruments() -> Vec<InstrumentModel> {
    let specs = vec![
        // --- Grand Pianos (13) ---
        AcousticSpec {
            id: "grand-ny-d",
            name: "Grand Concert NY D",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "9-foot New York concert grand modeled with dynamic felt hammers, crisp attack, and rich soundboard resonance.",
            inharmonicity_base: 0.00018,
            decay_scale: 1.05,
            string_length_m: 2.74,
            soundboard_impedance: 0.52,
            sympathetic_resonance: 0.48,
            duplex_scale_resonance: 0.38,
            blooming_energy: 0.30,
            quadratic_effect: 0.24,
            hammer_hardness: [0.40, 0.62, 0.88],
            hammer_noise: 0.35,
            hammer_tone: 0.54,
            strike_point: 0.125,
            spectrum_profile: [1.00, 0.94, 0.86, 0.74, 0.62, 0.49, 0.36, 0.29],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.70,
            octave_stretching: 1.00,
            dynamics_db: 40.0,
            condition: 0.04,
            damper_noise: 0.28,
            key_release_noise: 0.22,
            damping_duration: 0.50,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.64,
            eq_gains_db: [0.5, 0.0, 1.0],
            reverb: ReverbSpec::new("NY Concert Stage", 0.22, 1.85, 20.0, 14.0, 0.46, 0.54, 105.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "grand-hb-d",
            name: "Grand Concert HB D",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Warm, singing European concert grand with velvety upper register and balanced sympathetic sustain.",
            inharmonicity_base: 0.00016,
            decay_scale: 1.12,
            string_length_m: 2.74,
            soundboard_impedance: 0.48,
            sympathetic_resonance: 0.52,
            duplex_scale_resonance: 0.42,
            blooming_energy: 0.34,
            quadratic_effect: 0.20,
            hammer_hardness: [0.34, 0.54, 0.78],
            hammer_noise: 0.30,
            hammer_tone: 0.46,
            strike_point: 0.122,
            spectrum_profile: [1.05, 0.96, 0.88, 0.75, 0.58, 0.44, 0.32, 0.25],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.60,
            octave_stretching: 1.02,
            dynamics_db: 42.0,
            condition: 0.03,
            damper_noise: 0.25,
            key_release_noise: 0.20,
            damping_duration: 0.52,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.68,
            eq_gains_db: [0.6, 0.2, 0.6],
            reverb: ReverbSpec::new("Hamburg Philharmonic", 0.24, 2.05, 22.0, 16.0, 0.48, 0.50, 95.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "grand-steinway-b",
            name: "Concert Grand Model B",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "6'11\" semi-concert music room grand with focused midrange clarity, intimate projection, and agile touch.",
            inharmonicity_base: 0.00024,
            decay_scale: 0.96,
            string_length_m: 2.11,
            soundboard_impedance: 0.55,
            sympathetic_resonance: 0.44,
            duplex_scale_resonance: 0.35,
            blooming_energy: 0.26,
            quadratic_effect: 0.22,
            hammer_hardness: [0.37, 0.58, 0.84],
            hammer_noise: 0.33,
            hammer_tone: 0.50,
            strike_point: 0.128,
            spectrum_profile: [1.02, 0.95, 0.84, 0.70, 0.55, 0.42, 0.30, 0.22],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.75,
            octave_stretching: 1.04,
            dynamics_db: 39.0,
            condition: 0.05,
            damper_noise: 0.30,
            key_release_noise: 0.24,
            damping_duration: 0.48,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 0.88,
            stereo_width: 0.58,
            eq_gains_db: [0.4, 0.3, 0.8],
            reverb: ReverbSpec::new("Chamber Recital", 0.20, 1.55, 14.5, 11.0, 0.52, 0.53, 115.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "grand-sk-ex",
            name: "Shigeru Concert Grand SK-EX",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Hand-voiced Japanese 9-foot flagship featuring deeply resonant bass, singing treble sustain, and refined dynamic shading.",
            inharmonicity_base: 0.00015,
            decay_scale: 1.15,
            string_length_m: 2.78,
            soundboard_impedance: 0.46,
            sympathetic_resonance: 0.54,
            duplex_scale_resonance: 0.44,
            blooming_energy: 0.32,
            quadratic_effect: 0.19,
            hammer_hardness: [0.32, 0.52, 0.80],
            hammer_noise: 0.28,
            hammer_tone: 0.48,
            strike_point: 0.118,
            spectrum_profile: [1.08, 0.98, 0.86, 0.72, 0.56, 0.43, 0.31, 0.24],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.55,
            octave_stretching: 1.01,
            dynamics_db: 44.0,
            condition: 0.02,
            damper_noise: 0.24,
            key_release_noise: 0.19,
            damping_duration: 0.54,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.66,
            eq_gains_db: [0.7, -0.1, 0.7],
            reverb: ReverbSpec::new("Tokyo Suntory Hall", 0.25, 2.15, 24.0, 18.0, 0.44, 0.51, 90.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "grand-yfx",
            name: "Concert Grand CFX",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Brilliant modern 9-foot concert grand built for orchestral projection, crisp transient precision, and shimmering treble.",
            inharmonicity_base: 0.00017,
            decay_scale: 1.08,
            string_length_m: 2.75,
            soundboard_impedance: 0.50,
            sympathetic_resonance: 0.47,
            duplex_scale_resonance: 0.45,
            blooming_energy: 0.29,
            quadratic_effect: 0.26,
            hammer_hardness: [0.42, 0.65, 0.92],
            hammer_noise: 0.32,
            hammer_tone: 0.62,
            strike_point: 0.116,
            spectrum_profile: [0.98, 0.94, 0.90, 0.80, 0.68, 0.56, 0.44, 0.35],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.60,
            octave_stretching: 1.03,
            dynamics_db: 42.0,
            condition: 0.03,
            damper_noise: 0.26,
            key_release_noise: 0.21,
            damping_duration: 0.49,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.65,
            eq_gains_db: [0.3, 0.0, 1.5],
            reverb: ReverbSpec::new("Brilliant Symphony Hall", 0.23, 1.95, 21.5, 15.0, 0.45, 0.60, 110.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "grand-bosendorfer-280vc",
            name: "Vienna Concert Grand 280VC",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Austrian spruce-rim concert grand with extended sub-bass keyboard range, velvety warmth, and deep blooming sustain.",
            inharmonicity_base: 0.00014,
            decay_scale: 1.18,
            string_length_m: 2.80,
            soundboard_impedance: 0.44,
            sympathetic_resonance: 0.58,
            duplex_scale_resonance: 0.30,
            blooming_energy: 0.38,
            quadratic_effect: 0.18,
            hammer_hardness: [0.35, 0.55, 0.79],
            hammer_noise: 0.31,
            hammer_tone: 0.44,
            strike_point: 0.130,
            spectrum_profile: [1.12, 1.02, 0.87, 0.71, 0.54, 0.40, 0.28, 0.21],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.65,
            octave_stretching: 0.98,
            dynamics_db: 43.0,
            condition: 0.03,
            damper_noise: 0.27,
            key_release_noise: 0.21,
            damping_duration: 0.55,
            keyboard_range_extended: true,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.70,
            eq_gains_db: [1.1, 0.1, 0.4],
            reverb: ReverbSpec::new("Vienna Musikverein", 0.26, 2.25, 25.0, 19.0, 0.47, 0.48, 85.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "grand-bechstein-dg",
            name: "Berlin Concert Grand D 282",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "2.82-meter German concert grand celebrated for contrapuntal transparency, luminous harmonics, and long lyrical decay.",
            inharmonicity_base: 0.00015,
            decay_scale: 1.14,
            string_length_m: 2.82,
            soundboard_impedance: 0.49,
            sympathetic_resonance: 0.50,
            duplex_scale_resonance: 0.40,
            blooming_energy: 0.31,
            quadratic_effect: 0.21,
            hammer_hardness: [0.36, 0.58, 0.84],
            hammer_noise: 0.29,
            hammer_tone: 0.55,
            strike_point: 0.120,
            spectrum_profile: [1.00, 0.97, 0.89, 0.77, 0.63, 0.48, 0.35, 0.27],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.58,
            octave_stretching: 1.00,
            dynamics_db: 43.0,
            condition: 0.03,
            damper_noise: 0.25,
            key_release_noise: 0.19,
            damping_duration: 0.53,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.67,
            eq_gains_db: [0.4, 0.2, 1.0],
            reverb: ReverbSpec::new("Berlin Konzerthaus", 0.24, 2.00, 22.5, 16.5, 0.46, 0.55, 100.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "grand-petrof-275",
            name: "Bohemian Concert Grand P 275",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Central European concert grand with sweet, rounded romantic voice, woody low-mid warmth, and gentle hammer attack.",
            inharmonicity_base: 0.00019,
            decay_scale: 1.10,
            string_length_m: 2.75,
            soundboard_impedance: 0.53,
            sympathetic_resonance: 0.51,
            duplex_scale_resonance: 0.33,
            blooming_energy: 0.35,
            quadratic_effect: 0.20,
            hammer_hardness: [0.33, 0.51, 0.75],
            hammer_noise: 0.34,
            hammer_tone: 0.42,
            strike_point: 0.132,
            spectrum_profile: [1.08, 1.00, 0.88, 0.73, 0.55, 0.39, 0.27, 0.19],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.80,
            octave_stretching: 0.99,
            dynamics_db: 40.0,
            condition: 0.05,
            damper_noise: 0.29,
            key_release_noise: 0.23,
            damping_duration: 0.52,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 0.95,
            stereo_width: 0.62,
            eq_gains_db: [0.8, 0.5, 0.2],
            reverb: ReverbSpec::new("Prague Rudolfinum", 0.25, 2.10, 21.0, 15.5, 0.49, 0.47, 92.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "grand-steingraeber-e272",
            name: "Bayreuth Concert Grand E-272",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Artisanal Bayreuth concert grand offering extraordinary pianissimo delicacy alongside thunderous fortissimo headroom.",
            inharmonicity_base: 0.00016,
            decay_scale: 1.13,
            string_length_m: 2.72,
            soundboard_impedance: 0.47,
            sympathetic_resonance: 0.53,
            duplex_scale_resonance: 0.39,
            blooming_energy: 0.33,
            quadratic_effect: 0.23,
            hammer_hardness: [0.31, 0.56, 0.89],
            hammer_noise: 0.30,
            hammer_tone: 0.52,
            strike_point: 0.124,
            spectrum_profile: [1.04, 0.95, 0.87, 0.76, 0.61, 0.47, 0.34, 0.26],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.62,
            octave_stretching: 1.01,
            dynamics_db: 46.0,
            condition: 0.03,
            damper_noise: 0.26,
            key_release_noise: 0.20,
            damping_duration: 0.51,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.66,
            eq_gains_db: [0.6, 0.1, 0.9],
            reverb: ReverbSpec::new("Bayreuth Chamber Hall", 0.22, 1.90, 19.0, 14.0, 0.47, 0.54, 100.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "grand-grotrian-royal",
            name: "Braunschweig Concert Royal 277",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Star-braced German concert grand prized for homogeneous register transitions, creamy inner voices, and orchestral blend.",
            inharmonicity_base: 0.00017,
            decay_scale: 1.11,
            string_length_m: 2.77,
            soundboard_impedance: 0.51,
            sympathetic_resonance: 0.49,
            duplex_scale_resonance: 0.36,
            blooming_energy: 0.32,
            quadratic_effect: 0.21,
            hammer_hardness: [0.35, 0.55, 0.81],
            hammer_noise: 0.32,
            hammer_tone: 0.49,
            strike_point: 0.126,
            spectrum_profile: [1.03, 0.98, 0.88, 0.74, 0.59, 0.45, 0.33, 0.24],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.68,
            octave_stretching: 1.00,
            dynamics_db: 41.0,
            condition: 0.04,
            damper_noise: 0.27,
            key_release_noise: 0.21,
            damping_duration: 0.50,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.64,
            eq_gains_db: [0.5, 0.3, 0.7],
            reverb: ReverbSpec::new("German Scoring Stage", 0.21, 1.80, 18.5, 13.5, 0.48, 0.52, 105.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "grand-bluthner-1",
            name: "Leipzig Aliquot Grand Model 1",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Signature four-string aliquot treble scaling yielding luminous halo overtones, golden warmth, and singing sustain.",
            inharmonicity_base: 0.00015,
            decay_scale: 1.16,
            string_length_m: 2.80,
            soundboard_impedance: 0.45,
            sympathetic_resonance: 0.62,
            duplex_scale_resonance: 0.68,
            blooming_energy: 0.36,
            quadratic_effect: 0.19,
            hammer_hardness: [0.33, 0.52, 0.77],
            hammer_noise: 0.29,
            hammer_tone: 0.47,
            strike_point: 0.129,
            spectrum_profile: [1.06, 0.99, 0.89, 0.78, 0.64, 0.51, 0.39, 0.30],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.72,
            octave_stretching: 0.97,
            dynamics_db: 41.0,
            condition: 0.04,
            damper_noise: 0.26,
            key_release_noise: 0.20,
            damping_duration: 0.54,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.69,
            eq_gains_db: [0.7, 0.2, 0.9],
            reverb: ReverbSpec::new("Leipzig Gewandhaus", 0.25, 2.15, 23.0, 17.0, 0.46, 0.52, 95.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "grand-k2",
            name: "Studio Grand K2",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Balanced 2.13m parlor-studio grand combining crisp modern articulation with controlled close-mic room response.",
            inharmonicity_base: 0.00023,
            decay_scale: 0.98,
            string_length_m: 2.13,
            soundboard_impedance: 0.54,
            sympathetic_resonance: 0.42,
            duplex_scale_resonance: 0.34,
            blooming_energy: 0.25,
            quadratic_effect: 0.23,
            hammer_hardness: [0.38, 0.60, 0.85],
            hammer_noise: 0.34,
            hammer_tone: 0.53,
            strike_point: 0.123,
            spectrum_profile: [1.00, 0.92, 0.84, 0.72, 0.59, 0.46, 0.34, 0.26],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.70,
            octave_stretching: 1.03,
            dynamics_db: 38.0,
            condition: 0.04,
            damper_noise: 0.29,
            key_release_noise: 0.22,
            damping_duration: 0.47,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 0.85,
            stereo_width: 0.58,
            eq_gains_db: [0.3, 0.1, 0.8],
            reverb: ReverbSpec::new("Intimate Studio Room", 0.17, 1.35, 12.5, 9.0, 0.54, 0.56, 120.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "grand-yc5-rock",
            name: "Stage Rock Grand YC5",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Hard-voiced 2.00m pop and rock stage grand with lacquered hammers, tight bass punch, and studio bus compression.",
            inharmonicity_base: 0.00026,
            decay_scale: 0.92,
            string_length_m: 2.00,
            soundboard_impedance: 0.58,
            sympathetic_resonance: 0.36,
            duplex_scale_resonance: 0.40,
            blooming_energy: 0.22,
            quadratic_effect: 0.30,
            hammer_hardness: [0.48, 0.74, 0.98],
            hammer_noise: 0.40,
            hammer_tone: 0.70,
            strike_point: 0.112,
            spectrum_profile: [0.96, 0.92, 0.90, 0.84, 0.74, 0.64, 0.52, 0.42],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.85,
            octave_stretching: 1.06,
            dynamics_db: 32.0,
            condition: 0.06,
            damper_noise: 0.32,
            key_release_noise: 0.25,
            damping_duration: 0.44,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.62,
            eq_gains_db: [0.8, -0.4, 2.2],
            reverb: ReverbSpec::new("Live Arena Plate", 0.19, 1.50, 16.0, 12.0, 0.50, 0.64, 130.0),
            fx: FxSpec::with_compressor(0.52, 0.38, 0.55),
        },

        // --- Upright Piano (1) ---
        AcousticSpec {
            id: "upright-u4",
            name: "Studio Upright U4",
            category: InstrumentCategory::UprightPiano,
            exciter: ExciterType::FeltHammer,
            description: "Intimate studio upright piano with shorter strings, higher inharmonicity, and distinct mechanical action.",
            inharmonicity_base: 0.00042,
            decay_scale: 0.78,
            string_length_m: 1.31,
            soundboard_impedance: 0.64,
            sympathetic_resonance: 0.34,
            duplex_scale_resonance: 0.18,
            blooming_energy: 0.20,
            quadratic_effect: 0.25,
            hammer_hardness: [0.38, 0.58, 0.82],
            hammer_noise: 0.48,
            hammer_tone: 0.48,
            strike_point: 0.140,
            spectrum_profile: [1.04, 0.94, 0.82, 0.68, 0.52, 0.38, 0.26, 0.18],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 1.15,
            octave_stretching: 1.12,
            dynamics_db: 35.0,
            condition: 0.10,
            damper_noise: 0.42,
            key_release_noise: 0.34,
            damping_duration: 0.42,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 0.65,
            stereo_width: 0.50,
            eq_gains_db: [0.4, 0.6, 0.2],
            reverb: ReverbSpec::new("Wood Studio Room", 0.15, 1.10, 9.5, 7.0, 0.58, 0.48, 125.0),
            fx: FxSpec::acoustic_clean(),
        },

        // --- Electric Pianos (5) ---
        AcousticSpec {
            id: "vintage-tines-mk2",
            name: "Vintage Tines MKII",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::TineHammer,
            description: "Electro-acoustic tine piano with bell-like attack, asymmetric pickup warmth, and lush stereo chorus.",
            inharmonicity_base: 0.00004,
            decay_scale: 0.85,
            string_length_m: 0.38,
            soundboard_impedance: 0.32,
            sympathetic_resonance: 0.08,
            duplex_scale_resonance: 0.02,
            blooming_energy: 0.10,
            quadratic_effect: 0.12,
            hammer_hardness: [0.34, 0.56, 0.84],
            hammer_noise: 0.24,
            hammer_tone: 0.58,
            strike_point: 0.135,
            spectrum_profile: [1.20, 0.90, 0.45, 0.25, 0.15, 0.08, 0.05, 0.03],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.30,
            octave_stretching: 0.90,
            dynamics_db: 34.0,
            condition: 0.08,
            damper_noise: 0.18,
            key_release_noise: 0.16,
            damping_duration: 0.40,
            keyboard_range_extended: false,
            pickup_symmetry: 0.62,
            pickup_distance: 0.42,
            lid_position: 1.0,
            stereo_width: 0.72,
            eq_gains_db: [1.2, -0.5, 1.0],
            reverb: ReverbSpec::new("Vintage Spring Studio", 0.18, 1.45, 12.0, 10.0, 0.42, 0.50, 140.0),
            fx: FxSpec::with_chorus(0.26, 0.32),
        },
        AcousticSpec {
            id: "vintage-tines-mk1",
            name: "Vintage Tines MKI Suitcase",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::TineHammer,
            description: "Warm 1973 felt-tipped tine piano with mellow rounded bark, vintage suitcase preamp warmth, and stereo autopan tremolo.",
            inharmonicity_base: 0.00005,
            decay_scale: 0.90,
            string_length_m: 0.40,
            soundboard_impedance: 0.35,
            sympathetic_resonance: 0.09,
            duplex_scale_resonance: 0.02,
            blooming_energy: 0.14,
            quadratic_effect: 0.15,
            hammer_hardness: [0.28, 0.48, 0.76],
            hammer_noise: 0.22,
            hammer_tone: 0.44,
            strike_point: 0.148,
            spectrum_profile: [1.28, 0.96, 0.54, 0.32, 0.18, 0.09, 0.04, 0.02],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.40,
            octave_stretching: 0.92,
            dynamics_db: 36.0,
            condition: 0.11,
            damper_noise: 0.20,
            key_release_noise: 0.18,
            damping_duration: 0.42,
            keyboard_range_extended: false,
            pickup_symmetry: 0.72,
            pickup_distance: 0.36,
            lid_position: 0.92,
            stereo_width: 0.76,
            eq_gains_db: [1.6, 0.4, -0.3],
            reverb: ReverbSpec::new("Warm Tube Cabinet", 0.16, 1.25, 10.0, 8.0, 0.45, 0.44, 130.0),
            fx: FxSpec::with_tremolo(0.34, 0.45),
        },
        AcousticSpec {
            id: "vintage-reeds-200",
            name: "Vintage Reeds 200",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::ReedHammer,
            description: "Punchy reed electric piano with classic snarling midrange bark under hard velocities and vibrato tremolo.",
            inharmonicity_base: 0.00006,
            decay_scale: 0.72,
            string_length_m: 0.34,
            soundboard_impedance: 0.38,
            sympathetic_resonance: 0.04,
            duplex_scale_resonance: 0.01,
            blooming_energy: 0.12,
            quadratic_effect: 0.22,
            hammer_hardness: [0.36, 0.62, 0.90],
            hammer_noise: 0.28,
            hammer_tone: 0.60,
            strike_point: 0.115,
            spectrum_profile: [1.10, 0.75, 0.85, 0.40, 0.55, 0.22, 0.14, 0.08],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.25,
            octave_stretching: 0.94,
            dynamics_db: 32.0,
            condition: 0.10,
            damper_noise: 0.22,
            key_release_noise: 0.20,
            damping_duration: 0.36,
            keyboard_range_extended: false,
            pickup_symmetry: 0.55,
            pickup_distance: 0.48,
            lid_position: 0.95,
            stereo_width: 0.56,
            eq_gains_db: [0.6, 1.4, 0.4],
            reverb: ReverbSpec::new("70s Tracking Booth", 0.14, 1.10, 9.0, 7.5, 0.48, 0.49, 145.0),
            fx: FxSpec::with_tremolo(0.22, 0.58),
        },
        AcousticSpec {
            id: "clavinet-d6",
            name: "Clavinet D6",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::PluckPlectrum,
            description: "Funky amplified electro-mechanical clavichord with crisp rubber-tipped tangents, snappy release click, and tight compression.",
            inharmonicity_base: 0.00014,
            decay_scale: 0.54,
            string_length_m: 0.78,
            soundboard_impedance: 0.68,
            sympathetic_resonance: 0.12,
            duplex_scale_resonance: 0.08,
            blooming_energy: 0.08,
            quadratic_effect: 0.28,
            hammer_hardness: [0.58, 0.78, 0.96],
            hammer_noise: 0.45,
            hammer_tone: 0.76,
            strike_point: 0.072,
            spectrum_profile: [0.78, 0.94, 1.06, 0.98, 0.90, 0.76, 0.62, 0.48],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.35,
            octave_stretching: 0.96,
            dynamics_db: 28.0,
            condition: 0.09,
            damper_noise: 0.30,
            key_release_noise: 0.48,
            damping_duration: 0.28,
            keyboard_range_extended: false,
            pickup_symmetry: 0.80,
            pickup_distance: 0.28,
            lid_position: 1.0,
            stereo_width: 0.48,
            eq_gains_db: [-0.4, 1.8, 2.0],
            reverb: ReverbSpec::new("Funk Direct Room", 0.10, 0.85, 7.5, 6.0, 0.55, 0.58, 160.0),
            fx: FxSpec::with_compressor(0.55, 0.34, 0.62),
        },
        AcousticSpec {
            id: "pianet-n",
            name: "Pianet N",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::ReedHammer,
            description: "Classic adhesive-pad plucked-reed electric piano with woody chime attack, warm fundamental, and clean direct decay.",
            inharmonicity_base: 0.00005,
            decay_scale: 0.66,
            string_length_m: 0.36,
            soundboard_impedance: 0.40,
            sympathetic_resonance: 0.03,
            duplex_scale_resonance: 0.01,
            blooming_energy: 0.06,
            quadratic_effect: 0.14,
            hammer_hardness: [0.30, 0.48, 0.68],
            hammer_noise: 0.16,
            hammer_tone: 0.42,
            strike_point: 0.160,
            spectrum_profile: [1.22, 0.88, 0.58, 0.34, 0.20, 0.11, 0.06, 0.03],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.20,
            octave_stretching: 0.92,
            dynamics_db: 30.0,
            condition: 0.09,
            damper_noise: 0.15,
            key_release_noise: 0.26,
            damping_duration: 0.35,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.55,
            lid_position: 0.90,
            stereo_width: 0.52,
            eq_gains_db: [0.9, 0.7, -0.2],
            reverb: ReverbSpec::new("Small Amp Room", 0.15, 1.05, 8.5, 8.0, 0.46, 0.46, 135.0),
            fx: FxSpec::acoustic_clean(),
        },

        // --- Historical (3) ---
        AcousticSpec {
            id: "ruckers-harpsichord",
            name: "Flemish Harpsichord II",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::PluckPlectrum,
            description: "Plucked baroque two-manual harpsichord tuned to A=415 Hz with Werckmeister III temperament and crisp quill release.",
            inharmonicity_base: 0.00011,
            decay_scale: 0.62,
            string_length_m: 1.95,
            soundboard_impedance: 0.72,
            sympathetic_resonance: 0.28,
            duplex_scale_resonance: 0.12,
            blooming_energy: 0.10,
            quadratic_effect: 0.16,
            hammer_hardness: [0.62, 0.66, 0.72],
            hammer_noise: 0.44,
            hammer_tone: 0.74,
            strike_point: 0.080,
            spectrum_profile: [0.85, 0.95, 1.00, 0.92, 0.85, 0.75, 0.65, 0.55],
            diapason_hz: 415.0,
            temperament: TemperamentId::WerckmeisterIII,
            unison_width_cents: 1.60,
            octave_stretching: 0.82,
            dynamics_db: 18.0,
            condition: 0.14,
            damper_noise: 0.36,
            key_release_noise: 0.52,
            damping_duration: 0.32,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.60,
            eq_gains_db: [-0.6, 0.5, 1.8],
            reverb: ReverbSpec::new("Baroque Stone Chapel", 0.26, 2.20, 19.0, 16.0, 0.50, 0.58, 120.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "wiener-pianoforte-1808",
            name: "Wiener Pianoforte 1808",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::FeltHammer,
            description: "Early 19th-century Viennese leather-hammer fortepiano tuned to A=430 Hz Vallotti, with transparent wooden clarity.",
            inharmonicity_base: 0.00028,
            decay_scale: 0.68,
            string_length_m: 2.05,
            soundboard_impedance: 0.66,
            sympathetic_resonance: 0.36,
            duplex_scale_resonance: 0.08,
            blooming_energy: 0.18,
            quadratic_effect: 0.17,
            hammer_hardness: [0.44, 0.64, 0.86],
            hammer_noise: 0.42,
            hammer_tone: 0.58,
            strike_point: 0.105,
            spectrum_profile: [0.95, 0.98, 0.90, 0.76, 0.58, 0.42, 0.29, 0.20],
            diapason_hz: 430.0,
            temperament: TemperamentId::Vallotti,
            unison_width_cents: 1.80,
            octave_stretching: 0.88,
            dynamics_db: 32.0,
            condition: 0.18,
            damper_noise: 0.38,
            key_release_noise: 0.30,
            damping_duration: 0.38,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 0.90,
            stereo_width: 0.56,
            eq_gains_db: [-0.2, 0.6, 0.9],
            reverb: ReverbSpec::new("Viennese Salon 1808", 0.21, 1.55, 14.0, 11.0, 0.52, 0.52, 115.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "erard-grand-1849",
            name: "Erard Concert Grand 1849",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::FeltHammer,
            description: "Romantic Parisian straight-strung concert grand (A=435 Hz, Kirnberger III) uniting distinct bass color with crystalline treble.",
            inharmonicity_base: 0.00021,
            decay_scale: 0.88,
            string_length_m: 2.48,
            soundboard_impedance: 0.57,
            sympathetic_resonance: 0.45,
            duplex_scale_resonance: 0.20,
            blooming_energy: 0.24,
            quadratic_effect: 0.21,
            hammer_hardness: [0.38, 0.59, 0.85],
            hammer_noise: 0.36,
            hammer_tone: 0.54,
            strike_point: 0.118,
            spectrum_profile: [1.02, 0.94, 0.88, 0.78, 0.62, 0.48, 0.34, 0.24],
            diapason_hz: 435.0,
            temperament: TemperamentId::KirnbergerIII,
            unison_width_cents: 1.40,
            octave_stretching: 0.94,
            dynamics_db: 36.0,
            condition: 0.15,
            damper_noise: 0.34,
            key_release_noise: 0.26,
            damping_duration: 0.45,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 0.96,
            stereo_width: 0.60,
            eq_gains_db: [0.3, 0.4, 0.7],
            reverb: ReverbSpec::new("Paris Salon Pleyel", 0.23, 1.75, 16.5, 13.0, 0.49, 0.53, 105.0),
            fx: FxSpec::acoustic_clean(),
        },

        // --- Mallets (6) ---
        AcousticSpec {
            id: "vibraphone-vb",
            name: "Concert Vibraphone V-B",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletSoft,
            description: "Resonant aluminum bar vibraphone with pure fundamental decay, motor-fan tremolo shimmer, and lush hall ambience.",
            inharmonicity_base: 0.00002,
            decay_scale: 1.25,
            string_length_m: 0.52,
            soundboard_impedance: 0.25,
            sympathetic_resonance: 0.02,
            duplex_scale_resonance: 0.00,
            blooming_energy: 0.06,
            quadratic_effect: 0.05,
            hammer_hardness: [0.26, 0.45, 0.68],
            hammer_noise: 0.14,
            hammer_tone: 0.38,
            strike_point: 0.240,
            spectrum_profile: [1.25, 0.45, 0.18, 0.06, 0.02, 0.01, 0.00, 0.00],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.05,
            octave_stretching: 0.80,
            dynamics_db: 36.0,
            condition: 0.02,
            damper_noise: 0.12,
            key_release_noise: 0.08,
            damping_duration: 0.55,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.68,
            eq_gains_db: [0.5, 0.2, -0.4],
            reverb: ReverbSpec::new("Warm Jazz Hall", 0.30, 2.30, 22.0, 17.0, 0.45, 0.50, 95.0),
            fx: FxSpec::with_tremolo(0.24, 0.55),
        },
        AcousticSpec {
            id: "marimba-concert",
            name: "Concert Rosewood Marimba",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletSoft,
            description: "Hand-tuned Honduran rosewood bars struck with soft yarn mallets, deep woody fundamental, and fourth-partial resonance.",
            inharmonicity_base: 0.00003,
            decay_scale: 0.48,
            string_length_m: 0.55,
            soundboard_impedance: 0.62,
            sympathetic_resonance: 0.04,
            duplex_scale_resonance: 0.00,
            blooming_energy: 0.05,
            quadratic_effect: 0.08,
            hammer_hardness: [0.22, 0.40, 0.62],
            hammer_noise: 0.24,
            hammer_tone: 0.34,
            strike_point: 0.280,
            spectrum_profile: [1.32, 0.22, 0.08, 0.42, 0.04, 0.02, 0.01, 0.00],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.10,
            octave_stretching: 0.82,
            dynamics_db: 38.0,
            condition: 0.03,
            damper_noise: 0.06,
            key_release_noise: 0.05,
            damping_duration: 0.60,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.64,
            eq_gains_db: [1.2, 0.3, -0.6],
            reverb: ReverbSpec::new("Percussion Studio", 0.24, 1.65, 16.0, 12.0, 0.48, 0.46, 85.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "celesta-orchestral",
            name: "Orchestral Celesta",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::FeltHammer,
            description: "Bell-like steel plates struck by felt hammers over wooden resonator chambers for crystalline orchestral sparkle.",
            inharmonicity_base: 0.00007,
            decay_scale: 0.82,
            string_length_m: 0.45,
            soundboard_impedance: 0.40,
            sympathetic_resonance: 0.10,
            duplex_scale_resonance: 0.02,
            blooming_energy: 0.08,
            quadratic_effect: 0.06,
            hammer_hardness: [0.24, 0.42, 0.64],
            hammer_noise: 0.32,
            hammer_tone: 0.45,
            strike_point: 0.220,
            spectrum_profile: [1.22, 0.68, 0.34, 0.18, 0.09, 0.04, 0.02, 0.01],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.15,
            octave_stretching: 0.85,
            dynamics_db: 34.0,
            condition: 0.04,
            damper_noise: 0.24,
            key_release_noise: 0.18,
            damping_duration: 0.45,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 0.80,
            stereo_width: 0.58,
            eq_gains_db: [-0.4, 0.3, 1.2],
            reverb: ReverbSpec::new("Orchestral Pit Hall", 0.28, 2.15, 23.0, 18.0, 0.44, 0.56, 130.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "toy-piano",
            name: "Vintage Toy Piano",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletHard,
            description: "Miniature tuned steel rods struck by hard plastic hammers, producing nostalgic metallic inharmonicity and short percussive ring.",
            inharmonicity_base: 0.00110,
            decay_scale: 0.38,
            string_length_m: 0.26,
            soundboard_impedance: 0.78,
            sympathetic_resonance: 0.06,
            duplex_scale_resonance: 0.00,
            blooming_energy: 0.04,
            quadratic_effect: 0.14,
            hammer_hardness: [0.66, 0.82, 0.96],
            hammer_noise: 0.56,
            hammer_tone: 0.80,
            strike_point: 0.190,
            spectrum_profile: [0.88, 1.15, 0.92, 1.05, 0.72, 0.58, 0.40, 0.28],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 4.50,
            octave_stretching: 0.75,
            dynamics_db: 24.0,
            condition: 0.28,
            damper_noise: 0.35,
            key_release_noise: 0.30,
            damping_duration: 0.30,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 0.75,
            stereo_width: 0.45,
            eq_gains_db: [-1.2, 0.9, 1.8],
            reverb: ReverbSpec::new("Nursery Wood Room", 0.18, 0.95, 8.0, 6.0, 0.56, 0.60, 160.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "steel-drum",
            name: "Trinidadian Steel Drum",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletHard,
            description: "Hand-hammered Caribbean steelpan modeled with octave-coupled partial shimmer, lively transient snap, and open courtyard ring.",
            inharmonicity_base: 0.00034,
            decay_scale: 0.74,
            string_length_m: 0.42,
            soundboard_impedance: 0.48,
            sympathetic_resonance: 0.24,
            duplex_scale_resonance: 0.16,
            blooming_energy: 0.18,
            quadratic_effect: 0.32,
            hammer_hardness: [0.52, 0.72, 0.90],
            hammer_noise: 0.36,
            hammer_tone: 0.68,
            strike_point: 0.200,
            spectrum_profile: [1.12, 0.96, 0.55, 0.74, 0.38, 0.26, 0.18, 0.12],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 2.20,
            octave_stretching: 0.88,
            dynamics_db: 32.0,
            condition: 0.14,
            damper_noise: 0.08,
            key_release_noise: 0.06,
            damping_duration: 0.52,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.66,
            eq_gains_db: [0.2, 1.0, 1.3],
            reverb: ReverbSpec::new("Island Courtyard", 0.23, 1.50, 15.0, 12.0, 0.52, 0.57, 120.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "kalimba-african",
            name: "African Kalimba",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::PluckPlectrum,
            description: "Thumb-plucked spring-steel lamellae mounted on a hollow gourd resonator with pure Zarlino harmonics and intimate wood bloom.",
            inharmonicity_base: 0.00038,
            decay_scale: 0.58,
            string_length_m: 0.28,
            soundboard_impedance: 0.60,
            sympathetic_resonance: 0.14,
            duplex_scale_resonance: 0.04,
            blooming_energy: 0.08,
            quadratic_effect: 0.15,
            hammer_hardness: [0.40, 0.58, 0.76],
            hammer_noise: 0.38,
            hammer_tone: 0.52,
            strike_point: 0.150,
            spectrum_profile: [1.28, 0.78, 0.42, 0.22, 0.12, 0.08, 0.04, 0.02],
            diapason_hz: 440.0,
            temperament: TemperamentId::Zarlino,
            unison_width_cents: 0.90,
            octave_stretching: 0.84,
            dynamics_db: 30.0,
            condition: 0.08,
            damper_noise: 0.10,
            key_release_noise: 0.14,
            damping_duration: 0.46,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 0.90,
            stereo_width: 0.54,
            eq_gains_db: [0.8, 0.6, 0.0],
            reverb: ReverbSpec::new("Warm Gourd Ambience", 0.20, 1.30, 11.0, 9.0, 0.50, 0.48, 110.0),
            fx: FxSpec::acoustic_clean(),
        },

        // --- Plucked (2) ---
        AcousticSpec {
            id: "classical-guitar",
            name: "Classical Concert Guitar",
            category: InstrumentCategory::Plucked,
            exciter: ExciterType::PluckPlectrum,
            description: "Nylon-string Spanish concert guitar model with warm cedar soundboard impedance and expressive pluck position.",
            inharmonicity_base: 0.00022,
            decay_scale: 0.65,
            string_length_m: 0.65,
            soundboard_impedance: 0.62,
            sympathetic_resonance: 0.22,
            duplex_scale_resonance: 0.05,
            blooming_energy: 0.15,
            quadratic_effect: 0.14,
            hammer_hardness: [0.36, 0.54, 0.74],
            hammer_noise: 0.26,
            hammer_tone: 0.48,
            strike_point: 0.160,
            spectrum_profile: [1.10, 0.88, 0.70, 0.52, 0.38, 0.25, 0.16, 0.10],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.35,
            octave_stretching: 0.92,
            dynamics_db: 35.0,
            condition: 0.05,
            damper_noise: 0.20,
            key_release_noise: 0.18,
            damping_duration: 0.44,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 0.95,
            stereo_width: 0.52,
            eq_gains_db: [0.7, 0.3, 0.2],
            reverb: ReverbSpec::new("Cedar Guitar Salon", 0.22, 1.60, 13.5, 11.0, 0.50, 0.49, 100.0),
            fx: FxSpec::acoustic_clean(),
        },
        AcousticSpec {
            id: "concert-harp",
            name: "Grand Concert Pedal Harp",
            category: InstrumentCategory::Plucked,
            exciter: ExciterType::PluckPlectrum,
            description: "47-string double-action orchestral concert harp with sweeping sympathetic string halo, Pythagorean purity, and deep sustain.",
            inharmonicity_base: 0.00010,
            decay_scale: 1.22,
            string_length_m: 1.82,
            soundboard_impedance: 0.42,
            sympathetic_resonance: 0.64,
            duplex_scale_resonance: 0.15,
            blooming_energy: 0.28,
            quadratic_effect: 0.12,
            hammer_hardness: [0.30, 0.48, 0.68],
            hammer_noise: 0.18,
            hammer_tone: 0.44,
            strike_point: 0.210,
            spectrum_profile: [1.18, 0.92, 0.66, 0.44, 0.28, 0.18, 0.11, 0.06],
            diapason_hz: 442.0,
            temperament: TemperamentId::Pythagorean,
            unison_width_cents: 0.40,
            octave_stretching: 0.95,
            dynamics_db: 40.0,
            condition: 0.04,
            damper_noise: 0.14,
            key_release_noise: 0.12,
            damping_duration: 0.58,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.72,
            eq_gains_db: [0.8, 0.0, 0.5],
            reverb: ReverbSpec::new("Symphonic Harp Hall", 0.29, 2.45, 26.0, 20.0, 0.43, 0.52, 90.0),
            fx: FxSpec::acoustic_clean(),
        },

        // --- Prepared (1) ---
        AcousticSpec {
            id: "prepared-grand",
            name: "Prepared Experimental Grand",
            category: InstrumentCategory::Prepared,
            exciter: ExciterType::PreparedBolt,
            description: "Modified grand strings with metallic bolt overtones, wide unison beating, and percussive avant-garde timbre.",
            inharmonicity_base: 0.00078,
            decay_scale: 0.70,
            string_length_m: 2.74,
            soundboard_impedance: 0.74,
            sympathetic_resonance: 0.46,
            duplex_scale_resonance: 0.28,
            blooming_energy: 0.22,
            quadratic_effect: 0.40,
            hammer_hardness: [0.52, 0.74, 0.96],
            hammer_noise: 0.55,
            hammer_tone: 0.72,
            strike_point: 0.310,
            spectrum_profile: [0.80, 1.15, 0.45, 1.20, 0.35, 1.10, 0.90, 1.15],
            diapason_hz: 440.0,
            temperament: TemperamentId::WellTempered,
            unison_width_cents: 14.0,
            octave_stretching: 1.15,
            dynamics_db: 38.0,
            condition: 0.55,
            damper_noise: 0.46,
            key_release_noise: 0.38,
            damping_duration: 0.40,
            keyboard_range_extended: false,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.78,
            eq_gains_db: [0.5, 0.8, 1.6],
            reverb: ReverbSpec::new("Avant-Garde Loft", 0.27, 2.35, 20.0, 15.0, 0.48, 0.58, 100.0),
            fx: FxSpec::with_delay(0.18, 0.28, 0.42),
        },

        // --- Synthesizer (1) ---
        AcousticSpec {
            id: "syngular-hybrid",
            name: "Syngular Hybrid Synth",
            category: InstrumentCategory::Synthesizer,
            exciter: ExciterType::SynthHybrid,
            description: "Physical-modeling acoustic-synth hybrid combining blooming resonator harmonics, wide chorus detune, and ping-pong delay.",
            inharmonicity_base: 0.00003,
            decay_scale: 1.32,
            string_length_m: 2.40,
            soundboard_impedance: 0.28,
            sympathetic_resonance: 0.38,
            duplex_scale_resonance: 0.48,
            blooming_energy: 0.42,
            quadratic_effect: 0.26,
            hammer_hardness: [0.42, 0.66, 0.92],
            hammer_noise: 0.12,
            hammer_tone: 0.65,
            strike_point: 0.100,
            spectrum_profile: [1.15, 1.05, 0.88, 0.92, 0.74, 0.68, 0.54, 0.46],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 6.50,
            octave_stretching: 1.00,
            dynamics_db: 38.0,
            condition: 0.02,
            damper_noise: 0.08,
            key_release_noise: 0.06,
            damping_duration: 0.62,
            keyboard_range_extended: true,
            pickup_symmetry: 0.50,
            pickup_distance: 0.50,
            lid_position: 1.0,
            stereo_width: 0.88,
            eq_gains_db: [1.2, 0.0, 1.4],
            reverb: ReverbSpec::new("Cosmic Cathedral", 0.34, 3.40, 32.0, 24.0, 0.42, 0.58, 85.0),
            fx: FxSpec::with_delay_and_chorus(0.22, 0.42, 0.48, 0.28, 0.38),
        },
    ];

    specs.into_iter().map(build_instrument).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn factory_instrument_count_and_unique_metadata() {
        let instruments = build_factory_instruments();
        assert!(
            instruments.len() >= 26,
            "Expected at least 26 factory instruments, found {}",
            instruments.len()
        );

        let expected_ids = [
            "grand-ny-d",
            "grand-hb-d",
            "grand-steinway-b",
            "grand-sk-ex",
            "grand-yfx",
            "grand-bosendorfer-280vc",
            "grand-bechstein-dg",
            "grand-petrof-275",
            "grand-steingraeber-e272",
            "grand-grotrian-royal",
            "grand-bluthner-1",
            "grand-k2",
            "grand-yc5-rock",
            "upright-u4",
            "vintage-tines-mk2",
            "vintage-tines-mk1",
            "vintage-reeds-200",
            "clavinet-d6",
            "pianet-n",
            "ruckers-harpsichord",
            "wiener-pianoforte-1808",
            "erard-grand-1849",
            "vibraphone-vb",
            "marimba-concert",
            "celesta-orchestral",
            "toy-piano",
            "steel-drum",
            "kalimba-african",
            "classical-guitar",
            "concert-harp",
            "prepared-grand",
            "syngular-hybrid",
        ];

        let mut seen_ids = HashSet::new();
        let mut seen_names = HashSet::new();
        for inst in &instruments {
            assert!(!inst.id.trim().is_empty(), "Instrument ID must not be empty");
            assert!(!inst.name.trim().is_empty(), "Instrument name must not be empty for {}", inst.id);
            assert!(
                !inst.description.trim().is_empty(),
                "Instrument description must not be empty for {}",
                inst.id
            );
            assert!(
                seen_ids.insert(inst.id.clone()),
                "Duplicate instrument ID detected: {}",
                inst.id
            );
            assert!(
                seen_names.insert(inst.name.clone()),
                "Duplicate instrument name detected: {}",
                inst.name
            );
        }

        for expected in expected_ids {
            assert!(
                seen_ids.contains(expected),
                "Missing expected factory instrument ID: {}",
                expected
            );
        }
        assert_eq!(instruments[0].id, "grand-ny-d");
    }

    #[test]
    fn all_eight_instrument_categories_covered() {
        let instruments = build_factory_instruments();
        let categories: Vec<InstrumentCategory> = instruments.iter().map(|i| i.category).collect();

        let required = [
            InstrumentCategory::GrandPiano,
            InstrumentCategory::UprightPiano,
            InstrumentCategory::ElectricPiano,
            InstrumentCategory::Historical,
            InstrumentCategory::Mallet,
            InstrumentCategory::Plucked,
            InstrumentCategory::Prepared,
            InstrumentCategory::Synthesizer,
        ];

        for cat in required {
            assert!(
                categories.contains(&cat),
                "Category {:?} must be represented in factory instruments",
                cat
            );
        }

        // Verify at least 13 grand pianos and all 8 exciters are utilized
        let grand_count = instruments
            .iter()
            .filter(|i| i.category == InstrumentCategory::GrandPiano)
            .count();
        assert!(grand_count >= 13, "Expected 13 grand pianos, got {}", grand_count);

        let exciters: Vec<ExciterType> = instruments.iter().map(|i| i.exciter).collect();
        for exc in [
            ExciterType::FeltHammer,
            ExciterType::TineHammer,
            ExciterType::ReedHammer,
            ExciterType::PluckPlectrum,
            ExciterType::MalletHard,
            ExciterType::MalletSoft,
            ExciterType::PreparedBolt,
            ExciterType::SynthHybrid,
        ] {
            assert!(
                exciters.contains(&exc),
                "Exciter {:?} must be represented across factory instruments",
                exc
            );
        }
    }

    #[test]
    fn physical_parameter_sanity_bounds() {
        let instruments = build_factory_instruments();

        for inst in &instruments {
            let p = &inst.default_params;
            assert!(
                inst.inharmonicity_base.is_finite() && inst.inharmonicity_base > 0.0,
                "{}: inharmonicity_base must be finite and > 0.0 (got {})",
                inst.id,
                inst.inharmonicity_base
            );
            assert!(
                inst.decay_scale.is_finite() && inst.decay_scale > 0.0,
                "{}: decay_scale must be finite and > 0.0 (got {})",
                inst.id,
                inst.decay_scale
            );
            assert!(
                p.design.string_length_m.is_finite() && p.design.string_length_m > 0.2,
                "{}: string_length_m must be > 0.2m (got {})",
                inst.id,
                p.design.string_length_m
            );
            assert!(
                p.tuning.diapason_hz.is_finite()
                    && (400.0..=480.0).contains(&p.tuning.diapason_hz),
                "{}: diapason_hz must be in [400.0, 480.0] (got {})",
                inst.id,
                p.tuning.diapason_hz
            );
            assert!(
                p.design.soundboard_impedance > 0.0 && p.design.soundboard_impedance < 1.2,
                "{}: soundboard_impedance out of range ({})",
                inst.id,
                p.design.soundboard_impedance
            );
            assert!(
                p.voicing.strike_point > 0.0 && p.voicing.strike_point < 0.5,
                "{}: strike_point must lie in (0.0, 0.5) (got {})",
                inst.id,
                p.voicing.strike_point
            );
            assert!(
                (0.0..=1.0).contains(&p.design.sympathetic_resonance),
                "{}: sympathetic_resonance out of [0, 1] ({})",
                inst.id,
                p.design.sympathetic_resonance
            );
            assert!(
                p.voicing.hammer_hardness_piano > 0.0
                    && p.voicing.hammer_hardness_piano <= p.voicing.hammer_hardness_mezzo
                    && p.voicing.hammer_hardness_mezzo <= p.voicing.hammer_hardness_forte
                    && p.voicing.hammer_hardness_forte <= 1.2,
                "{}: non-monotonic or out-of-range hammer hardness",
                inst.id
            );
            let spec_sum: f32 = p.voicing.spectrum_profile.iter().sum();
            assert!(
                p.voicing.spectrum_profile.iter().all(|v| v.is_finite() && *v >= 0.0)
                    && spec_sum > 0.5,
                "{}: invalid spectrum_profile {:?}",
                inst.id,
                p.voicing.spectrum_profile
            );
            assert!(
                !p.reverb.preset_name.trim().is_empty()
                    && p.reverb.mix >= 0.0
                    && p.reverb.mix <= 1.0
                    && p.reverb.duration_s > 0.2,
                "{}: invalid reverb config",
                inst.id
            );
        }
    }

    #[test]
    fn acoustic_differentiation_across_families() {
        let instruments = build_factory_instruments();

        let upright = instruments
            .iter()
            .find(|i| i.id == "upright-u4")
            .expect("upright-u4 missing");

        let grands: Vec<&InstrumentModel> = instruments
            .iter()
            .filter(|i| i.category == InstrumentCategory::GrandPiano)
            .collect();

        for grand in &grands {
            assert!(
                upright.default_params.design.string_length_m
                    < grand.default_params.design.string_length_m,
                "Upright string_length_m ({}) should be shorter than grand {} ({})",
                upright.default_params.design.string_length_m,
                grand.id,
                grand.default_params.design.string_length_m
            );
            assert!(
                upright.inharmonicity_base > grand.inharmonicity_base,
                "Upright inharmonicity ({}) should exceed grand {} ({})",
                upright.inharmonicity_base,
                grand.id,
                grand.inharmonicity_base
            );
        }

        // Every instrument has a unique spectrum_profile signature
        let mut profiles: Vec<[f32; 8]> = Vec::with_capacity(instruments.len());
        for inst in &instruments {
            let prof = inst.default_params.voicing.spectrum_profile;
            assert!(
                !profiles.iter().any(|existing| {
                    existing
                        .iter()
                        .zip(prof.iter())
                        .all(|(a, b)| (a - b).abs() < 1e-5)
                }),
                "Instrument {} shares an identical spectrum_profile with another model",
                inst.id
            );
            profiles.push(prof);
        }

        // Historical instruments use period pitch (< 440 Hz) and non-Equal temperaments
        for hist in instruments
            .iter()
            .filter(|i| i.category == InstrumentCategory::Historical)
        {
            assert!(
                hist.default_params.tuning.diapason_hz < 440.0,
                "Historical {} should use period diapason < 440 Hz",
                hist.id
            );
            assert_ne!(
                hist.default_params.tuning.temperament,
                TemperamentId::Equal,
                "Historical {} should use a historical temperament",
                hist.id
            );
        }

        // Electric pianos & mallets have much lower sympathetic string resonance than concert grands
        let ny_grand = instruments.iter().find(|i| i.id == "grand-ny-d").unwrap();
        for ep in instruments
            .iter()
            .filter(|i| i.category == InstrumentCategory::ElectricPiano)
        {
            assert!(
                ep.default_params.design.sympathetic_resonance
                    < ny_grand.default_params.design.sympathetic_resonance * 0.5,
                "Electric piano {} should have lower sympathetic resonance than concert grand",
                ep.id
            );
        }

        // Blüthner aliquot model emphasizes duplex scale resonance
        let bluthner = instruments
            .iter()
            .find(|i| i.id == "grand-bluthner-1")
            .unwrap();
        assert!(bluthner.default_params.design.duplex_scale_resonance > 0.60);

        // Synth hybrid enables both Delay and Chorus FX
        let syngular = instruments
            .iter()
            .find(|i| i.id == "syngular-hybrid")
            .unwrap();
        assert!(syngular.default_params.fx[0].enabled);
        assert!(syngular.default_params.fx[1].enabled);
    }
}
