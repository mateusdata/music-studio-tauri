use crate::contracts::{
    EngineParams, ExciterType, InstrumentModel, PedalState, TemperamentId,
};
use std::f32::consts::PI;

pub const MAX_POLYPHONY: usize = 128;
const NUM_PARTIALS: usize = 8;

/// Computes exact fundamental frequency (Hz) applying Diapason, Temperament, Octave Stretching,
/// Pitch Bend, and per-note PRO tuning offset.
pub fn compute_note_frequency(midi_note: u8, params: &EngineParams) -> f32 {
    let base_semi = midi_note as f32 - 69.0;
    let stretched_semi = base_semi
        + (params.tuning.octave_stretching - 1.0) * (base_semi / 12.0) * 0.045 * base_semi.abs().sqrt();
    let mut freq = params.tuning.diapason_hz * 2.0_f32.powf((stretched_semi + params.tuning.pitch_bend_semitones) / 12.0);

    let pc = (midi_note % 12) as usize;
    let ratio = temperament_ratio(params.tuning.temperament, pc);
    freq *= ratio;

    if let Some(ne) = params.note_edit.get(&midi_note) {
        freq *= 2.0_f32.powf(ne.detune_cents / 1200.0);
    }
    freq.clamp(12.0, 16000.0)
}

/// Temperament frequency multiplier relative to 12-TET (A unchanged = 1.0),
/// derived from the Pianoteq manual Section 17 equations.
pub fn temperament_ratio(temp: TemperamentId, pitch_class: usize) -> f32 {
    let pc = pitch_class % 12;
    match temp {
        TemperamentId::Equal => 1.0,
        TemperamentId::Pythagorean => {
            let delta = 1.5_f32 / 2.0_f32.powf(7.0 / 12.0);
            let exps = [-3.0, -8.0, -1.0, -6.0, 1.0, -4.0, 3.0, -2.0, -7.0, 0.0, -5.0, 2.0];
            delta.powf(exps[pc])
        }
        TemperamentId::Zarlino => {
            let ratio2c = [
                1.0,
                25.0 / 24.0,
                9.0 / 8.0,
                32.0 / 27.0,
                5.0 / 4.0,
                4.0 / 3.0,
                45.0 / 32.0,
                3.0 / 2.0,
                25.0 / 16.0,
                5.0 / 3.0,
                16.0 / 9.0,
                15.0 / 8.0,
            ];
            let a_norm = ratio2c[9] / 2.0_f32.powf(9.0 / 12.0);
            (ratio2c[pc] / 2.0_f32.powf(pc as f32 / 12.0)) / a_norm
        }
        TemperamentId::MeanTone => {
            let q = 5.0_f32.powf(0.25);
            let ratio2c = [
                1.0,
                5.0 * q.powi(3) / 16.0,
                q * q / 2.0,
                4.0 * q / 5.0,
                5.0 / 4.0,
                2.0 / q,
                5.0 * q * q / 8.0,
                q,
                25.0 / 16.0,
                q.powi(3) / 2.0,
                4.0 * q * q / 5.0,
                5.0 * q / 4.0,
            ];
            let a_norm = ratio2c[9] / 2.0_f32.powf(9.0 / 12.0);
            (ratio2c[pc] / 2.0_f32.powf(pc as f32 / 12.0)) / a_norm
        }
        TemperamentId::WellTempered => {
            let ratio2c = [
                1.0, 1.053498, 1.118034, 1.185185, 1.253134, 1.333333,
                1.404664, 1.495349, 1.580247, 1.671851, 1.777778, 1.879701,
            ];
            let a_norm = ratio2c[9] / 2.0_f32.powf(9.0 / 12.0);
            (ratio2c[pc] / 2.0_f32.powf(pc as f32 / 12.0)) / a_norm
        }
        TemperamentId::WerckmeisterIII => {
            let ratio2c = [
                1.0,
                256.0 / 243.0,
                1.1174,
                32.0 / 27.0,
                1.2528,
                4.0 / 3.0,
                1024.0 / 729.0,
                1.4949,
                128.0 / 81.0,
                1.6704,
                16.0 / 9.0,
                1.8792,
            ];
            let a_norm = ratio2c[9] / 2.0_f32.powf(9.0 / 12.0);
            (ratio2c[pc] / 2.0_f32.powf(pc as f32 / 12.0)) / a_norm
        }
        TemperamentId::KirnbergerIII => {
            let cents = [0.0, 90.225, 193.157, 294.135, 386.314, 498.045, 590.224, 696.578, 792.180, 889.735, 996.090, 1088.269];
            let a_dev = cents[9] - 900.0;
            2.0_f32.powf((cents[pc] - pc as f32 * 100.0 - a_dev) / 1200.0)
        }
        TemperamentId::Vallotti => {
            let cents = [0.0, 94.135, 196.090, 298.045, 392.180, 501.955, 592.180, 698.045, 796.090, 894.135, 1000.0, 1090.225];
            let a_dev = cents[9] - 900.0;
            2.0_f32.powf((cents[pc] - pc as f32 * 100.0 - a_dev) / 1200.0)
        }
    }
}

#[derive(Clone)]
struct Voice {
    active: bool,
    midi_note: u8,
    key_down: bool,
    sustained: bool,
    sostenuto_held: bool,
    harmonic_held: bool,
    velocity: f32,
    freq: f32,
    age_samples: u64,
    phases_a: [f32; NUM_PARTIALS],
    phases_b: [f32; NUM_PARTIALS],
    phase_incs_a: [f32; NUM_PARTIALS],
    phase_incs_b: [f32; NUM_PARTIALS],
    partial_amps: [f32; NUM_PARTIALS],
    partial_decays: [f32; NUM_PARTIALS],
    partial_levels: [f32; NUM_PARTIALS],
    hammer_noise_env: f32,
    hammer_noise_lp: f32,
    damper_noise_env: f32,
    pan: f32,
    exciter: ExciterType,
    bloom_state: f32,
    rng_state: u32,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            active: false,
            midi_note: 60,
            key_down: false,
            sustained: false,
            sostenuto_held: false,
            harmonic_held: false,
            velocity: 0.0,
            freq: 261.63,
            age_samples: 0,
            phases_a: [0.0; NUM_PARTIALS],
            phases_b: [0.0; NUM_PARTIALS],
            phase_incs_a: [0.0; NUM_PARTIALS],
            phase_incs_b: [0.0; NUM_PARTIALS],
            partial_amps: [0.0; NUM_PARTIALS],
            partial_decays: [0.9999; NUM_PARTIALS],
            partial_levels: [0.0; NUM_PARTIALS],
            hammer_noise_env: 0.0,
            hammer_noise_lp: 0.0,
            damper_noise_env: 0.0,
            pan: 0.0,
            exciter: ExciterType::FeltHammer,
            bloom_state: 0.0,
            rng_state: 0x12345678,
        }
    }
}

impl Voice {
    #[inline]
    fn next_noise(&mut self) -> f32 {
        self.rng_state = self.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let bits = (self.rng_state >> 9) | 0x3f800000;
        f32::from_bits(bits) * 2.0 - 3.0
    }
}

pub struct PianoSynthEngine {
    sample_rate: f32,
    voices: Vec<Voice>,
    sympathetic_lp: f32,
    sympathetic_bp: f32,
    sympathetic_energy: f32,
    pedal_thump_env: f32,
    note_counter: u64,
    rng: u32,
}

impl PianoSynthEngine {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate: sample_rate.max(22050.0),
            voices: vec![Voice::default(); MAX_POLYPHONY],
            sympathetic_lp: 0.0,
            sympathetic_bp: 0.0,
            sympathetic_energy: 0.0,
            pedal_thump_env: 0.0,
            note_counter: 0,
            rng: 0x9E3779B9,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(22050.0);
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.active).count()
    }

    pub fn active_notes(&self) -> Vec<u8> {
        let mut notes: Vec<u8> = self
            .voices
            .iter()
            .filter(|v| v.active && v.key_down)
            .map(|v| v.midi_note)
            .collect();
        notes.sort_unstable();
        notes.dedup();
        notes
    }

    pub fn all_notes_off(&mut self) {
        for v in &mut self.voices {
            v.active = false;
            v.key_down = false;
            v.sustained = false;
            v.sostenuto_held = false;
        }
    }

    pub fn note_on(
        &mut self,
        midi_note: u8,
        velocity_u8: u8,
        params: &EngineParams,
        instrument: &InstrumentModel,
        pedals: &PedalState,
    ) {
        if velocity_u8 == 0 {
            self.note_off(midi_note, params, pedals);
            return;
        }

        // Check keyboard range if not extended
        if !params.action.keyboard_range_extended && !(21..=108).contains(&midi_note) {
            return;
        }

        // Allocate voice (reuse same note if already sounding softly or oldest released voice)
        let mut slot_idx = self.voices.iter().position(|v| !v.active);
        if slot_idx.is_none() {
            slot_idx = self
                .voices
                .iter()
                .enumerate()
                .filter(|(_, v)| !v.key_down)
                .max_by_key(|(_, v)| v.age_samples)
                .map(|(i, _)| i);
        }
        if slot_idx.is_none() {
            slot_idx = self
                .voices
                .iter()
                .enumerate()
                .max_by_key(|(_, v)| v.age_samples)
                .map(|(i, _)| i);
        }
        let idx = slot_idx.unwrap_or(0);

        self.note_counter = self.note_counter.wrapping_add(1);
        let vel_norm = (velocity_u8 as f32 / 127.0).clamp(0.01, 1.0);
        // Apply dynamics range and velocity offset
        let dyn_curve = (params.dynamics_db / 50.0).clamp(0.2, 2.0);
        let vel_eff = (vel_norm + params.velocity_offset * 0.25).clamp(0.02, 1.0).powf(dyn_curve);

        let freq = compute_note_frequency(midi_note, params);
        let note_edit = params.note_edit.get(&midi_note);
        let note_gain = note_edit.map(|ne| 10.0_f32.powf(ne.gain_db / 20.0)).unwrap_or(1.0);

        // Soft pedal attenuation & felt softening
        let soft_atten = 1.0 - pedals.soft * (1.0 - params.voicing.soft_level * 0.65);
        let mute_atten = 1.0 - params.action.mute * 0.85;

        // Velocity-interpolated hammer hardness (Piano=0.32, Mezzo=0.55, Forte=0.77)
        let hardness_base = if vel_norm < 0.55 {
            let t = (vel_norm / 0.55).clamp(0.0, 1.0);
            params.voicing.hammer_hardness_piano * (1.0 - t)
                + params.voicing.hammer_hardness_mezzo * t
        } else {
            let t = ((vel_norm - 0.55) / 0.45).clamp(0.0, 1.0);
            params.voicing.hammer_hardness_mezzo * (1.0 - t)
                + params.voicing.hammer_hardness_forte * t
        };
        let hardness = (hardness_base
            + note_edit.map(|ne| ne.hammer_hardness_offset).unwrap_or(0.0)
            - pedals.soft * 0.22
            - pedals.celeste * 0.30
            - pedals.buff_stop * 0.35)
            .clamp(0.05, 1.25);

        // Inharmonicity B scales inversely with string length squared and rises in treble
        let length_factor = (2.74 / params.design.string_length_m.max(0.8)).powi(2);
        let midi_rel = (midi_note as f32 - 21.0) / 87.0;
        let b_inharm = instrument.inharmonicity_base
            * length_factor
            * (0.4 + 4.5 * midi_rel.powi(2))
            * (1.0 + params.condition * 0.35);

        // Unison trichord detune (cents)
        let unison_cents = (params.tuning.unison_width_cents
            + params.condition * 12.0)
            * if midi_note < 36 { 0.35 } else { 1.0 };
        let detune_ratio = 2.0_f32.powf(unison_cents / 1200.0);

        let nyquist = self.sample_rate * 0.45;
        let base_decay_s = (7.5 - 5.8 * midi_rel) * instrument.decay_scale
            * (1.35 - params.design.soundboard_impedance * 0.65);

        let v = &mut self.voices[idx];
        v.active = true;
        v.midi_note = midi_note;
        v.key_down = true;
        v.sustained = false;
        v.sostenuto_held = pedals.sostenuto > 0.5 && pedals.super_sostenuto;
        v.harmonic_held = pedals.harmonic > 0.5;
        v.velocity = vel_eff * soft_atten * mute_atten * note_gain;
        v.freq = freq;
        v.age_samples = 0;
        v.exciter = instrument.exciter;
        v.bloom_state = 0.0;
        v.rng_state = (self.note_counter as u32).wrapping_mul(747796405).wrapping_add(midi_note as u32 + 13);

        // Natural stereo keyboard image (-0.78 bass to +0.78 treble)
        v.pan = ((midi_note as f32 - 60.0) / 48.0).clamp(-0.82, 0.82);

        let strike_pt = params.voicing.strike_point.clamp(0.03, 0.48);
        let hammer_bright = 0.4 + hardness * 1.45;

        for p_idx in 0..NUM_PARTIALS {
            let n = (p_idx + 1) as f32;
            let partial_freq = freq * n * (1.0 + b_inharm * n * n).sqrt();
            if partial_freq >= nyquist {
                v.partial_amps[p_idx] = 0.0;
                v.partial_levels[p_idx] = 0.0;
                v.phase_incs_a[p_idx] = 0.0;
                v.phase_incs_b[p_idx] = 0.0;
                continue;
            }

            let inc = 2.0 * PI * partial_freq / self.sample_rate;
            v.phase_incs_a[p_idx] = inc;
            v.phase_incs_b[p_idx] = inc * detune_ratio;
            v.phases_a[p_idx] = 0.0;
            v.phases_b[p_idx] = (p_idx as f32) * 0.37; // Slight unison phase spread

            // Strike point comb filter: sin(n * pi * strike_point)
            let strike_comb = (n * PI * strike_pt).sin().abs().max(0.12);
            // Spectral rolloff controlled by hammer hardness & velocity
            let rolloff = (1.0 / n.powf(1.35 / hammer_bright)).clamp(0.02, 1.0);
            let prof_weight = params.voicing.spectrum_profile[p_idx].clamp(0.0, 2.0);
            // Electric piano / harpsichord exciter harmonic emphasis
            let exciter_shape = match instrument.exciter {
                ExciterType::TineHammer => if p_idx == 0 { 1.4 } else if p_idx < 3 { 0.8 } else { 0.25 },
                ExciterType::ReedHammer => if p_idx % 2 == 0 { 1.15 } else { 0.45 },
                ExciterType::PluckPlectrum => 1.15 * (0.85 + 0.15 * n),
                ExciterType::MalletHard | ExciterType::MalletSoft => {
                    if p_idx == 0 { 1.3 } else if p_idx == 1 { 0.55 } else { 0.12 }
                }
                ExciterType::PreparedBolt => if p_idx % 2 == 1 { 1.35 } else { 0.7 },
                _ => 1.0,
            };

            let amp = v.velocity * strike_comb * rolloff * prof_weight * exciter_shape * 0.24;
            v.partial_amps[p_idx] = amp;
            v.partial_levels[p_idx] = amp;

            let partial_decay_s = (base_decay_s / (1.0 + 0.48 * (n - 1.0))).max(0.08);
            v.partial_decays[p_idx] = (-1.0 / (partial_decay_s * self.sample_rate)).exp();
        }

        v.hammer_noise_env = v.velocity * params.voicing.hammer_noise * 0.35;
        v.hammer_noise_lp = 0.0;
        v.damper_noise_env = 0.0;
    }

    pub fn note_off(&mut self, midi_note: u8, params: &EngineParams, pedals: &PedalState) {
        let sustain_on = if pedals.reversed_sustain {
            pedals.sustain < 0.5
        } else {
            pedals.sustain >= 0.5
        };

        for v in &mut self.voices {
            if v.active && v.midi_note == midi_note && v.key_down {
                v.key_down = false;
                if sustain_on {
                    v.sustained = true;
                } else if !v.sostenuto_held && !v.harmonic_held {
                    v.damper_noise_env = v.velocity * params.action.damper_noise * 0.20;
                }
            }
        }
    }

    pub fn update_pedals(&mut self, pedals: &PedalState, prev_pedals: &PedalState, params: &EngineParams) {
        // Sostenuto latch on press
        if pedals.sostenuto >= 0.5 && prev_pedals.sostenuto < 0.5 {
            for v in &mut self.voices {
                if v.active && v.key_down {
                    v.sostenuto_held = true;
                }
            }
        } else if pedals.sostenuto < 0.5 && prev_pedals.sostenuto >= 0.5 {
            for v in &mut self.voices {
                v.sostenuto_held = false;
            }
        }

        let sustain_on = if pedals.reversed_sustain {
            pedals.sustain < 0.5
        } else {
            pedals.sustain >= 0.5
        };
        let prev_sustain_on = if prev_pedals.reversed_sustain {
            prev_pedals.sustain < 0.5
        } else {
            prev_pedals.sustain >= 0.5
        };

        if (sustain_on != prev_sustain_on) && params.action.pedal_noise > 0.01 {
            self.pedal_thump_env = params.action.pedal_noise * 0.14;
        }

        if !sustain_on {
            for v in &mut self.voices {
                if v.active && v.sustained && !v.key_down {
                    v.sustained = false;
                }
            }
        }
    }

    /// Renders `out_l` and `out_r` stereo buffers in-place (zero allocation).
    pub fn process_block(
        &mut self,
        out_l: &mut [f32],
        out_r: &mut [f32],
        params: &EngineParams,
        pedals: &PedalState,
    ) {
        let len = out_l.len().min(out_r.len());
        let sustain_on = if pedals.reversed_sustain {
            pedals.sustain < 0.5
        } else {
            pedals.sustain >= 0.5
        };

        let damper_release_s = (0.045 + params.action.damping_duration * 0.18).max(0.025);
        let release_coeff = (-1.0 / (damper_release_s * self.sample_rate)).exp();
        let hammer_lp_coeff = (0.18 + params.voicing.hammer_tone * 0.68).clamp(0.05, 0.92);
        let unison_mix = params.tuning.unison_balance.clamp(0.0, 1.0);
        let bloom_amt = params.design.blooming_energy * 0.25;
        let symp_amt = params.design.sympathetic_resonance * (0.35 + if sustain_on { 0.65 } else { 0.0 });
        let duplex_amt = params.design.duplex_scale_resonance * 0.15;

        for i in 0..len {
            let mut sum_l = 0.0_f32;
            let mut sum_r = 0.0_f32;
            let mut body_energy = 0.0_f32;

            for v in &mut self.voices {
                if !v.active {
                    continue;
                }

                let is_held = v.key_down || (v.sustained && sustain_on) || v.sostenuto_held || v.harmonic_held;
                let mut sample = 0.0_f32;
                let mut max_level = 0.0_f32;

                // Attack smoothing (first ~1.5 ms)
                let attack_env = (v.age_samples as f32 / (self.sample_rate * 0.0015)).min(1.0);
                // Blooming swell for sustained acoustic tone
                v.bloom_state = v.bloom_state * 0.9995 + (1.0 - v.bloom_state) * 0.0005;
                let bloom_mult = 1.0 + bloom_amt * (1.0 - (-(v.age_samples as f32) / (self.sample_rate * 0.35)).exp());

                for p_idx in 0..NUM_PARTIALS {
                    let level = v.partial_levels[p_idx];
                    if level < 1e-6 {
                        continue;
                    }
                    let pa = v.phases_a[p_idx];
                    let pb = v.phases_b[p_idx];
                    let s_a = pa.sin();
                    let s_b = pb.sin();
                    let mixed = s_a * (1.0 - unison_mix * 0.45) + s_b * (unison_mix * 0.45);
                    sample += mixed * level * if p_idx >= 2 { bloom_mult } else { 1.0 };

                    let mut next_pa = pa + v.phase_incs_a[p_idx];
                    if next_pa > 2.0 * PI {
                        next_pa -= 2.0 * PI;
                    }
                    v.phases_a[p_idx] = next_pa;

                    let mut next_pb = pb + v.phase_incs_b[p_idx];
                    if next_pb > 2.0 * PI {
                        next_pb -= 2.0 * PI;
                    }
                    v.phases_b[p_idx] = next_pb;

                    let decay = if is_held {
                        v.partial_decays[p_idx]
                    } else {
                        v.partial_decays[p_idx].min(release_coeff)
                    };
                    v.partial_levels[p_idx] = level * decay;
                    if v.partial_levels[p_idx] > max_level {
                        max_level = v.partial_levels[p_idx];
                    }
                }

                // Duplex shimmer on treble notes
                if v.midi_note >= 60 && duplex_amt > 0.001 && max_level > 0.001 {
                    let duplex_sig = (v.phases_a[2] * 1.5).sin() * v.partial_levels[0] * duplex_amt;
                    sample += duplex_sig;
                }

                // Hammer percussion transient burst
                if v.hammer_noise_env > 1e-5 {
                    let noise = v.next_noise();
                    v.hammer_noise_lp = v.hammer_noise_lp * (1.0 - hammer_lp_coeff) + noise * hammer_lp_coeff;
                    sample += v.hammer_noise_lp * v.hammer_noise_env;
                    v.hammer_noise_env *= 0.988;
                }

                // Damper release felt thump
                if v.damper_noise_env > 1e-5 {
                    let d_noise = v.next_noise();
                    sample += d_noise * v.damper_noise_env * 0.25;
                    v.damper_noise_env *= 0.992;
                }

                // Subtle nonlinear quadratic soundboard saturation
                let quad = params.design.quadratic_effect * 0.12;
                sample = sample * attack_env + quad * sample * sample;

                let pan_r = ((v.pan + 1.0) * 0.5).clamp(0.0, 1.0);
                let gain_l = (1.0 - pan_r).sqrt();
                let gain_r = pan_r.sqrt();

                sum_l += sample * gain_l;
                sum_r += sample * gain_r;
                body_energy += sample;

                v.age_samples += 1;
                if max_level < 1e-5 && v.hammer_noise_env < 1e-5 && v.damper_noise_env < 1e-5 {
                    v.active = false;
                }
            }

            // Sympathetic string resonance body tank
            if symp_amt > 0.001 {
                self.sympathetic_energy = self.sympathetic_energy * 0.9994 + body_energy * 0.0006;
                self.sympathetic_lp = self.sympathetic_lp * 0.97 + self.sympathetic_energy * 0.03;
                self.sympathetic_bp = self.sympathetic_bp * 0.992 + (self.sympathetic_energy - self.sympathetic_lp) * 0.08;
                let res = self.sympathetic_bp * symp_amt * 0.45;
                sum_l += res;
                sum_r += res * 0.96;
            }

            // Pedal mechanical thump
            if self.pedal_thump_env > 1e-5 {
                self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
                let thump = ((self.rng >> 16) as f32 / 65535.0 - 0.5) * self.pedal_thump_env;
                sum_l += thump;
                sum_r += thump;
                self.pedal_thump_env *= 0.995;
            }

            // Master volume & condition/tuning output
            let vol = 10.0_f32.powf(params.volume_db / 20.0);
            out_l[i] = sum_l * vol;
            out_r[i] = sum_r * vol;
        }
    }
}
