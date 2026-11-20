use crate::contracts::{EngineParams, InstrumentModel, PresetData};
use std::path::PathBuf;

pub fn build_factory_presets(instruments: &[InstrumentModel]) -> Vec<PresetData> {
    let mut presets = Vec::new();
    for inst in instruments {
        presets.push(PresetData {
            id: format!("{}-default", inst.id),
            name: format!("{} Studio", inst.name),
            instrument_id: inst.id.clone(),
            category: "Studio".to_string(),
            description: format!("Balanced factory studio voicing for {}", inst.name),
            params: inst.default_params.clone(),
            is_factory: true,
        });

        // Felt / Intimate variant for pianos
        let mut felt = inst.default_params.clone();
        felt.voicing.hammer_hardness_piano *= 0.68;
        felt.voicing.hammer_hardness_mezzo *= 0.72;
        felt.voicing.hammer_hardness_forte *= 0.75;
        felt.voicing.hammer_noise = (felt.voicing.hammer_noise + 0.15).min(1.0);
        felt.output.lid_position = 0.35;
        felt.reverb.mix = 0.28;
        presets.push(PresetData {
            id: format!("{}-felt", inst.id),
            name: format!("{} Intimate Felt", inst.name),
            instrument_id: inst.id.clone(),
            category: "Intimate".to_string(),
            description: format!("Softened felt hammers and close microphone intimacy for {}", inst.name),
            params: felt,
            is_factory: true,
        });

        // Concert / Binaural variant
        let mut binaural = inst.default_params.clone();
        binaural.output.binaural_mode = true;
        binaural.output.stereo_width = 0.80;
        binaural.reverb.duration_s = 2.4;
        binaural.reverb.mix = 0.26;
        presets.push(PresetData {
            id: format!("{}-binaural", inst.id),
            name: format!("{} Binaural Player", inst.name),
            instrument_id: inst.id.clone(),
            category: "Binaural".to_string(),
            description: format!("Player-perspective 3D binaural acoustic stage for {}", inst.name),
            params: binaural,
            is_factory: true,
        });
    }
    presets
}

/// Linearly interpolates between Preset A and Preset B using morph factor `t` in [0.0, 1.0].
pub fn morph_engine_params(a: &EngineParams, b: &EngineParams, t: f32) -> EngineParams {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    let mut out = if t < 0.5 { a.clone() } else { b.clone() };
    out.volume_db = lerp(a.volume_db, b.volume_db);
    out.dynamics_db = lerp(a.dynamics_db, b.dynamics_db);
    out.velocity_offset = lerp(a.velocity_offset, b.velocity_offset);
    out.condition = lerp(a.condition, b.condition);
    out.post_effect_gain_db = lerp(a.post_effect_gain_db, b.post_effect_gain_db);
    out.morphing = t;

    out.tuning.diapason_hz = lerp(a.tuning.diapason_hz, b.tuning.diapason_hz);
    out.tuning.unison_width_cents = lerp(a.tuning.unison_width_cents, b.tuning.unison_width_cents);
    out.tuning.octave_stretching = lerp(a.tuning.octave_stretching, b.tuning.octave_stretching);

    out.voicing.hammer_hardness_piano = lerp(a.voicing.hammer_hardness_piano, b.voicing.hammer_hardness_piano);
    out.voicing.hammer_hardness_mezzo = lerp(a.voicing.hammer_hardness_mezzo, b.voicing.hammer_hardness_mezzo);
    out.voicing.hammer_hardness_forte = lerp(a.voicing.hammer_hardness_forte, b.voicing.hammer_hardness_forte);
    out.voicing.hammer_noise = lerp(a.voicing.hammer_noise, b.voicing.hammer_noise);
    out.voicing.hammer_tone = lerp(a.voicing.hammer_tone, b.voicing.hammer_tone);
    out.voicing.strike_point = lerp(a.voicing.strike_point, b.voicing.strike_point);
    for i in 0..8 {
        out.voicing.spectrum_profile[i] = lerp(a.voicing.spectrum_profile[i], b.voicing.spectrum_profile[i]);
    }

    out.design.soundboard_impedance = lerp(a.design.soundboard_impedance, b.design.soundboard_impedance);
    out.design.string_length_m = lerp(a.design.string_length_m, b.design.string_length_m);
    out.design.sympathetic_resonance = lerp(a.design.sympathetic_resonance, b.design.sympathetic_resonance);
    out.design.duplex_scale_resonance = lerp(a.design.duplex_scale_resonance, b.design.duplex_scale_resonance);
    out.design.blooming_energy = lerp(a.design.blooming_energy, b.design.blooming_energy);

    out.output.lid_position = lerp(a.output.lid_position, b.output.lid_position);
    out.output.stereo_width = lerp(a.output.stereo_width, b.output.stereo_width);
    out.reverb.mix = lerp(a.reverb.mix, b.reverb.mix);
    out.reverb.duration_s = lerp(a.reverb.duration_s, b.reverb.duration_s);
    out
}

pub fn user_presets_dir() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("piano-tect-tauri").join("presets")
}

pub fn save_user_preset(preset: &PresetData) -> Result<PathBuf, String> {
    let dir = user_presets_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let safe_id: String = preset
        .id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let path = dir.join(format!("{}.json", safe_id));
    let json = serde_json::to_string_pretty(preset).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn load_user_presets() -> Vec<PresetData> {
    let dir = user_presets_dir();
    let mut loaded = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(raw) = std::fs::read_to_string(entry.path()) {
                    if let Ok(preset) = serde_json::from_str::<PresetData>(&raw) {
                        loaded.push(preset);
                    }
                }
            }
        }
    }
    loaded
}
