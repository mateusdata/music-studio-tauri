use crate::contracts::{
    BounceParams, EffectSlot, EffectType, EngineParams, ExciterType, InstrumentCategory,
    InstrumentModel, NoteFxParams, PlayMode, PlayModeParams, TemperamentId,
};

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
struct FxSlotSpec {
    enabled: bool,
    effect_type: EffectType,
    mix: f32,
    param_1: f32,
    param_2: f32,
    param_3: f32,
    param_4: f32,
}

impl FxSlotSpec {
    const fn off(effect_type: EffectType, mix: f32, param_1: f32, param_2: f32) -> Self {
        Self {
            enabled: false,
            effect_type,
            mix,
            param_1,
            param_2,
            param_3: 0.55,
            param_4: 0.50,
        }
    }

    const fn on(effect_type: EffectType, mix: f32, param_1: f32, param_2: f32) -> Self {
        Self {
            enabled: true,
            effect_type,
            mix,
            param_1,
            param_2,
            param_3: 0.55,
            param_4: 0.50,
        }
    }
}

#[derive(Clone, Copy)]
struct FxSpec {
    slots: [FxSlotSpec; 3],
}

impl FxSpec {
    const fn acoustic_clean() -> Self {
        Self {
            slots: [
                FxSlotSpec::off(EffectType::Delay, 0.20, 0.35, 0.36),
                FxSlotSpec::off(EffectType::Chorus, 0.18, 0.32, 0.45),
                FxSlotSpec::off(EffectType::Compressor, 0.40, 0.42, 0.38),
            ],
        }
    }

    const fn custom(s0: FxSlotSpec, s1: FxSlotSpec, s2: FxSlotSpec) -> Self {
        Self {
            slots: [s0, s1, s2],
        }
    }

    const fn with_chorus(mix: f32, rate: f32) -> Self {
        Self::custom(
            FxSlotSpec::off(EffectType::Delay, 0.18, 0.35, 0.36),
            FxSlotSpec::on(EffectType::Chorus, mix, rate, 0.45),
            FxSlotSpec::off(EffectType::Compressor, 0.40, 0.42, 0.38),
        )
    }

    const fn with_tremolo(mix: f32, rate: f32) -> Self {
        Self::custom(
            FxSlotSpec::off(EffectType::Delay, 0.18, 0.35, 0.36),
            FxSlotSpec::on(EffectType::Tremolo, mix, rate, 0.45),
            FxSlotSpec::off(EffectType::Compressor, 0.40, 0.42, 0.38),
        )
    }

    const fn with_compressor(mix: f32, thresh: f32, ratio: f32) -> Self {
        Self::custom(
            FxSlotSpec::off(EffectType::Delay, 0.18, 0.35, 0.36),
            FxSlotSpec::off(EffectType::Chorus, 0.18, 0.32, 0.45),
            FxSlotSpec::on(EffectType::Compressor, mix, thresh, ratio),
        )
    }

    const fn with_delay(mix: f32, time: f32, feedback: f32) -> Self {
        Self::custom(
            FxSlotSpec::on(EffectType::Delay, mix, time, feedback),
            FxSlotSpec::off(EffectType::Chorus, 0.18, 0.32, 0.45),
            FxSlotSpec::off(EffectType::Compressor, 0.40, 0.42, 0.38),
        )
    }
}

#[derive(Clone, Copy)]
struct BounceSpec {
    enabled: bool,
    delay_ms: f32,
    velocity_sensitivity: f32,
    delay_loss: f32,
    velocity_loss: f32,
    humanization: f32,
    max_bounces: u8,
}

impl BounceSpec {
    const fn none() -> Self {
        Self {
            enabled: false,
            delay_ms: 120.0,
            velocity_sensitivity: 0.50,
            delay_loss: 0.15,
            velocity_loss: 0.35,
            humanization: 0.10,
            max_bounces: 6,
        }
    }

    const fn standby(delay_ms: f32, delay_loss: f32, velocity_loss: f32, max_bounces: u8) -> Self {
        Self {
            enabled: false,
            delay_ms,
            velocity_sensitivity: 0.55,
            delay_loss,
            velocity_loss,
            humanization: 0.12,
            max_bounces,
        }
    }

    const fn active(
        delay_ms: f32,
        velocity_sensitivity: f32,
        delay_loss: f32,
        velocity_loss: f32,
        humanization: f32,
        max_bounces: u8,
    ) -> Self {
        Self {
            enabled: true,
            delay_ms,
            velocity_sensitivity,
            delay_loss,
            velocity_loss,
            humanization,
            max_bounces,
        }
    }
}

#[derive(Clone, Copy)]
struct NoteFxSpec {
    lfo_rate_hz: f32,
    onset_duration_ms: f32,
    vibrato_cents: f32,
    vibrato_offset_cents: f32,
    tremolo_depth_db: f32,
    attack_envelope: f32,
    smear: f32,
}

impl NoteFxSpec {
    const fn flat() -> Self {
        Self {
            lfo_rate_hz: 5.2,
            onset_duration_ms: 80.0,
            vibrato_cents: 0.0,
            vibrato_offset_cents: 0.0,
            tremolo_depth_db: 0.0,
            attack_envelope: 0.50,
            smear: 0.0,
        }
    }

    const fn expressive(
        lfo_rate_hz: f32,
        onset_duration_ms: f32,
        vibrato_cents: f32,
        vibrato_offset_cents: f32,
        tremolo_depth_db: f32,
        attack_envelope: f32,
        smear: f32,
    ) -> Self {
        Self {
            lfo_rate_hz,
            onset_duration_ms,
            vibrato_cents,
            vibrato_offset_cents,
            tremolo_depth_db,
            attack_envelope,
            smear,
        }
    }
}

#[derive(Clone, Copy)]
struct PlayModeSpec {
    mode: PlayMode,
    guitar_legato: bool,
    guitar_body: f32,
}

impl PlayModeSpec {
    const fn standard() -> Self {
        Self {
            mode: PlayMode::Standard,
            guitar_legato: false,
            guitar_body: 0.50,
        }
    }

    const fn diatonic_harp() -> Self {
        Self {
            mode: PlayMode::DiatonicHarp,
            guitar_legato: false,
            guitar_body: 0.50,
        }
    }

    const fn guitar(guitar_body: f32, guitar_legato: bool) -> Self {
        Self {
            mode: PlayMode::Guitar,
            guitar_legato,
            guitar_body,
        }
    }
}

#[derive(Clone, Copy)]
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
    bounce: BounceSpec,
    note_fx: NoteFxSpec,
    play_mode: PlayModeSpec,
}

const DEFAULT_SPEC: AcousticSpec = AcousticSpec {
    id: "",
    name: "",
    category: InstrumentCategory::GrandPiano,
    exciter: ExciterType::FeltHammer,
    description: "",
    inharmonicity_base: 0.00018,
    decay_scale: 1.0,
    string_length_m: 2.74,
    soundboard_impedance: 0.52,
    sympathetic_resonance: 0.45,
    duplex_scale_resonance: 0.35,
    blooming_energy: 0.28,
    quadratic_effect: 0.22,
    hammer_hardness: [0.38, 0.58, 0.82],
    hammer_noise: 0.35,
    hammer_tone: 0.50,
    strike_point: 0.125,
    spectrum_profile: [1.0, 0.92, 0.84, 0.72, 0.60, 0.48, 0.34, 0.28],
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
    stereo_width: 0.62,
    eq_gains_db: [0.5, 0.0, 1.0],
    reverb: ReverbSpec::new("Concert Hall", 0.22, 1.8, 18.0, 14.0, 0.45, 0.52, 110.0),
    fx: FxSpec::acoustic_clean(),
    bounce: BounceSpec::none(),
    note_fx: NoteFxSpec::flat(),
    play_mode: PlayModeSpec::standard(),
};

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

    params.fx = [0, 1, 2].map(|idx| {
        let s = spec.fx.slots[idx];
        EffectSlot {
            enabled: s.enabled,
            effect_type: s.effect_type,
            mix: s.mix,
            param_1: s.param_1,
            param_2: s.param_2,
            param_3: s.param_3,
            param_4: s.param_4,
        }
    });

    params.reverb.enabled = true;
    params.reverb.preset_name = spec.reverb.preset_name.to_string();
    params.reverb.mix = spec.reverb.mix;
    params.reverb.duration_s = spec.reverb.duration_s;
    params.reverb.room_size_m = spec.reverb.room_size_m;
    params.reverb.pre_delay_ms = spec.reverb.pre_delay_ms;
    params.reverb.early_reflections = spec.reverb.early_reflections;
    params.reverb.tone = spec.reverb.tone;
    params.reverb.highpass_hz = spec.reverb.highpass_hz;

    params.bounce = BounceParams {
        enabled: spec.bounce.enabled,
        delay_ms: spec.bounce.delay_ms,
        sync_tempo: false,
        velocity_sensitivity: spec.bounce.velocity_sensitivity,
        delay_loss: spec.bounce.delay_loss,
        velocity_loss: spec.bounce.velocity_loss,
        humanization: spec.bounce.humanization,
        max_bounces: spec.bounce.max_bounces,
    };

    params.note_fx = NoteFxParams {
        lfo_rate_hz: spec.note_fx.lfo_rate_hz,
        onset_duration_ms: spec.note_fx.onset_duration_ms,
        vibrato_cents: spec.note_fx.vibrato_cents,
        vibrato_offset_cents: spec.note_fx.vibrato_offset_cents,
        tremolo_depth_db: spec.note_fx.tremolo_depth_db,
        attack_envelope: spec.note_fx.attack_envelope,
        smear: spec.note_fx.smear,
        ..NoteFxParams::default()
    };

    params.play_mode = PlayModeParams {
        mode: spec.play_mode.mode,
        harp_pedals: [0; 7],
        guitar_fret: 0,
        guitar_legato: spec.play_mode.guitar_legato,
        guitar_body: spec.play_mode.guitar_body,
    };

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

/// Builds the complete 62-model factory library spanning all 8 instrument categories
/// and Pianoteq 9.3.1 instrument families (modern grands, uprights, electric pianos,
/// KIViR historical keyboards & bells, chromatic percussion/idiophones, plucked strings,
/// prepared grand, and hybrid physical synthesizer).
pub fn build_factory_instruments() -> Vec<InstrumentModel> {
    let specs = vec![
        // =====================================================================
        // 1. Modern Acoustic Grand Pianos (14)
        // =====================================================================
        AcousticSpec {
            id: "grand-ny-d",
            name: "Grand Steinway D (New York)",
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
            lid_position: 1.0,
            stereo_width: 0.64,
            eq_gains_db: [0.5, 0.0, 1.0],
            reverb: ReverbSpec::new("NY Concert Stage", 0.22, 1.85, 20.0, 14.0, 0.46, 0.54, 105.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-hb-d",
            name: "Grand Steinway D (Hamburg)",
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
            lid_position: 1.0,
            stereo_width: 0.68,
            eq_gains_db: [0.6, 0.2, 0.6],
            reverb: ReverbSpec::new("Hamburg Philharmonic", 0.24, 2.05, 22.0, 16.0, 0.48, 0.50, 95.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-steinway-b",
            name: "Grand Steinway B",
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
            lid_position: 0.88,
            stereo_width: 0.58,
            eq_gains_db: [0.4, 0.3, 0.8],
            reverb: ReverbSpec::new("Chamber Recital", 0.20, 1.55, 14.5, 11.0, 0.52, 0.53, 115.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-sk-ex",
            name: "Grand Shigeru Kawai SK-EX",
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
            lid_position: 1.0,
            stereo_width: 0.66,
            eq_gains_db: [0.7, -0.1, 0.7],
            reverb: ReverbSpec::new("Tokyo Suntory Hall", 0.25, 2.15, 24.0, 18.0, 0.44, 0.51, 90.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-yfx",
            name: "Grand Japanese YFX",
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
            lid_position: 1.0,
            stereo_width: 0.65,
            eq_gains_db: [0.3, 0.0, 1.5],
            reverb: ReverbSpec::new("Brilliant Symphony Hall", 0.23, 1.95, 21.5, 15.0, 0.45, 0.60, 110.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-bosendorfer-280vc",
            name: "Grand Bösendorfer 280VC",
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
            lid_position: 1.0,
            stereo_width: 0.70,
            eq_gains_db: [1.1, 0.1, 0.4],
            reverb: ReverbSpec::new("Vienna Musikverein", 0.26, 2.25, 25.0, 19.0, 0.47, 0.48, 85.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-bechstein-dg",
            name: "Grand C. Bechstein DG",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "2.82-meter Berlin concert grand celebrated for contrapuntal transparency, luminous harmonics, and long lyrical decay.",
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
            lid_position: 1.0,
            stereo_width: 0.67,
            eq_gains_db: [0.4, 0.2, 1.0],
            reverb: ReverbSpec::new("Berlin Konzerthaus", 0.24, 2.00, 22.5, 16.5, 0.46, 0.55, 100.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-petrof-275",
            name: "Grand Ant. Petrof 275",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Handcrafted Bohemian 275cm concert grand with sweet, rounded romantic voice, woody low-mid warmth, and singing sustain.",
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
            lid_position: 0.95,
            stereo_width: 0.62,
            eq_gains_db: [0.8, 0.5, 0.2],
            reverb: ReverbSpec::new("Prague Rudolfinum", 0.25, 2.10, 21.0, 15.5, 0.49, 0.47, 92.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-petrof-mistral",
            name: "Grand Petrof 284 Mistral",
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Flagship 2.84-meter Czech concert grand delivering thunderous bass authority, orchestral sustain, and rich central-European color.",
            inharmonicity_base: 0.00015,
            decay_scale: 1.17,
            string_length_m: 2.84,
            soundboard_impedance: 0.47,
            sympathetic_resonance: 0.56,
            duplex_scale_resonance: 0.37,
            blooming_energy: 0.37,
            quadratic_effect: 0.22,
            hammer_hardness: [0.35, 0.56, 0.83],
            hammer_noise: 0.31,
            hammer_tone: 0.48,
            strike_point: 0.127,
            spectrum_profile: [1.10, 1.01, 0.90, 0.76, 0.60, 0.44, 0.30, 0.22],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.72,
            octave_stretching: 0.98,
            dynamics_db: 44.0,
            condition: 0.03,
            damper_noise: 0.26,
            key_release_noise: 0.20,
            damping_duration: 0.55,
            lid_position: 1.0,
            stereo_width: 0.68,
            eq_gains_db: [1.0, 0.3, 0.5],
            reverb: ReverbSpec::new("Hradec Concert Hall", 0.25, 2.20, 23.5, 17.5, 0.47, 0.50, 88.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-steingraeber-e272",
            name: "Grand Steingraeber E-272",
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
            lid_position: 1.0,
            stereo_width: 0.66,
            eq_gains_db: [0.6, 0.1, 0.9],
            reverb: ReverbSpec::new("Bayreuth Chamber Hall", 0.22, 1.90, 19.0, 14.0, 0.47, 0.54, 100.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-grotrian-royal",
            name: "Grand Grotrian Concert Royal",
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
            lid_position: 1.0,
            stereo_width: 0.64,
            eq_gains_db: [0.5, 0.3, 0.7],
            reverb: ReverbSpec::new("German Scoring Stage", 0.21, 1.80, 18.5, 13.5, 0.48, 0.52, 105.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-bluthner-1",
            name: "Grand Blüthner Model 1",
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
            lid_position: 1.0,
            stereo_width: 0.69,
            eq_gains_db: [0.7, 0.2, 0.9],
            reverb: ReverbSpec::new("Leipzig Gewandhaus", 0.25, 2.15, 23.0, 17.0, 0.46, 0.52, 95.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-k2",
            name: "Grand K2",
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
            lid_position: 0.85,
            stereo_width: 0.58,
            eq_gains_db: [0.3, 0.1, 0.8],
            reverb: ReverbSpec::new("Intimate Studio Room", 0.17, 1.35, 12.5, 9.0, 0.54, 0.56, 120.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "grand-yc5-rock",
            name: "Grand YC5 Rock",
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
            lid_position: 1.0,
            stereo_width: 0.62,
            eq_gains_db: [0.8, -0.4, 2.2],
            reverb: ReverbSpec::new("Live Arena Plate", 0.19, 1.50, 16.0, 12.0, 0.50, 0.64, 130.0),
            fx: FxSpec::with_compressor(0.52, 0.38, 0.55),
            ..DEFAULT_SPEC
        },

        // =====================================================================
        // 2. Acoustic Upright Pianos (2)
        // =====================================================================
        AcousticSpec {
            id: "upright-u4",
            name: "Upright U4 Studio",
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
            lid_position: 0.65,
            stereo_width: 0.50,
            eq_gains_db: [0.4, 0.6, 0.2],
            reverb: ReverbSpec::new("Wood Studio Room", 0.15, 1.10, 9.5, 7.0, 0.58, 0.48, 125.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "upright-u4-honky",
            name: "Upright U4 Honky-Tonk / Tack",
            category: InstrumentCategory::UprightPiano,
            exciter: ExciterType::FeltHammer,
            description: "Vintage saloon tack upright with wide unison detuning, hardened thumbtack felt tips, metallic upper partials, and ragged room punch.",
            inharmonicity_base: 0.00048,
            decay_scale: 0.68,
            string_length_m: 1.28,
            soundboard_impedance: 0.70,
            sympathetic_resonance: 0.30,
            duplex_scale_resonance: 0.14,
            blooming_energy: 0.16,
            quadratic_effect: 0.32,
            hammer_hardness: [0.60, 0.78, 0.96],
            hammer_noise: 0.62,
            hammer_tone: 0.74,
            strike_point: 0.146,
            spectrum_profile: [0.92, 0.98, 1.04, 0.90, 0.80, 0.68, 0.54, 0.42],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 11.50,
            octave_stretching: 1.16,
            dynamics_db: 28.0,
            condition: 0.52,
            damper_noise: 0.48,
            key_release_noise: 0.40,
            damping_duration: 0.36,
            lid_position: 0.78,
            stereo_width: 0.56,
            eq_gains_db: [-0.4, 1.2, 1.8],
            reverb: ReverbSpec::new("Saloon Timber Hall", 0.18, 1.05, 10.5, 7.5, 0.60, 0.56, 140.0),
            fx: FxSpec::custom(
                FxSlotSpec::off(EffectType::Delay, 0.15, 0.25, 0.30),
                FxSlotSpec::on(EffectType::Chorus, 0.14, 0.22, 0.30),
                FxSlotSpec::on(EffectType::Compressor, 0.42, 0.42, 0.50),
            ),
            ..DEFAULT_SPEC
        },

        // =====================================================================
        // 3. Electro-Acoustic Pianos (8)
        // =====================================================================
        AcousticSpec {
            id: "vintage-tines-mk2",
            name: "Vintage Tines MKII",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::TineHammer,
            description: "Electro-acoustic tine piano with bell-like attack, asymmetric pickup warmth, stereo chorus, and tube combo amp sheen.",
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
            pickup_symmetry: 0.62,
            pickup_distance: 0.42,
            stereo_width: 0.72,
            eq_gains_db: [1.2, -0.5, 1.0],
            reverb: ReverbSpec::new("Vintage Spring Studio", 0.18, 1.45, 12.0, 10.0, 0.42, 0.50, 140.0),
            fx: FxSpec::custom(
                FxSlotSpec::off(EffectType::Delay, 0.18, 0.32, 0.36),
                FxSlotSpec::on(EffectType::Chorus, 0.26, 0.32, 0.45),
                FxSlotSpec::on(EffectType::Amp, 0.35, 0.30, 0.55),
            ),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "vintage-tines-mk1",
            name: "Vintage Tines MKI",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::TineHammer,
            description: "Warm 1973 felt-tipped suitcase tine piano featuring mellow bark, 70s phaser sweep, stereo autopan tremolo, and cabinet amp.",
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
            pickup_symmetry: 0.72,
            pickup_distance: 0.36,
            lid_position: 0.92,
            stereo_width: 0.76,
            eq_gains_db: [1.6, 0.4, -0.3],
            reverb: ReverbSpec::new("Warm Tube Cabinet", 0.16, 1.25, 10.0, 8.0, 0.45, 0.44, 130.0),
            fx: FxSpec::custom(
                FxSlotSpec::on(EffectType::Phaser, 0.24, 0.36, 0.48),
                FxSlotSpec::on(EffectType::Tremolo, 0.34, 0.45, 0.50),
                FxSlotSpec::on(EffectType::Amp, 0.30, 0.28, 0.50),
            ),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "vintage-reeds-200",
            name: "Vintage Reeds 200A",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::ReedHammer,
            description: "Punchy reed electric piano with classic snarling midrange bark under hard velocities, tremolo vibrato, and speaker amp grit.",
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
            pickup_symmetry: 0.55,
            pickup_distance: 0.48,
            lid_position: 0.95,
            stereo_width: 0.56,
            eq_gains_db: [0.6, 1.4, 0.4],
            reverb: ReverbSpec::new("70s Tracking Booth", 0.14, 1.10, 9.0, 7.5, 0.48, 0.49, 145.0),
            fx: FxSpec::custom(
                FxSlotSpec::off(EffectType::Delay, 0.15, 0.28, 0.30),
                FxSlotSpec::on(EffectType::Tremolo, 0.26, 0.56, 0.50),
                FxSlotSpec::on(EffectType::Amp, 0.40, 0.38, 0.60),
            ),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "vintage-reeds-140b",
            name: "Vintage Reeds 140B",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::ReedHammer,
            description: "Early 1960s tube-preamp reed piano with darker woody bass, softer felt transient, smooth tremolo, and soulful amp compression.",
            inharmonicity_base: 0.00007,
            decay_scale: 0.76,
            string_length_m: 0.37,
            soundboard_impedance: 0.41,
            sympathetic_resonance: 0.05,
            duplex_scale_resonance: 0.01,
            blooming_energy: 0.11,
            quadratic_effect: 0.20,
            hammer_hardness: [0.30, 0.54, 0.82],
            hammer_noise: 0.25,
            hammer_tone: 0.48,
            strike_point: 0.126,
            spectrum_profile: [1.16, 0.84, 0.76, 0.36, 0.44, 0.18, 0.10, 0.05],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.32,
            octave_stretching: 0.93,
            dynamics_db: 34.0,
            condition: 0.13,
            damper_noise: 0.24,
            key_release_noise: 0.22,
            damping_duration: 0.38,
            pickup_symmetry: 0.58,
            pickup_distance: 0.44,
            lid_position: 0.90,
            stereo_width: 0.54,
            eq_gains_db: [1.2, 0.9, -0.2],
            reverb: ReverbSpec::new("60s Tube Club", 0.16, 1.20, 10.5, 8.5, 0.47, 0.45, 130.0),
            fx: FxSpec::custom(
                FxSlotSpec::off(EffectType::Chorus, 0.16, 0.28, 0.40),
                FxSlotSpec::on(EffectType::Tremolo, 0.22, 0.44, 0.45),
                FxSlotSpec::on(EffectType::Amp, 0.36, 0.32, 0.52),
            ),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "clavinet-d6",
            name: "Clavinet D6",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::PluckPlectrum,
            description: "Funky amplified electro-mechanical clavichord with rubber-tipped tangents, snappy release click, auto-wah filter, amp drive, and tight compressor.",
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
            pickup_symmetry: 0.80,
            pickup_distance: 0.28,
            stereo_width: 0.48,
            eq_gains_db: [-0.4, 1.8, 2.0],
            reverb: ReverbSpec::new("Funk Direct Room", 0.10, 0.85, 7.5, 6.0, 0.55, 0.58, 160.0),
            fx: FxSpec::custom(
                FxSlotSpec::on(EffectType::Wah, 0.48, 0.44, 0.62),
                FxSlotSpec::on(EffectType::Amp, 0.45, 0.40, 0.65),
                FxSlotSpec::on(EffectType::Compressor, 0.55, 0.34, 0.62),
            ),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "pianet-n",
            name: "Pianet N",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::ReedHammer,
            description: "Classic adhesive-pad plucked-reed electric piano with woody chime attack, warm fundamental, phaser sheen, and vintage amp.",
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
            pickup_symmetry: 0.50,
            pickup_distance: 0.55,
            lid_position: 0.90,
            stereo_width: 0.52,
            eq_gains_db: [0.9, 0.7, -0.2],
            reverb: ReverbSpec::new("Small Amp Room", 0.15, 1.05, 8.5, 8.0, 0.46, 0.46, 135.0),
            fx: FxSpec::custom(
                FxSlotSpec::on(EffectType::Phaser, 0.22, 0.28, 0.45),
                FxSlotSpec::off(EffectType::Chorus, 0.16, 0.30, 0.40),
                FxSlotSpec::on(EffectType::Amp, 0.28, 0.25, 0.48),
            ),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "pianet-t",
            name: "Pianet T",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::ReedHammer,
            description: "Late-1970s passive rubber-suction-pad reed piano delivering ultra-warm mellow midrange, gentle phaser, and tremolo pulse.",
            inharmonicity_base: 0.00006,
            decay_scale: 0.62,
            string_length_m: 0.35,
            soundboard_impedance: 0.43,
            sympathetic_resonance: 0.03,
            duplex_scale_resonance: 0.01,
            blooming_energy: 0.05,
            quadratic_effect: 0.13,
            hammer_hardness: [0.26, 0.44, 0.64],
            hammer_noise: 0.14,
            hammer_tone: 0.36,
            strike_point: 0.172,
            spectrum_profile: [1.26, 0.84, 0.48, 0.26, 0.14, 0.08, 0.04, 0.02],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.22,
            octave_stretching: 0.91,
            dynamics_db: 28.0,
            condition: 0.08,
            damper_noise: 0.14,
            key_release_noise: 0.22,
            damping_duration: 0.34,
            pickup_symmetry: 0.54,
            pickup_distance: 0.58,
            lid_position: 0.86,
            stereo_width: 0.54,
            eq_gains_db: [1.1, 0.5, -0.6],
            reverb: ReverbSpec::new("Analog Tape Lounge", 0.16, 1.15, 9.5, 8.5, 0.44, 0.44, 125.0),
            fx: FxSpec::custom(
                FxSlotSpec::on(EffectType::Phaser, 0.20, 0.24, 0.42),
                FxSlotSpec::on(EffectType::Tremolo, 0.20, 0.40, 0.48),
                FxSlotSpec::off(EffectType::Compressor, 0.38, 0.40, 0.42),
            ),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "electra-piano",
            name: "Electra-Piano",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::TineHammer,
            description: "1960s stage electro-mechanical upright piano with assertive rock chime, tape delay echo, phaser color, and driven combo amp.",
            inharmonicity_base: 0.00009,
            decay_scale: 0.74,
            string_length_m: 0.44,
            soundboard_impedance: 0.46,
            sympathetic_resonance: 0.07,
            duplex_scale_resonance: 0.03,
            blooming_energy: 0.13,
            quadratic_effect: 0.24,
            hammer_hardness: [0.40, 0.64, 0.90],
            hammer_noise: 0.32,
            hammer_tone: 0.64,
            strike_point: 0.122,
            spectrum_profile: [1.12, 0.98, 0.76, 0.52, 0.36, 0.22, 0.14, 0.08],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.48,
            octave_stretching: 0.95,
            dynamics_db: 32.0,
            condition: 0.12,
            damper_noise: 0.24,
            key_release_noise: 0.22,
            damping_duration: 0.38,
            pickup_symmetry: 0.66,
            pickup_distance: 0.38,
            lid_position: 0.94,
            stereo_width: 0.62,
            eq_gains_db: [0.5, 1.0, 1.2],
            reverb: ReverbSpec::new("Classic Rock Stage", 0.18, 1.40, 14.0, 11.0, 0.48, 0.55, 130.0),
            fx: FxSpec::custom(
                FxSlotSpec::on(EffectType::Delay, 0.16, 0.30, 0.36),
                FxSlotSpec::on(EffectType::Phaser, 0.22, 0.32, 0.45),
                FxSlotSpec::on(EffectType::Amp, 0.46, 0.44, 0.60),
            ),
            ..DEFAULT_SPEC
        },

        // =====================================================================
        // 4. Historical & KIViR Collection (14)
        // =====================================================================
        AcousticSpec {
            id: "ruckers-harpsichord",
            name: "H. Ruckers II Harpsichord",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::PluckPlectrum,
            description: "Plucked 1624 Flemish two-manual harpsichord tuned to A=415 Hz Werckmeister III with vibrant quill release.",
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
            stereo_width: 0.60,
            eq_gains_db: [-0.6, 0.5, 1.8],
            reverb: ReverbSpec::new("Baroque Stone Chapel", 0.26, 2.20, 19.0, 16.0, 0.50, 0.58, 120.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "salodiensis-virginal-1600",
            name: "J. Salodiensis Virginal (1600)",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::PluckPlectrum,
            description: "Renaissance Italian polygonal virginal from 1600 with central-plucked hollow sweetness, meantone thirds, and delicate cedar body.",
            inharmonicity_base: 0.00013,
            decay_scale: 0.56,
            string_length_m: 1.42,
            soundboard_impedance: 0.76,
            sympathetic_resonance: 0.22,
            duplex_scale_resonance: 0.06,
            blooming_energy: 0.08,
            quadratic_effect: 0.14,
            hammer_hardness: [0.58, 0.62, 0.68],
            hammer_noise: 0.40,
            hammer_tone: 0.66,
            strike_point: 0.190,
            spectrum_profile: [1.08, 0.96, 0.88, 0.74, 0.62, 0.48, 0.36, 0.24],
            diapason_hz: 415.0,
            temperament: TemperamentId::MeanTone,
            unison_width_cents: 1.30,
            octave_stretching: 0.80,
            dynamics_db: 16.0,
            condition: 0.16,
            damper_noise: 0.32,
            key_release_noise: 0.48,
            damping_duration: 0.30,
            lid_position: 0.90,
            stereo_width: 0.54,
            eq_gains_db: [-0.4, 0.8, 1.2],
            reverb: ReverbSpec::new("Venetian Palazzo", 0.24, 1.85, 15.5, 13.0, 0.52, 0.55, 125.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "giusti-harpsichord-1680",
            name: "G. Giusti Harpsichord (1680)",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::PluckPlectrum,
            description: "1680 Italian cypress harpsichord with crisp immediate pluck, lean clear bass, and singing Vallotti brilliance.",
            inharmonicity_base: 0.00010,
            decay_scale: 0.60,
            string_length_m: 2.08,
            soundboard_impedance: 0.70,
            sympathetic_resonance: 0.26,
            duplex_scale_resonance: 0.10,
            blooming_energy: 0.09,
            quadratic_effect: 0.17,
            hammer_hardness: [0.64, 0.68, 0.75],
            hammer_noise: 0.46,
            hammer_tone: 0.78,
            strike_point: 0.092,
            spectrum_profile: [0.88, 0.98, 1.04, 0.95, 0.82, 0.70, 0.56, 0.44],
            diapason_hz: 415.0,
            temperament: TemperamentId::Vallotti,
            unison_width_cents: 1.45,
            octave_stretching: 0.83,
            dynamics_db: 19.0,
            condition: 0.14,
            damper_noise: 0.35,
            key_release_noise: 0.50,
            damping_duration: 0.31,
            stereo_width: 0.58,
            eq_gains_db: [-0.5, 0.4, 1.6],
            reverb: ReverbSpec::new("Roman Baroque Gallery", 0.25, 2.00, 17.5, 14.5, 0.50, 0.57, 120.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "schmidt-square-1780",
            name: "Ph. Schmidt Square (1780)",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::FeltHammer,
            description: "Late 18th-century German square fortepiano with light uncovered wooden/leather hammers, intimate lute-like shimmer, and Kirnberger III tuning.",
            inharmonicity_base: 0.00036,
            decay_scale: 0.58,
            string_length_m: 1.38,
            soundboard_impedance: 0.72,
            sympathetic_resonance: 0.28,
            duplex_scale_resonance: 0.04,
            blooming_energy: 0.14,
            quadratic_effect: 0.15,
            hammer_hardness: [0.48, 0.68, 0.88],
            hammer_noise: 0.46,
            hammer_tone: 0.64,
            strike_point: 0.100,
            spectrum_profile: [0.92, 0.98, 0.94, 0.82, 0.66, 0.50, 0.36, 0.25],
            diapason_hz: 422.0,
            temperament: TemperamentId::KirnbergerIII,
            unison_width_cents: 2.10,
            octave_stretching: 0.86,
            dynamics_db: 28.0,
            condition: 0.20,
            damper_noise: 0.40,
            key_release_noise: 0.32,
            damping_duration: 0.35,
            lid_position: 0.85,
            stereo_width: 0.52,
            eq_gains_db: [-0.4, 0.7, 1.1],
            reverb: ReverbSpec::new("Weimar Clavichord Parlor", 0.18, 1.30, 11.0, 9.0, 0.54, 0.52, 125.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "dohnal-1795",
            name: "J. Dohnal (1795)",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::FeltHammer,
            description: "Classical 1795 Viennese wooden-frame fortepiano by Joseph Dohnal, ideal for Mozart and early Beethoven with agile translucent bass.",
            inharmonicity_base: 0.00031,
            decay_scale: 0.64,
            string_length_m: 1.92,
            soundboard_impedance: 0.68,
            sympathetic_resonance: 0.34,
            duplex_scale_resonance: 0.06,
            blooming_energy: 0.16,
            quadratic_effect: 0.16,
            hammer_hardness: [0.46, 0.66, 0.87],
            hammer_noise: 0.44,
            hammer_tone: 0.60,
            strike_point: 0.102,
            spectrum_profile: [0.94, 0.99, 0.92, 0.78, 0.60, 0.44, 0.30, 0.21],
            diapason_hz: 425.0,
            temperament: TemperamentId::WellTempered,
            unison_width_cents: 1.90,
            octave_stretching: 0.87,
            dynamics_db: 30.0,
            condition: 0.19,
            damper_noise: 0.39,
            key_release_noise: 0.31,
            damping_duration: 0.36,
            lid_position: 0.90,
            stereo_width: 0.55,
            eq_gains_db: [-0.3, 0.6, 1.0],
            reverb: ReverbSpec::new("Kremsegg Abbey Hall", 0.20, 1.45, 13.0, 10.5, 0.52, 0.52, 120.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "broadwood-1796",
            name: "J. Broadwood (1796)",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::FeltHammer,
            description: "English 1796 John Broadwood grand fortepiano featuring deeper bass sustain, sturdy oak frame weight, and cantabile London tone.",
            inharmonicity_base: 0.00026,
            decay_scale: 0.76,
            string_length_m: 2.24,
            soundboard_impedance: 0.62,
            sympathetic_resonance: 0.40,
            duplex_scale_resonance: 0.10,
            blooming_energy: 0.20,
            quadratic_effect: 0.18,
            hammer_hardness: [0.40, 0.60, 0.82],
            hammer_noise: 0.38,
            hammer_tone: 0.52,
            strike_point: 0.114,
            spectrum_profile: [1.04, 0.96, 0.86, 0.72, 0.54, 0.39, 0.27, 0.18],
            diapason_hz: 428.0,
            temperament: TemperamentId::Vallotti,
            unison_width_cents: 1.75,
            octave_stretching: 0.90,
            dynamics_db: 34.0,
            condition: 0.17,
            damper_noise: 0.36,
            key_release_noise: 0.28,
            damping_duration: 0.42,
            lid_position: 0.92,
            stereo_width: 0.57,
            eq_gains_db: [0.4, 0.4, 0.6],
            reverb: ReverbSpec::new("London Hanover Square", 0.22, 1.65, 15.0, 12.0, 0.50, 0.50, 110.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "wiener-pianoforte-1808",
            name: "J. Weimes Pianoforte (1808)",
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
            lid_position: 0.90,
            stereo_width: 0.56,
            eq_gains_db: [-0.2, 0.6, 0.9],
            reverb: ReverbSpec::new("Viennese Salon 1808", 0.21, 1.55, 14.0, 11.0, 0.52, 0.52, 115.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "besendorfer-1829",
            name: "I. Besendorfer (1829)",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::FeltHammer,
            description: "1829 Ignaz Bösendorfer Viennese grand bridging Classical elegance and Romantic warmth with rich spruce resonance.",
            inharmonicity_base: 0.00025,
            decay_scale: 0.74,
            string_length_m: 2.18,
            soundboard_impedance: 0.61,
            sympathetic_resonance: 0.39,
            duplex_scale_resonance: 0.12,
            blooming_energy: 0.21,
            quadratic_effect: 0.18,
            hammer_hardness: [0.41, 0.61, 0.84],
            hammer_noise: 0.39,
            hammer_tone: 0.54,
            strike_point: 0.112,
            spectrum_profile: [1.01, 0.97, 0.89, 0.75, 0.57, 0.41, 0.28, 0.19],
            diapason_hz: 432.0,
            temperament: TemperamentId::WellTempered,
            unison_width_cents: 1.65,
            octave_stretching: 0.91,
            dynamics_db: 34.0,
            condition: 0.16,
            damper_noise: 0.36,
            key_release_noise: 0.28,
            damping_duration: 0.41,
            lid_position: 0.92,
            stereo_width: 0.58,
            eq_gains_db: [0.2, 0.5, 0.7],
            reverb: ReverbSpec::new("Schubertiade Chamber", 0.21, 1.60, 14.5, 11.5, 0.51, 0.51, 112.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "pleyel-1835",
            name: "I. Pleyel (1835)",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::FeltHammer,
            description: "1835 Ignace Pleyel Parisian salon grand favored by Chopin, offering silvery poetic treble, delicate damping, and intimate nuance.",
            inharmonicity_base: 0.00023,
            decay_scale: 0.82,
            string_length_m: 2.30,
            soundboard_impedance: 0.59,
            sympathetic_resonance: 0.43,
            duplex_scale_resonance: 0.16,
            blooming_energy: 0.22,
            quadratic_effect: 0.19,
            hammer_hardness: [0.36, 0.57, 0.82],
            hammer_noise: 0.35,
            hammer_tone: 0.56,
            strike_point: 0.115,
            spectrum_profile: [1.00, 0.95, 0.90, 0.79, 0.65, 0.50, 0.36, 0.26],
            diapason_hz: 434.0,
            temperament: TemperamentId::KirnbergerIII,
            unison_width_cents: 1.50,
            octave_stretching: 0.93,
            dynamics_db: 35.0,
            condition: 0.15,
            damper_noise: 0.34,
            key_release_noise: 0.27,
            damping_duration: 0.44,
            lid_position: 0.94,
            stereo_width: 0.59,
            eq_gains_db: [0.1, 0.4, 0.9],
            reverb: ReverbSpec::new("Chopin Salon Rue Rochechouart", 0.22, 1.65, 15.0, 12.0, 0.50, 0.53, 108.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "frenzel-1841",
            name: "J. Frenzel (1841)",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::FeltHammer,
            description: "1841 Austrian Biedermeier concert piano with singing middle register, clear wooden bass articulation, and responsive leather-felt touch.",
            inharmonicity_base: 0.00022,
            decay_scale: 0.84,
            string_length_m: 2.36,
            soundboard_impedance: 0.58,
            sympathetic_resonance: 0.44,
            duplex_scale_resonance: 0.18,
            blooming_energy: 0.23,
            quadratic_effect: 0.20,
            hammer_hardness: [0.39, 0.58, 0.83],
            hammer_noise: 0.36,
            hammer_tone: 0.53,
            strike_point: 0.116,
            spectrum_profile: [1.03, 0.96, 0.87, 0.75, 0.59, 0.45, 0.32, 0.22],
            diapason_hz: 435.0,
            temperament: TemperamentId::WellTempered,
            unison_width_cents: 1.45,
            octave_stretching: 0.94,
            dynamics_db: 36.0,
            condition: 0.15,
            damper_noise: 0.33,
            key_release_noise: 0.26,
            damping_duration: 0.44,
            lid_position: 0.95,
            stereo_width: 0.59,
            eq_gains_db: [0.3, 0.5, 0.6],
            reverb: ReverbSpec::new("Linz Redoutensaal", 0.22, 1.70, 16.0, 12.5, 0.49, 0.52, 106.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "erard-grand-1849",
            name: "S. Erard (1849)",
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
            lid_position: 0.96,
            stereo_width: 0.60,
            eq_gains_db: [0.3, 0.4, 0.7],
            reverb: ReverbSpec::new("Paris Salon Pleyel", 0.23, 1.75, 16.5, 13.0, 0.49, 0.53, 105.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "streicher-1852",
            name: "J.B. Streicher (1852)",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::FeltHammer,
            description: "1852 Johann Baptist Streicher Viennese concert grand beloved by Brahms, blending robust orchestral bass with dry lyrical focus.",
            inharmonicity_base: 0.00022,
            decay_scale: 0.86,
            string_length_m: 2.42,
            soundboard_impedance: 0.56,
            sympathetic_resonance: 0.46,
            duplex_scale_resonance: 0.22,
            blooming_energy: 0.25,
            quadratic_effect: 0.20,
            hammer_hardness: [0.37, 0.57, 0.82],
            hammer_noise: 0.34,
            hammer_tone: 0.51,
            strike_point: 0.120,
            spectrum_profile: [1.05, 0.97, 0.86, 0.74, 0.58, 0.43, 0.31, 0.21],
            diapason_hz: 436.0,
            temperament: TemperamentId::WellTempered,
            unison_width_cents: 1.35,
            octave_stretching: 0.95,
            dynamics_db: 37.0,
            condition: 0.14,
            damper_noise: 0.32,
            key_release_noise: 0.25,
            damping_duration: 0.46,
            lid_position: 0.96,
            stereo_width: 0.61,
            eq_gains_db: [0.5, 0.3, 0.6],
            reverb: ReverbSpec::new("Vienna Brahms Hall", 0.23, 1.80, 17.0, 13.5, 0.48, 0.51, 102.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "ny-steinway-square-1858",
            name: "NY Steinway Square (1858)",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::FeltHammer,
            description: "Early 1858 New York cast-iron overstrung square grand piano combining parlor warmth with unexpected American bass projection.",
            inharmonicity_base: 0.00033,
            decay_scale: 0.80,
            string_length_m: 1.78,
            soundboard_impedance: 0.62,
            sympathetic_resonance: 0.38,
            duplex_scale_resonance: 0.16,
            blooming_energy: 0.20,
            quadratic_effect: 0.22,
            hammer_hardness: [0.40, 0.60, 0.84],
            hammer_noise: 0.40,
            hammer_tone: 0.55,
            strike_point: 0.134,
            spectrum_profile: [1.04, 0.93, 0.83, 0.70, 0.53, 0.38, 0.26, 0.17],
            diapason_hz: 438.0,
            temperament: TemperamentId::WellTempered,
            unison_width_cents: 1.70,
            octave_stretching: 0.98,
            dynamics_db: 35.0,
            condition: 0.16,
            damper_noise: 0.36,
            key_release_noise: 0.29,
            damping_duration: 0.43,
            lid_position: 0.88,
            stereo_width: 0.56,
            eq_gains_db: [0.6, 0.4, 0.5],
            reverb: ReverbSpec::new("1858 Bowery Parlor", 0.19, 1.40, 12.5, 10.0, 0.52, 0.50, 115.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "bechstein-1899",
            name: "C. Bechstein (1899)",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::FeltHammer,
            description: "Fin-de-siècle 1899 Berlin concert grand tuned to A=435 Hz with lush Romantic wool felt, velvety inner voices, and noble sustain.",
            inharmonicity_base: 0.00018,
            decay_scale: 0.98,
            string_length_m: 2.62,
            soundboard_impedance: 0.52,
            sympathetic_resonance: 0.50,
            duplex_scale_resonance: 0.32,
            blooming_energy: 0.30,
            quadratic_effect: 0.20,
            hammer_hardness: [0.34, 0.54, 0.78],
            hammer_noise: 0.31,
            hammer_tone: 0.48,
            strike_point: 0.123,
            spectrum_profile: [1.06, 0.97, 0.87, 0.74, 0.59, 0.45, 0.33, 0.23],
            diapason_hz: 435.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 1.05,
            octave_stretching: 0.98,
            dynamics_db: 40.0,
            condition: 0.12,
            damper_noise: 0.29,
            key_release_noise: 0.23,
            damping_duration: 0.50,
            lid_position: 0.98,
            stereo_width: 0.64,
            eq_gains_db: [0.6, 0.3, 0.5],
            reverb: ReverbSpec::new("Belle Époque Conservatory", 0.24, 1.95, 19.5, 15.0, 0.48, 0.51, 98.0),
            ..DEFAULT_SPEC
        },

        // =====================================================================
        // 5. Chromatic Percussion, Mallets & Steelpans (12)
        // =====================================================================
        AcousticSpec {
            id: "vibraphone-vb",
            name: "Vibraphone V-B",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletSoft,
            description: "Resonant aluminum bar vibraphone with soft mallets, pure fundamental decay, motor-fan tremolo shimmer, and lush hall ambience.",
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
            stereo_width: 0.68,
            eq_gains_db: [0.5, 0.2, -0.4],
            reverb: ReverbSpec::new("Warm Jazz Hall", 0.30, 2.30, 22.0, 17.0, 0.45, 0.50, 95.0),
            fx: FxSpec::with_tremolo(0.24, 0.55),
            bounce: BounceSpec::standby(130.0, 0.18, 0.38, 6),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "vibraphone-vm",
            name: "Vibraphone V-M",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletSoft,
            description: "Medium-cord mallet vibraphone offering clearer bar attack, subtle metallic second-partial bloom, and slower warm fan vibrato.",
            inharmonicity_base: 0.000025,
            decay_scale: 1.18,
            string_length_m: 0.50,
            soundboard_impedance: 0.28,
            sympathetic_resonance: 0.03,
            duplex_scale_resonance: 0.00,
            blooming_energy: 0.08,
            quadratic_effect: 0.07,
            hammer_hardness: [0.34, 0.56, 0.80],
            hammer_noise: 0.20,
            hammer_tone: 0.48,
            strike_point: 0.215,
            spectrum_profile: [1.20, 0.58, 0.26, 0.11, 0.04, 0.02, 0.01, 0.00],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.08,
            octave_stretching: 0.81,
            dynamics_db: 36.0,
            condition: 0.03,
            damper_noise: 0.14,
            key_release_noise: 0.10,
            damping_duration: 0.52,
            stereo_width: 0.66,
            eq_gains_db: [0.3, 0.4, 0.2],
            reverb: ReverbSpec::new("Blue Note Club", 0.27, 2.05, 19.0, 15.0, 0.47, 0.52, 100.0),
            fx: FxSpec::with_tremolo(0.28, 0.42),
            bounce: BounceSpec::standby(115.0, 0.16, 0.36, 6),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "celesta-orchestral",
            name: "Celesta",
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
            lid_position: 0.80,
            stereo_width: 0.58,
            eq_gains_db: [-0.4, 0.3, 1.2],
            reverb: ReverbSpec::new("Orchestral Pit Hall", 0.28, 2.15, 23.0, 18.0, 0.44, 0.56, 130.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "glockenspiel",
            name: "Glockenspiel",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletHard,
            description: "High-carbon orchestral steel bells struck by hard brass mallets, yielding piercing chime clarity and shimmering overtone sustain.",
            inharmonicity_base: 0.00012,
            decay_scale: 1.08,
            string_length_m: 0.32,
            soundboard_impedance: 0.22,
            sympathetic_resonance: 0.04,
            duplex_scale_resonance: 0.01,
            blooming_energy: 0.06,
            quadratic_effect: 0.09,
            hammer_hardness: [0.70, 0.86, 0.99],
            hammer_noise: 0.42,
            hammer_tone: 0.84,
            strike_point: 0.180,
            spectrum_profile: [1.18, 0.82, 0.62, 0.46, 0.30, 0.20, 0.12, 0.06],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.08,
            octave_stretching: 0.78,
            dynamics_db: 32.0,
            condition: 0.02,
            damper_noise: 0.08,
            key_release_noise: 0.05,
            damping_duration: 0.62,
            stereo_width: 0.60,
            eq_gains_db: [-1.0, 0.4, 2.2],
            reverb: ReverbSpec::new("Symphony Brass Stage", 0.31, 2.40, 26.0, 20.0, 0.43, 0.62, 165.0),
            bounce: BounceSpec::standby(95.0, 0.14, 0.32, 8),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "toy-piano",
            name: "Toy Piano",
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
            lid_position: 0.75,
            stereo_width: 0.45,
            eq_gains_db: [-1.2, 0.9, 1.8],
            reverb: ReverbSpec::new("Nursery Wood Room", 0.18, 0.95, 8.0, 6.0, 0.56, 0.60, 160.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "kalimba-african",
            name: "Kalimba",
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
            lid_position: 0.90,
            stereo_width: 0.54,
            eq_gains_db: [0.8, 0.6, 0.0],
            reverb: ReverbSpec::new("Warm Gourd Ambience", 0.20, 1.30, 11.0, 9.0, 0.50, 0.48, 110.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "marimba-concert",
            name: "Marimba",
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
            stereo_width: 0.64,
            eq_gains_db: [1.2, 0.3, -0.6],
            reverb: ReverbSpec::new("Percussion Studio", 0.24, 1.65, 16.0, 12.0, 0.48, 0.46, 85.0),
            bounce: BounceSpec::standby(110.0, 0.15, 0.35, 6),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "xylophone",
            name: "Xylophone",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletHard,
            description: "Hardwood rosewood bars quint-tuned to the third harmonic and struck by hard phenolic mallets for crisp orchestral projection.",
            inharmonicity_base: 0.00008,
            decay_scale: 0.34,
            string_length_m: 0.46,
            soundboard_impedance: 0.74,
            sympathetic_resonance: 0.03,
            duplex_scale_resonance: 0.00,
            blooming_energy: 0.04,
            quadratic_effect: 0.11,
            hammer_hardness: [0.64, 0.82, 0.98],
            hammer_noise: 0.48,
            hammer_tone: 0.78,
            strike_point: 0.250,
            spectrum_profile: [1.14, 0.28, 0.68, 0.14, 0.08, 0.04, 0.02, 0.01],
            diapason_hz: 442.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.14,
            octave_stretching: 0.80,
            dynamics_db: 34.0,
            condition: 0.04,
            damper_noise: 0.08,
            key_release_noise: 0.06,
            damping_duration: 0.48,
            stereo_width: 0.60,
            eq_gains_db: [-0.5, 1.0, 1.4],
            reverb: ReverbSpec::new("Orchestral Stage Right", 0.22, 1.50, 18.0, 14.0, 0.50, 0.55, 125.0),
            bounce: BounceSpec::standby(90.0, 0.12, 0.30, 8),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "steel-drum",
            name: "Steel Drum",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletHard,
            description: "Hand-hammered Caribbean steelpan modeled with strong blooming energy transfer, octave-coupled overtone shimmer, and courtyard ring.",
            inharmonicity_base: 0.00034,
            decay_scale: 0.74,
            string_length_m: 0.42,
            soundboard_impedance: 0.48,
            sympathetic_resonance: 0.24,
            duplex_scale_resonance: 0.16,
            blooming_energy: 0.34,
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
            stereo_width: 0.66,
            eq_gains_db: [0.2, 1.0, 1.3],
            reverb: ReverbSpec::new("Island Courtyard", 0.23, 1.50, 15.0, 12.0, 0.52, 0.57, 120.0),
            bounce: BounceSpec::standby(105.0, 0.16, 0.34, 6),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "spacedrum",
            name: "Spacedrum",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletSoft,
            description: "French hand-played steelpan shell with Helmholtz central dome resonance, velvety blooming halo, and subtle ambient delay.",
            inharmonicity_base: 0.00019,
            decay_scale: 0.94,
            string_length_m: 0.48,
            soundboard_impedance: 0.38,
            sympathetic_resonance: 0.30,
            duplex_scale_resonance: 0.20,
            blooming_energy: 0.28,
            quadratic_effect: 0.24,
            hammer_hardness: [0.30, 0.50, 0.72],
            hammer_noise: 0.24,
            hammer_tone: 0.46,
            strike_point: 0.225,
            spectrum_profile: [1.24, 0.86, 0.52, 0.38, 0.20, 0.12, 0.06, 0.03],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 1.10,
            octave_stretching: 0.90,
            dynamics_db: 36.0,
            condition: 0.06,
            damper_noise: 0.06,
            key_release_noise: 0.05,
            damping_duration: 0.56,
            stereo_width: 0.70,
            eq_gains_db: [0.8, 0.2, 0.4],
            reverb: ReverbSpec::new("Meditative Vault", 0.28, 2.15, 20.0, 16.0, 0.44, 0.50, 95.0),
            fx: FxSpec::with_delay(0.16, 0.36, 0.42),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "hand-pan",
            name: "Hand Pan",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletSoft,
            description: "Nitrided steel Hang-style idiophone played with fingertips, producing warm fundamental, coupled octave/fifth partials, and ethereal delay.",
            inharmonicity_base: 0.00016,
            decay_scale: 0.98,
            string_length_m: 0.46,
            soundboard_impedance: 0.36,
            sympathetic_resonance: 0.32,
            duplex_scale_resonance: 0.22,
            blooming_energy: 0.26,
            quadratic_effect: 0.22,
            hammer_hardness: [0.26, 0.46, 0.68],
            hammer_noise: 0.20,
            hammer_tone: 0.42,
            strike_point: 0.245,
            spectrum_profile: [1.28, 0.82, 0.60, 0.30, 0.16, 0.09, 0.04, 0.02],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.85,
            octave_stretching: 0.91,
            dynamics_db: 36.0,
            condition: 0.05,
            damper_noise: 0.05,
            key_release_noise: 0.04,
            damping_duration: 0.58,
            stereo_width: 0.72,
            eq_gains_db: [1.0, 0.1, 0.3],
            reverb: ReverbSpec::new("Brocéliande Clearing", 0.29, 2.25, 21.0, 17.0, 0.43, 0.49, 90.0),
            fx: FxSpec::with_delay(0.14, 0.32, 0.38),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "tank-drum",
            name: "Tank Drum",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletSoft,
            description: "Slit-tongue steel tank drum with rounded soft-mallet bloom, pure fundamental singing sustain, and tranquil woody harmonic decay.",
            inharmonicity_base: 0.00014,
            decay_scale: 1.04,
            string_length_m: 0.38,
            soundboard_impedance: 0.34,
            sympathetic_resonance: 0.26,
            duplex_scale_resonance: 0.12,
            blooming_energy: 0.20,
            quadratic_effect: 0.18,
            hammer_hardness: [0.24, 0.42, 0.64],
            hammer_noise: 0.18,
            hammer_tone: 0.36,
            strike_point: 0.260,
            spectrum_profile: [1.34, 0.52, 0.28, 0.14, 0.07, 0.03, 0.02, 0.01],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.65,
            octave_stretching: 0.89,
            dynamics_db: 35.0,
            condition: 0.04,
            damper_noise: 0.05,
            key_release_noise: 0.04,
            damping_duration: 0.60,
            stereo_width: 0.66,
            eq_gains_db: [1.1, 0.2, -0.3],
            reverb: ReverbSpec::new("Zen Sanctuary", 0.27, 2.10, 18.0, 14.0, 0.45, 0.47, 88.0),
            ..DEFAULT_SPEC
        },

        // =====================================================================
        // 6. Plucked Strings: Guitar & Harps (4)
        // =====================================================================
        AcousticSpec {
            id: "classical-guitar",
            name: "Classical Guitar",
            category: InstrumentCategory::Plucked,
            exciter: ExciterType::PluckPlectrum,
            description: "Nylon-string Spanish concert guitar in Guitar PlayMode with warm cedar body resonance, hammer-on/pull-off legato, and gentle stereo chorus.",
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
            lid_position: 0.95,
            stereo_width: 0.52,
            eq_gains_db: [0.7, 0.3, 0.2],
            reverb: ReverbSpec::new("Cedar Guitar Salon", 0.22, 1.60, 13.5, 11.0, 0.50, 0.49, 100.0),
            fx: FxSpec::with_chorus(0.14, 0.24),
            play_mode: PlayModeSpec::guitar(0.72, true),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "concert-harp",
            name: "Concert Harp",
            category: InstrumentCategory::Plucked,
            exciter: ExciterType::PluckPlectrum,
            description: "47-string Salvi double-action pedal harp in DiatonicHarp PlayMode with sweeping sympathetic string halo, Pythagorean purity, and stereo chorus.",
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
            stereo_width: 0.72,
            eq_gains_db: [0.8, 0.0, 0.5],
            reverb: ReverbSpec::new("Symphonic Harp Hall", 0.29, 2.45, 26.0, 20.0, 0.43, 0.52, 90.0),
            fx: FxSpec::with_chorus(0.15, 0.22),
            play_mode: PlayModeSpec::diatonic_harp(),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "celtic-harp",
            name: "Celtic Harp",
            category: InstrumentCategory::Plucked,
            exciter: ExciterType::PluckPlectrum,
            description: "34-string Aoyama nylon-strung lever harp in DiatonicHarp PlayMode with bright singing pluck, Zarlino intonation, and misty delay echo.",
            inharmonicity_base: 0.00015,
            decay_scale: 0.94,
            string_length_m: 1.28,
            soundboard_impedance: 0.50,
            sympathetic_resonance: 0.52,
            duplex_scale_resonance: 0.10,
            blooming_energy: 0.22,
            quadratic_effect: 0.13,
            hammer_hardness: [0.34, 0.52, 0.72],
            hammer_noise: 0.22,
            hammer_tone: 0.50,
            strike_point: 0.195,
            spectrum_profile: [1.14, 0.94, 0.72, 0.50, 0.34, 0.22, 0.13, 0.07],
            diapason_hz: 440.0,
            temperament: TemperamentId::Zarlino,
            unison_width_cents: 0.35,
            octave_stretching: 0.93,
            dynamics_db: 38.0,
            condition: 0.05,
            damper_noise: 0.16,
            key_release_noise: 0.14,
            damping_duration: 0.52,
            stereo_width: 0.66,
            eq_gains_db: [0.5, 0.3, 0.7],
            reverb: ReverbSpec::new("Highland Stone Keep", 0.27, 2.15, 20.5, 16.0, 0.47, 0.54, 100.0),
            fx: FxSpec::with_delay(0.14, 0.34, 0.40),
            play_mode: PlayModeSpec::diatonic_harp(),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "triple-harp",
            name: "Triple Harp",
            category: InstrumentCategory::Plucked,
            exciter: ExciterType::PluckPlectrum,
            description: "Italian Baroque three-row harp in DiatonicHarp PlayMode tuned to A=415 Hz Meantone with cascading multi-rank sympathetic shimmer.",
            inharmonicity_base: 0.00012,
            decay_scale: 1.14,
            string_length_m: 1.60,
            soundboard_impedance: 0.46,
            sympathetic_resonance: 0.72,
            duplex_scale_resonance: 0.24,
            blooming_energy: 0.26,
            quadratic_effect: 0.14,
            hammer_hardness: [0.32, 0.49, 0.70],
            hammer_noise: 0.20,
            hammer_tone: 0.46,
            strike_point: 0.202,
            spectrum_profile: [1.16, 0.90, 0.68, 0.48, 0.31, 0.19, 0.11, 0.05],
            diapason_hz: 415.0,
            temperament: TemperamentId::MeanTone,
            unison_width_cents: 0.55,
            octave_stretching: 0.91,
            dynamics_db: 37.0,
            condition: 0.08,
            damper_noise: 0.15,
            key_release_noise: 0.13,
            damping_duration: 0.56,
            stereo_width: 0.70,
            eq_gains_db: [0.6, 0.2, 0.6],
            reverb: ReverbSpec::new("Florentine Baroque Chapel", 0.28, 2.35, 22.0, 17.5, 0.46, 0.52, 95.0),
            fx: FxSpec::with_chorus(0.12, 0.20),
            play_mode: PlayModeSpec::diatonic_harp(),
            ..DEFAULT_SPEC
        },

        // =====================================================================
        // 7. Prepared & Physical Synthesizer (2)
        // =====================================================================
        AcousticSpec {
            id: "prepared-grand",
            name: "Prepared Experimental Grand",
            category: InstrumentCategory::Prepared,
            exciter: ExciterType::PreparedBolt,
            description: "Modified grand strings with metallic bolt overtones, wide unison beating, rhythmic bounce strikes, phaser sweep, and tape delay.",
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
            stereo_width: 0.78,
            eq_gains_db: [0.5, 0.8, 1.6],
            reverb: ReverbSpec::new("Avant-Garde Loft", 0.27, 2.35, 20.0, 15.0, 0.48, 0.58, 100.0),
            fx: FxSpec::custom(
                FxSlotSpec::on(EffectType::Delay, 0.20, 0.30, 0.46),
                FxSlotSpec::on(EffectType::Phaser, 0.22, 0.38, 0.52),
                FxSlotSpec::on(EffectType::Compressor, 0.44, 0.38, 0.54),
            ),
            bounce: BounceSpec::active(145.0, 0.55, 0.22, 0.45, 0.22, 3),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "syngular-hybrid",
            name: "Syngular Physical Synth",
            category: InstrumentCategory::Synthesizer,
            exciter: ExciterType::SynthHybrid,
            description: "Physical-modeling acoustic-synth hybrid combining blooming resonator harmonics, NoteFX vibrato and smear, stereo delay, chorus, and phaser.",
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
            stereo_width: 0.88,
            eq_gains_db: [1.2, 0.0, 1.4],
            reverb: ReverbSpec::new("Cosmic Cathedral", 0.34, 3.40, 32.0, 24.0, 0.42, 0.58, 85.0),
            fx: FxSpec::custom(
                FxSlotSpec::on(EffectType::Delay, 0.24, 0.44, 0.50),
                FxSlotSpec::on(EffectType::Chorus, 0.30, 0.38, 0.50),
                FxSlotSpec::on(EffectType::Phaser, 0.18, 0.26, 0.44),
            ),
            note_fx: NoteFxSpec::expressive(4.8, 140.0, 9.5, 2.5, 1.8, 0.62, 0.38),
            ..DEFAULT_SPEC
        },

        // =====================================================================
        // 8. KIViR Instruments & Bells (6)
        // =====================================================================
        AcousticSpec {
            id: "kivir-clavichord",
            name: "Clavichord",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::PluckPlectrum,
            description: "Intimate 18th-century fretted clavichord with brass tangent strike, subtle expressive pitch sensitivity, and Well-Tempered tuning at A=415 Hz.",
            inharmonicity_base: 0.00017,
            decay_scale: 0.50,
            string_length_m: 1.18,
            soundboard_impedance: 0.68,
            sympathetic_resonance: 0.20,
            duplex_scale_resonance: 0.04,
            blooming_energy: 0.07,
            quadratic_effect: 0.14,
            hammer_hardness: [0.30, 0.46, 0.64],
            hammer_noise: 0.34,
            hammer_tone: 0.52,
            strike_point: 0.145,
            spectrum_profile: [1.06, 0.88, 0.72, 0.54, 0.38, 0.25, 0.15, 0.08],
            diapason_hz: 415.0,
            temperament: TemperamentId::WellTempered,
            unison_width_cents: 1.40,
            octave_stretching: 0.86,
            dynamics_db: 24.0,
            condition: 0.16,
            damper_noise: 0.22,
            key_release_noise: 0.28,
            damping_duration: 0.34,
            lid_position: 0.85,
            stereo_width: 0.48,
            eq_gains_db: [-0.2, 0.6, 0.9],
            reverb: ReverbSpec::new("KIViR Clavichord Study", 0.19, 1.25, 10.0, 8.0, 0.54, 0.50, 120.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "kivir-cymbalum",
            name: "Cymbalum",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletHard,
            description: "Hungarian concert cimbalom hammered dulcimer with bright multi-course steel string overtones, mallet bounce flutter, and shimmering open resonance.",
            inharmonicity_base: 0.00024,
            decay_scale: 1.06,
            string_length_m: 1.35,
            soundboard_impedance: 0.46,
            sympathetic_resonance: 0.66,
            duplex_scale_resonance: 0.34,
            blooming_energy: 0.24,
            quadratic_effect: 0.25,
            hammer_hardness: [0.56, 0.76, 0.94],
            hammer_noise: 0.44,
            hammer_tone: 0.72,
            strike_point: 0.110,
            spectrum_profile: [0.94, 0.96, 1.02, 0.88, 0.74, 0.62, 0.48, 0.36],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 2.40,
            octave_stretching: 0.94,
            dynamics_db: 38.0,
            condition: 0.08,
            damper_noise: 0.18,
            key_release_noise: 0.14,
            damping_duration: 0.56,
            stereo_width: 0.72,
            eq_gains_db: [0.2, 0.5, 1.6],
            reverb: ReverbSpec::new("Budapest Salon Hall", 0.26, 1.95, 18.0, 14.0, 0.48, 0.56, 105.0),
            bounce: BounceSpec::active(92.0, 0.45, 0.14, 0.44, 0.14, 3),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "kivir-harpsichord",
            name: "Harpsichord (KIViR)",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::PluckPlectrum,
            description: "KIViR French-Italian plucked harpsichord (Blanchet/Grimaldi character) tuned to A=415 Hz Werckmeister III with crisp quill bite.",
            inharmonicity_base: 0.00012,
            decay_scale: 0.64,
            string_length_m: 2.12,
            soundboard_impedance: 0.69,
            sympathetic_resonance: 0.30,
            duplex_scale_resonance: 0.14,
            blooming_energy: 0.11,
            quadratic_effect: 0.17,
            hammer_hardness: [0.60, 0.65, 0.71],
            hammer_noise: 0.43,
            hammer_tone: 0.72,
            strike_point: 0.086,
            spectrum_profile: [0.90, 0.97, 1.02, 0.90, 0.79, 0.66, 0.52, 0.40],
            diapason_hz: 415.0,
            temperament: TemperamentId::WerckmeisterIII,
            unison_width_cents: 1.50,
            octave_stretching: 0.84,
            dynamics_db: 20.0,
            condition: 0.13,
            damper_noise: 0.34,
            key_release_noise: 0.50,
            damping_duration: 0.32,
            stereo_width: 0.60,
            eq_gains_db: [-0.5, 0.5, 1.5],
            reverb: ReverbSpec::new("KIViR Blanchet Gallery", 0.25, 2.05, 18.0, 15.0, 0.50, 0.57, 118.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "kivir-walter-pianoforte",
            name: "Pianoforte: Walter",
            category: InstrumentCategory::Historical,
            exciter: ExciterType::FeltHammer,
            description: "Anton Walter Viennese fortepiano (Mozart's concert instrument) at A=430 Hz Vallotti with crisp buckskin hammers and clear singing bass.",
            inharmonicity_base: 0.00029,
            decay_scale: 0.66,
            string_length_m: 2.02,
            soundboard_impedance: 0.65,
            sympathetic_resonance: 0.35,
            duplex_scale_resonance: 0.07,
            blooming_energy: 0.17,
            quadratic_effect: 0.16,
            hammer_hardness: [0.45, 0.65, 0.85],
            hammer_noise: 0.43,
            hammer_tone: 0.61,
            strike_point: 0.104,
            spectrum_profile: [0.96, 0.99, 0.88, 0.74, 0.56, 0.40, 0.27, 0.18],
            diapason_hz: 430.0,
            temperament: TemperamentId::Vallotti,
            unison_width_cents: 1.75,
            octave_stretching: 0.89,
            dynamics_db: 31.0,
            condition: 0.18,
            damper_noise: 0.37,
            key_release_noise: 0.29,
            damping_duration: 0.37,
            lid_position: 0.92,
            stereo_width: 0.56,
            eq_gains_db: [-0.1, 0.5, 0.9],
            reverb: ReverbSpec::new("Mozart Saal Vienna", 0.21, 1.50, 13.5, 11.0, 0.52, 0.53, 116.0),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "kivir-cp80",
            name: "Electro-acoustic piano: CP-80",
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::FeltHammer,
            description: "80s electric grand piano combining real struck harp strings and piezo pickups with stereo chorus, tremolo shimmer, and stage amp.",
            inharmonicity_base: 0.00028,
            decay_scale: 0.88,
            string_length_m: 1.56,
            soundboard_impedance: 0.56,
            sympathetic_resonance: 0.32,
            duplex_scale_resonance: 0.24,
            blooming_energy: 0.20,
            quadratic_effect: 0.26,
            hammer_hardness: [0.42, 0.64, 0.90],
            hammer_noise: 0.36,
            hammer_tone: 0.62,
            strike_point: 0.118,
            spectrum_profile: [1.04, 0.92, 0.84, 0.68, 0.52, 0.42, 0.30, 0.20],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 0.85,
            octave_stretching: 1.04,
            dynamics_db: 34.0,
            condition: 0.07,
            damper_noise: 0.28,
            key_release_noise: 0.22,
            damping_duration: 0.46,
            pickup_symmetry: 0.56,
            pickup_distance: 0.40,
            stereo_width: 0.76,
            eq_gains_db: [0.9, 0.2, 1.4],
            reverb: ReverbSpec::new("80s Arena Stage", 0.22, 1.85, 20.0, 16.0, 0.46, 0.58, 120.0),
            fx: FxSpec::custom(
                FxSlotSpec::on(EffectType::Chorus, 0.28, 0.34, 0.48),
                FxSlotSpec::on(EffectType::Tremolo, 0.20, 0.42, 0.45),
                FxSlotSpec::on(EffectType::Amp, 0.38, 0.34, 0.56),
            ),
            ..DEFAULT_SPEC
        },
        AcousticSpec {
            id: "kivir-tubular-bells",
            name: "Tubular Bells",
            category: InstrumentCategory::Mallet,
            exciter: ExciterType::MalletHard,
            description: "Orchestral cathedral chime tubes struck by rawhide mallets with majestic inharmonic bell overtones and sweeping cathedral decay.",
            inharmonicity_base: 0.00088,
            decay_scale: 1.48,
            string_length_m: 1.90,
            soundboard_impedance: 0.24,
            sympathetic_resonance: 0.22,
            duplex_scale_resonance: 0.14,
            blooming_energy: 0.16,
            quadratic_effect: 0.20,
            hammer_hardness: [0.60, 0.80, 0.98],
            hammer_noise: 0.45,
            hammer_tone: 0.76,
            strike_point: 0.185,
            spectrum_profile: [0.90, 1.08, 0.78, 1.12, 0.62, 0.48, 0.32, 0.22],
            diapason_hz: 440.0,
            temperament: TemperamentId::Equal,
            unison_width_cents: 1.80,
            octave_stretching: 0.92,
            dynamics_db: 38.0,
            condition: 0.04,
            damper_noise: 0.12,
            key_release_noise: 0.08,
            damping_duration: 0.64,
            stereo_width: 0.74,
            eq_gains_db: [0.6, 0.3, 1.2],
            reverb: ReverbSpec::new("Cathedral Bell Tower", 0.38, 4.20, 38.0, 28.0, 0.42, 0.55, 80.0),
            ..DEFAULT_SPEC
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
            instruments.len() >= 62,
            "Expected at least 62 factory instruments, found {}",
            instruments.len()
        );

        let expected_ids = [
            // Grand Pianos (14)
            "grand-ny-d",
            "grand-hb-d",
            "grand-steinway-b",
            "grand-sk-ex",
            "grand-yfx",
            "grand-bosendorfer-280vc",
            "grand-bechstein-dg",
            "grand-petrof-275",
            "grand-petrof-mistral",
            "grand-steingraeber-e272",
            "grand-grotrian-royal",
            "grand-bluthner-1",
            "grand-k2",
            "grand-yc5-rock",
            // Uprights (2)
            "upright-u4",
            "upright-u4-honky",
            // Electric Pianos (9)
            "vintage-tines-mk2",
            "vintage-tines-mk1",
            "vintage-reeds-200",
            "vintage-reeds-140b",
            "clavinet-d6",
            "pianet-n",
            "pianet-t",
            "electra-piano",
            "kivir-cp80",
            // Historical & KIViR Collection (17)
            "ruckers-harpsichord",
            "salodiensis-virginal-1600",
            "giusti-harpsichord-1680",
            "schmidt-square-1780",
            "dohnal-1795",
            "broadwood-1796",
            "wiener-pianoforte-1808",
            "besendorfer-1829",
            "pleyel-1835",
            "frenzel-1841",
            "erard-grand-1849",
            "streicher-1852",
            "ny-steinway-square-1858",
            "bechstein-1899",
            "kivir-clavichord",
            "kivir-harpsichord",
            "kivir-walter-pianoforte",
            // Mallets & Idiophones (14)
            "vibraphone-vb",
            "vibraphone-vm",
            "celesta-orchestral",
            "glockenspiel",
            "toy-piano",
            "kalimba-african",
            "marimba-concert",
            "xylophone",
            "steel-drum",
            "spacedrum",
            "hand-pan",
            "tank-drum",
            "kivir-cymbalum",
            "kivir-tubular-bells",
            // Plucked (4)
            "classical-guitar",
            "concert-harp",
            "celtic-harp",
            "triple-harp",
            // Prepared & Synthesizer (2)
            "prepared-grand",
            "syngular-hybrid",
        ];

        assert_eq!(expected_ids.len(), 62);

        let mut seen_ids = HashSet::new();
        let mut seen_names = HashSet::new();
        let mut seen_reverbs = HashSet::new();
        for inst in &instruments {
            assert!(!inst.id.trim().is_empty(), "Instrument ID must not be empty");
            assert!(
                !inst.name.trim().is_empty(),
                "Instrument name must not be empty for {}",
                inst.id
            );
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
            assert!(
                seen_reverbs.insert(inst.default_params.reverb.preset_name.clone()),
                "Duplicate reverb preset_name detected: {}",
                inst.default_params.reverb.preset_name
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
    fn all_eight_instrument_categories_and_exciters_covered() {
        let instruments = build_factory_instruments();

        let count_cat = |cat: InstrumentCategory| {
            instruments.iter().filter(|i| i.category == cat).count()
        };

        assert_eq!(count_cat(InstrumentCategory::GrandPiano), 14);
        assert_eq!(count_cat(InstrumentCategory::UprightPiano), 2);
        assert_eq!(count_cat(InstrumentCategory::ElectricPiano), 9);
        assert_eq!(count_cat(InstrumentCategory::Historical), 17);
        assert_eq!(count_cat(InstrumentCategory::Mallet), 14);
        assert_eq!(count_cat(InstrumentCategory::Plucked), 4);
        assert_eq!(count_cat(InstrumentCategory::Prepared), 1);
        assert_eq!(count_cat(InstrumentCategory::Synthesizer), 1);

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
    fn acoustic_differentiation_and_cycle2_feature_coverage() {
        let instruments = build_factory_instruments();
        let by_id = |id: &str| -> &InstrumentModel {
            instruments
                .iter()
                .find(|i| i.id == id)
                .unwrap_or_else(|| panic!("Missing instrument {}", id))
        };

        // Uprights have shorter strings and higher inharmonicity than all 14 grand pianos
        let grands: Vec<&InstrumentModel> = instruments
            .iter()
            .filter(|i| i.category == InstrumentCategory::GrandPiano)
            .collect();
        for upright in instruments
            .iter()
            .filter(|i| i.category == InstrumentCategory::UprightPiano)
        {
            for grand in &grands {
                assert!(
                    upright.default_params.design.string_length_m
                        < grand.default_params.design.string_length_m,
                    "Upright {} length should be shorter than grand {}",
                    upright.id,
                    grand.id
                );
                assert!(
                    upright.inharmonicity_base > grand.inharmonicity_base,
                    "Upright {} inharmonicity should exceed grand {}",
                    upright.id,
                    grand.id
                );
            }
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

        // Historical models use authentic pre-440Hz period pitch
        for hist in instruments
            .iter()
            .filter(|i| i.category == InstrumentCategory::Historical)
        {
            assert!(
                hist.default_params.tuning.diapason_hz < 440.0,
                "Historical {} should use period diapason < 440 Hz",
                hist.id
            );
        }

        // Extended keyboard range on Bösendorfer 280VC & Syngular Hybrid
        assert!(by_id("grand-bosendorfer-280vc").default_params.action.keyboard_range_extended);
        assert!(by_id("syngular-hybrid").default_params.action.keyboard_range_extended);

        // Plucked PlayModes
        assert_eq!(by_id("classical-guitar").default_params.play_mode.mode, PlayMode::Guitar);
        assert_eq!(by_id("concert-harp").default_params.play_mode.mode, PlayMode::DiatonicHarp);
        assert_eq!(by_id("celtic-harp").default_params.play_mode.mode, PlayMode::DiatonicHarp);
        assert_eq!(by_id("triple-harp").default_params.play_mode.mode, PlayMode::DiatonicHarp);

        // Prepared bounce & Syngular NoteFX vibrato/smear
        assert!(by_id("prepared-grand").default_params.bounce.enabled);
        assert!(by_id("syngular-hybrid").default_params.note_fx.vibrato_cents > 0.0);
        assert!(by_id("syngular-hybrid").default_params.note_fx.smear > 0.0);

        // Electric Piano FX showcase: Chorus, Phaser, Wah, Amp, Delay, Tremolo, Compressor
        let active_fx_types: Vec<EffectType> = instruments
            .iter()
            .flat_map(|i| i.default_params.fx.iter())
            .filter(|slot| slot.enabled)
            .map(|slot| slot.effect_type)
            .collect();
        for required_fx in [
            EffectType::Chorus,
            EffectType::Phaser,
            EffectType::Wah,
            EffectType::Amp,
            EffectType::Delay,
            EffectType::Tremolo,
            EffectType::Compressor,
        ] {
            assert!(
                active_fx_types.contains(&required_fx),
                "Expected active FX showcase for {:?}",
                required_fx
            );
        }

        // Specific authentic Clavinet Wah + Amp + Compressor check
        let clav = by_id("clavinet-d6");
        assert!(clav.default_params.fx[0].enabled && clav.default_params.fx[0].effect_type == EffectType::Wah);
        assert!(clav.default_params.fx[1].enabled && clav.default_params.fx[1].effect_type == EffectType::Amp);
        assert!(clav.default_params.fx[2].enabled && clav.default_params.fx[2].effect_type == EffectType::Compressor);

        // KIViR Pack Instruments & Bells checks
        let cp80 = by_id("kivir-cp80");
        assert!(cp80.default_params.fx[0].enabled && cp80.default_params.fx[0].effect_type == EffectType::Chorus);
        assert!(cp80.default_params.fx[1].enabled && cp80.default_params.fx[1].effect_type == EffectType::Tremolo);
        assert!(cp80.default_params.fx[2].enabled && cp80.default_params.fx[2].effect_type == EffectType::Amp);

        let cymb = by_id("kivir-cymbalum");
        assert!(cymb.default_params.bounce.enabled);
        assert!(cymb.default_params.design.sympathetic_resonance >= 0.60);

        let bells = by_id("kivir-tubular-bells");
        assert!(bells.inharmonicity_base > 0.0005);
        assert!(bells.decay_scale >= 1.40);
        assert!(bells.default_params.reverb.duration_s >= 3.5);
    }
}
