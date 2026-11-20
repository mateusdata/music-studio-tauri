use crate::contracts::{EffectType, EngineParams};
use std::f32::consts::PI;

const STAGE_SCALE_M: f32 = 3.0;
const SPEED_OF_SOUND_MPS: f32 = 343.0;
const MIC_REF_DIST_M: f32 = 0.96;
const REVERB_BASE_COMB_LENS: [f32; 4] = [1557.0, 1617.0, 1491.0, 1422.0];
const REVERB_ER_TAPS_L_MS: [f32; 6] = [7.3, 13.7, 21.1, 29.3, 38.9, 47.7];
const REVERB_ER_TAPS_R_MS: [f32; 6] = [9.1, 16.3, 24.7, 33.1, 41.9, 53.3];
const REVERB_ER_WEIGHTS: [f32; 6] = [0.28, -0.22, 0.18, -0.14, 0.11, -0.08];

#[derive(Clone, Default)]
struct BiquadState {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1_l: f32,
    z2_l: f32,
    z1_r: f32,
    z2_r: f32,
}

impl BiquadState {
    fn set_bypass(&mut self) {
        self.b0 = 1.0;
        self.b1 = 0.0;
        self.b2 = 0.0;
        self.a1 = 0.0;
        self.a2 = 0.0;
    }

    fn normalize(&mut self, b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) {
        let inv_a0 = 1.0 / a0.max(1e-6);
        self.b0 = b0 * inv_a0;
        self.b1 = b1 * inv_a0;
        self.b2 = b2 * inv_a0;
        self.a1 = a1 * inv_a0;
        self.a2 = a2 * inv_a0;
    }

    fn update_peaking(&mut self, sr: f32, freq: f32, gain_db: f32, q: f32) {
        if gain_db.abs() < 0.02 {
            self.set_bypass();
            return;
        }
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq.clamp(25.0, sr * 0.45) / sr;
        let alpha = w0.sin() / (2.0 * q.clamp(0.2, 10.0));
        let cos_w0 = w0.cos();

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

        self.normalize(b0, b1, b2, a0, a1, a2);
    }

    fn update_low_shelf(&mut self, sr: f32, freq: f32, gain_db: f32, q: f32) {
        if gain_db.abs() < 0.02 {
            self.set_bypass();
            return;
        }
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq.clamp(25.0, sr * 0.45) / sr;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q.clamp(0.3, 4.0));
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha;

        self.normalize(b0, b1, b2, a0, a1, a2);
    }

    fn update_high_shelf(&mut self, sr: f32, freq: f32, gain_db: f32, q: f32) {
        if gain_db.abs() < 0.02 {
            self.set_bypass();
            return;
        }
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq.clamp(25.0, sr * 0.45) / sr;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q.clamp(0.3, 4.0));
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha;

        self.normalize(b0, b1, b2, a0, a1, a2);
    }

    #[inline]
    fn process_stereo(&mut self, l: f32, r: f32) -> (f32, f32) {
        let out_l = self.b0 * l + self.z1_l;
        self.z1_l = self.b1 * l - self.a1 * out_l + self.z2_l;
        self.z2_l = self.b2 * l - self.a2 * out_l;

        let out_r = self.b0 * r + self.z1_r;
        self.z1_r = self.b1 * r - self.a1 * out_r + self.z2_r;
        self.z2_r = self.b2 * r - self.a2 * out_r;
        (out_l, out_r)
    }
}

pub struct EffectsChain {
    sample_rate: f32,
    mic_delay_bufs: [Vec<f32>; 3],
    mic_write_pos: usize,
    eq_filters: [BiquadState; 3],
    slot_eq_filters: [BiquadState; 3],
    slot_comp_env: [f32; 3],
    slot_chorus_phase: [f32; 3],
    delay_buf_l: Vec<f32>,
    delay_buf_r: Vec<f32>,
    delay_write_pos: usize,
    chorus_buf_l: Vec<f32>,
    chorus_buf_r: Vec<f32>,
    chorus_pos: usize,
    reverb_pre_buf_l: Vec<f32>,
    reverb_pre_buf_r: Vec<f32>,
    reverb_pre_pos: usize,
    reverb_combs_l: [Vec<f32>; 4],
    reverb_combs_r: [Vec<f32>; 4],
    reverb_idx: [usize; 4],
    reverb_lp_l: [f32; 4],
    reverb_lp_r: [f32; 4],
    hp_prev_in_l: f32,
    hp_prev_out_l: f32,
    hp_prev_in_r: f32,
    hp_prev_out_r: f32,
}

impl EffectsChain {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate.max(22050.0);
        let max_mic_delay = (sr * 0.25).ceil() as usize;
        let max_delay = (sr * 1.5).ceil() as usize;
        let max_chorus = (sr * 0.08).ceil() as usize;
        let max_reverb_pre = (sr * 0.5).ceil() as usize;
        let scale = sr / 44100.0;
        let c_max_lens = [
            ((REVERB_BASE_COMB_LENS[0] * scale * 2.5).ceil() as usize).max(256),
            ((REVERB_BASE_COMB_LENS[1] * scale * 2.5).ceil() as usize).max(256),
            ((REVERB_BASE_COMB_LENS[2] * scale * 2.5).ceil() as usize).max(256),
            ((REVERB_BASE_COMB_LENS[3] * scale * 2.5).ceil() as usize).max(256),
        ];
        Self {
            sample_rate: sr,
            mic_delay_bufs: [
                vec![0.0; max_mic_delay.max(4096)],
                vec![0.0; max_mic_delay.max(4096)],
                vec![0.0; max_mic_delay.max(4096)],
            ],
            mic_write_pos: 0,
            eq_filters: [BiquadState::default(), BiquadState::default(), BiquadState::default()],
            slot_eq_filters: [BiquadState::default(), BiquadState::default(), BiquadState::default()],
            slot_comp_env: [0.0; 3],
            slot_chorus_phase: [0.0; 3],
            delay_buf_l: vec![0.0; max_delay.max(4096)],
            delay_buf_r: vec![0.0; max_delay.max(4096)],
            delay_write_pos: 0,
            chorus_buf_l: vec![0.0; max_chorus.max(1024)],
            chorus_buf_r: vec![0.0; max_chorus.max(1024)],
            chorus_pos: 0,
            reverb_pre_buf_l: vec![0.0; max_reverb_pre.max(8192)],
            reverb_pre_buf_r: vec![0.0; max_reverb_pre.max(8192)],
            reverb_pre_pos: 0,
            reverb_combs_l: [
                vec![0.0; c_max_lens[0]],
                vec![0.0; c_max_lens[1]],
                vec![0.0; c_max_lens[2]],
                vec![0.0; c_max_lens[3]],
            ],
            reverb_combs_r: [
                vec![0.0; c_max_lens[0] + 23],
                vec![0.0; c_max_lens[1] + 23],
                vec![0.0; c_max_lens[2] + 23],
                vec![0.0; c_max_lens[3] + 23],
            ],
            reverb_idx: [0; 4],
            reverb_lp_l: [0.0; 4],
            reverb_lp_r: [0.0; 4],
            hp_prev_in_l: 0.0,
            hp_prev_out_l: 0.0,
            hp_prev_in_r: 0.0,
            hp_prev_out_r: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        *self = Self::new(sr);
    }

    pub fn process_block(&mut self, buf_l: &mut [f32], buf_r: &mut [f32], params: &EngineParams) {
        let len = buf_l.len().min(buf_r.len());

        if params.eq_enabled {
            for i in 0..3 {
                let b = &params.eq_bands[i];
                self.eq_filters[i].update_peaking(self.sample_rate, b.freq_hz, b.gain_db, b.q);
            }
        }

        // Precompute FX slot ParametricEq coefficients once per block
        for (slot_idx, slot) in params.fx.iter().enumerate() {
            if slot.enabled && slot.effect_type == EffectType::ParametricEq {
                let freq = 40.0 * 400.0_f32.powf(slot.param_1.clamp(0.0, 1.0));
                let gain_db = (slot.param_2.clamp(0.0, 1.0) - 0.5) * 30.0;
                let q = 0.4 + slot.param_3.clamp(0.0, 1.0) * 5.6;
                if slot.param_4 < 0.25 {
                    self.slot_eq_filters[slot_idx].update_low_shelf(self.sample_rate, freq, gain_db, q);
                } else if slot.param_4 > 0.75 {
                    self.slot_eq_filters[slot_idx].update_high_shelf(self.sample_rate, freq, gain_db, q);
                } else {
                    self.slot_eq_filters[slot_idx].update_peaking(self.sample_rate, freq, gain_db, q);
                }
            }
        }

        // Precompute 3-microphone stage spatializer parameters (zero heap allocations)
        let mic_buf_len = self.mic_delay_bufs[0].len();
        let max_mic_delay_samps = (mic_buf_len.saturating_sub(2)) as f32;
        let mut mic_enabled = [false; 3];
        let mut mic_w_l = [0.5_f32; 3];
        let mut mic_w_r = [0.5_f32; 3];
        let mut mic_delay_int = [0_usize; 3];
        let mut mic_delay_frac = [0.0_f32; 3];
        let mut mic_out_l = [0.0_f32; 3];
        let mut mic_out_r = [0.0_f32; 3];

        for (k, mic) in params.output.mics.iter().enumerate() {
            if !mic.enabled {
                continue;
            }
            mic_enabled[k] = true;
            let mx = sanitize(mic.x, 0.5).clamp(-10.0, 10.0);
            let my = sanitize(mic.y, 0.4).clamp(-10.0, 10.0);
            let mz = sanitize(mic.z, 0.65).clamp(-10.0, 10.0);

            // Stereo soundboard pickup weights from bass (x=0.30) and treble (x=0.70) radiation centers
            let dist_src_l = (hypot3(mx - 0.30, my - 0.30, mz - 0.45) * STAGE_SCALE_M).max(0.25);
            let dist_src_r = (hypot3(mx - 0.70, my - 0.30, mz - 0.45) * STAGE_SCALE_M).max(0.25);
            let inv_l = 1.0 / dist_src_l;
            let inv_r = 1.0 / dist_src_r;
            let base_wl = inv_l / (inv_l + inv_r);
            let az_bias = mic.azimuth.clamp(-90.0, 90.0).to_radians().sin() * 0.2;
            let wl = (base_wl - az_bias).clamp(0.05, 0.95);
            mic_w_l[k] = wl;
            mic_w_r[k] = 1.0 - wl;

            // Acoustic center distance and propagation delay
            let dist_c_m = (hypot3(mx - 0.50, my - 0.30, mz - 0.45) * STAGE_SCALE_M).max(0.25);
            let dist_atten = if params.output.mic_level_compensation {
                1.0
            } else {
                (MIC_REF_DIST_M / dist_c_m).clamp(0.05, 2.0)
            };

            let delay_samps = if params.output.mic_delay_compensation {
                0.0
            } else {
                ((dist_c_m / SPEED_OF_SOUND_MPS) * self.sample_rate).clamp(0.0, max_mic_delay_samps)
            };
            let d_floor = delay_samps.floor();
            mic_delay_int[k] = d_floor as usize;
            mic_delay_frac[k] = delay_samps - d_floor;

            let az_rad = mic.azimuth.to_radians();
            let el_rad = mic.elevation.to_radians();
            let dir_gain = 0.65 + 0.35 * az_rad.cos().max(0.0) * el_rad.cos().max(0.0);
            let mic_gain = mic.volume.clamp(0.0, 2.0) * dist_atten * dir_gain * 0.75;

            let pan_norm = ((mic.pan.clamp(-1.0, 1.0) + 1.0) * 0.5).clamp(0.0, 1.0);
            let angle = pan_norm * (PI * 0.5);
            mic_out_l[k] = mic_gain * angle.cos();
            mic_out_r[k] = mic_gain * angle.sin();
        }

        // Lid position gentle darkening when closed
        let lid_gain = 0.72 + params.output.lid_position.clamp(0.0, 1.0) * 0.28;
        // Stereo width mid/side matrix
        let width = (params.output.stereo_width * 2.0).clamp(0.0, 2.0);
        // Binaural slight crossfeed
        let crossfeed = if params.output.binaural_mode { 0.14 } else { 0.0 };

        // Precompute reverb parameters
        let rev_pre_len = self.reverb_pre_buf_l.len();
        let max_pre_samps = rev_pre_len.saturating_sub(2);
        let pre_delay_samps = ((params.reverb.pre_delay_ms.clamp(0.0, 220.0) * 0.001) * self.sample_rate).round() as usize;
        let pre_delay_samps = pre_delay_samps.min(max_pre_samps);

        let room_scale = (params.reverb.room_size_m.clamp(3.0, 60.0) / 18.0).clamp(0.25, 2.5);
        let er_level = params.reverb.early_reflections.clamp(0.0, 1.5);
        let mut er_taps_l = [0_usize; 6];
        let mut er_taps_r = [0_usize; 6];
        for t_idx in 0..6 {
            let tap_l = (REVERB_ER_TAPS_L_MS[t_idx] * room_scale * 0.001 * self.sample_rate).round() as usize;
            let tap_r = (REVERB_ER_TAPS_R_MS[t_idx] * room_scale * 0.001 * self.sample_rate).round() as usize;
            er_taps_l[t_idx] = (pre_delay_samps + tap_l).min(max_pre_samps);
            er_taps_r[t_idx] = (pre_delay_samps + tap_r).min(max_pre_samps);
        }

        let scale = self.sample_rate / 44100.0;
        let comb_room_scale = (0.55 + 0.45 * room_scale).clamp(0.45, 2.2);
        let mut active_comb_len_l = [256_usize; 4];
        let mut active_comb_len_r = [256_usize; 4];
        for c in 0..4 {
            active_comb_len_l[c] = ((REVERB_BASE_COMB_LENS[c] * scale * comb_room_scale) as usize)
                .clamp(64, self.reverb_combs_l[c].len());
            active_comb_len_r[c] = (((REVERB_BASE_COMB_LENS[c] * scale * comb_room_scale) as usize) + 23)
                .clamp(64, self.reverb_combs_r[c].len());
        }

        let reverb_fb = (0.62 + (params.reverb.duration_s / 6.0).clamp(0.0, 0.34)).min(0.95);
        let reverb_damp = (1.0 - params.reverb.tone).clamp(0.1, 0.85);
        let hp_dt = 1.0 / self.sample_rate;
        let hp_rc = 1.0 / (2.0 * PI * params.reverb.highpass_hz.clamp(20.0, 2000.0));
        let hp_alpha = (hp_rc / (hp_rc + hp_dt)).clamp(0.5, 0.998);

        let post_gain = 10.0_f32.powf(params.post_effect_gain_db / 20.0);
        let lim_thresh = 10.0_f32.powf(params.limiter.threshold_db / 20.0).clamp(0.1, 1.0);
        let lim_sharpness = params.limiter.sharpness.clamp(0.0, 1.0);
        let lim_gain = 10.0_f32.powf(params.limiter.gain_db / 20.0);

        for i in 0..len {
            let dry_in_l = sanitize(buf_l[i], 0.0) * lid_gain;
            let dry_in_r = sanitize(buf_r[i], 0.0) * lid_gain;

            // 3-microphone stage spatializer
            let mut l = 0.0_f32;
            let mut r = 0.0_f32;
            for k in 0..3 {
                let mono_in = dry_in_l * mic_w_l[k] + dry_in_r * mic_w_r[k];
                self.mic_delay_bufs[k][self.mic_write_pos] = mono_in;
                if mic_enabled[k] {
                    let d_int = mic_delay_int[k];
                    let frac = mic_delay_frac[k];
                    let idx0 = (self.mic_write_pos + mic_buf_len - d_int) % mic_buf_len;
                    let idx1 = (self.mic_write_pos + mic_buf_len - d_int - 1) % mic_buf_len;
                    let delayed = self.mic_delay_bufs[k][idx0] * (1.0 - frac)
                        + self.mic_delay_bufs[k][idx1] * frac;
                    l += delayed * mic_out_l[k];
                    r += delayed * mic_out_r[k];
                }
            }
            self.mic_write_pos = (self.mic_write_pos + 1) % mic_buf_len;

            // Stereo width M/S
            let mid = 0.5 * (l + r);
            let side = 0.5 * (l - r) * width;
            l = mid + side;
            r = mid - side;

            if crossfeed > 0.0 {
                let nl = l * (1.0 - crossfeed) + r * crossfeed;
                let nr = r * (1.0 - crossfeed) + l * crossfeed;
                l = nl;
                r = nr;
            }

            // 3-band EQ
            if params.eq_enabled {
                for f in &mut self.eq_filters {
                    let (el, er) = f.process_stereo(l, r);
                    l = el;
                    r = er;
                }
            }

            // FX Slots (Delay, ParametricEq, Chorus, Tremolo, Compressor)
            for (slot_idx, slot) in params.fx.iter().enumerate() {
                if !slot.enabled {
                    continue;
                }
                let mix = slot.mix.clamp(0.0, 1.0);
                match slot.effect_type {
                    EffectType::Delay => {
                        let delay_samps = ((0.06 + slot.param_1.clamp(0.0, 1.0) * 0.55) * self.sample_rate) as usize;
                        let delay_samps = delay_samps.clamp(1, self.delay_buf_l.len() - 1);
                        let read_pos = (self.delay_write_pos + self.delay_buf_l.len() - delay_samps)
                            % self.delay_buf_l.len();
                        let dl = self.delay_buf_l[read_pos];
                        let dr = self.delay_buf_r[read_pos];
                        let fb = (slot.param_2 * 0.72).clamp(0.0, 0.82);
                        self.delay_buf_l[self.delay_write_pos] = l + dr * fb;
                        self.delay_buf_r[self.delay_write_pos] = r + dl * fb;
                        self.delay_write_pos = (self.delay_write_pos + 1) % self.delay_buf_l.len();
                        l += dl * mix;
                        r += dr * mix;
                    }
                    EffectType::ParametricEq => {
                        let (eq_l, eq_r) = self.slot_eq_filters[slot_idx].process_stereo(l, r);
                        l = l * (1.0 - mix) + eq_l * mix;
                        r = r * (1.0 - mix) + eq_r * mix;
                    }
                    EffectType::Chorus | EffectType::Tremolo => {
                        self.slot_chorus_phase[slot_idx] +=
                            2.0 * PI * (0.4 + slot.param_1.clamp(0.0, 1.0) * 3.5) / self.sample_rate;
                        if self.slot_chorus_phase[slot_idx] > 2.0 * PI {
                            self.slot_chorus_phase[slot_idx] -= 2.0 * PI;
                        }
                        let lfo = 0.5 + 0.5 * self.slot_chorus_phase[slot_idx].sin();
                        if slot.effect_type == EffectType::Tremolo {
                            let mod_l = 1.0 - mix * lfo;
                            let mod_r = 1.0 - mix * (1.0 - lfo);
                            l *= mod_l;
                            r *= mod_r;
                        } else {
                            let mod_delay = (0.004 + lfo * 0.012) * self.sample_rate;
                            let c_len = self.chorus_buf_l.len();
                            let d_clamped = mod_delay.clamp(1.0, (c_len - 2) as f32);
                            let tap_int = d_clamped.floor() as usize;
                            let frac = d_clamped - tap_int as f32;
                            let rpos0 = (self.chorus_pos + c_len - tap_int) % c_len;
                            let rpos1 = (self.chorus_pos + c_len - tap_int - 1) % c_len;
                            let cl = self.chorus_buf_l[rpos0] * (1.0 - frac) + self.chorus_buf_l[rpos1] * frac;
                            let cr = self.chorus_buf_r[rpos0] * (1.0 - frac) + self.chorus_buf_r[rpos1] * frac;
                            self.chorus_buf_l[self.chorus_pos] = l;
                            self.chorus_buf_r[self.chorus_pos] = r;
                            self.chorus_pos = (self.chorus_pos + 1) % c_len;
                            l = l * (1.0 - mix * 0.5) + cl * mix;
                            r = r * (1.0 - mix * 0.5) + cr * mix;
                        }
                    }
                    EffectType::Compressor => {
                        let thresh_db = -36.0 + slot.param_1.clamp(0.0, 1.0) * 32.0;
                        let ratio = 1.0 + slot.param_2.clamp(0.0, 1.0) * 11.0;
                        let attack_s = 0.001 + slot.param_3.clamp(0.0, 1.0) * 0.049;
                        let release_s = 0.02 + slot.param_4.clamp(0.0, 1.0) * 0.45;
                        let attack_coeff = (-1.0 / (attack_s * self.sample_rate)).exp();
                        let release_coeff = (-1.0 / (release_s * self.sample_rate)).exp();

                        let detector = l.abs().max(r.abs());
                        let env = &mut self.slot_comp_env[slot_idx];
                        if detector > *env {
                            *env = attack_coeff * *env + (1.0 - attack_coeff) * detector;
                        } else {
                            *env = release_coeff * *env + (1.0 - release_coeff) * detector;
                        }

                        let env_db = 20.0 * (*env).max(1e-7).log10();
                        let knee_db = 6.0_f32;
                        let over = env_db - thresh_db;
                        let gain_red_db = if 2.0 * over < -knee_db {
                            0.0
                        } else if 2.0 * over.abs() <= knee_db {
                            let x = over + knee_db * 0.5;
                            ((1.0 / ratio) - 1.0) * (x * x) / (2.0 * knee_db)
                        } else {
                            (1.0 / ratio - 1.0) * over
                        };
                        let comp_gain = 10.0_f32.powf(gain_red_db / 20.0);
                        let wet_l = l * comp_gain;
                        let wet_r = r * comp_gain;
                        l = l * (1.0 - mix) + wet_l * mix;
                        r = r * (1.0 - mix) + wet_r * mix;
                    }
                }
            }

            // Algorithmic Stereo Reverb with Pre-Delay + Early Reflections + Late Combs
            if params.reverb.enabled && params.reverb.mix > 0.001 {
                self.reverb_pre_buf_l[self.reverb_pre_pos] = l;
                self.reverb_pre_buf_r[self.reverb_pre_pos] = r;

                let pre_idx = (self.reverb_pre_pos + rev_pre_len - pre_delay_samps) % rev_pre_len;
                let pre_l = self.reverb_pre_buf_l[pre_idx];
                let pre_r = self.reverb_pre_buf_r[pre_idx];

                let mut er_l = 0.0_f32;
                let mut er_r = 0.0_f32;
                if er_level > 0.001 {
                    for t_idx in 0..6 {
                        let idx_l = (self.reverb_pre_pos + rev_pre_len - er_taps_l[t_idx]) % rev_pre_len;
                        let idx_r = (self.reverb_pre_pos + rev_pre_len - er_taps_r[t_idx]) % rev_pre_len;
                        er_l += self.reverb_pre_buf_l[idx_l] * REVERB_ER_WEIGHTS[t_idx];
                        er_r += self.reverb_pre_buf_r[idx_r] * REVERB_ER_WEIGHTS[t_idx];
                    }
                    er_l *= er_level;
                    er_r *= er_level;
                }
                self.reverb_pre_pos = (self.reverb_pre_pos + 1) % rev_pre_len;

                let mut comb_sum_l = 0.0_f32;
                let mut comb_sum_r = 0.0_f32;
                let rev_in_l = (pre_l + er_l * 0.25) * 0.22;
                let rev_in_r = (pre_r + er_r * 0.25) * 0.22;
                for c in 0..4 {
                    let idx_l = self.reverb_idx[c] % active_comb_len_l[c];
                    let out_cl = self.reverb_combs_l[c][idx_l];
                    self.reverb_lp_l[c] = out_cl * (1.0 - reverb_damp) + self.reverb_lp_l[c] * reverb_damp;
                    self.reverb_combs_l[c][idx_l] = rev_in_l + self.reverb_lp_l[c] * reverb_fb;
                    comb_sum_l += out_cl;

                    let idx_r = self.reverb_idx[c] % active_comb_len_r[c];
                    let out_cr = self.reverb_combs_r[c][idx_r];
                    self.reverb_lp_r[c] = out_cr * (1.0 - reverb_damp) + self.reverb_lp_r[c] * reverb_damp;
                    self.reverb_combs_r[c][idx_r] = rev_in_r + self.reverb_lp_r[c] * reverb_fb;
                    comb_sum_r += out_cr;

                    self.reverb_idx[c] = self.reverb_idx[c].wrapping_add(1);
                }

                let wet_l = er_l * 0.45 + comb_sum_l * 0.85;
                let wet_r = er_r * 0.45 + comb_sum_r * 0.85;

                // Highpass on wet tail
                let hp_l = hp_alpha * (self.hp_prev_out_l + wet_l - self.hp_prev_in_l);
                self.hp_prev_in_l = wet_l;
                self.hp_prev_out_l = hp_l;

                let hp_r = hp_alpha * (self.hp_prev_out_r + wet_r - self.hp_prev_in_r);
                self.hp_prev_in_r = wet_r;
                self.hp_prev_out_r = hp_r;

                let mix = params.reverb.mix.clamp(0.0, 1.0);
                l = l * (1.0 - mix * 0.35) + hp_l * mix;
                r = r * (1.0 - mix * 0.35) + hp_r * mix;
            }

            l *= post_gain;
            r *= post_gain;

            // Brickwall Soft Limiter
            if params.limiter.enabled {
                l = soft_limit(l * lim_gain, lim_thresh, lim_sharpness);
                r = soft_limit(r * lim_gain, lim_thresh, lim_sharpness);
            }

            buf_l[i] = if l.is_finite() { l.clamp(-1.0, 1.0) } else { 0.0 };
            buf_r[i] = if r.is_finite() { r.clamp(-1.0, 1.0) } else { 0.0 };
        }
    }
}

#[inline]
fn sanitize(x: f32, fallback: f32) -> f32 {
    if x.is_finite() { x } else { fallback }
}

#[inline]
fn hypot3(dx: f32, dy: f32, dz: f32) -> f32 {
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[inline]
fn soft_limit(x: f32, thresh: f32, sharpness: f32) -> f32 {
    let abs = x.abs();
    if abs <= thresh {
        x
    } else {
        let drive = 0.6 + sharpness * 1.4;
        let excess = ((abs - thresh) / (1.0 - thresh + 1e-4)) * drive;
        let compressed = thresh + (1.0 - thresh) * excess.tanh();
        x.signum() * compressed.min(0.995)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{EffectSlot, EqBand};

    fn make_clean_params() -> EngineParams {
        let mut p = EngineParams::default();
        p.eq_enabled = false;
        p.reverb.enabled = false;
        p.limiter.enabled = false;
        p.post_effect_gain_db = 0.0;
        p.output.lid_position = 1.0;
        p.output.stereo_width = 0.5;
        p.output.mic_delay_compensation = true;
        p.output.mic_level_compensation = true;
        for slot in &mut p.fx {
            slot.enabled = false;
        }
        p
    }

    fn rms(slice: &[f32]) -> f32 {
        let sum_sq: f32 = slice.iter().map(|x| x * x).sum();
        (sum_sq / slice.len().max(1) as f32).sqrt()
    }

    fn fill_sine(buf_l: &mut [f32], buf_r: &mut [f32], sr: f32, freq_hz: f32, amp: f32) {
        for i in 0..buf_l.len().min(buf_r.len()) {
            let s = amp * (2.0 * PI * freq_hz * (i as f32) / sr).sin();
            buf_l[i] = s;
            buf_r[i] = s;
        }
    }

    #[test]
    fn test_mic_spatializer_level_and_delay_compensation() {
        let sr = 48000.0;
        let n = 1024;

        // Single room mic moved far away (y=1.8 -> ~4.5m -> ~630 samples at 48kHz)
        let mut base = make_clean_params();
        base.output.mics[0].enabled = false;
        base.output.mics[1].enabled = false;
        base.output.mics[2].enabled = true;
        base.output.mics[2].x = 0.5;
        base.output.mics[2].y = 1.8;
        base.output.mics[2].z = 0.65;
        base.output.mics[2].volume = 1.0;
        base.output.mics[2].pan = 0.0;

        // 1. Delay compensation ON vs OFF with impulse at sample 0
        let mut p_uncomp = base.clone();
        p_uncomp.output.mic_delay_compensation = false;
        p_uncomp.output.mic_level_compensation = false;

        let mut p_comp = base.clone();
        p_comp.output.mic_delay_compensation = true;
        p_comp.output.mic_level_compensation = true;

        let mut fx_uncomp = EffectsChain::new(sr);
        let mut fx_comp = EffectsChain::new(sr);
        let mut l_u = vec![0.0_f32; n];
        let mut r_u = vec![0.0_f32; n];
        l_u[0] = 1.0;
        r_u[0] = 1.0;
        let mut l_c = l_u.clone();
        let mut r_c = r_u.clone();

        fx_uncomp.process_block(&mut l_u, &mut r_u, &p_uncomp);
        fx_comp.process_block(&mut l_c, &mut r_c, &p_comp);

        let peak_idx_u = l_u
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
            .unwrap()
            .0;
        let peak_idx_c = l_c
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
            .unwrap()
            .0;

        assert_eq!(peak_idx_c, 0, "Delay-compensated mic must arrive at sample 0");
        assert!(
            peak_idx_u > 400 && peak_idx_u < 800,
            "Uncompensated far mic must arrive delayed (~630 samples), got {}",
            peak_idx_u
        );

        // 2. Level compensation ON must be louder than OFF for far mic
        let peak_val_u = l_u[peak_idx_u].abs();
        let peak_val_c = l_c[peak_idx_c].abs();
        assert!(
            peak_val_c > peak_val_u * 2.5,
            "Level compensation ON ({}) must exceed distance-attenuated OFF ({})",
            peak_val_c,
            peak_val_u
        );
    }

    #[test]
    fn test_eq_boost_and_cut_spectral_rms() {
        let sr = 48000.0;
        let n = 4096;

        // Test both 3-band EQ and FX ParametricEq slot
        let mut p_flat = make_clean_params();
        p_flat.eq_enabled = true;
        p_flat.eq_bands = [
            EqBand { freq_hz: 100.0, gain_db: 0.0, q: 1.0 },
            EqBand { freq_hz: 1000.0, gain_db: 0.0, q: 1.2 },
            EqBand { freq_hz: 6000.0, gain_db: 0.0, q: 1.0 },
        ];

        let mut p_boost = p_flat.clone();
        p_boost.eq_bands[1].gain_db = 12.0;

        let mut p_cut = p_flat.clone();
        p_cut.eq_bands[1].gain_db = -12.0;

        let mut buf_l_flat = vec![0.0; n];
        let mut buf_r_flat = vec![0.0; n];
        fill_sine(&mut buf_l_flat, &mut buf_r_flat, sr, 1000.0, 0.2);

        let mut buf_l_boost = buf_l_flat.clone();
        let mut buf_r_boost = buf_r_flat.clone();
        let mut buf_l_cut = buf_l_flat.clone();
        let mut buf_r_cut = buf_r_flat.clone();

        EffectsChain::new(sr).process_block(&mut buf_l_flat, &mut buf_r_flat, &p_flat);
        EffectsChain::new(sr).process_block(&mut buf_l_boost, &mut buf_r_boost, &p_boost);
        EffectsChain::new(sr).process_block(&mut buf_l_cut, &mut buf_r_cut, &p_cut);

        let rms_flat = rms(&buf_l_flat[512..]);
        let rms_boost = rms(&buf_l_boost[512..]);
        let rms_cut = rms(&buf_l_cut[512..]);

        assert!(
            rms_boost > rms_flat * 2.5,
            "12 dB EQ boost at 1kHz must raise RMS significantly ({} vs {})",
            rms_boost,
            rms_flat
        );
        assert!(
            rms_cut < rms_flat * 0.4,
            "-12 dB EQ cut at 1kHz must attenuate RMS significantly ({} vs {})",
            rms_cut,
            rms_flat
        );

        // FX ParametricEq slot (Peaking around 800 Hz: param_1=0.5)
        let mut p_fx_boost = make_clean_params();
        p_fx_boost.fx[0] = EffectSlot {
            enabled: true,
            effect_type: EffectType::ParametricEq,
            mix: 1.0,
            param_1: 0.5, // 800 Hz
            param_2: 0.9, // +12 dB
            param_3: 0.3,
            param_4: 0.5, // Peaking
        };
        let mut p_fx_cut = p_fx_boost.clone();
        p_fx_cut.fx[0].param_2 = 0.1; // -12 dB

        let mut fl_b = vec![0.0; n];
        let mut fr_b = vec![0.0; n];
        fill_sine(&mut fl_b, &mut fr_b, sr, 800.0, 0.2);
        let mut fl_c = fl_b.clone();
        let mut fr_c = fr_b.clone();
        let mut fl_0 = fl_b.clone();
        let mut fr_0 = fr_b.clone();

        EffectsChain::new(sr).process_block(&mut fl_0, &mut fr_0, &make_clean_params());
        EffectsChain::new(sr).process_block(&mut fl_b, &mut fr_b, &p_fx_boost);
        EffectsChain::new(sr).process_block(&mut fl_c, &mut fr_c, &p_fx_cut);

        assert!(rms(&fl_b[512..]) > rms(&fl_0[512..]) * 2.5);
        assert!(rms(&fl_c[512..]) < rms(&fl_0[512..]) * 0.4);

        // FX ParametricEq LowShelf (param_4 = 0.0) & HighShelf (param_4 = 1.0)
        let mut p_ls = make_clean_params();
        p_ls.fx[0] = EffectSlot {
            enabled: true,
            effect_type: EffectType::ParametricEq,
            mix: 1.0,
            param_1: 0.3, // ~240 Hz
            param_2: 0.9, // +12 dB
            param_3: 0.4,
            param_4: 0.0, // LowShelf
        };
        let mut p_hs = p_ls.clone();
        p_hs.fx[0].param_1 = 0.75; // ~3578 Hz
        p_hs.fx[0].param_4 = 1.0; // HighShelf

        let mut low_l = vec![0.0; n];
        let mut low_r = vec![0.0; n];
        fill_sine(&mut low_l, &mut low_r, sr, 80.0, 0.2);
        let mut low_ls_l = low_l.clone();
        let mut low_ls_r = low_r.clone();
        EffectsChain::new(sr).process_block(&mut low_l, &mut low_r, &make_clean_params());
        EffectsChain::new(sr).process_block(&mut low_ls_l, &mut low_ls_r, &p_ls);
        assert!(rms(&low_ls_l[512..]) > rms(&low_l[512..]) * 2.5);

        let mut hi_l = vec![0.0; n];
        let mut hi_r = vec![0.0; n];
        fill_sine(&mut hi_l, &mut hi_r, sr, 8000.0, 0.2);
        let mut hi_hs_l = hi_l.clone();
        let mut hi_hs_r = hi_r.clone();
        EffectsChain::new(sr).process_block(&mut hi_l, &mut hi_r, &make_clean_params());
        EffectsChain::new(sr).process_block(&mut hi_hs_l, &mut hi_hs_r, &p_hs);
        assert!(rms(&hi_hs_l[512..]) > rms(&hi_l[512..]) * 2.5);
    }

    #[test]
    fn test_delay_echo_tail() {
        let sr = 48000.0;
        let mut params = make_clean_params();
        params.fx[0] = EffectSlot {
            enabled: true,
            effect_type: EffectType::Delay,
            mix: 0.8,
            param_1: 0.2, // (0.06 + 0.2*0.55) = 0.17s = 8160 samples
            param_2: 0.65,
            param_3: 0.5,
            param_4: 0.5,
        };

        let mut fx = EffectsChain::new(sr);
        let total = 24000; // 0.5 seconds
        let mut buf_l = vec![0.0_f32; total];
        let mut buf_r = vec![0.0_f32; total];
        // Short burst at start
        buf_l[0] = 0.9;
        buf_r[0] = 0.6;

        fx.process_block(&mut buf_l, &mut buf_r, &params);

        let expected_echo = ((0.06 + 0.2 * 0.55) * sr) as usize;
        let first_echo_window = &buf_r[expected_echo - 8..expected_echo + 8];
        let second_echo_window = &buf_l[expected_echo * 2 - 8..expected_echo * 2 + 8];

        let max_e1 = first_echo_window.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
        let max_e2 = second_echo_window.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
        assert!(max_e1 > 0.2, "First delay echo must appear near {} samples (got {})", expected_echo, max_e1);
        assert!(max_e2 > 0.05 && max_e2 < max_e1, "Second feedback echo must appear and decay ({} vs {})", max_e2, max_e1);
    }

    #[test]
    fn test_chorus_and_tremolo_modulation() {
        let sr = 48000.0;
        let n = 24000; // 0.5s covers >1 LFO cycle at ~2 Hz

        // 1. Tremolo modulation depth
        let mut p_trem = make_clean_params();
        p_trem.fx[1] = EffectSlot {
            enabled: true,
            effect_type: EffectType::Tremolo,
            mix: 0.9,
            param_1: 0.5, // ~2.15 Hz
            param_2: 0.5,
            param_3: 0.5,
            param_4: 0.5,
        };
        let mut tl = vec![0.0; n];
        let mut tr = vec![0.0; n];
        fill_sine(&mut tl, &mut tr, sr, 440.0, 0.5);
        EffectsChain::new(sr).process_block(&mut tl, &mut tr, &p_trem);

        let win = 1024;
        let mut min_rms = f32::MAX;
        let mut max_rms = 0.0_f32;
        for chunk in tl.chunks_exact(win) {
            let r = rms(chunk);
            min_rms = min_rms.min(r);
            max_rms = max_rms.max(r);
        }
        assert!(
            max_rms > min_rms * 3.0,
            "Tremolo must strongly modulate amplitude envelope (max {} vs min {})",
            max_rms,
            min_rms
        );

        // 2. Chorus time-varying delay creates L/R difference and differs from dry
        let mut p_ch = make_clean_params();
        p_ch.fx[1] = EffectSlot {
            enabled: true,
            effect_type: EffectType::Chorus,
            mix: 0.85,
            param_1: 0.6,
            param_2: 0.5,
            param_3: 0.5,
            param_4: 0.5,
        };
        let mut cl = vec![0.0; n];
        let mut cr = vec![0.0; n];
        fill_sine(&mut cl, &mut cr, sr, 440.0, 0.4);
        let dry_l = cl.clone();
        EffectsChain::new(sr).process_block(&mut cl, &mut cr, &p_ch);

        let diff_rms = rms(&cl[2048..].iter().zip(&dry_l[2048..]).map(|(a, b)| a - b).collect::<Vec<_>>());
        assert!(diff_rms > 0.05, "Chorus must meaningfully modulate carrier signal (diff RMS {})", diff_rms);
    }

    #[test]
    fn test_compressor_gain_reduction() {
        let sr = 48000.0;
        let n = 8192;

        let p_dry = make_clean_params();
        let mut p_comp = make_clean_params();
        p_comp.fx[2] = EffectSlot {
            enabled: true,
            effect_type: EffectType::Compressor,
            mix: 1.0,
            param_1: 0.15, // threshold ~ -31.2 dB
            param_2: 0.85, // ratio ~ 10.35:1
            param_3: 0.05, // fast attack ~ 3.5 ms
            param_4: 0.40, // release ~ 200 ms
        };

        // Loud signal above threshold -> strong gain reduction
        let mut dry_l = vec![0.0; n];
        let mut dry_r = vec![0.0; n];
        fill_sine(&mut dry_l, &mut dry_r, sr, 300.0, 0.8);
        let mut comp_l = dry_l.clone();
        let mut comp_r = dry_r.clone();

        EffectsChain::new(sr).process_block(&mut dry_l, &mut dry_r, &p_dry);
        EffectsChain::new(sr).process_block(&mut comp_l, &mut comp_r, &p_comp);

        let rms_dry_loud = rms(&dry_l[2048..]);
        let rms_comp_loud = rms(&comp_l[2048..]);
        assert!(
            rms_comp_loud < rms_dry_loud * 0.35,
            "Compressor must strongly attenuate loud signal above threshold ({} vs {})",
            rms_comp_loud,
            rms_dry_loud
        );

        // Quiet signal well below threshold -> near-unity gain
        let mut q_dry_l = vec![0.0; n];
        let mut q_dry_r = vec![0.0; n];
        fill_sine(&mut q_dry_l, &mut q_dry_r, sr, 300.0, 0.005); // ~ -46 dB
        let mut q_comp_l = q_dry_l.clone();
        let mut q_comp_r = q_dry_r.clone();

        EffectsChain::new(sr).process_block(&mut q_dry_l, &mut q_dry_r, &p_dry);
        EffectsChain::new(sr).process_block(&mut q_comp_l, &mut q_comp_r, &p_comp);

        let rms_q_dry = rms(&q_dry_l[2048..]);
        let rms_q_comp = rms(&q_comp_l[2048..]);
        assert!(
            (rms_q_comp / rms_q_dry) > 0.95,
            "Compressor must leave quiet signal below threshold uncompressed ({} vs {})",
            rms_q_comp,
            rms_q_dry
        );
    }

    #[test]
    fn test_reverb_tail_predelay_and_early_reflections() {
        let sr = 48000.0;
        let total = 24000; // 500 ms

        // 1. Pre-delay check: 50 ms = 2400 samples
        let mut p_pre = make_clean_params();
        p_pre.reverb.enabled = true;
        p_pre.reverb.mix = 1.0;
        p_pre.reverb.pre_delay_ms = 50.0;
        p_pre.reverb.early_reflections = 0.6;
        p_pre.reverb.duration_s = 1.8;

        let mut l = vec![0.0_f32; total];
        let mut r = vec![0.0_f32; total];
        l[0] = 1.0;
        r[0] = 1.0;
        EffectsChain::new(sr).process_block(&mut l, &mut r, &p_pre);

        // Samples [10 .. 2300] (~0.2ms to ~47.9ms) must be completely silent before pre-delay expires
        let pre_gap_peak = l[10..2300].iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
        let tail_rms = rms(&l[2400..12000]);
        assert!(
            pre_gap_peak < 1e-6,
            "Reverb must remain silent during pre-delay window, got peak {}",
            pre_gap_peak
        );
        assert!(
            tail_rms > 0.002,
            "Reverb impulse tail must carry non-zero energy after pre-delay (RMS {})",
            tail_rms
        );

        // 2. Burst tail energy check (256-sample tone burst -> rich reverb tail)
        let mut burst_l = vec![0.0_f32; total];
        let mut burst_r = vec![0.0_f32; total];
        fill_sine(&mut burst_l[..256], &mut burst_r[..256], sr, 440.0, 0.8);
        let mut dry_l = burst_l.clone();
        let mut dry_r = burst_r.clone();
        EffectsChain::new(sr).process_block(&mut dry_l, &mut dry_r, &make_clean_params());
        EffectsChain::new(sr).process_block(&mut burst_l, &mut burst_r, &p_pre);
        assert_eq!(rms(&dry_l[2400..12000]), 0.0);
        assert!(
            rms(&burst_l[2400..12000]) > 0.015,
            "Reverb burst tail must sustain significant energy (RMS {})",
            rms(&burst_l[2400..12000])
        );

        // 3. Early reflections energy & room size scaling check
        let mut p_er_off = make_clean_params();
        p_er_off.reverb.enabled = true;
        p_er_off.reverb.mix = 1.0;
        p_er_off.reverb.pre_delay_ms = 0.0;
        p_er_off.reverb.room_size_m = 18.0;
        p_er_off.reverb.duration_s = 0.2;
        p_er_off.reverb.early_reflections = 0.0;

        let mut p_er_on = p_er_off.clone();
        p_er_on.reverb.early_reflections = 1.0;

        let mut p_big_room = p_er_on.clone();
        p_big_room.reverb.room_size_m = 36.0;

        let mut l0 = vec![0.0; 4800];
        let mut r0 = vec![0.0; 4800];
        l0[0] = 1.0;
        r0[0] = 1.0;
        let mut l1 = l0.clone();
        let mut r1 = r0.clone();
        let mut l2 = l0.clone();
        let mut r2 = r0.clone();

        EffectsChain::new(sr).process_block(&mut l0, &mut r0, &p_er_off);
        EffectsChain::new(sr).process_block(&mut l1, &mut r1, &p_er_on);
        EffectsChain::new(sr).process_block(&mut l2, &mut r2, &p_big_room);

        // First 25ms window contains early reflection taps before combs kick in (~35ms)
        let early_rms_off = rms(&l0[10..1200]);
        let early_rms_on = rms(&l1[10..1200]);
        let big_room_first_ms = rms(&l2[10..600]);
        let med_room_first_ms = rms(&l1[10..600]);
        assert!(
            early_rms_on > early_rms_off + 0.002,
            "Early reflections must populate early window ({} vs {})",
            early_rms_on,
            early_rms_off
        );
        assert!(
            (big_room_first_ms - med_room_first_ms).abs() > 0.001,
            "Room size scaling must alter early reflection tap timing ({} vs {})",
            big_room_first_ms,
            med_room_first_ms
        );
    }

    #[test]
    fn test_brickwall_limiter_and_zero_nan_inf() {
        let sr = 48000.0;
        let n = 4096;
        let mut params = EngineParams::default();
        params.limiter.enabled = true;
        params.limiter.threshold_db = -1.0;
        params.limiter.sharpness = 0.8;
        params.limiter.gain_db = 12.0;
        params.post_effect_gain_db = 12.0;
        for slot in &mut params.fx {
            slot.enabled = true;
        }

        let mut fx = EffectsChain::new(sr);
        let mut buf_l = vec![0.0; n];
        let mut buf_r = vec![0.0; n];
        fill_sine(&mut buf_l, &mut buf_r, sr, 220.0, 5.0);
        // Inject pathological NaN / Inf values to verify sanitization
        buf_l[10] = f32::NAN;
        buf_r[11] = f32::INFINITY;
        buf_l[12] = f32::NEG_INFINITY;

        fx.process_block(&mut buf_l, &mut buf_r, &params);

        for (idx, (&l, &r)) in buf_l.iter().zip(buf_r.iter()).enumerate() {
            assert!(l.is_finite() && r.is_finite(), "Sample {} became non-finite: ({}, {})", idx, l, r);
            assert!(l.abs() <= 1.0 && r.abs() <= 1.0, "Limiter ceiling exceeded at {}: ({}, {})", idx, l, r);
        }

        // Also verify sample rate re-init runs cleanly without NaN/Inf
        fx.set_sample_rate(96000.0);
        fill_sine(&mut buf_l, &mut buf_r, 96000.0, 440.0, 0.5);
        fx.process_block(&mut buf_l, &mut buf_r, &params);
        assert!(buf_l.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
        assert!(buf_r.iter().all(|x| x.is_finite() && x.abs() <= 1.0));
    }
}
