use crate::contracts::{
    EngineParams, ExciterType, InstrumentModel, PedalState, TemperamentId,
};
use std::f32::consts::PI;

pub const MAX_POLYPHONY: usize = 128;
const NUM_PARTIALS: usize = 8;

#[inline]
fn sanitize_sample_rate(sr: f32) -> f32 {
    if sr.is_finite() {
        sr.clamp(22050.0, 384000.0)
    } else {
        48000.0
    }
}

#[inline]
fn finite_or(val: f32, fallback: f32) -> f32 {
    if val.is_finite() { val } else { fallback }
}

#[inline]
fn safety_limit(x: f32) -> f32 {
    if !x.is_finite() {
        return 0.0;
    }
    let abs = x.abs();
    let thresh = 0.88_f32;
    if abs <= thresh {
        x
    } else {
        let excess = (abs - thresh) / (1.0 - thresh);
        let limited = thresh + (1.0 - thresh) * 0.96 * excess.tanh();
        x.signum() * limited.min(1.0)
    }
}

/// Computes exact fundamental frequency (Hz) applying Diapason, Temperament, Octave Stretching,
/// Pitch Bend, and per-note PRO tuning offset.
pub fn compute_note_frequency(midi_note: u8, params: &EngineParams) -> f32 {
    let diapason = finite_or(params.tuning.diapason_hz, 440.0).clamp(100.0, 1200.0);
    let stretch = finite_or(params.tuning.octave_stretching, 1.0).clamp(0.5, 2.0);
    let pitch_bend = finite_or(params.tuning.pitch_bend_semitones, 0.0).clamp(-24.0, 24.0);

    let base_semi = midi_note as f32 - 69.0;
    let stretched_semi = base_semi
        + (stretch - 1.0) * (base_semi / 12.0) * 0.045 * base_semi.abs().sqrt();
    let mut freq = diapason * 2.0_f32.powf((stretched_semi + pitch_bend) / 12.0);

    let pc = (midi_note % 12) as usize;
    let ratio = temperament_ratio(params.tuning.temperament, pc);
    freq *= ratio;

    if let Some(ne) = params.note_edit.get(&midi_note) {
        let detune = finite_or(ne.detune_cents, 0.0).clamp(-200.0, 200.0);
        freq *= 2.0_f32.powf(detune / 1200.0);
    }
    if freq.is_finite() {
        freq.clamp(12.0, 16000.0)
    } else {
        440.0
    }
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
    key_release_noise_env: f32,
    key_release_lp: f32,
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
            key_release_noise_env: 0.0,
            key_release_lp: 0.0,
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
            sample_rate: sanitize_sample_rate(sample_rate),
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
        let new_sr = sanitize_sample_rate(sr);
        let old_sr = self.sample_rate;
        if (new_sr - old_sr).abs() < 1e-3 {
            self.sample_rate = new_sr;
            return;
        }
        let ratio = old_sr / new_sr;
        let nyquist_inc = 2.0 * PI * 0.45;
        for v in &mut self.voices {
            if !v.active {
                continue;
            }
            for p_idx in 0..NUM_PARTIALS {
                if v.phase_incs_a[p_idx] > 0.0 {
                    let next_a = v.phase_incs_a[p_idx] * ratio;
                    let next_b = v.phase_incs_b[p_idx] * ratio;
                    if !next_a.is_finite() || next_a >= nyquist_inc {
                        v.partial_amps[p_idx] = 0.0;
                        v.partial_levels[p_idx] = 0.0;
                        v.phase_incs_a[p_idx] = 0.0;
                        v.phase_incs_b[p_idx] = 0.0;
                    } else {
                        v.phase_incs_a[p_idx] = next_a;
                        v.phase_incs_b[p_idx] = if next_b.is_finite() {
                            next_b.clamp(0.0, nyquist_inc * 0.99)
                        } else {
                            next_a
                        };
                        v.partial_decays[p_idx] = v.partial_decays[p_idx].clamp(0.0, 0.999999).powf(ratio);
                    }
                }
            }
        }
        self.sample_rate = new_sr;
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
            v.harmonic_held = false;
            v.hammer_noise_env = 0.0;
            v.damper_noise_env = 0.0;
            v.key_release_noise_env = 0.0;
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
        let dyn_curve = (finite_or(params.dynamics_db, 40.0) / 50.0).clamp(0.2, 2.0);
        let vel_offset = finite_or(params.velocity_offset, 0.0).clamp(-1.0, 1.0);
        let vel_eff = (vel_norm + vel_offset * 0.25).clamp(0.02, 1.0).powf(dyn_curve);

        let freq = compute_note_frequency(midi_note, params);
        let note_edit = params.note_edit.get(&midi_note);
        let note_gain_db = note_edit.map(|ne| finite_or(ne.gain_db, 0.0).clamp(-24.0, 24.0)).unwrap_or(0.0);
        let note_gain = 10.0_f32.powf(note_gain_db / 20.0);

        // Soft pedal attenuation & felt softening
        let soft_pedal = finite_or(pedals.soft, 0.0).clamp(0.0, 1.0);
        let celeste_pedal = finite_or(pedals.celeste, 0.0).clamp(0.0, 1.0);
        let buff_pedal = finite_or(pedals.buff_stop, 0.0).clamp(0.0, 1.0);
        let soft_atten = 1.0 - soft_pedal * (1.0 - finite_or(params.voicing.soft_level, 0.65).clamp(0.0, 1.0) * 0.65);
        let mute_atten = 1.0 - finite_or(params.action.mute, 0.0).clamp(0.0, 1.0) * 0.85;

        // Velocity-interpolated hammer hardness (Piano=0.32, Mezzo=0.55, Forte=0.77)
        let h_piano = finite_or(params.voicing.hammer_hardness_piano, 0.38);
        let h_mezzo = finite_or(params.voicing.hammer_hardness_mezzo, 0.58);
        let h_forte = finite_or(params.voicing.hammer_hardness_forte, 0.82);
        let hardness_base = if vel_norm < 0.55 {
            let t = (vel_norm / 0.55).clamp(0.0, 1.0);
            h_piano * (1.0 - t) + h_mezzo * t
        } else {
            let t = ((vel_norm - 0.55) / 0.45).clamp(0.0, 1.0);
            h_mezzo * (1.0 - t) + h_forte * t
        };
        let hardness = (hardness_base
            + note_edit.map(|ne| finite_or(ne.hammer_hardness_offset, 0.0)).unwrap_or(0.0)
            - soft_pedal * 0.22
            - celeste_pedal * 0.30
            - buff_pedal * 0.35)
            .clamp(0.05, 1.25);

        // Inharmonicity B scales inversely with string length squared and rises in treble
        let string_len = finite_or(params.design.string_length_m, 2.74).max(0.8);
        let length_factor = (2.74 / string_len).powi(2);
        let midi_rel = ((midi_note as f32 - 21.0) / 87.0).clamp(0.0, 1.2);
        let condition = finite_or(params.condition, 0.04).clamp(0.0, 2.0);
        let b_inharm = (finite_or(instrument.inharmonicity_base, 0.00018).max(0.0)
            * length_factor
            * (0.4 + 4.5 * midi_rel.powi(2))
            * (1.0 + condition * 0.35))
            .clamp(0.0, 0.1);

        // Unison trichord detune (cents)
        let unison_cents = (finite_or(params.tuning.unison_width_cents, 0.7).clamp(0.0, 50.0)
            + condition * 12.0)
            * if midi_note < 36 { 0.35 } else { 1.0 };
        let detune_ratio = 2.0_f32.powf(unison_cents / 1200.0);

        let nyquist = self.sample_rate * 0.45;
        let impedance = finite_or(params.design.soundboard_impedance, 0.52).clamp(0.0, 1.5);
        let base_decay_s = ((7.5 - 5.8 * midi_rel).max(0.4)
            * finite_or(instrument.decay_scale, 1.0).clamp(0.1, 5.0)
            * (1.35 - impedance * 0.65).max(0.15))
            .max(0.1);

        let v = &mut self.voices[idx];
        v.active = true;
        v.midi_note = midi_note;
        v.key_down = true;
        v.sustained = false;
        v.sostenuto_held = pedals.sostenuto >= 0.5 && pedals.super_sostenuto;
        v.harmonic_held = pedals.harmonic >= 0.5;
        v.velocity = vel_eff * soft_atten * mute_atten * note_gain;
        v.freq = freq;
        v.age_samples = 0;
        v.exciter = instrument.exciter;
        v.bloom_state = 0.0;
        v.rng_state = (self.note_counter as u32).wrapping_mul(747796405).wrapping_add(midi_note as u32 + 13);

        // Natural stereo keyboard image (-0.78 bass to +0.78 treble)
        v.pan = ((midi_note as f32 - 60.0) / 48.0).clamp(-0.82, 0.82);

        let strike_pt = finite_or(params.voicing.strike_point, 0.125).clamp(0.03, 0.48);
        let hammer_bright = 0.4 + hardness * 1.45;

        for p_idx in 0..NUM_PARTIALS {
            let n = (p_idx + 1) as f32;
            let partial_freq = freq * n * (1.0 + b_inharm * n * n).sqrt();
            if !partial_freq.is_finite() || partial_freq >= nyquist {
                v.partial_amps[p_idx] = 0.0;
                v.partial_levels[p_idx] = 0.0;
                v.phase_incs_a[p_idx] = 0.0;
                v.phase_incs_b[p_idx] = 0.0;
                continue;
            }

            let inc = 2.0 * PI * partial_freq / self.sample_rate;
            v.phase_incs_a[p_idx] = inc;
            v.phase_incs_b[p_idx] = (inc * detune_ratio).min(2.0 * PI * 0.48);
            v.phases_a[p_idx] = 0.0;
            v.phases_b[p_idx] = (p_idx as f32) * 0.37; // Slight unison phase spread

            // Strike point comb filter: sin(n * pi * strike_point)
            let strike_comb = (n * PI * strike_pt).sin().abs().max(0.12);
            // Spectral rolloff controlled by hammer hardness & velocity
            let rolloff = (1.0 / n.powf(1.35 / hammer_bright)).clamp(0.02, 1.0);
            let prof_weight = finite_or(params.voicing.spectrum_profile[p_idx], 1.0).clamp(0.0, 2.0);
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

        v.hammer_noise_env = v.velocity * finite_or(params.voicing.hammer_noise, 0.35).clamp(0.0, 2.0) * 0.35;
        v.hammer_noise_lp = 0.0;
        v.damper_noise_env = 0.0;
        v.key_release_noise_env = 0.0;
        v.key_release_lp = 0.0;
    }

    pub fn note_off(&mut self, midi_note: u8, params: &EngineParams, pedals: &PedalState) {
        let sustain_on = if pedals.reversed_sustain {
            pedals.sustain < 0.5
        } else {
            pedals.sustain >= 0.5
        };
        let key_rel_amt = finite_or(params.action.key_release_noise, 0.22).clamp(0.0, 2.0);
        let damper_amt = finite_or(params.action.damper_noise, 0.28).clamp(0.0, 2.0);

        for v in &mut self.voices {
            if v.active && v.midi_note == midi_note && v.key_down {
                v.key_down = false;
                v.harmonic_held = pedals.harmonic >= 0.5;
                if key_rel_amt > 1e-4 {
                    v.key_release_noise_env = key_rel_amt * (0.25 + 0.75 * v.velocity.min(1.0)) * 0.16;
                }
                if sustain_on {
                    v.sustained = true;
                } else if !v.sostenuto_held && !v.harmonic_held {
                    v.damper_noise_env = v.velocity * damper_amt * 0.20;
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

        // Harmonic pedal latch on press / clear on release
        if pedals.harmonic >= 0.5 && prev_pedals.harmonic < 0.5 {
            for v in &mut self.voices {
                if v.active && v.key_down {
                    v.harmonic_held = true;
                }
            }
        } else if pedals.harmonic < 0.5 && prev_pedals.harmonic >= 0.5 {
            for v in &mut self.voices {
                v.harmonic_held = false;
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

        let pedal_noise = finite_or(params.action.pedal_noise, 0.25).clamp(0.0, 2.0);
        if (sustain_on != prev_sustain_on) && pedal_noise > 0.01 {
            self.pedal_thump_env = pedal_noise * 0.14;
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
        if len == 0 {
            return;
        }
        let sustain_on = if pedals.reversed_sustain {
            pedals.sustain < 0.5
        } else {
            pedals.sustain >= 0.5
        };
        let rattle_amt = if pedals.rattle.is_finite() && pedals.rattle > 0.01 {
            pedals.rattle.clamp(0.0, 1.0)
        } else {
            0.0
        };

        let damper_release_s = (0.045 + finite_or(params.action.damping_duration, 0.5).clamp(0.0, 2.0) * 0.18).max(0.025);
        let release_coeff = (-1.0 / (damper_release_s * self.sample_rate)).exp();
        let hammer_lp_coeff = (0.18 + finite_or(params.voicing.hammer_tone, 0.5) * 0.68).clamp(0.05, 0.92);
        let unison_mix = finite_or(params.tuning.unison_balance, 0.5).clamp(0.0, 1.0);
        let bloom_amt = finite_or(params.design.blooming_energy, 0.28).clamp(0.0, 2.0) * 0.25;
        let symp_amt = finite_or(params.design.sympathetic_resonance, 0.45).clamp(0.0, 2.0)
            * (0.35 + if sustain_on { 0.65 } else { 0.0 });
        let duplex_amt = finite_or(params.design.duplex_scale_resonance, 0.35).clamp(0.0, 2.0) * 0.15;
        let quad = finite_or(params.design.quadratic_effect, 0.22).clamp(0.0, 2.0) * 0.12;
        let vol = 10.0_f32.powf(finite_or(params.volume_db, -4.0).clamp(-80.0, 24.0) / 20.0);

        let hammer_noise_decay = (-1.0 / (0.0018 * self.sample_rate)).exp();
        let damper_noise_decay = (-1.0 / (0.0026 * self.sample_rate)).exp();
        let key_release_decay = (-1.0 / (0.0045 * self.sample_rate)).exp();
        let key_release_lp_coeff = (1.0 - (-2.0 * PI * 380.0 / self.sample_rate).exp()).clamp(0.02, 0.5);
        let bloom_tau = self.sample_rate * 0.35;
        let bloom_state_decay = (-1.0 / (0.0416 * self.sample_rate)).exp();
        let attack_samples = self.sample_rate * 0.0015;
        let symp_in_coeff = (1.0 - (-1.0 / (0.035 * self.sample_rate)).exp()).clamp(1e-5, 0.05);
        let symp_lp_coeff = (1.0 - (-1.0 / (0.0007 * self.sample_rate)).exp()).clamp(0.001, 0.25);
        let symp_bp_decay = (-1.0 / (0.0026 * self.sample_rate)).exp();
        let pedal_thump_decay = (-1.0 / (0.0042 * self.sample_rate)).exp();
        let nyq_inc = PI * 0.9;

        for i in 0..len {
            let mut sum_l = 0.0_f32;
            let mut sum_r = 0.0_f32;
            let mut body_energy = 0.0_f32;

            for v in &mut self.voices {
                if !v.active {
                    continue;
                }

                let full_held = v.key_down || (v.sustained && sustain_on) || v.sostenuto_held;
                let harmonic_only_held = !full_held && v.harmonic_held && pedals.harmonic >= 0.5;
                let mut sample = 0.0_f32;
                let mut max_level = 0.0_f32;

                // Attack smoothing (first ~1.5 ms)
                let attack_env = (v.age_samples as f32 / attack_samples).min(1.0);
                // Blooming swell for sustained acoustic tone
                v.bloom_state = v.bloom_state * bloom_state_decay + (1.0 - v.bloom_state) * (1.0 - bloom_state_decay);
                let bloom_mult = 1.0 + bloom_amt * (1.0 - (-(v.age_samples as f32) / bloom_tau).exp());

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
                    if next_pa >= 2.0 * PI {
                        next_pa -= 2.0 * PI;
                    }
                    v.phases_a[p_idx] = next_pa;

                    let mut next_pb = pb + v.phase_incs_b[p_idx];
                    if next_pb >= 2.0 * PI {
                        next_pb -= 2.0 * PI;
                    }
                    v.phases_b[p_idx] = next_pb;

                    let decay = if full_held {
                        v.partial_decays[p_idx]
                    } else if harmonic_only_held {
                        if p_idx == 0 {
                            v.partial_decays[p_idx].min(release_coeff)
                        } else {
                            v.partial_decays[p_idx]
                        }
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

                // Rattle pedal (parchment strip buzz / jangle harmonic modulation)
                if rattle_amt > 0.0 && max_level > 1e-4 {
                    let phi0 = v.phases_a[0];
                    let fund_inc = v.phase_incs_a[0];
                    let buzz_gate = phi0.sin().max(0.0);
                    let rattle_env = (v.partial_levels[0] * 1.6
                        + v.partial_levels[1] * 0.8
                        + v.partial_levels[2] * 0.4)
                        .min(0.6);
                    let mut jangle = 0.0_f32;
                    if fund_inc > 0.0 {
                        if fund_inc * 3.0 < nyq_inc { jangle += (phi0 * 3.0).sin() * 0.45; }
                        if fund_inc * 5.0 < nyq_inc { jangle += (phi0 * 5.0).sin() * 0.42; }
                        if fund_inc * 7.0 < nyq_inc { jangle += (phi0 * 7.0).sin() * 0.36; }
                        if fund_inc * 9.0 < nyq_inc { jangle += (phi0 * 9.0).sin() * 0.30; }
                        if fund_inc * 11.0 < nyq_inc { jangle += (phi0 * 11.0).sin() * 0.24; }
                    }
                    let sizzle = v.next_noise() * 0.28;
                    let rattle_mod = (jangle + sizzle) * buzz_gate * rattle_env * rattle_amt * 1.65;
                    sample = sample * (1.0 - 0.28 * rattle_amt * buzz_gate) + rattle_mod;
                }

                // Hammer percussion transient burst
                if v.hammer_noise_env > 1e-5 {
                    let noise = v.next_noise();
                    v.hammer_noise_lp = v.hammer_noise_lp * (1.0 - hammer_lp_coeff) + noise * hammer_lp_coeff;
                    sample += v.hammer_noise_lp * v.hammer_noise_env;
                    v.hammer_noise_env *= hammer_noise_decay;
                }

                // Damper release felt thump
                if v.damper_noise_env > 1e-5 {
                    let d_noise = v.next_noise();
                    sample += d_noise * v.damper_noise_env * 0.25;
                    v.damper_noise_env *= damper_noise_decay;
                }

                // Key-release mechanical action noise
                if v.key_release_noise_env > 1e-5 {
                    let kr_noise = v.next_noise();
                    v.key_release_lp = v.key_release_lp * (1.0 - key_release_lp_coeff)
                        + kr_noise * key_release_lp_coeff;
                    sample += v.key_release_lp * v.key_release_noise_env;
                    v.key_release_noise_env *= key_release_decay;
                }

                // Subtle nonlinear quadratic soundboard saturation
                sample = sample * attack_env + quad * sample * sample;
                if !sample.is_finite() {
                    sample = 0.0;
                }

                let pan_r = ((v.pan + 1.0) * 0.5).clamp(0.0, 1.0);
                let gain_l = (1.0 - pan_r).sqrt();
                let gain_r = pan_r.sqrt();

                sum_l += sample * gain_l;
                sum_r += sample * gain_r;
                body_energy += sample;

                v.age_samples += 1;
                if max_level < 1e-5
                    && v.hammer_noise_env < 1e-5
                    && v.damper_noise_env < 1e-5
                    && v.key_release_noise_env < 1e-5
                {
                    v.active = false;
                }
            }

            // Sympathetic string resonance body tank
            if symp_amt > 0.001 {
                self.sympathetic_energy = self.sympathetic_energy * (1.0 - symp_in_coeff) + body_energy * symp_in_coeff;
                self.sympathetic_lp = self.sympathetic_lp * (1.0 - symp_lp_coeff) + self.sympathetic_energy * symp_lp_coeff;
                self.sympathetic_bp = self.sympathetic_bp * symp_bp_decay + (self.sympathetic_energy - self.sympathetic_lp) * 0.08;
                if !self.sympathetic_energy.is_finite() || !self.sympathetic_bp.is_finite() {
                    self.sympathetic_energy = 0.0;
                    self.sympathetic_lp = 0.0;
                    self.sympathetic_bp = 0.0;
                }
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
                self.pedal_thump_env *= pedal_thump_decay;
            }

            // Master volume & bounded finite output
            out_l[i] = safety_limit(sum_l * vol);
            out_r[i] = safety_limit(sum_r * vol);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{InstrumentCategory, NoteEditOffset};

    fn test_instrument() -> InstrumentModel {
        InstrumentModel {
            id: "test-grand".to_string(),
            name: "Test Grand".to_string(),
            category: InstrumentCategory::GrandPiano,
            exciter: ExciterType::FeltHammer,
            description: "Unit test concert grand".to_string(),
            inharmonicity_base: 0.00018,
            decay_scale: 1.05,
            default_params: EngineParams::default(),
        }
    }

    fn clean_params() -> EngineParams {
        let mut p = EngineParams::default();
        p.voicing.hammer_noise = 0.0;
        p.action.damper_noise = 0.0;
        p.action.key_release_noise = 0.0;
        p.action.pedal_noise = 0.0;
        p.design.sympathetic_resonance = 0.0;
        p.design.duplex_scale_resonance = 0.0;
        p.design.blooming_energy = 0.0;
        p.design.quadratic_effect = 0.0;
        p.tuning.unison_width_cents = 0.0;
        p.volume_db = 0.0;
        p
    }

    fn render_mono(
        synth: &mut PianoSynthEngine,
        frames: usize,
        block_size: usize,
        params: &EngineParams,
        pedals: &PedalState,
    ) -> Vec<f32> {
        let mut mono = Vec::with_capacity(frames);
        let mut buf_l = vec![0.0_f32; block_size];
        let mut buf_r = vec![0.0_f32; block_size];
        let mut done = 0;
        while done < frames {
            let n = block_size.min(frames - done);
            synth.process_block(&mut buf_l[..n], &mut buf_r[..n], params, pedals);
            for i in 0..n {
                assert!(buf_l[i].is_finite() && buf_r[i].is_finite(), "non-finite sample at {}", done + i);
                mono.push(0.5 * (buf_l[i] + buf_r[i]));
            }
            done += n;
        }
        mono
    }

    fn rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
        (sq / samples.len() as f64).sqrt() as f32
    }

    /// Goertzel DFT power at target frequency `freq_hz`.
    fn goertzel_power(samples: &[f32], sample_rate: f32, freq_hz: f32) -> f32 {
        let n = samples.len();
        if n == 0 {
            return 0.0;
        }
        let omega = 2.0 * std::f64::consts::PI * (freq_hz as f64) / (sample_rate as f64);
        let coeff = 2.0 * omega.cos();
        let mut s_prev = 0.0_f64;
        let mut s_prev2 = 0.0_f64;
        for (idx, &x) in samples.iter().enumerate() {
            // Hann window to avoid spectral leakage across partials
            let w = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * idx as f64 / (n as f64)).cos());
            let s = (x as f64) * w + coeff * s_prev - s_prev2;
            s_prev2 = s_prev;
            s_prev = s;
        }
        let power = s_prev2 * s_prev2 + s_prev * s_prev - coeff * s_prev * s_prev2;
        (power / ((n as f64) * (n as f64))).max(0.0) as f32
    }

    fn interval_cents(f_low: f32, f_high: f32) -> f32 {
        1200.0 * (f_high / f_low).log2()
    }

    #[test]
    fn test_diapason_and_a4_tuning_accuracy() {
        let mut params = EngineParams::default();
        params.tuning.temperament = TemperamentId::Equal;
        params.tuning.octave_stretching = 1.0;

        // Standard A4 = 440 Hz
        params.tuning.diapason_hz = 440.0;
        assert!((compute_note_frequency(69, &params) - 440.0).abs() < 1e-4);
        assert!((compute_note_frequency(57, &params) - 220.0).abs() < 1e-4);
        assert!((compute_note_frequency(81, &params) - 880.0).abs() < 1e-4);
        assert!((compute_note_frequency(60, &params) - 261.62558).abs() < 1e-3);

        // Baroque A4 = 415 Hz
        params.tuning.diapason_hz = 415.0;
        assert!((compute_note_frequency(69, &params) - 415.0).abs() < 1e-4);
        assert!((compute_note_frequency(57, &params) - 207.5).abs() < 1e-4);

        // Orchestral A4 = 442 Hz
        params.tuning.diapason_hz = 442.0;
        assert!((compute_note_frequency(69, &params) - 442.0).abs() < 1e-4);
        assert!((compute_note_frequency(81, &params) - 884.0).abs() < 1e-4);

        // Pitch bend & octave stretching
        params.tuning.diapason_hz = 440.0;
        params.tuning.pitch_bend_semitones = 2.0;
        assert!((compute_note_frequency(69, &params) - 440.0 * 2.0_f32.powf(2.0 / 12.0)).abs() < 1e-3);

        params.tuning.pitch_bend_semitones = 0.0;
        params.tuning.octave_stretching = 1.3;
        assert!((compute_note_frequency(69, &params) - 440.0).abs() < 1e-4);
        assert!(compute_note_frequency(81, &params) > 880.0);
        assert!(compute_note_frequency(57, &params) < 220.0);

        // Non-finite input guard
        params.tuning.diapason_hz = f32::NAN;
        assert!(compute_note_frequency(69, &params).is_finite());
    }

    #[test]
    fn test_all_8_temperaments_a4_and_intervals() {
        let all_temps = [
            TemperamentId::Equal,
            TemperamentId::Pythagorean,
            TemperamentId::Zarlino,
            TemperamentId::MeanTone,
            TemperamentId::WellTempered,
            TemperamentId::WerckmeisterIII,
            TemperamentId::KirnbergerIII,
            TemperamentId::Vallotti,
        ];

        for temp in all_temps {
            let ratio_a = temperament_ratio(temp, 9);
            assert!(
                (ratio_a - 1.0).abs() < 1e-6,
                "A4 ratio for {:?} was {}, expected 1.0",
                temp,
                ratio_a
            );
            let mut params = EngineParams::default();
            params.tuning.temperament = temp;
            params.tuning.diapason_hz = 440.0;
            let f_a4 = compute_note_frequency(69, &params);
            assert!(
                (f_a4 - 440.0).abs() < 1e-3,
                "A4 frequency for {:?} was {}, expected 440.0",
                temp,
                f_a4
            );
        }

        let check_intervals = |temp: TemperamentId, expected_c_g_cents: f32, expected_c_e_cents: f32| {
            let mut params = EngineParams::default();
            params.tuning.temperament = temp;
            let c4 = compute_note_frequency(60, &params);
            let e4 = compute_note_frequency(64, &params);
            let g4 = compute_note_frequency(67, &params);
            let cg = interval_cents(c4, g4);
            let ce = interval_cents(c4, e4);
            assert!(
                (cg - expected_c_g_cents).abs() < 0.08,
                "{:?} C4-G4 fifth was {:.4} cents, expected {:.4}",
                temp,
                cg,
                expected_c_g_cents
            );
            assert!(
                (ce - expected_c_e_cents).abs() < 0.08,
                "{:?} C4-E4 third was {:.4} cents, expected {:.4}",
                temp,
                ce,
                expected_c_e_cents
            );
        };

        check_intervals(TemperamentId::Equal, 700.0, 400.0);
        check_intervals(TemperamentId::Pythagorean, 701.955, 407.820);
        check_intervals(TemperamentId::Zarlino, 701.955, 386.314);
        check_intervals(TemperamentId::MeanTone, 696.578, 386.314);
        check_intervals(TemperamentId::WellTempered, 696.578, 390.648);
        check_intervals(TemperamentId::WerckmeisterIII, 696.059, 390.186);
        check_intervals(TemperamentId::KirnbergerIII, 696.578, 386.314);
        check_intervals(TemperamentId::Vallotti, 698.045, 392.180);

        // Additional characteristic fifth checks (Pythagorean wolf F#4->C#5, Zarlino D4->A4, Vallotti F#4->C#5 pure)
        let mut p = EngineParams::default();
        p.tuning.temperament = TemperamentId::Pythagorean;
        let wolf = interval_cents(compute_note_frequency(66, &p), compute_note_frequency(73, &p));
        assert!((wolf - 678.495).abs() < 0.1, "Pythagorean wolf fifth was {}", wolf);

        p.tuning.temperament = TemperamentId::Zarlino;
        let da_fifth = interval_cents(compute_note_frequency(62, &p), compute_note_frequency(69, &p));
        assert!((da_fifth - 680.449).abs() < 0.1, "Zarlino D-A fifth was {}", da_fifth);

        p.tuning.temperament = TemperamentId::Vallotti;
        let pure_sharp_fifth = interval_cents(compute_note_frequency(66, &p), compute_note_frequency(73, &p));
        assert!((pure_sharp_fifth - 701.955).abs() < 0.1, "Vallotti F#-C# fifth was {}", pure_sharp_fifth);
    }

    #[test]
    fn test_velocity_dynamics_monotonicity() {
        let sr = 48000.0_f32;
        let inst = test_instrument();
        let params = clean_params();
        let pedals = PedalState::default();
        let f0 = compute_note_frequency(60, &params);

        let velocities = [24_u8, 56, 92, 127];
        let mut prev_rms = 0.0_f32;
        let mut prev_upper = 0.0_f32;
        let mut prev_ratio = 0.0_f32;

        for &vel in &velocities {
            let mut synth = PianoSynthEngine::new(sr);
            synth.note_on(60, vel, &params, &inst, &pedals);
            let buf = render_mono(&mut synth, 4096, 256, &params, &pedals);
            let level = rms(&buf);
            let p1 = goertzel_power(&buf, sr, f0);
            let upper: f32 = (2..=6)
                .map(|n| goertzel_power(&buf, sr, f0 * n as f32))
                .sum();
            let ratio = upper / p1.max(1e-12);

            assert!(level > prev_rms, "vel={} rms {} <= prev {}", vel, level, prev_rms);
            assert!(upper > prev_upper, "vel={} upper {} <= prev {}", vel, upper, prev_upper);
            assert!(ratio > prev_ratio, "vel={} ratio {} <= prev {}", vel, ratio, prev_ratio);

            prev_rms = level;
            prev_upper = upper;
            prev_ratio = ratio;
        }
    }

    #[test]
    fn test_sustain_and_reversed_sustain_pedals() {
        let sr = 48000.0_f32;
        let inst = test_instrument();
        let params = clean_params();

        // Normal sustain off vs on
        let mut synth_dry = PianoSynthEngine::new(sr);
        let mut synth_sus = PianoSynthEngine::new(sr);
        let dry_pedals = PedalState::default();
        let mut sus_pedals = PedalState::default();
        sus_pedals.sustain = 1.0;

        synth_dry.note_on(60, 100, &params, &inst, &dry_pedals);
        synth_sus.note_on(60, 100, &params, &inst, &sus_pedals);
        let _ = render_mono(&mut synth_dry, 2048, 256, &params, &dry_pedals);
        let _ = render_mono(&mut synth_sus, 2048, 256, &params, &sus_pedals);

        synth_dry.note_off(60, &params, &dry_pedals);
        synth_sus.note_off(60, &params, &sus_pedals);

        let dry_tail = render_mono(&mut synth_dry, 28000, 256, &params, &dry_pedals);
        let sus_tail = render_mono(&mut synth_sus, 28000, 256, &params, &sus_pedals);

        let dry_end_rms = rms(&dry_tail[24000..]);
        let sus_end_rms = rms(&sus_tail[24000..]);
        assert!(sus_end_rms > dry_end_rms * 10.0, "sus={} dry={}", sus_end_rms, dry_end_rms);
        assert_eq!(synth_sus.active_voice_count(), 1);

        // Release sustain pedal -> voice damps out
        synth_sus.update_pedals(&dry_pedals, &sus_pedals, &params);
        let _ = render_mono(&mut synth_sus, 80000, 256, &params, &dry_pedals);
        assert_eq!(synth_sus.active_voice_count(), 0);

        // Reversed sustain: pedal=0 holds, pedal=1 damps
        let mut rev_hold = PedalState::default();
        rev_hold.reversed_sustain = true;
        rev_hold.sustain = 0.0;
        let mut synth_rev = PianoSynthEngine::new(sr);
        synth_rev.note_on(60, 100, &params, &inst, &rev_hold);
        let _ = render_mono(&mut synth_rev, 2048, 256, &params, &rev_hold);
        synth_rev.note_off(60, &params, &rev_hold);
        let rev_tail = render_mono(&mut synth_rev, 28000, 256, &params, &rev_hold);
        assert!(rms(&rev_tail[24000..]) > dry_end_rms * 10.0);
    }

    #[test]
    fn test_sostenuto_pedal_selective_hold() {
        let sr = 48000.0_f32;
        let inst = test_instrument();
        let params = clean_params();
        let mut synth = PianoSynthEngine::new(sr);
        let mut pedals = PedalState::default();

        // Strike C4 before pressing sostenuto
        synth.note_on(60, 100, &params, &inst, &pedals);
        let _ = render_mono(&mut synth, 1024, 256, &params, &pedals);

        // Latch sostenuto
        let prev = pedals.clone();
        pedals.sostenuto = 1.0;
        synth.update_pedals(&pedals, &prev, &params);

        // Strike G4 after sostenuto is already down
        synth.note_on(67, 100, &params, &inst, &pedals);
        let _ = render_mono(&mut synth, 1024, 256, &params, &pedals);

        // Release both keys
        synth.note_off(60, &params, &pedals);
        synth.note_off(67, &params, &pedals);

        let tail = render_mono(&mut synth, 24000, 256, &params, &pedals);
        let window = &tail[16000..24000];
        let p_c4 = goertzel_power(window, sr, compute_note_frequency(60, &params));
        let p_g4 = goertzel_power(window, sr, compute_note_frequency(67, &params));

        assert!(p_c4 > p_g4 * 50.0, "C4 should sustain while G4 damps: p_c4={}, p_g4={}", p_c4, p_g4);

        // Releasing sostenuto releases C4
        let prev2 = pedals.clone();
        pedals.sostenuto = 0.0;
        synth.update_pedals(&pedals, &prev2, &params);
        let _ = render_mono(&mut synth, 80000, 256, &params, &pedals);
        assert_eq!(synth.active_voice_count(), 0);
    }

    #[test]
    fn test_soft_pedal_reduces_level_and_brightness() {
        let sr = 48000.0_f32;
        let inst = test_instrument();
        let params = clean_params();
        let f0 = compute_note_frequency(60, &params);

        let mut synth_norm = PianoSynthEngine::new(sr);
        let p_norm = PedalState::default();
        synth_norm.note_on(60, 100, &params, &inst, &p_norm);
        let buf_norm = render_mono(&mut synth_norm, 4096, 256, &params, &p_norm);

        let mut synth_soft = PianoSynthEngine::new(sr);
        let mut p_soft = PedalState::default();
        p_soft.soft = 1.0;
        synth_soft.note_on(60, 100, &params, &inst, &p_soft);
        let buf_soft = render_mono(&mut synth_soft, 4096, 256, &params, &p_soft);

        assert!(rms(&buf_soft) < rms(&buf_norm) * 0.9);
        let bright_norm: f32 = (2..=6).map(|n| goertzel_power(&buf_norm, sr, f0 * n as f32)).sum::<f32>()
            / goertzel_power(&buf_norm, sr, f0).max(1e-12);
        let bright_soft: f32 = (2..=6).map(|n| goertzel_power(&buf_soft, sr, f0 * n as f32)).sum::<f32>()
            / goertzel_power(&buf_soft, sr, f0).max(1e-12);
        assert!(bright_soft < bright_norm, "soft brightness {} should be < normal {}", bright_soft, bright_norm);
    }

    #[test]
    fn test_harmonic_pedal_damps_fundamental_sustains_overtones() {
        let sr = 48000.0_f32;
        let inst = test_instrument();
        let params = clean_params();
        let f0 = compute_note_frequency(60, &params);

        let run_case = |pedals: &PedalState| -> (f32, f32, f32) {
            let mut synth = PianoSynthEngine::new(sr);
            synth.note_on(60, 110, &params, &inst, pedals);
            let _ = render_mono(&mut synth, 2048, 256, &params, pedals);
            synth.note_off(60, &params, pedals);
            let tail = render_mono(&mut synth, 28000, 256, &params, pedals);
            let w = &tail[19808..28000];
            let p1 = goertzel_power(w, sr, f0);
            let p2 = goertzel_power(w, sr, f0 * 2.0);
            let p3 = goertzel_power(w, sr, f0 * 3.0);
            (p1, p2, p3)
        };

        let mut p_sustain = PedalState::default();
        p_sustain.sustain = 1.0;
        let mut p_harmonic = PedalState::default();
        p_harmonic.harmonic = 1.0;
        let p_dry = PedalState::default();

        let (sus_p1, sus_p2, sus_p3) = run_case(&p_sustain);
        let (harm_p1, harm_p2, harm_p3) = run_case(&p_harmonic);
        let (dry_p1, dry_p2, dry_p3) = run_case(&p_dry);

        // Fundamental damped rapidly relative to sustain pedal
        assert!(
            harm_p1 < sus_p1 * 0.05,
            "harmonic fundamental {} should be heavily damped vs sustain {}",
            harm_p1,
            sus_p1
        );
        assert!(
            harm_p1 < dry_p1 * 2.0 + 1e-7,
            "harmonic fundamental {} should damp near dry {}",
            harm_p1,
            dry_p1
        );

        // Upper overtones (partials >= 2) remain sustained relative to dry release
        assert!(
            harm_p2 > dry_p2 * 25.0 && harm_p3 > dry_p3 * 25.0,
            "harmonic partials >=2 should sustain: harm=({}, {}), dry=({}, {})",
            harm_p2,
            harm_p3,
            dry_p2,
            dry_p3
        );
        assert!(
            (harm_p2 / sus_p2) > 0.7 && (harm_p3 / sus_p3) > 0.7,
            "harmonic partials >=2 should retain sustain energy vs sustain pedal"
        );
    }

    #[test]
    fn test_rattle_pedal_buzz_and_threshold() {
        let sr = 48000.0_f32;
        let inst = test_instrument();
        let params = clean_params();
        let f0 = compute_note_frequency(48, &params); // C3

        let p_zero = PedalState::default();
        let mut p_below = PedalState::default();
        p_below.rattle = 0.005; // <= 0.01 threshold
        let mut p_rattle = PedalState::default();
        p_rattle.rattle = 0.85;

        let mut s_zero = PianoSynthEngine::new(sr);
        let mut s_below = PianoSynthEngine::new(sr);
        let mut s_rattle = PianoSynthEngine::new(sr);

        s_zero.note_on(48, 110, &params, &inst, &p_zero);
        s_below.note_on(48, 110, &params, &inst, &p_below);
        s_rattle.note_on(48, 110, &params, &inst, &p_rattle);

        let b_zero = render_mono(&mut s_zero, 8192, 256, &params, &p_zero);
        let b_below = render_mono(&mut s_below, 8192, 256, &params, &p_below);
        let b_rattle = render_mono(&mut s_rattle, 8192, 256, &params, &p_rattle);

        let max_below_diff = b_zero
            .iter()
            .zip(b_below.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        assert!(max_below_diff < 1e-6, "rattle <= 0.01 must not alter signal");

        // Rattle injects buzz harmonics (5th/7th/9th/11th partials) & amplitude modulation
        let jangle_zero = goertzel_power(&b_zero, sr, f0 * 5.0)
            + goertzel_power(&b_zero, sr, f0 * 7.0)
            + goertzel_power(&b_zero, sr, f0 * 9.0)
            + goertzel_power(&b_zero, sr, f0 * 11.0);
        let jangle_rattle = goertzel_power(&b_rattle, sr, f0 * 5.0)
            + goertzel_power(&b_rattle, sr, f0 * 7.0)
            + goertzel_power(&b_rattle, sr, f0 * 9.0)
            + goertzel_power(&b_rattle, sr, f0 * 11.0);
        let high_buzz_zero = goertzel_power(&b_zero, sr, f0 * 9.0)
            + goertzel_power(&b_zero, sr, f0 * 11.0);
        let high_buzz_rattle = goertzel_power(&b_rattle, sr, f0 * 9.0)
            + goertzel_power(&b_rattle, sr, f0 * 11.0);
        assert!(
            jangle_rattle > jangle_zero * 2.5,
            "rattle should boost buzz/jangle overtones: rattle={}, zero={}",
            jangle_rattle,
            jangle_zero
        );
        assert!(
            high_buzz_rattle > high_buzz_zero * 50.0,
            "rattle should generate high buzz harmonics 9/11: rattle={}, zero={}",
            high_buzz_rattle,
            high_buzz_zero
        );
    }

    #[test]
    fn test_key_release_mechanical_noise() {
        let sr = 48000.0_f32;
        let inst = test_instrument();
        let mut pedals = PedalState::default();
        pedals.sustain = 1.0; // Isolate key-release mechanical noise from damper noise

        let run_release = |kr_level: f32| -> Vec<f32> {
            let mut params = clean_params();
            params.action.key_release_noise = kr_level;
            let mut synth = PianoSynthEngine::new(sr);
            synth.note_on(60, 100, &params, &inst, &pedals);
            let _ = render_mono(&mut synth, 2048, 256, &params, &pedals);
            synth.note_off(60, &params, &pedals);
            render_mono(&mut synth, 1024, 256, &params, &pedals)
        };

        let dry = run_release(0.0);
        let med = run_release(0.35);
        let high = run_release(1.0);

        let diff_rms = |a: &[f32], b: &[f32]| -> f32 {
            let diffs: Vec<f32> = a.iter().zip(b.iter()).map(|(x, y)| x - y).collect();
            rms(&diffs)
        };

        let d_med = diff_rms(&med, &dry);
        let d_high = diff_rms(&high, &dry);
        assert!(d_med > 1e-4, "medium key_release_noise should produce transient: {}", d_med);
        assert!(d_high > d_med * 2.0, "higher key_release_noise should scale monotonically: med={}, high={}", d_med, d_high);
    }

    #[test]
    fn test_pro_note_edit_per_key_offsets() {
        let sr = 48000.0_f32;
        let inst = test_instrument();
        let pedals = PedalState::default();

        // 1. Detune cents
        let mut params = clean_params();
        let base_c4 = compute_note_frequency(60, &params);
        let base_d4 = compute_note_frequency(62, &params);
        params.note_edit.insert(
            60,
            NoteEditOffset {
                detune_cents: 50.0,
                gain_db: 0.0,
                hammer_hardness_offset: 0.0,
                impedance_offset: 0.0,
            },
        );
        let detuned_c4 = compute_note_frequency(60, &params);
        assert!((detuned_c4 - base_c4 * 2.0_f32.powf(50.0 / 1200.0)).abs() < 1e-3);
        assert!((compute_note_frequency(62, &params) - base_d4).abs() < 1e-5);

        // 2. Per-key gain_db (+6 dB vs -6 dB)
        let mut p_plus = clean_params();
        p_plus.note_edit.insert(
            60,
            NoteEditOffset { gain_db: 6.0, ..Default::default() },
        );
        let mut p_minus = clean_params();
        p_minus.note_edit.insert(
            60,
            NoteEditOffset { gain_db: -6.0, ..Default::default() },
        );
        let mut s_plus = PianoSynthEngine::new(sr);
        let mut s_minus = PianoSynthEngine::new(sr);
        s_plus.note_on(60, 80, &p_plus, &inst, &pedals);
        s_minus.note_on(60, 80, &p_minus, &inst, &pedals);
        let r_plus = rms(&render_mono(&mut s_plus, 4096, 256, &p_plus, &pedals));
        let r_minus = rms(&render_mono(&mut s_minus, 4096, 256, &p_minus, &pedals));
        let gain_ratio = r_plus / r_minus.max(1e-9);
        assert!((gain_ratio - 3.981).abs() < 0.25, "12dB diff should yield ~3.98x RMS, got {}", gain_ratio);

        // 3. Per-key hammer_hardness_offset
        let mut p_soft_h = clean_params();
        p_soft_h.note_edit.insert(
            60,
            NoteEditOffset { hammer_hardness_offset: -0.25, ..Default::default() },
        );
        let mut p_hard_h = clean_params();
        p_hard_h.note_edit.insert(
            60,
            NoteEditOffset { hammer_hardness_offset: 0.45, ..Default::default() },
        );
        let mut s_sh = PianoSynthEngine::new(sr);
        let mut s_hh = PianoSynthEngine::new(sr);
        s_sh.note_on(60, 80, &p_soft_h, &inst, &pedals);
        s_hh.note_on(60, 80, &p_hard_h, &inst, &pedals);
        let b_sh = render_mono(&mut s_sh, 4096, 256, &p_soft_h, &pedals);
        let b_hh = render_mono(&mut s_hh, 4096, 256, &p_hard_h, &pedals);

        let br_sh: f32 = (2..=6).map(|n| goertzel_power(&b_sh, sr, base_c4 * n as f32)).sum::<f32>()
            / goertzel_power(&b_sh, sr, base_c4).max(1e-12);
        let br_hh: f32 = (2..=6).map(|n| goertzel_power(&b_hh, sr, base_c4 * n as f32)).sum::<f32>()
            / goertzel_power(&b_hh, sr, base_c4).max(1e-12);
        assert!(br_hh > br_sh * 1.5, "harder hammer offset brightness {} <= soft {}", br_hh, br_sh);
    }

    #[test]
    fn test_sample_rate_and_dynamic_buffer_sizes() {
        let inst = test_instrument();
        let params = clean_params();
        let pedals = PedalState::default();

        let mut synth = PianoSynthEngine::new(48000.0);
        synth.note_on(69, 100, &params, &inst, &pedals); // A4 = 440Hz
        let _ = render_mono(&mut synth, 1024, 256, &params, &pedals);

        // Switch sample rate mid-note to 96kHz and check pitch stays 440Hz
        synth.set_sample_rate(96000.0);
        let buf_96k = render_mono(&mut synth, 8192, 513, &params, &pedals);
        let p_440 = goertzel_power(&buf_96k, 96000.0, 440.0);
        let p_220 = goertzel_power(&buf_96k, 96000.0, 220.0);
        assert!(p_440 > p_220 * 50.0, "A4 pitch drifted after set_sample_rate(96000): p440={}, p220={}", p_440, p_220);

        // Invalid sample rates guard
        synth.set_sample_rate(f32::NAN);
        synth.set_sample_rate(-44100.0);
        synth.set_sample_rate(44100.0);

        // Dynamic / irregular / zero / mismatched buffer sizes
        for &bs in &[0_usize, 1, 7, 64, 127, 256, 513, 2048, 4096] {
            let mut l = vec![0.0_f32; bs];
            let mut r = vec![0.0_f32; bs + 13];
            synth.process_block(&mut l, &mut r, &params, &pedals);
            for &s in l.iter().chain(r.iter()) {
                assert!(s.is_finite() && s.abs() <= 1.0);
            }
        }
    }

    #[test]
    fn test_64_voice_polyphony_stress() {
        let sr = 48000.0_f32;
        let inst = test_instrument();
        let mut params = EngineParams::default();
        params.volume_db = 0.0;
        let mut pedals = PedalState::default();
        pedals.sustain = 1.0;
        pedals.rattle = 0.5;

        let mut synth = PianoSynthEngine::new(sr);
        for note in 24_u8..88_u8 {
            synth.note_on(note, 127, &params, &inst, &pedals);
        }
        assert_eq!(synth.active_voice_count(), 64);

        let mut buf_l = vec![0.0_f32; 256];
        let mut buf_r = vec![0.0_f32; 256];
        let mut max_abs = 0.0_f32;
        let mut sq_sum = 0.0_f64;
        let total_blocks = (sr as usize * 2) / 256;

        for b in 0..total_blocks {
            synth.process_block(&mut buf_l, &mut buf_r, &params, &pedals);
            for i in 0..256 {
                let l = buf_l[i];
                let r = buf_r[i];
                assert!(l.is_finite(), "NaN/Inf in L at block {} sample {}", b, i);
                assert!(r.is_finite(), "NaN/Inf in R at block {} sample {}", b, i);
                assert!(l.abs() <= 1.0, "L exceeded 1.0: {}", l);
                assert!(r.abs() <= 1.0, "R exceeded 1.0: {}", r);
                max_abs = max_abs.max(l.abs()).max(r.abs());
                sq_sum += (l as f64) * (l as f64) + (r as f64) * (r as f64);
            }
        }
        let total_rms = (sq_sum / (total_blocks * 256 * 2) as f64).sqrt();
        assert!(total_rms > 0.05, "64-voice stress produced unexpected silence");
        assert!(max_abs <= 1.0);

        synth.all_notes_off();
        assert_eq!(synth.active_voice_count(), 0);
    }
}
