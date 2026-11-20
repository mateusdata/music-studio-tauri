use crate::contracts::{
    EngineParams, ExciterType, InstrumentModel, PedalState, PlayMode, TemperamentId,
};
use std::f32::consts::PI;

pub const MAX_POLYPHONY: usize = 128;
const NUM_PARTIALS: usize = 8;
const MAX_BOUNCE_EVENTS: usize = 64;
const DIATONIC_PC_TO_DEG: [usize; 12] = [0, 0, 1, 1, 2, 3, 3, 4, 4, 5, 5, 6];

#[derive(Clone, Copy)]
struct BounceEvent {
    active: bool,
    samples_until_trigger: u64,
    midi_note: u8,
    velocity_u8: u8,
    exciter: ExciterType,
    inharmonicity_base: f32,
    decay_scale: f32,
}

impl Default for BounceEvent {
    fn default() -> Self {
        Self {
            active: false,
            samples_until_trigger: 0,
            midi_note: 60,
            velocity_u8: 64,
            exciter: ExciterType::FeltHammer,
            inharmonicity_base: 0.00018,
            decay_scale: 1.0,
        }
    }
}

#[inline]
fn eval_lfo(phase: f32, shape: f32, skew: f32) -> f32 {
    let u = phase.rem_euclid(1.0);
    let sk = skew.clamp(0.05, 0.95);
    let u_skewed = if u < sk {
        0.5 * (u / sk)
    } else {
        0.5 + 0.5 * ((u - sk) / (1.0 - sk))
    };
    let tri = if u_skewed < 0.25 {
        4.0 * u_skewed
    } else if u_skewed < 0.75 {
        2.0 - 4.0 * u_skewed
    } else {
        4.0 * u_skewed - 4.0
    };
    let sine = (2.0 * PI * u_skewed).sin();
    let sh = shape.clamp(0.0, 1.0);
    tri * (1.0 - sh) + sine * sh
}

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

/// Computes exact fundamental frequency (Hz) applying PlayMode (Diatonic Harp / Guitar Fret),
/// Diapason, Temperament, Octave Stretching, Pitch Bend, and per-note PRO tuning offset.
pub fn compute_note_frequency(midi_note: u8, params: &EngineParams) -> f32 {
    let diapason = finite_or(params.tuning.diapason_hz, 440.0).clamp(100.0, 1200.0);
    let stretch = finite_or(params.tuning.octave_stretching, 1.0).clamp(0.5, 2.0);
    let pitch_bend = finite_or(params.tuning.pitch_bend_semitones, 0.0).clamp(-24.0, 24.0);

    let play_shift: i16 = match params.play_mode.mode {
        PlayMode::Standard => 0,
        PlayMode::DiatonicHarp => {
            let deg = DIATONIC_PC_TO_DEG[(midi_note % 12) as usize];
            params.play_mode.harp_pedals[deg].clamp(-1, 1) as i16
        }
        PlayMode::Guitar => {
            if params.play_mode.guitar_fret > 0 {
                (params.play_mode.guitar_fret as i16).min(36)
            } else {
                0
            }
        }
    };

    let effective_midi = (midi_note as i16 + play_shift).clamp(0, 127);
    let base_semi = effective_midi as f32 - 69.0;
    let stretched_semi = base_semi
        + (stretch - 1.0) * (base_semi / 12.0) * 0.045 * base_semi.abs().sqrt();
    let mut freq = diapason * 2.0_f32.powf((stretched_semi + pitch_bend) / 12.0);

    let pc = (effective_midi.rem_euclid(12)) as usize;
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
    lfo_phase: f32,
    vel_norm_raw: f32,
    is_guitar_legato: bool,
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
            lfo_phase: 0.0,
            vel_norm_raw: 0.0,
            is_guitar_legato: false,
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
    bounce_events: [BounceEvent; MAX_BOUNCE_EVENTS],
    global_lfo_phase: f32,
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
            bounce_events: [BounceEvent::default(); MAX_BOUNCE_EVENTS],
            global_lfo_phase: 0.0,
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
        let sr_scale = (new_sr as f64) / (old_sr as f64);
        for b in &mut self.bounce_events {
            if b.active {
                b.samples_until_trigger = ((b.samples_until_trigger as f64) * sr_scale).round().max(1.0) as u64;
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
        for b in &mut self.bounce_events {
            b.active = false;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn trigger_voice(
        &mut self,
        midi_note: u8,
        velocity_u8: u8,
        params: &EngineParams,
        exciter: ExciterType,
        inharmonicity_base: f32,
        decay_scale: f32,
        pedals: &PedalState,
        is_bounce: bool,
    ) {
        // Check keyboard range if not extended
        if !params.action.keyboard_range_extended && !(21..=108).contains(&midi_note) {
            return;
        }

        // Allocate voice (reuse free slot, then oldest released voice, then oldest held voice)
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
        let play_mode = params.play_mode.mode;
        let harp_hardness_scale = if play_mode == PlayMode::DiatonicHarp { 0.88 } else { 1.0 };
        let hardness = ((hardness_base * harp_hardness_scale)
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
        let b_inharm = (finite_or(inharmonicity_base, 0.00018).max(0.0)
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
        let play_decay_scale = match play_mode {
            PlayMode::DiatonicHarp => 1.15,
            PlayMode::Guitar => 0.82 + 0.36 * finite_or(params.play_mode.guitar_body, 0.5).clamp(0.0, 1.5),
            PlayMode::Standard => 1.0,
        };
        let base_decay_s = ((7.5 - 5.8 * midi_rel).max(0.4)
            * finite_or(decay_scale, 1.0).clamp(0.1, 5.0)
            * play_decay_scale
            * (1.35 - impedance * 0.65).max(0.15))
            .max(0.1);

        let guitar_body = finite_or(params.play_mode.guitar_body, 0.5).clamp(0.0, 1.5);
        let guitar_legato = play_mode == PlayMode::Guitar && params.play_mode.guitar_legato;
        let lfo_init_phase = if params.note_fx.lfo_phase_locked {
            self.global_lfo_phase
        } else {
            ((self.note_counter as f32) * 0.37).fract()
        };

        let v = &mut self.voices[idx];
        v.active = true;
        v.midi_note = midi_note;
        v.key_down = true;
        v.sustained = false;
        let _ = is_bounce;
        v.sostenuto_held = pedals.sostenuto >= 0.5 && pedals.super_sostenuto;
        v.harmonic_held = pedals.harmonic >= 0.5;
        v.velocity = vel_eff * soft_atten * mute_atten * note_gain;
        v.freq = freq;
        v.age_samples = 0;
        v.exciter = exciter;
        v.bloom_state = 0.0;
        v.lfo_phase = lfo_init_phase;
        v.vel_norm_raw = vel_norm;
        v.is_guitar_legato = guitar_legato;
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
            let exciter_shape = match exciter {
                ExciterType::TineHammer => if p_idx == 0 { 1.4 } else if p_idx < 3 { 0.8 } else { 0.25 },
                ExciterType::ReedHammer => if p_idx % 2 == 0 { 1.15 } else { 0.45 },
                ExciterType::PluckPlectrum => 1.15 * (0.85 + 0.15 * n),
                ExciterType::MalletHard | ExciterType::MalletSoft => {
                    if p_idx == 0 { 1.3 } else if p_idx == 1 { 0.55 } else { 0.12 }
                }
                ExciterType::PreparedBolt => if p_idx % 2 == 1 { 1.35 } else { 0.7 },
                _ => 1.0,
            };
            let play_shape = match play_mode {
                PlayMode::DiatonicHarp => if p_idx < 3 { 1.10 } else { 0.82 },
                PlayMode::Guitar => if p_idx < 3 { 0.85 + 0.30 * guitar_body } else { 0.90 },
                PlayMode::Standard => 1.0,
            };

            let amp = v.velocity * strike_comb * rolloff * prof_weight * exciter_shape * play_shape * 0.24;
            v.partial_amps[p_idx] = amp;
            v.partial_levels[p_idx] = amp;

            let partial_decay_s = (base_decay_s / (1.0 + 0.48 * (n - 1.0))).max(0.08);
            v.partial_decays[p_idx] = (-1.0 / (partial_decay_s * self.sample_rate)).exp();
        }

        let legato_noise_scale = if guitar_legato { 0.30 } else { 1.0 };
        v.hammer_noise_env =
            v.velocity * finite_or(params.voicing.hammer_noise, 0.35).clamp(0.0, 2.0) * 0.35 * legato_noise_scale;
        v.hammer_noise_lp = 0.0;
        v.damper_noise_env = 0.0;
        v.key_release_noise_env = 0.0;
        v.key_release_lp = 0.0;
    }

    fn schedule_bounces(
        &mut self,
        midi_note: u8,
        velocity_u8: u8,
        params: &EngineParams,
        instrument: &InstrumentModel,
    ) {
        let max_bounces = params.bounce.max_bounces.min(16);
        if max_bounces == 0 {
            return;
        }
        let vel_norm = (velocity_u8 as f32 / 127.0).clamp(0.01, 1.0);
        let vel_sens = finite_or(params.bounce.velocity_sensitivity, 0.5).clamp(0.0, 1.0);
        let delay_scale = (1.0 - vel_sens * 0.45 * (vel_norm - 0.5)).clamp(0.4, 1.6);
        let mut cur_delay_ms = (finite_or(params.bounce.delay_ms, 120.0).clamp(15.0, 2000.0)) * delay_scale;
        let delay_loss = finite_or(params.bounce.delay_loss, 0.15).clamp(0.0, 0.85);
        let vel_loss = finite_or(params.bounce.velocity_loss, 0.35).clamp(0.05, 0.90);
        let humanize = finite_or(params.bounce.humanization, 0.1).clamp(0.0, 1.0);

        let mut cur_vel = velocity_u8 as f32;
        let mut cumulative_samples = 0_u64;

        for _ in 0..max_bounces {
            cur_vel *= 1.0 - vel_loss;
            let next_vel_u8 = cur_vel.round().clamp(0.0, 127.0) as u8;
            if next_vel_u8 < 2 {
                break;
            }
            self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
            let jitter = ((self.rng >> 16) as f32 / 65535.0 - 0.5) * 2.0 * humanize * 0.12;
            let step_ms = (cur_delay_ms * (1.0 + jitter)).clamp(10.0, 2000.0);
            let step_samples = ((step_ms * 0.001) * self.sample_rate).round().max(1.0) as u64;
            cumulative_samples = cumulative_samples.saturating_add(step_samples);
            cur_delay_ms = (cur_delay_ms * (1.0 - delay_loss)).clamp(10.0, 2000.0);

            let slot = self
                .bounce_events
                .iter()
                .position(|b| !b.active)
                .or_else(|| {
                    self.bounce_events
                        .iter()
                        .enumerate()
                        .max_by_key(|(_, b)| b.samples_until_trigger)
                        .map(|(i, _)| i)
                });
            if let Some(s_idx) = slot {
                self.bounce_events[s_idx] = BounceEvent {
                    active: true,
                    samples_until_trigger: cumulative_samples,
                    midi_note,
                    velocity_u8: next_vel_u8,
                    exciter: instrument.exciter,
                    inharmonicity_base: instrument.inharmonicity_base,
                    decay_scale: instrument.decay_scale,
                };
            }
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
        if !params.action.keyboard_range_extended && !(21..=108).contains(&midi_note) {
            return;
        }

        self.trigger_voice(
            midi_note,
            velocity_u8,
            params,
            instrument.exciter,
            instrument.inharmonicity_base,
            instrument.decay_scale,
            pedals,
            false,
        );

        if params.bounce.enabled {
            self.schedule_bounces(midi_note, velocity_u8, params, instrument);
        }
    }

    pub fn note_off(&mut self, midi_note: u8, params: &EngineParams, pedals: &PedalState) {
        let sustain_on = if pedals.reversed_sustain {
            pedals.sustain < 0.5
        } else {
            pedals.sustain >= 0.5
        };
        let key_rel_amt = finite_or(params.action.key_release_noise, 0.22).clamp(0.0, 2.0);
        let damper_amt = finite_or(params.action.damper_noise, 0.28).clamp(0.0, 2.0);

        for b in &mut self.bounce_events {
            if b.active && b.midi_note == midi_note {
                b.active = false;
            }
        }

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
        let legato_release_coeff = (-1.0 / ((damper_release_s * 1.8).max(0.05) * self.sample_rate)).exp();
        let hammer_lp_coeff = (0.18 + finite_or(params.voicing.hammer_tone, 0.5) * 0.68).clamp(0.05, 0.92);
        let unison_mix = finite_or(params.tuning.unison_balance, 0.5).clamp(0.0, 1.0);
        let bloom_amt = finite_or(params.design.blooming_energy, 0.28).clamp(0.0, 2.0) * 0.25;
        let symp_play_scale = if params.play_mode.mode == PlayMode::Guitar {
            0.4 + finite_or(params.play_mode.guitar_body, 0.5).clamp(0.0, 2.0) * 1.2
        } else {
            1.0
        };
        let symp_amt = finite_or(params.design.sympathetic_resonance, 0.45).clamp(0.0, 2.0)
            * (0.35 + if sustain_on { 0.65 } else { 0.0 })
            * symp_play_scale;
        let duplex_amt = finite_or(params.design.duplex_scale_resonance, 0.35).clamp(0.0, 2.0) * 0.15;
        let quad = finite_or(params.design.quadratic_effect, 0.22).clamp(0.0, 2.0) * 0.12;
        let vol = 10.0_f32.powf(finite_or(params.volume_db, -4.0).clamp(-80.0, 24.0) / 20.0);

        let lfo_rate_hz = finite_or(params.note_fx.lfo_rate_hz, 5.2).clamp(0.05, 40.0);
        let lfo_inc = lfo_rate_hz / self.sample_rate;
        let lfo_shape = finite_or(params.note_fx.lfo_shape, 0.5).clamp(0.0, 1.0);
        let lfo_skew = finite_or(params.note_fx.lfo_skew, 0.5).clamp(0.05, 0.95);
        let onset_samples = (finite_or(params.note_fx.onset_duration_ms, 80.0).clamp(0.0, 5000.0) * 0.001) * self.sample_rate;
        let vib_base_cents = finite_or(params.note_fx.vibrato_cents, 0.0).clamp(-200.0, 200.0);
        let vib_vel_cents = finite_or(params.note_fx.vibrato_offset_cents, 0.0).clamp(-200.0, 200.0);
        let trem_db = finite_or(params.note_fx.tremolo_depth_db, 0.0).clamp(0.0, 24.0);
        let trem_phase_offset = finite_or(params.note_fx.tremolo_phase, 0.0).clamp(0.0, 1.0) * 0.5;
        let smear = finite_or(params.note_fx.smear, 0.0).clamp(0.0, 1.0);
        let attack_param = finite_or(params.note_fx.attack_envelope, 0.5).clamp(0.0, 1.0);
        let attack_base_s = if attack_param <= 0.5 {
            0.0003 + (attack_param / 0.5) * 0.0012
        } else {
            0.0015 + ((attack_param - 0.5) / 0.5).powi(2) * 0.080
        };
        let attack_samples = attack_base_s * self.sample_rate;
        let active_note_fx = vib_base_cents.abs() > 1e-4 || vib_vel_cents.abs() > 1e-4 || trem_db > 1e-4;

        let hammer_noise_decay = (-1.0 / (0.0018 * self.sample_rate)).exp();
        let damper_noise_decay = (-1.0 / (0.0026 * self.sample_rate)).exp();
        let key_release_decay = (-1.0 / (0.0045 * self.sample_rate)).exp();
        let key_release_lp_coeff = (1.0 - (-2.0 * PI * 380.0 / self.sample_rate).exp()).clamp(0.02, 0.5);
        let bloom_tau = self.sample_rate * 0.35;
        let bloom_state_decay = (-1.0 / (0.0416 * self.sample_rate)).exp();
        let symp_in_coeff = (1.0 - (-1.0 / (0.035 * self.sample_rate)).exp()).clamp(1e-5, 0.05);
        let symp_lp_coeff = (1.0 - (-1.0 / (0.0007 * self.sample_rate)).exp()).clamp(0.001, 0.25);
        let symp_bp_decay = (-1.0 / (0.0026 * self.sample_rate)).exp();
        let pedal_thump_decay = (-1.0 / (0.0042 * self.sample_rate)).exp();
        let nyq_inc = PI * 0.9;

        for i in 0..len {
            // Sample-accurate Mallet Bounce event dispatch (zero allocation)
            for b_idx in 0..MAX_BOUNCE_EVENTS {
                if !self.bounce_events[b_idx].active {
                    continue;
                }
                if self.bounce_events[b_idx].samples_until_trigger > 0 {
                    self.bounce_events[b_idx].samples_until_trigger -= 1;
                }
                if self.bounce_events[b_idx].samples_until_trigger == 0 {
                    let ev = self.bounce_events[b_idx];
                    self.bounce_events[b_idx].active = false;
                    self.trigger_voice(
                        ev.midi_note,
                        ev.velocity_u8,
                        params,
                        ev.exciter,
                        ev.inharmonicity_base,
                        ev.decay_scale,
                        pedals,
                        true,
                    );
                }
            }
            self.global_lfo_phase = (self.global_lfo_phase + lfo_inc).fract();

            let mut sum_l = 0.0_f32;
            let mut sum_r = 0.0_f32;
            let mut body_energy = 0.0_f32;

            for v in &mut self.voices {
                if !v.active {
                    continue;
                }

                let full_held = v.key_down || (v.sustained && sustain_on) || v.sostenuto_held;
                let harmonic_only_held = !full_held && v.harmonic_held && pedals.harmonic >= 0.5;
                let voice_release_coeff = if v.is_guitar_legato {
                    legato_release_coeff
                } else {
                    release_coeff
                };
                let mut sample_l = 0.0_f32;
                let mut sample_r = 0.0_f32;
                let mut max_level = 0.0_f32;

                // Attack smoothing (scaled by NoteFx attack_envelope and guitar legato)
                let v_attack_samples = if v.is_guitar_legato {
                    attack_samples * 2.2
                } else {
                    attack_samples
                };
                let attack_env = (v.age_samples as f32 / v_attack_samples.max(1.0)).min(1.0);

                // Per-voice LFO modulation (vibrato + stereo tremolo)
                let (pitch_mult, trem_gain_l, trem_gain_r) = if active_note_fx {
                    let onset_env = if onset_samples < 1.0 {
                        1.0
                    } else {
                        (v.age_samples as f32 / onset_samples).min(1.0)
                    };
                    let lfo_l = eval_lfo(v.lfo_phase, lfo_shape, lfo_skew);
                    let lfo_r = if trem_phase_offset > 1e-4 {
                        eval_lfo((v.lfo_phase + trem_phase_offset).fract(), lfo_shape, lfo_skew)
                    } else {
                        lfo_l
                    };
                    let vib_cents = (vib_base_cents + vib_vel_cents * v.vel_norm_raw).clamp(-200.0, 200.0);
                    let p_mult = if vib_cents.abs() > 1e-4 {
                        2.0_f32.powf((lfo_l * onset_env * vib_cents) / 1200.0)
                    } else {
                        1.0
                    };
                    let (tg_l, tg_r) = if trem_db > 1e-4 {
                        let att_l = -trem_db * 0.5 * (1.0 - lfo_l) * onset_env;
                        let att_r = -trem_db * 0.5 * (1.0 - lfo_r) * onset_env;
                        (10.0_f32.powf(att_l / 20.0), 10.0_f32.powf(att_r / 20.0))
                    } else {
                        (1.0, 1.0)
                    };
                    (p_mult, tg_l, tg_r)
                } else {
                    (1.0, 1.0, 1.0)
                };
                v.lfo_phase = (v.lfo_phase + lfo_inc).fract();

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
                    let partial_weight = level * if p_idx >= 2 { bloom_mult } else { 1.0 };
                    let mixed = s_a * (1.0 - unison_mix * 0.45) + s_b * (unison_mix * 0.45);
                    if smear > 1e-4 {
                        let stereo_sep = (s_a - s_b) * (smear * 0.28);
                        sample_l += (mixed + stereo_sep) * partial_weight;
                        sample_r += (mixed - stereo_sep) * partial_weight;
                    } else {
                        let w = mixed * partial_weight;
                        sample_l += w;
                        sample_r += w;
                    }

                    let inc_a = (v.phase_incs_a[p_idx] * pitch_mult).min(2.0 * PI * 0.48);
                    let smear_factor = if smear > 1e-4 {
                        let sign = if p_idx % 2 == 0 { 1.0 } else { -1.0 };
                        1.0 + sign * smear * 0.004 * (p_idx as f32 + 1.0)
                    } else {
                        1.0
                    };
                    let inc_b = (v.phase_incs_b[p_idx] * pitch_mult * smear_factor).min(2.0 * PI * 0.48);

                    let mut next_pa = pa + inc_a;
                    if next_pa >= 2.0 * PI {
                        next_pa -= 2.0 * PI;
                    }
                    v.phases_a[p_idx] = next_pa;

                    let mut next_pb = pb + inc_b;
                    if next_pb >= 2.0 * PI {
                        next_pb -= 2.0 * PI;
                    }
                    v.phases_b[p_idx] = next_pb;

                    let decay = if full_held {
                        v.partial_decays[p_idx]
                    } else if harmonic_only_held {
                        if p_idx == 0 {
                            v.partial_decays[p_idx].min(voice_release_coeff)
                        } else {
                            v.partial_decays[p_idx]
                        }
                    } else {
                        v.partial_decays[p_idx].min(voice_release_coeff)
                    };
                    v.partial_levels[p_idx] = level * decay;
                    if v.partial_levels[p_idx] > max_level {
                        max_level = v.partial_levels[p_idx];
                    }
                }

                // Duplex shimmer on treble notes (integer 2x of 3rd partial avoids phase wrap discontinuity)
                if v.midi_note >= 60 && duplex_amt > 0.001 && max_level > 0.001 {
                    let duplex_sig = (v.phases_a[2] * 2.0).sin() * v.partial_levels[0] * duplex_amt;
                    sample_l += duplex_sig;
                    sample_r += duplex_sig;
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
                    let am = 1.0 - 0.28 * rattle_amt * buzz_gate;
                    sample_l = sample_l * am + rattle_mod;
                    sample_r = sample_r * am + rattle_mod;
                }

                // Hammer percussion transient burst
                if v.hammer_noise_env > 1e-5 {
                    let noise = v.next_noise();
                    v.hammer_noise_lp = v.hammer_noise_lp * (1.0 - hammer_lp_coeff) + noise * hammer_lp_coeff;
                    let hn = v.hammer_noise_lp * v.hammer_noise_env;
                    sample_l += hn;
                    sample_r += hn;
                    v.hammer_noise_env *= hammer_noise_decay;
                }

                // Damper release felt thump
                if v.damper_noise_env > 1e-5 {
                    let d_noise = v.next_noise() * v.damper_noise_env * 0.25;
                    sample_l += d_noise;
                    sample_r += d_noise;
                    v.damper_noise_env *= damper_noise_decay;
                }

                // Key-release mechanical action noise
                if v.key_release_noise_env > 1e-5 {
                    let kr_noise = v.next_noise();
                    v.key_release_lp = v.key_release_lp * (1.0 - key_release_lp_coeff)
                        + kr_noise * key_release_lp_coeff;
                    let kr = v.key_release_lp * v.key_release_noise_env;
                    sample_l += kr;
                    sample_r += kr;
                    v.key_release_noise_env *= key_release_decay;
                }

                // Subtle nonlinear quadratic soundboard saturation & amplitude tremolo
                sample_l = (sample_l * attack_env + quad * sample_l * sample_l) * trem_gain_l;
                sample_r = (sample_r * attack_env + quad * sample_r * sample_r) * trem_gain_r;
                if !sample_l.is_finite() {
                    sample_l = 0.0;
                }
                if !sample_r.is_finite() {
                    sample_r = 0.0;
                }

                let pan_r = ((v.pan + 1.0) * 0.5).clamp(0.0, 1.0);
                let gain_l = (1.0 - pan_r).sqrt();
                let gain_r = pan_r.sqrt();

                sum_l += sample_l * gain_l;
                sum_r += sample_r * gain_r;
                body_energy += 0.5 * (sample_l + sample_r);

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

    #[test]
    fn test_sympathetic_and_duplex_scale_resonance_isolation() {
        let sr = 48000.0_f32;
        let inst = test_instrument();

        // 1. Sympathetic tank isolation
        let mut p_dry = clean_params();
        p_dry.design.sympathetic_resonance = 0.0;
        let mut p_symp = clean_params();
        p_symp.design.sympathetic_resonance = 1.0;

        let mut pedals = PedalState::default();
        pedals.sustain = 1.0;

        let mut s_dry = PianoSynthEngine::new(sr);
        let mut s_symp = PianoSynthEngine::new(sr);
        s_dry.note_on(60, 110, &p_dry, &inst, &pedals);
        s_symp.note_on(60, 110, &p_symp, &inst, &pedals);

        let _ = render_mono(&mut s_dry, 4096, 256, &p_dry, &pedals);
        let _ = render_mono(&mut s_symp, 4096, 256, &p_symp, &pedals);
        s_dry.note_off(60, &p_dry, &pedals);
        s_symp.note_off(60, &p_symp, &pedals);

        let tail_dry = render_mono(&mut s_dry, 16384, 256, &p_dry, &pedals);
        let tail_symp = render_mono(&mut s_symp, 16384, 256, &p_symp, &pedals);
        let diff_symp: Vec<f32> = tail_symp.iter().zip(tail_dry.iter()).map(|(a, b)| a - b).collect();
        let symp_diff_rms = rms(&diff_symp);
        assert!(symp_diff_rms > 1e-4, "sympathetic resonance should alter tail: {}", symp_diff_rms);

        // 2. Duplex scale shimmer isolation on treble (MIDI >= 60) vs bass (MIDI < 60)
        let mut p_dup0 = clean_params();
        p_dup0.design.duplex_scale_resonance = 0.0;
        let mut p_dup1 = clean_params();
        p_dup1.design.duplex_scale_resonance = 1.0;

        let pedals_off = PedalState::default();
        let mut s_t0 = PianoSynthEngine::new(sr);
        let mut s_t1 = PianoSynthEngine::new(sr);
        s_t0.note_on(72, 110, &p_dup0, &inst, &pedals_off); // C5 >= 60
        s_t1.note_on(72, 110, &p_dup1, &inst, &pedals_off);
        let b_t0 = render_mono(&mut s_t0, 8192, 256, &p_dup0, &pedals_off);
        let b_t1 = render_mono(&mut s_t1, 8192, 256, &p_dup1, &pedals_off);
        let treble_diff: Vec<f32> = b_t1.iter().zip(b_t0.iter()).map(|(a, b)| a - b).collect();
        let treble_diff_rms = rms(&treble_diff);
        assert!(
            treble_diff_rms > 1e-3,
            "duplex shimmer should appear on treble >= 60: diff_rms={}",
            treble_diff_rms
        );
        // Verify duplex energy sits at 2x partial 3 (~6 * f0)
        let f_c5 = compute_note_frequency(72, &p_dup0);
        let shimmer_diff_pow = goertzel_power(&treble_diff, sr, f_c5 * 6.0);
        assert!(shimmer_diff_pow > 1e-6, "duplex shimmer spectral peak missing: {}", shimmer_diff_pow);

        let mut s_b0 = PianoSynthEngine::new(sr);
        let mut s_b1 = PianoSynthEngine::new(sr);
        s_b0.note_on(48, 110, &p_dup0, &inst, &pedals_off); // C3 < 60
        s_b1.note_on(48, 110, &p_dup1, &inst, &pedals_off);
        let b_b0 = render_mono(&mut s_b0, 4096, 256, &p_dup0, &pedals_off);
        let b_b1 = render_mono(&mut s_b1, 4096, 256, &p_dup1, &pedals_off);
        let bass_diff = b_b0.iter().zip(b_b1.iter()).map(|(a, b)| (a - b).abs()).fold(0.0_f32, f32::max);
        assert!(bass_diff < 1e-6, "duplex resonance should only engage on treble notes >= 60");
    }

    #[test]
    fn test_hammer_damper_and_pedal_mechanical_noises_isolation() {
        let sr = 48000.0_f32;
        let inst = test_instrument();
        let pedals = PedalState::default();

        // 1. Hammer noise isolation
        let p_h0 = clean_params();
        let mut p_h1 = clean_params();
        p_h1.voicing.hammer_noise = 1.0;

        let mut s_h0 = PianoSynthEngine::new(sr);
        let mut s_h1 = PianoSynthEngine::new(sr);
        s_h0.note_on(60, 100, &p_h0, &inst, &pedals);
        s_h1.note_on(60, 100, &p_h1, &inst, &pedals);
        let b_h0 = render_mono(&mut s_h0, 512, 256, &p_h0, &pedals);
        let b_h1 = render_mono(&mut s_h1, 512, 256, &p_h1, &pedals);
        let diff_hammer: Vec<f32> = b_h1.iter().zip(b_h0.iter()).map(|(a, b)| a - b).collect();
        assert!(rms(&diff_hammer) > 1e-3, "hammer_noise=1.0 should inject attack transient");

        // 2. Damper noise isolation on note_off
        let p_d0 = clean_params();
        let mut p_d1 = clean_params();
        p_d1.action.damper_noise = 1.0;

        let mut s_d0 = PianoSynthEngine::new(sr);
        let mut s_d1 = PianoSynthEngine::new(sr);
        s_d0.note_on(60, 100, &p_d0, &inst, &pedals);
        s_d1.note_on(60, 100, &p_d1, &inst, &pedals);
        let _ = render_mono(&mut s_d0, 2048, 256, &p_d0, &pedals);
        let _ = render_mono(&mut s_d1, 2048, 256, &p_d1, &pedals);
        s_d0.note_off(60, &p_d0, &pedals);
        s_d1.note_off(60, &p_d1, &pedals);
        let rel_d0 = render_mono(&mut s_d0, 512, 256, &p_d0, &pedals);
        let rel_d1 = render_mono(&mut s_d1, 512, 256, &p_d1, &pedals);
        let diff_damper: Vec<f32> = rel_d1.iter().zip(rel_d0.iter()).map(|(a, b)| a - b).collect();
        assert!(rms(&diff_damper) > 1e-4, "damper_noise=1.0 should inject release thump");

        // 3. Pedal mechanical noise isolation with zero active notes
        let mut p_p0 = clean_params();
        p_p0.action.pedal_noise = 0.0;
        let mut p_p1 = clean_params();
        p_p1.action.pedal_noise = 1.0;

        let mut prev_p = PedalState::default();
        let mut next_p = PedalState::default();
        next_p.sustain = 1.0;

        let mut s_p0 = PianoSynthEngine::new(sr);
        let mut s_p1 = PianoSynthEngine::new(sr);
        s_p0.update_pedals(&next_p, &prev_p, &p_p0);
        s_p1.update_pedals(&next_p, &prev_p, &p_p1);
        let out_p0 = render_mono(&mut s_p0, 512, 256, &p_p0, &next_p);
        let out_p1 = render_mono(&mut s_p1, 512, 256, &p_p1, &next_p);
        prev_p = next_p;
        let _ = prev_p;
        assert!(rms(&out_p0) < 1e-6);
        assert!(rms(&out_p1) > 1e-3, "pedal_noise=1.0 should produce thump on sustain press");
    }

    #[test]
    fn test_mallet_bounce_multi_strike_decay() {
        let sr = 48000.0_f32;
        let mut inst = test_instrument();
        inst.decay_scale = 0.10;

        let pedals = PedalState::default();

        let mut p_single = clean_params();
        p_single.bounce.enabled = false;
        p_single.voicing.hammer_noise = 0.8;

        let mut p_bounce = clean_params();
        p_bounce.voicing.hammer_noise = 0.8;
        p_bounce.bounce.enabled = true;
        p_bounce.bounce.delay_ms = 120.0;
        p_bounce.bounce.velocity_sensitivity = 0.0;
        p_bounce.bounce.delay_loss = 0.10;
        p_bounce.bounce.velocity_loss = 0.25;
        p_bounce.bounce.humanization = 0.0;
        p_bounce.bounce.max_bounces = 5;

        let mut s_single = PianoSynthEngine::new(sr);
        let mut s_bounce = PianoSynthEngine::new(sr);

        s_single.note_on(60, 115, &p_single, &inst, &pedals);
        s_bounce.note_on(60, 115, &p_bounce, &inst, &pedals);

        // Render 1.2s (57600 samples)
        let b_single = render_mono(&mut s_single, 57600, 256, &p_single, &pedals);
        let b_bounce = render_mono(&mut s_bounce, 57600, 256, &p_bounce, &pedals);

        // Before bounce 1 (120ms = 5760 samples), signals must match identically
        let pre_bounce_diff = b_bounce[..5750]
            .iter()
            .zip(b_single[..5750].iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        assert!(pre_bounce_diff < 1e-6, "unexpected signal diff before bounce 1: {}", pre_bounce_diff);

        // Overall late tail energy is strongly sustained/re-excited by bounces
        let w_single_tail = rms(&b_single[20000..]);
        let w_bounce_tail = rms(&b_bounce[20000..]);
        assert!(
            w_bounce_tail > w_single_tail * 1.4,
            "bounce strikes should re-excite tail vs single strike: bounce={}, single={}",
            w_bounce_tail,
            w_single_tail
        );

        // Isolate attack transient bursts at bounce timestamps (120ms=5760, 228ms=10944, 325.2ms=15610)
        let hp_burst_rms = |buf: &[f32], start: usize| -> f32 {
            let hp: Vec<f32> = buf[start..start + 256]
                .windows(2)
                .map(|w| w[1] - w[0])
                .collect();
            rms(&hp)
        };
        let t0 = hp_burst_rms(&b_bounce, 0);
        let t1 = hp_burst_rms(&b_bounce, 5760);
        let t2 = hp_burst_rms(&b_bounce, 10944);
        let t3 = hp_burst_rms(&b_bounce, 15610);
        let single_late_hp = hp_burst_rms(&b_single, 10944);

        assert!(t2 > single_late_hp * 2.0, "bounce transient should exceed decayed single tail");
        assert!(
            t0 > t1 && t1 > t2 && t2 > t3,
            "mallet bounce strike transients should decay monotonically: t0={}, t1={}, t2={}, t3={}",
            t0,
            t1,
            t2,
            t3
        );
    }

    #[test]
    fn test_notefx_lfo_vibrato_tremolo_and_smear() {
        let sr = 48000.0_f32;
        let inst = test_instrument();
        let pedals = PedalState::default();

        // 1. Vibrato
        let p_dry = clean_params();
        let mut p_vib = clean_params();
        p_vib.note_fx.vibrato_cents = 50.0;
        p_vib.note_fx.lfo_rate_hz = 6.0;
        p_vib.note_fx.onset_duration_ms = 0.0;

        let mut s_dry = PianoSynthEngine::new(sr);
        let mut s_vib = PianoSynthEngine::new(sr);
        s_dry.note_on(69, 100, &p_dry, &inst, &pedals);
        s_vib.note_on(69, 100, &p_vib, &inst, &pedals);
        let b_dry = render_mono(&mut s_dry, 16384, 256, &p_dry, &pedals);
        let b_vib = render_mono(&mut s_vib, 16384, 256, &p_vib, &pedals);
        let vib_diff: Vec<f32> = b_vib.iter().zip(b_dry.iter()).map(|(a, b)| a - b).collect();
        assert!(rms(&vib_diff) > 0.01, "vibrato should modulate phase increments: {}", rms(&vib_diff));

        // 2. Stereo Tremolo (depth + phase offset)
        let mut p_trem = clean_params();
        p_trem.note_fx.tremolo_depth_db = 12.0;
        p_trem.note_fx.tremolo_phase = 1.0; // 180-deg L/R stereo offset
        p_trem.note_fx.lfo_rate_hz = 5.0;
        p_trem.note_fx.onset_duration_ms = 0.0;

        let mut s_trem = PianoSynthEngine::new(sr);
        s_trem.note_on(60, 100, &p_trem, &inst, &pedals); // Center-panned C4
        let mut l = vec![0.0_f32; 12000];
        let mut r = vec![0.0_f32; 12000];
        s_trem.process_block(&mut l, &mut r, &p_trem, &pedals);
        let lr_diff: Vec<f32> = l.iter().zip(r.iter()).map(|(a, b)| a - b).collect();
        assert!(rms(&lr_diff) > 0.005, "stereo tremolo phase offset should create L/R difference: {}", rms(&lr_diff));

        // 3. Smear stereo spread & attack_envelope
        let mut p_smear = clean_params();
        p_smear.note_fx.smear = 0.9;
        p_smear.note_fx.attack_envelope = 1.0;

        let mut s_smear = PianoSynthEngine::new(sr);
        s_smear.note_on(60, 100, &p_smear, &inst, &pedals);
        let mut sl = vec![0.0_f32; 4096];
        let mut sr_buf = vec![0.0_f32; 4096];
        s_smear.process_block(&mut sl, &mut sr_buf, &p_smear, &pedals);
        let smear_lr: Vec<f32> = sl.iter().zip(sr_buf.iter()).map(|(a, b)| a - b).collect();
        assert!(rms(&smear_lr) > 1e-4, "smear should introduce stereo partial spread");
        assert!(rms(&sl[..256]) < rms(&sl[2048..4096]) * 0.5, "slow attack_envelope=1.0 should ramp onset");
    }

    #[test]
    fn test_diatonic_harp_pedals_and_guitar_mode() {
        let sr = 48000.0_f32;
        let inst = test_instrument();
        let pedals = PedalState::default();

        // 1. Diatonic Harp: C pedal +1 -> C4 shifts to C#4; E pedal -1 -> E4 shifts to Eb4
        let base_params = clean_params();
        let f_cs4 = compute_note_frequency(61, &base_params);
        let f_eb4 = compute_note_frequency(63, &base_params);

        let mut harp_params = clean_params();
        harp_params.play_mode.mode = PlayMode::DiatonicHarp;
        harp_params.play_mode.harp_pedals = [1, 0, -1, 0, 0, 0, 0]; // C# and Eb

        assert!((compute_note_frequency(60, &harp_params) - f_cs4).abs() < 1e-3);
        assert!((compute_note_frequency(64, &harp_params) - f_eb4).abs() < 1e-3);

        let mut s_harp = PianoSynthEngine::new(sr);
        s_harp.note_on(60, 100, &harp_params, &inst, &pedals);
        let b_harp = render_mono(&mut s_harp, 8192, 256, &harp_params, &pedals);
        let f_c4 = compute_note_frequency(60, &base_params);
        assert!(
            goertzel_power(&b_harp, sr, f_cs4) > goertzel_power(&b_harp, sr, f_c4) * 50.0,
            "harp C pedal +1 should render audio at C#4"
        );

        // 2. Guitar Mode: capo/fret +5 shifts C4 to F4 (perfect fourth)
        let f_f4 = compute_note_frequency(65, &base_params);
        let mut gtr_params = clean_params();
        gtr_params.play_mode.mode = PlayMode::Guitar;
        gtr_params.play_mode.guitar_fret = 5;
        gtr_params.play_mode.guitar_body = 0.85;
        gtr_params.play_mode.guitar_legato = true;

        assert!((compute_note_frequency(60, &gtr_params) - f_f4).abs() < 1e-3);

        let mut s_gtr = PianoSynthEngine::new(sr);
        s_gtr.note_on(60, 100, &gtr_params, &inst, &pedals);
        let b_gtr = render_mono(&mut s_gtr, 8192, 256, &gtr_params, &pedals);
        assert!(
            goertzel_power(&b_gtr, sr, f_f4) > goertzel_power(&b_gtr, sr, f_c4) * 50.0,
            "guitar fret +5 should render audio at F4"
        );
    }
}
