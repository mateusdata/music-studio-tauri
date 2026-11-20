use crate::contracts::{
    EngineParams, ExciterType, InstrumentCategory, InstrumentModel, TemperamentId,
};

pub fn build_factory_instruments() -> Vec<InstrumentModel> {
    let mut list = Vec::new();

    // 1. Concert Grand NY
    let mut ny_params = EngineParams::default();
    ny_params.design.string_length_m = 2.74;
    ny_params.voicing.hammer_hardness_piano = 0.40;
    ny_params.voicing.hammer_hardness_mezzo = 0.62;
    ny_params.voicing.hammer_hardness_forte = 0.88;
    list.push(InstrumentModel {
        id: "grand-ny-d".to_string(),
        name: "Grand Concert NY D".to_string(),
        category: InstrumentCategory::GrandPiano,
        exciter: ExciterType::FeltHammer,
        description: "9-foot New York concert grand modeled with dynamic felt hammers, crisp attack, and rich soundboard resonance.".to_string(),
        inharmonicity_base: 0.00018,
        decay_scale: 1.05,
        default_params: ny_params,
    });

    // 2. Concert Grand Hamburg
    let mut hb_params = EngineParams::default();
    hb_params.design.string_length_m = 2.74;
    hb_params.voicing.hammer_hardness_piano = 0.34;
    hb_params.voicing.hammer_hardness_mezzo = 0.54;
    hb_params.voicing.hammer_hardness_forte = 0.78;
    hb_params.voicing.spectrum_profile = [1.05, 0.96, 0.88, 0.75, 0.58, 0.44, 0.32, 0.25];
    list.push(InstrumentModel {
        id: "grand-hb-d".to_string(),
        name: "Grand Concert HB D".to_string(),
        category: InstrumentCategory::GrandPiano,
        exciter: ExciterType::FeltHammer,
        description: "Warm, singing European concert grand with velvety upper register and balanced sympathetic sustain.".to_string(),
        inharmonicity_base: 0.00016,
        decay_scale: 1.12,
        default_params: hb_params,
    });

    // 3. Upright Studio U4
    let mut u4_params = EngineParams::default();
    u4_params.design.string_length_m = 1.31;
    u4_params.design.soundboard_impedance = 0.64;
    u4_params.voicing.hammer_noise = 0.48;
    u4_params.action.damper_noise = 0.42;
    list.push(InstrumentModel {
        id: "upright-u4".to_string(),
        name: "Studio Upright U4".to_string(),
        category: InstrumentCategory::UprightPiano,
        exciter: ExciterType::FeltHammer,
        description: "Intimate studio upright piano with shorter strings, higher inharmonicity, and distinct mechanical action.".to_string(),
        inharmonicity_base: 0.00042,
        decay_scale: 0.78,
        default_params: u4_params,
    });

    // 4. Vintage Tines MKII
    let mut tines_params = EngineParams::default();
    tines_params.voicing.spectrum_profile = [1.2, 0.9, 0.45, 0.25, 0.15, 0.08, 0.05, 0.03];
    tines_params.design.sympathetic_resonance = 0.08;
    tines_params.fx[1].enabled = true;
    tines_params.reverb.mix = 0.18;
    list.push(InstrumentModel {
        id: "vintage-tines-mk2".to_string(),
        name: "Vintage Tines MKII".to_string(),
        category: InstrumentCategory::ElectricPiano,
        exciter: ExciterType::TineHammer,
        description: "Electro-acoustic tine piano with bell-like attack, asymmetric pickup warmth, and stereo chorus.".to_string(),
        inharmonicity_base: 0.00004,
        decay_scale: 0.85,
        default_params: tines_params,
    });

    // 5. Vintage Reeds 200A
    let mut reeds_params = EngineParams::default();
    reeds_params.voicing.spectrum_profile = [1.1, 0.75, 0.85, 0.40, 0.55, 0.22, 0.14, 0.08];
    reeds_params.design.sympathetic_resonance = 0.04;
    list.push(InstrumentModel {
        id: "vintage-reeds-200".to_string(),
        name: "Vintage Reeds 200".to_string(),
        category: InstrumentCategory::ElectricPiano,
        exciter: ExciterType::ReedHammer,
        description: "Punchy reed electric piano with classic bark under hard velocities.".to_string(),
        inharmonicity_base: 0.00006,
        decay_scale: 0.72,
        default_params: reeds_params,
    });

    // 6. Historical Harpsichord
    let mut harp_params = EngineParams::default();
    harp_params.tuning.diapason_hz = 415.0;
    harp_params.tuning.temperament = TemperamentId::WerckmeisterIII;
    harp_params.voicing.strike_point = 0.08;
    harp_params.voicing.spectrum_profile = [0.85, 0.95, 1.0, 0.92, 0.85, 0.75, 0.65, 0.55];
    list.push(InstrumentModel {
        id: "ruckers-harpsichord".to_string(),
        name: "Flemish Harpsichord II".to_string(),
        category: InstrumentCategory::Historical,
        exciter: ExciterType::PluckPlectrum,
        description: "Plucked baroque harpsichord tuned to A=415 Hz with Werckmeister III temperament.".to_string(),
        inharmonicity_base: 0.00011,
        decay_scale: 0.62,
        default_params: harp_params,
    });

    // 7. Concert Vibraphone
    let mut vibe_params = EngineParams::default();
    vibe_params.voicing.spectrum_profile = [1.25, 0.45, 0.18, 0.06, 0.02, 0.01, 0.0, 0.0];
    vibe_params.design.sympathetic_resonance = 0.02;
    vibe_params.reverb.mix = 0.30;
    list.push(InstrumentModel {
        id: "vibraphone-vb".to_string(),
        name: "Concert Vibraphone V-B".to_string(),
        category: InstrumentCategory::Mallet,
        exciter: ExciterType::MalletSoft,
        description: "Resonant aluminum bar vibraphone with pure fundamental decay and lush hall ambience.".to_string(),
        inharmonicity_base: 0.00002,
        decay_scale: 1.25,
        default_params: vibe_params,
    });

    // 8. Classical Concert Guitar
    let mut gtr_params = EngineParams::default();
    gtr_params.voicing.strike_point = 0.16;
    gtr_params.design.string_length_m = 0.65;
    gtr_params.voicing.spectrum_profile = [1.1, 0.88, 0.70, 0.52, 0.38, 0.25, 0.16, 0.10];
    list.push(InstrumentModel {
        id: "classical-guitar".to_string(),
        name: "Classical Concert Guitar".to_string(),
        category: InstrumentCategory::Plucked,
        exciter: ExciterType::PluckPlectrum,
        description: "Nylon-string concert guitar model with warm wooden body impedance.".to_string(),
        inharmonicity_base: 0.00022,
        decay_scale: 0.65,
        default_params: gtr_params,
    });

    // 9. Prepared Experimental Grand
    let mut prep_params = EngineParams::default();
    prep_params.condition = 0.55;
    prep_params.tuning.unison_width_cents = 14.0;
    prep_params.voicing.strike_point = 0.31;
    prep_params.voicing.spectrum_profile = [0.8, 1.15, 0.45, 1.2, 0.35, 1.1, 0.9, 1.15];
    list.push(InstrumentModel {
        id: "prepared-grand".to_string(),
        name: "Prepared Experimental Grand".to_string(),
        category: InstrumentCategory::Prepared,
        exciter: ExciterType::PreparedBolt,
        description: "Modified grand strings with metallic bolt overtones, wide unison beating, and percussive timbre.".to_string(),
        inharmonicity_base: 0.00078,
        decay_scale: 0.70,
        default_params: prep_params,
    });

    list
}
