use crate::contracts::{EffectSlot, EffectType, EngineParams, InstrumentCategory, InstrumentModel, PresetData};
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
        felt.reverb.preset_name = "Chamber".to_string();
        felt.reverb.mix = 0.28;
        presets.push(PresetData {
            id: format!("{}-felt", inst.id),
            name: format!("{} Intimate Felt", inst.name),
            instrument_id: inst.id.clone(),
            category: "Intimate".to_string(),
            description: format!("Softened felt hammers and close chamber intimacy for {}", inst.name),
            params: felt,
            is_factory: true,
        });

        // Binaural Player variant
        let mut binaural = inst.default_params.clone();
        binaural.output.binaural_mode = true;
        binaural.output.stereo_width = 0.80;
        binaural.reverb.preset_name = "Concert Hall".to_string();
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

        // Grand Concert Stage variant
        let mut concert = inst.default_params.clone();
        concert.output.lid_position = 1.0;
        concert.output.stereo_width = 0.76;
        concert.output.mics[2].enabled = true;
        concert.output.mics[2].volume = 0.42;
        concert.reverb.enabled = true;
        concert.reverb.preset_name = "Concert Hall".to_string();
        concert.reverb.duration_s = 2.7;
        concert.reverb.room_size_m = 28.0;
        concert.reverb.pre_delay_ms = 22.0;
        concert.reverb.early_reflections = 0.62;
        concert.reverb.mix = 0.32;
        presets.push(PresetData {
            id: format!("{}-concert", inst.id),
            name: format!("{} Concert Stage", inst.name),
            instrument_id: inst.id.clone(),
            category: "Concert".to_string(),
            description: format!("Full-lid concert hall ambience with room ribbon microphone for {}", inst.name),
            params: concert,
            is_factory: true,
        });

        // FX Showcase variant tailored to instrument category
        let mut fx_params = inst.default_params.clone();
        let (fx_name, fx_desc) = match inst.category {
            InstrumentCategory::ElectricPiano | InstrumentCategory::Synthesizer => {
                fx_params.fx[0] = EffectSlot {
                    enabled: true,
                    effect_type: EffectType::Phaser,
                    mix: 0.55,
                    param_1: 0.28,
                    param_2: 0.72,
                    param_3: 0.52,
                    param_4: 0.48,
                };
                fx_params.fx[1] = EffectSlot {
                    enabled: true,
                    effect_type: EffectType::Amp,
                    mix: 0.65,
                    param_1: 0.38,
                    param_2: 0.62,
                    param_3: 0.55,
                    param_4: 0.45,
                };
                fx_params.fx[2] = EffectSlot {
                    enabled: true,
                    effect_type: EffectType::Delay,
                    mix: 0.30,
                    param_1: 0.34,
                    param_2: 0.42,
                    param_3: 0.65,
                    param_4: 0.85,
                };
                fx_params.reverb.enabled = true;
                fx_params.reverb.preset_name = "Spring".to_string();
                fx_params.reverb.mix = 0.26;
                (
                    format!("{} Vintage Phaser & Amp", inst.name),
                    format!("Classic stereo phaser, warm tube amp cabinet, and spring reverb for {}", inst.name),
                )
            }
            InstrumentCategory::Plucked | InstrumentCategory::Prepared => {
                fx_params.fx[0] = EffectSlot {
                    enabled: true,
                    effect_type: EffectType::Wah,
                    mix: 0.60,
                    param_1: 0.35,
                    param_2: 0.78,
                    param_3: 0.68,
                    param_4: 0.58,
                };
                fx_params.fx[1] = EffectSlot {
                    enabled: true,
                    effect_type: EffectType::Flanger,
                    mix: 0.42,
                    param_1: 0.22,
                    param_2: 0.55,
                    param_3: 0.70,
                    param_4: 0.50,
                };
                fx_params.fx[2] = EffectSlot {
                    enabled: true,
                    effect_type: EffectType::Delay,
                    mix: 0.36,
                    param_1: 0.38,
                    param_2: 0.50,
                    param_3: 0.60,
                    param_4: 0.90,
                };
                fx_params.reverb.enabled = true;
                fx_params.reverb.preset_name = "Taj Mahal".to_string();
                fx_params.reverb.duration_s = 3.2;
                fx_params.reverb.mix = 0.34;
                (
                    format!("{} Ambient Wah & Echo", inst.name),
                    format!("Envelope auto-wah, stereo flanger, ping-pong echo, and Taj Mahal dome for {}", inst.name),
                )
            }
            _ => {
                fx_params.fx[0] = EffectSlot {
                    enabled: true,
                    effect_type: EffectType::Chorus,
                    mix: 0.28,
                    param_1: 0.24,
                    param_2: 0.48,
                    param_3: 0.40,
                    param_4: 0.50,
                };
                fx_params.fx[1] = EffectSlot {
                    enabled: true,
                    effect_type: EffectType::Delay,
                    mix: 0.32,
                    param_1: 0.40,
                    param_2: 0.45,
                    param_3: 0.55,
                    param_4: 0.85,
                };
                fx_params.fx[2] = EffectSlot {
                    enabled: true,
                    effect_type: EffectType::Compressor,
                    mix: 0.45,
                    param_1: 0.35,
                    param_2: 0.40,
                    param_3: 0.40,
                    param_4: 0.50,
                };
                fx_params.reverb.enabled = true;
                fx_params.reverb.preset_name = "Cathedral".to_string();
                fx_params.reverb.duration_s = 3.6;
                fx_params.reverb.room_size_m = 38.0;
                fx_params.reverb.mix = 0.38;
                (
                    format!("{} Cathedral Echo", inst.name),
                    format!("Lush stereo chorus, ping-pong delay, and expansive Cathedral acoustics for {}", inst.name),
                )
            }
        };
        presets.push(PresetData {
            id: format!("{}-fx", inst.id),
            name: fx_name,
            instrument_id: inst.id.clone(),
            category: "FX Showcase".to_string(),
            description: fx_desc,
            params: fx_params,
            is_factory: true,
        });
    }
    presets
}

/// Linearly interpolates between Preset A and Preset B using morph factor `t` in [0.0, 1.0].
pub fn morph_engine_params(a: &EngineParams, b: &EngineParams, t: f32) -> EngineParams {
    let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 };
    let lerp = |x: f32, y: f32| {
        if t <= 0.0 {
            x
        } else if t >= 1.0 {
            y
        } else {
            x + (y - x) * t
        }
    };
    let mut out = if t < 0.5 { a.clone() } else { b.clone() };
    out.volume_db = lerp(a.volume_db, b.volume_db);
    out.dynamics_db = lerp(a.dynamics_db, b.dynamics_db);
    out.velocity_offset = lerp(a.velocity_offset, b.velocity_offset);
    out.condition = lerp(a.condition, b.condition);
    out.post_effect_gain_db = lerp(a.post_effect_gain_db, b.post_effect_gain_db);
    out.morphing = t;

    out.tuning.diapason_hz = lerp(a.tuning.diapason_hz, b.tuning.diapason_hz);
    out.tuning.unison_width_cents = lerp(a.tuning.unison_width_cents, b.tuning.unison_width_cents);
    out.tuning.unison_balance = lerp(a.tuning.unison_balance, b.tuning.unison_balance);
    out.tuning.octave_stretching = lerp(a.tuning.octave_stretching, b.tuning.octave_stretching);
    out.tuning.direct_sound_duration = lerp(a.tuning.direct_sound_duration, b.tuning.direct_sound_duration);
    out.tuning.pitch_bend_semitones = lerp(a.tuning.pitch_bend_semitones, b.tuning.pitch_bend_semitones);

    out.voicing.hammer_hardness_piano = lerp(a.voicing.hammer_hardness_piano, b.voicing.hammer_hardness_piano);
    out.voicing.hammer_hardness_mezzo = lerp(a.voicing.hammer_hardness_mezzo, b.voicing.hammer_hardness_mezzo);
    out.voicing.hammer_hardness_forte = lerp(a.voicing.hammer_hardness_forte, b.voicing.hammer_hardness_forte);
    out.voicing.hammer_noise = lerp(a.voicing.hammer_noise, b.voicing.hammer_noise);
    out.voicing.hammer_tone = lerp(a.voicing.hammer_tone, b.voicing.hammer_tone);
    out.voicing.strike_point = lerp(a.voicing.strike_point, b.voicing.strike_point);
    out.voicing.soft_level = lerp(a.voicing.soft_level, b.voicing.soft_level);
    out.voicing.pickup_symmetry = lerp(a.voicing.pickup_symmetry, b.voicing.pickup_symmetry);
    out.voicing.pickup_distance = lerp(a.voicing.pickup_distance, b.voicing.pickup_distance);
    for i in 0..8 {
        out.voicing.spectrum_profile[i] = lerp(a.voicing.spectrum_profile[i], b.voicing.spectrum_profile[i]);
    }

    out.design.soundboard_impedance = lerp(a.design.soundboard_impedance, b.design.soundboard_impedance);
    out.design.impedance_cutoff = lerp(a.design.impedance_cutoff, b.design.impedance_cutoff);
    out.design.impedance_slope = lerp(a.design.impedance_slope, b.design.impedance_slope);
    out.design.string_length_m = lerp(a.design.string_length_m, b.design.string_length_m);
    out.design.sympathetic_resonance = lerp(a.design.sympathetic_resonance, b.design.sympathetic_resonance);
    out.design.duplex_scale_resonance = lerp(a.design.duplex_scale_resonance, b.design.duplex_scale_resonance);
    out.design.blooming_energy = lerp(a.design.blooming_energy, b.design.blooming_energy);
    out.design.blooming_inertia = lerp(a.design.blooming_inertia, b.design.blooming_inertia);
    out.design.quadratic_effect = lerp(a.design.quadratic_effect, b.design.quadratic_effect);

    out.action.damper_position = lerp(a.action.damper_position, b.action.damper_position);
    out.action.damper_noise = lerp(a.action.damper_noise, b.action.damper_noise);
    out.action.pedal_noise = lerp(a.action.pedal_noise, b.action.pedal_noise);
    out.action.key_release_noise = lerp(a.action.key_release_noise, b.action.key_release_noise);
    out.action.damping_duration = lerp(a.action.damping_duration, b.action.damping_duration);
    out.action.mute = lerp(a.action.mute, b.action.mute);

    out.output.lid_position = lerp(a.output.lid_position, b.output.lid_position);
    out.output.stereo_width = lerp(a.output.stereo_width, b.output.stereo_width);
    out.output.head_x = lerp(a.output.head_x, b.output.head_x);
    out.output.head_y = lerp(a.output.head_y, b.output.head_y);
    out.output.head_z = lerp(a.output.head_z, b.output.head_z);
    out.output.head_diameter = lerp(a.output.head_diameter, b.output.head_diameter);
    out.output.head_angle = lerp(a.output.head_angle, b.output.head_angle);
    for i in 0..3 {
        out.output.mics[i].x = lerp(a.output.mics[i].x, b.output.mics[i].x);
        out.output.mics[i].y = lerp(a.output.mics[i].y, b.output.mics[i].y);
        out.output.mics[i].z = lerp(a.output.mics[i].z, b.output.mics[i].z);
        out.output.mics[i].azimuth = lerp(a.output.mics[i].azimuth, b.output.mics[i].azimuth);
        out.output.mics[i].elevation = lerp(a.output.mics[i].elevation, b.output.mics[i].elevation);
        out.output.mics[i].volume = lerp(a.output.mics[i].volume, b.output.mics[i].volume);
        out.output.mics[i].pan = lerp(a.output.mics[i].pan, b.output.mics[i].pan);
    }

    for i in 0..3 {
        out.eq_bands[i].freq_hz = lerp(a.eq_bands[i].freq_hz, b.eq_bands[i].freq_hz);
        out.eq_bands[i].gain_db = lerp(a.eq_bands[i].gain_db, b.eq_bands[i].gain_db);
        out.eq_bands[i].q = lerp(a.eq_bands[i].q, b.eq_bands[i].q);
    }

    for i in 0..3 {
        out.fx[i].mix = lerp(a.fx[i].mix, b.fx[i].mix);
        out.fx[i].param_1 = lerp(a.fx[i].param_1, b.fx[i].param_1);
        out.fx[i].param_2 = lerp(a.fx[i].param_2, b.fx[i].param_2);
        out.fx[i].param_3 = lerp(a.fx[i].param_3, b.fx[i].param_3);
        out.fx[i].param_4 = lerp(a.fx[i].param_4, b.fx[i].param_4);
    }

    out.reverb.mix = lerp(a.reverb.mix, b.reverb.mix);
    out.reverb.duration_s = lerp(a.reverb.duration_s, b.reverb.duration_s);
    out.reverb.room_size_m = lerp(a.reverb.room_size_m, b.reverb.room_size_m);
    out.reverb.pre_delay_ms = lerp(a.reverb.pre_delay_ms, b.reverb.pre_delay_ms);
    out.reverb.early_reflections = lerp(a.reverb.early_reflections, b.reverb.early_reflections);
    out.reverb.tone = lerp(a.reverb.tone, b.reverb.tone);
    out.reverb.highpass_hz = lerp(a.reverb.highpass_hz, b.reverb.highpass_hz);

    out.limiter.threshold_db = lerp(a.limiter.threshold_db, b.limiter.threshold_db);
    out.limiter.sharpness = lerp(a.limiter.sharpness, b.limiter.sharpness);
    out.limiter.gain_db = lerp(a.limiter.gain_db, b.limiter.gain_db);

    out.bounce.delay_ms = lerp(a.bounce.delay_ms, b.bounce.delay_ms);
    out.bounce.velocity_sensitivity = lerp(a.bounce.velocity_sensitivity, b.bounce.velocity_sensitivity);
    out.bounce.delay_loss = lerp(a.bounce.delay_loss, b.bounce.delay_loss);
    out.bounce.velocity_loss = lerp(a.bounce.velocity_loss, b.bounce.velocity_loss);
    out.bounce.humanization = lerp(a.bounce.humanization, b.bounce.humanization);

    out.note_fx.lfo_shape = lerp(a.note_fx.lfo_shape, b.note_fx.lfo_shape);
    out.note_fx.lfo_skew = lerp(a.note_fx.lfo_skew, b.note_fx.lfo_skew);
    out.note_fx.lfo_rate_hz = lerp(a.note_fx.lfo_rate_hz, b.note_fx.lfo_rate_hz);
    out.note_fx.onset_duration_ms = lerp(a.note_fx.onset_duration_ms, b.note_fx.onset_duration_ms);
    out.note_fx.vibrato_cents = lerp(a.note_fx.vibrato_cents, b.note_fx.vibrato_cents);
    out.note_fx.vibrato_offset_cents = lerp(a.note_fx.vibrato_offset_cents, b.note_fx.vibrato_offset_cents);
    out.note_fx.tremolo_depth_db = lerp(a.note_fx.tremolo_depth_db, b.note_fx.tremolo_depth_db);
    out.note_fx.tremolo_phase = lerp(a.note_fx.tremolo_phase, b.note_fx.tremolo_phase);
    out.note_fx.attack_envelope = lerp(a.note_fx.attack_envelope, b.note_fx.attack_envelope);
    out.note_fx.smear = lerp(a.note_fx.smear, b.note_fx.smear);

    out.play_mode.guitar_body = lerp(a.play_mode.guitar_body, b.play_mode.guitar_body);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{EffectType, ExciterType, InstrumentCategory, TemperamentId};
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    fn build_two_distinct_params() -> (EngineParams, EngineParams) {
        let mut a = EngineParams::default();
        a.volume_db = -12.0;
        a.dynamics_db = 24.0;
        a.velocity_offset = -0.2;
        a.condition = 0.0;
        a.post_effect_gain_db = -3.0;
        a.tuning.diapason_hz = 415.0;
        a.tuning.unison_width_cents = 0.2;
        a.tuning.octave_stretching = 0.8;
        a.voicing.hammer_hardness_piano = 0.2;
        a.voicing.hammer_hardness_mezzo = 0.4;
        a.voicing.hammer_hardness_forte = 0.6;
        a.voicing.spectrum_profile = [0.6; 8];
        a.design.soundboard_impedance = 0.3;
        a.design.string_length_m = 1.5;
        a.action.damper_noise = 0.1;
        a.output.lid_position = 0.2;
        a.output.stereo_width = 0.2;
        a.output.mics[0].volume = 0.4;
        a.output.mics[2].x = 0.3;
        a.eq_bands[0].gain_db = -6.0;
        a.eq_bands[1].freq_hz = 800.0;
        a.fx[0].effect_type = EffectType::Delay;
        a.fx[0].mix = 0.1;
        a.fx[0].param_1 = 0.2;
        a.reverb.mix = 0.1;
        a.reverb.duration_s = 0.8;
        a.reverb.room_size_m = 10.0;
        a.reverb.pre_delay_ms = 5.0;
        a.reverb.early_reflections = 0.1;
        a.limiter.threshold_db = -6.0;
        a.bounce.delay_ms = 60.0;
        a.bounce.velocity_sensitivity = 0.2;
        a.bounce.delay_loss = 0.05;
        a.bounce.velocity_loss = 0.2;
        a.bounce.humanization = 0.0;
        a.note_fx.lfo_shape = 0.1;
        a.note_fx.lfo_skew = 0.2;
        a.note_fx.lfo_rate_hz = 2.0;
        a.note_fx.onset_duration_ms = 20.0;
        a.note_fx.vibrato_cents = 5.0;
        a.note_fx.vibrato_offset_cents = -2.0;
        a.note_fx.tremolo_depth_db = 1.0;
        a.note_fx.tremolo_phase = 0.0;
        a.note_fx.attack_envelope = 0.2;
        a.note_fx.smear = 0.1;
        a.play_mode.guitar_body = 0.2;

        let mut b = EngineParams::default();
        b.volume_db = 2.0;
        b.dynamics_db = 56.0;
        b.velocity_offset = 0.2;
        b.condition = 0.8;
        b.post_effect_gain_db = 5.0;
        b.tuning.diapason_hz = 445.0;
        b.tuning.unison_width_cents = 4.2;
        b.tuning.octave_stretching = 1.4;
        b.voicing.hammer_hardness_piano = 0.8;
        b.voicing.hammer_hardness_mezzo = 0.9;
        b.voicing.hammer_hardness_forte = 1.0;
        b.voicing.spectrum_profile = [1.2; 8];
        b.design.soundboard_impedance = 0.9;
        b.design.string_length_m = 3.1;
        b.action.damper_noise = 0.7;
        b.output.lid_position = 1.0;
        b.output.stereo_width = 1.0;
        b.output.mics[0].volume = 1.0;
        b.output.mics[2].x = 0.9;
        b.eq_bands[0].gain_db = 6.0;
        b.eq_bands[1].freq_hz = 2000.0;
        b.fx[0].effect_type = EffectType::Delay;
        b.fx[0].mix = 0.9;
        b.fx[0].param_1 = 0.8;
        b.reverb.mix = 0.7;
        b.reverb.duration_s = 3.6;
        b.reverb.room_size_m = 34.0;
        b.reverb.pre_delay_ms = 45.0;
        b.reverb.early_reflections = 0.9;
        b.limiter.threshold_db = -0.2;
        b.bounce.delay_ms = 260.0;
        b.bounce.velocity_sensitivity = 0.8;
        b.bounce.delay_loss = 0.45;
        b.bounce.velocity_loss = 0.6;
        b.bounce.humanization = 0.6;
        b.note_fx.lfo_shape = 0.9;
        b.note_fx.lfo_skew = 0.8;
        b.note_fx.lfo_rate_hz = 10.0;
        b.note_fx.onset_duration_ms = 180.0;
        b.note_fx.vibrato_cents = 35.0;
        b.note_fx.vibrato_offset_cents = 10.0;
        b.note_fx.tremolo_depth_db = 9.0;
        b.note_fx.tremolo_phase = 1.0;
        b.note_fx.attack_envelope = 0.8;
        b.note_fx.smear = 0.7;
        b.play_mode.guitar_body = 0.8;

        (a, b)
    }

    #[test]
    fn test_preset_morph_ab_linearity() {
        let (a, b) = build_two_distinct_params();

        // t = 0.0 -> exact A
        let m0 = morph_engine_params(&a, &b, 0.0);
        assert!(approx_eq(m0.volume_db, a.volume_db, 1e-6));
        assert!(approx_eq(m0.tuning.diapason_hz, a.tuning.diapason_hz, 1e-6));
        assert!(approx_eq(m0.voicing.hammer_hardness_piano, a.voicing.hammer_hardness_piano, 1e-6));
        assert!(approx_eq(m0.design.string_length_m, a.design.string_length_m, 1e-6));
        assert!(approx_eq(m0.action.damper_noise, a.action.damper_noise, 1e-6));
        assert!(approx_eq(m0.output.mics[0].volume, a.output.mics[0].volume, 1e-6));
        assert!(approx_eq(m0.eq_bands[0].gain_db, a.eq_bands[0].gain_db, 1e-6));
        assert!(approx_eq(m0.fx[0].param_1, a.fx[0].param_1, 1e-6));
        assert!(approx_eq(m0.reverb.pre_delay_ms, a.reverb.pre_delay_ms, 1e-6));
        assert!(approx_eq(m0.reverb.room_size_m, a.reverb.room_size_m, 1e-6));
        assert!(approx_eq(m0.reverb.early_reflections, a.reverb.early_reflections, 1e-6));
        assert!(approx_eq(m0.limiter.threshold_db, a.limiter.threshold_db, 1e-6));

        // t = 0.5 -> exact arithmetic midpoint
        let m_mid = morph_engine_params(&a, &b, 0.5);
        assert!(approx_eq(m_mid.volume_db, (a.volume_db + b.volume_db) * 0.5, 1e-5));
        assert!(approx_eq(m_mid.dynamics_db, (a.dynamics_db + b.dynamics_db) * 0.5, 1e-5));
        assert!(approx_eq(m_mid.tuning.diapason_hz, 430.0, 1e-5));
        assert!(approx_eq(m_mid.voicing.spectrum_profile[3], 0.9, 1e-5));
        assert!(approx_eq(m_mid.design.string_length_m, 2.3, 1e-5));
        assert!(approx_eq(m_mid.action.damper_noise, 0.4, 1e-5));
        assert!(approx_eq(m_mid.output.lid_position, 0.6, 1e-5));
        assert!(approx_eq(m_mid.output.mics[0].volume, 0.7, 1e-5));
        assert!(approx_eq(m_mid.output.mics[2].x, 0.6, 1e-5));
        assert!(approx_eq(m_mid.eq_bands[0].gain_db, 0.0, 1e-5));
        assert!(approx_eq(m_mid.eq_bands[1].freq_hz, 1400.0, 1e-4));
        assert!(approx_eq(m_mid.fx[0].mix, 0.5, 1e-5));
        assert!(approx_eq(m_mid.fx[0].param_1, 0.5, 1e-5));
        assert!(approx_eq(m_mid.reverb.mix, 0.4, 1e-5));
        assert!(approx_eq(m_mid.reverb.room_size_m, 22.0, 1e-5));
        assert!(approx_eq(m_mid.reverb.pre_delay_ms, 25.0, 1e-5));
        assert!(approx_eq(m_mid.reverb.early_reflections, 0.5, 1e-5));
        assert!(approx_eq(m_mid.limiter.threshold_db, -3.1, 1e-5));
        assert!(approx_eq(m_mid.bounce.delay_ms, 160.0, 1e-4));
        assert!(approx_eq(m_mid.bounce.velocity_sensitivity, 0.5, 1e-5));
        assert!(approx_eq(m_mid.bounce.delay_loss, 0.25, 1e-5));
        assert!(approx_eq(m_mid.bounce.velocity_loss, 0.4, 1e-5));
        assert!(approx_eq(m_mid.bounce.humanization, 0.3, 1e-5));
        assert!(approx_eq(m_mid.note_fx.lfo_shape, 0.5, 1e-5));
        assert!(approx_eq(m_mid.note_fx.lfo_skew, 0.5, 1e-5));
        assert!(approx_eq(m_mid.note_fx.lfo_rate_hz, 6.0, 1e-5));
        assert!(approx_eq(m_mid.note_fx.onset_duration_ms, 100.0, 1e-4));
        assert!(approx_eq(m_mid.note_fx.vibrato_cents, 20.0, 1e-5));
        assert!(approx_eq(m_mid.note_fx.vibrato_offset_cents, 4.0, 1e-5));
        assert!(approx_eq(m_mid.note_fx.tremolo_depth_db, 5.0, 1e-5));
        assert!(approx_eq(m_mid.note_fx.tremolo_phase, 0.5, 1e-5));
        assert!(approx_eq(m_mid.note_fx.attack_envelope, 0.5, 1e-5));
        assert!(approx_eq(m_mid.note_fx.smear, 0.4, 1e-5));
        assert!(approx_eq(m_mid.play_mode.guitar_body, 0.5, 1e-5));

        // t = 1.0 -> exact B
        let m1 = morph_engine_params(&a, &b, 1.0);
        assert!(approx_eq(m1.volume_db, b.volume_db, 1e-6));
        assert!(approx_eq(m1.tuning.diapason_hz, b.tuning.diapason_hz, 1e-6));
        assert!(approx_eq(m1.voicing.hammer_hardness_forte, b.voicing.hammer_hardness_forte, 1e-6));
        assert!(approx_eq(m1.output.mics[2].x, b.output.mics[2].x, 1e-6));
        assert!(approx_eq(m1.eq_bands[1].freq_hz, b.eq_bands[1].freq_hz, 1e-6));
        assert!(approx_eq(m1.reverb.pre_delay_ms, b.reverb.pre_delay_ms, 1e-6));
        assert!(approx_eq(m1.limiter.threshold_db, b.limiter.threshold_db, 1e-6));

        // Clamping out-of-range t
        let m_under = morph_engine_params(&a, &b, -2.0);
        let m_over = morph_engine_params(&a, &b, 3.0);
        assert!(approx_eq(m_under.volume_db, a.volume_db, 1e-6));
        assert!(approx_eq(m_over.volume_db, b.volume_db, 1e-6));
    }

    #[test]
    fn test_factory_presets_and_json_roundtrip() {
        let inst = InstrumentModel {
            id: "test-grand".to_string(),
            name: "Test Grand".to_string(),
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Test model".to_string(),
            inharmonicity_base: 0.00018,
            decay_scale: 1.0,
            default_params: EngineParams::default(),
        };

        let ep_inst = InstrumentModel {
            id: "test-ep".to_string(),
            name: "Test EP".to_string(),
            category: InstrumentCategory::ElectricPiano,
            exciter: ExciterType::TineHammer,
            description: "Test EP model".to_string(),
            inharmonicity_base: 0.00004,
            decay_scale: 0.85,
            default_params: EngineParams::default(),
        };

        let factory = build_factory_presets(&[inst, ep_inst]);
        assert_eq!(factory.len(), 10);
        assert!(factory.iter().all(|p| p.is_factory));
        assert!(factory.iter().any(|p| p.id == "test-grand-concert"));
        assert!(factory.iter().any(|p| p.id == "test-grand-fx" && p.params.reverb.preset_name == "Cathedral"));
        assert!(factory.iter().any(|p| p.id == "test-ep-fx" && p.params.fx[0].effect_type == EffectType::Phaser && p.params.fx[1].effect_type == EffectType::Amp));

        let mut custom = factory[1].clone();
        custom.id = "user-custom-roundtrip".to_string();
        custom.name = "User Custom Roundtrip".to_string();
        custom.is_factory = false;
        custom.params.tuning.temperament = TemperamentId::KirnbergerIII;
        custom.params.reverb.pre_delay_ms = 27.5;
        custom.params.fx[2].enabled = true;
        custom.params.fx[2].effect_type = EffectType::Compressor;

        // Pure JSON round-trip
        let json = serde_json::to_string_pretty(&custom).expect("serialize preset");
        let parsed: PresetData = serde_json::from_str(&json).expect("deserialize preset");
        assert_eq!(parsed.id, custom.id);
        assert_eq!(parsed.name, custom.name);
        assert_eq!(parsed.is_factory, false);
        assert_eq!(parsed.params.tuning.temperament, TemperamentId::KirnbergerIII);
        assert!(approx_eq(parsed.params.reverb.pre_delay_ms, 27.5, 1e-6));
        assert_eq!(parsed.params.fx[2].effect_type, EffectType::Compressor);

        // Disk save/load round-trip using an isolated XDG_DATA_HOME temp directory
        let _guard = ENV_LOCK.lock().unwrap();
        let prev_xdg = std::env::var_os("XDG_DATA_HOME");
        let temp_root = std::env::temp_dir().join(format!("piano_tect_preset_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_root);
        std::env::set_var("XDG_DATA_HOME", &temp_root);

        let saved_path = save_user_preset(&custom).expect("save_user_preset must succeed");
        assert!(saved_path.exists());
        let loaded = load_user_presets();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, custom.id);
        assert!(approx_eq(loaded[0].params.reverb.pre_delay_ms, 27.5, 1e-6));

        let _ = std::fs::remove_dir_all(&temp_root);
        match prev_xdg {
            Some(v) => std::env::set_var("XDG_DATA_HOME", v),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
    }
}
