use crate::contracts::{EffectType, EngineParams};
use std::f32::consts::PI;

const STAGE_SCALE_M: f32 = 3.0;
const SPEED_OF_SOUND_MPS: f32 = 343.0;
const MIC_REF_DIST_M: f32 = 0.96;
const REVERB_BASE_COMB_LENS: [f32; 4] = [1557.0, 1617.0, 1491.0, 1422.0];
const REVERB_BASE_AP_LENS: [f32; 2] = [225.0, 556.0];
const REVERB_ER_TAPS_L_MS: [f32; 6] = [7.3, 13.7, 21.1, 29.3, 38.9, 47.7];
const REVERB_ER_TAPS_R_MS: [f32; 6] = [9.1, 16.3, 24.7, 33.1, 41.9, 53.3];
const REVERB_ER_WEIGHTS: [f32; 6] = [0.28, -0.22, 0.18, -0.14, 0.11, -0.08];

#[derive(Clone, Copy)]
struct ReverbProfile {
    er_gain: f32,
    er_spread: f32,
    fb_scale: f32,
    damp_bias: f32,
    comb_spread: f32,
    dispersion: f32,
}

fn reverb_environment_profile(name: &str) -> ReverbProfile {
    match name.trim().to_ascii_lowercase().as_str() {
        "cathedral" => ReverbProfile {
            er_gain: 0.85,
            er_spread: 1.45,
            fb_scale: 1.08,
            damp_bias: 0.12,
            comb_spread: 1.45,
            dispersion: 0.45,
        },
        "studio" => ReverbProfile {
            er_gain: 1.25,
            er_spread: 0.55,
            fb_scale: 0.84,
            damp_bias: 0.08,
            comb_spread: 0.62,
            dispersion: 0.10,
        },
        "chamber" => ReverbProfile {
            er_gain: 1.35,
            er_spread: 0.78,
            fb_scale: 0.92,
            damp_bias: 0.05,
            comb_spread: 0.80,
            dispersion: 0.18,
        },
        "taj mahal" => ReverbProfile {
            er_gain: 1.15,
            er_spread: 1.65,
            fb_scale: 1.12,
            damp_bias: -0.18,
            comb_spread: 1.60,
            dispersion: 0.58,
        },
        "plate" => ReverbProfile {
            er_gain: 0.40,
            er_spread: 0.45,
            fb_scale: 1.02,
            damp_bias: -0.22,
            comb_spread: 0.90,
            dispersion: 0.68,
        },
        "spring" => ReverbProfile {
            er_gain: 0.65,
            er_spread: 0.60,
            fb_scale: 0.88,
            damp_bias: 0.18,
            comb_spread: 0.52,
            dispersion: 0.74,
        },
        "speakers" => ReverbProfile {
            er_gain: 1.45,
            er_spread: 0.40,
            fb_scale: 0.76,
            damp_bias: 0.22,
            comb_spread: 0.48,
            dispersion: 0.35,
        },
        "broceliande" => ReverbProfile {
            er_gain: 1.10,
            er_spread: 1.55,
            fb_scale: 1.04,
            damp_bias: -0.10,
            comb_spread: 1.32,
            dispersion: 0.50,
        },
        "club" => ReverbProfile {
            er_gain: 1.30,
            er_spread: 0.68,
            fb_scale: 0.87,
            damp_bias: 0.16,
            comb_spread: 0.72,
            dispersion: 0.20,
        },
        _ => ReverbProfile {
            // "Concert Hall" default
            er_gain: 1.0,
            er_spread: 1.0,
            fb_scale: 1.0,
            damp_bias: 0.0,
            comb_spread: 1.0,
            dispersion: 0.25,
        },
    }
}

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
    lid_lp_l: f32,
    lid_lp_r: f32,
    binaural_buf_l: Vec<f32>,
    binaural_buf_r: Vec<f32>,
    binaural_pos: usize,
    binaural_shadow_l: f32,
    binaural_shadow_r: f32,
    mic_delay_bufs: [Vec<f32>; 3],
    mic_write_pos: usize,
    eq_filters: [BiquadState; 3],
    slot_eq_filters: [BiquadState; 3],
    slot_comp_env: [f32; 3],
    slot_lfo_phase: [f32; 3],
    slot_delay_buf_l: [Vec<f32>; 3],
    slot_delay_buf_r: [Vec<f32>; 3],
    slot_delay_pos: [usize; 3],
    slot_delay_lp_l: [f32; 3],
    slot_delay_lp_r: [f32; 3],
    slot_chorus_buf_l: [Vec<f32>; 3],
    slot_chorus_buf_r: [Vec<f32>; 3],
    slot_chorus_pos: [usize; 3],
    slot_flanger_buf_l: [Vec<f32>; 3],
    slot_flanger_buf_r: [Vec<f32>; 3],
    slot_flanger_pos: [usize; 3],
    slot_phaser_ap_l: [[f32; 4]; 3],
    slot_phaser_ap_r: [[f32; 4]; 3],
    slot_phaser_fb_l: [f32; 3],
    slot_phaser_fb_r: [f32; 3],
    slot_wah_lp_l: [f32; 3],
    slot_wah_lp_r: [f32; 3],
    slot_wah_bp_l: [f32; 3],
    slot_wah_bp_r: [f32; 3],
    slot_wah_env: [f32; 3],
    slot_fuzz_lp_l: [f32; 3],
    slot_fuzz_lp_r: [f32; 3],
    slot_amp_lp_l: [f32; 3],
    slot_amp_lp_r: [f32; 3],
    slot_amp_cab_lp_l: [f32; 3],
    slot_amp_cab_lp_r: [f32; 3],
    slot_amp_cab_bp_l: [f32; 3],
    slot_amp_cab_bp_r: [f32; 3],
    slot_amp_pres_lp_l: [f32; 3],
    slot_amp_pres_lp_r: [f32; 3],
    reverb_pre_buf_l: Vec<f32>,
    reverb_pre_buf_r: Vec<f32>,
    reverb_pre_pos: usize,
    reverb_combs_l: [Vec<f32>; 4],
    reverb_combs_r: [Vec<f32>; 4],
    reverb_idx: [usize; 4],
    reverb_lp_l: [f32; 4],
    reverb_lp_r: [f32; 4],
    reverb_ap_l: [Vec<f32>; 2],
    reverb_ap_r: [Vec<f32>; 2],
    reverb_ap_idx: [usize; 2],
    hp_prev_in_l: f32,
    hp_prev_out_l: f32,
    hp_prev_in_r: f32,
    hp_prev_out_r: f32,
}

impl EffectsChain {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate.max(22050.0);
        let max_binaural = ((sr * 0.01).ceil() as usize).max(512);
        let max_mic_delay = (sr * 0.25).ceil() as usize;
        let max_delay = ((sr * 1.0).ceil() as usize).max(4096);
        let max_chorus = ((sr * 0.08).ceil() as usize).max(1024);
        let max_flanger = ((sr * 0.045).ceil() as usize).max(2048);
        let max_reverb_pre = (sr * 0.5).ceil() as usize;
        let scale = sr / 44100.0;
        let c_max_lens = [
            ((REVERB_BASE_COMB_LENS[0] * scale * 2.5).ceil() as usize).max(256),
            ((REVERB_BASE_COMB_LENS[1] * scale * 2.5).ceil() as usize).max(256),
            ((REVERB_BASE_COMB_LENS[2] * scale * 2.5).ceil() as usize).max(256),
            ((REVERB_BASE_COMB_LENS[3] * scale * 2.5).ceil() as usize).max(256),
        ];
        let ap_lens = [
            ((REVERB_BASE_AP_LENS[0] * scale).round() as usize).max(64),
            ((REVERB_BASE_AP_LENS[1] * scale).round() as usize).max(128),
        ];
        Self {
            sample_rate: sr,
            lid_lp_l: 0.0,
            lid_lp_r: 0.0,
            binaural_buf_l: vec![0.0; max_binaural],
            binaural_buf_r: vec![0.0; max_binaural],
            binaural_pos: 0,
            binaural_shadow_l: 0.0,
            binaural_shadow_r: 0.0,
            mic_delay_bufs: [
                vec![0.0; max_mic_delay.max(4096)],
                vec![0.0; max_mic_delay.max(4096)],
                vec![0.0; max_mic_delay.max(4096)],
            ],
            mic_write_pos: 0,
            eq_filters: [BiquadState::default(), BiquadState::default(), BiquadState::default()],
            slot_eq_filters: [BiquadState::default(), BiquadState::default(), BiquadState::default()],
            slot_comp_env: [0.0; 3],
            slot_lfo_phase: [0.0; 3],
            slot_delay_buf_l: [vec![0.0; max_delay], vec![0.0; max_delay], vec![0.0; max_delay]],
            slot_delay_buf_r: [vec![0.0; max_delay], vec![0.0; max_delay], vec![0.0; max_delay]],
            slot_delay_pos: [0; 3],
            slot_delay_lp_l: [0.0; 3],
            slot_delay_lp_r: [0.0; 3],
            slot_chorus_buf_l: [vec![0.0; max_chorus], vec![0.0; max_chorus], vec![0.0; max_chorus]],
            slot_chorus_buf_r: [vec![0.0; max_chorus], vec![0.0; max_chorus], vec![0.0; max_chorus]],
            slot_chorus_pos: [0; 3],
            slot_flanger_buf_l: [vec![0.0; max_flanger], vec![0.0; max_flanger], vec![0.0; max_flanger]],
            slot_flanger_buf_r: [vec![0.0; max_flanger], vec![0.0; max_flanger], vec![0.0; max_flanger]],
            slot_flanger_pos: [0; 3],
            slot_phaser_ap_l: [[0.0; 4]; 3],
            slot_phaser_ap_r: [[0.0; 4]; 3],
            slot_phaser_fb_l: [0.0; 3],
            slot_phaser_fb_r: [0.0; 3],
            slot_wah_lp_l: [0.0; 3],
            slot_wah_lp_r: [0.0; 3],
            slot_wah_bp_l: [0.0; 3],
            slot_wah_bp_r: [0.0; 3],
            slot_wah_env: [0.0; 3],
            slot_fuzz_lp_l: [0.0; 3],
            slot_fuzz_lp_r: [0.0; 3],
            slot_amp_lp_l: [0.0; 3],
            slot_amp_lp_r: [0.0; 3],
            slot_amp_cab_lp_l: [0.0; 3],
            slot_amp_cab_lp_r: [0.0; 3],
            slot_amp_cab_bp_l: [0.0; 3],
            slot_amp_cab_bp_r: [0.0; 3],
            slot_amp_pres_lp_l: [0.0; 3],
            slot_amp_pres_lp_r: [0.0; 3],
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
            reverb_ap_l: [vec![0.0; ap_lens[0]], vec![0.0; ap_lens[1]]],
            reverb_ap_r: [vec![0.0; ap_lens[0] + 17], vec![0.0; ap_lens[1] + 19]],
            reverb_ap_idx: [0; 2],
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

        // Lid position gentle level scaling + 1-pole low-pass darkening when closed
        let lid = params.output.lid_position.clamp(0.0, 1.0);
        let lid_gain = 0.72 + lid * 0.28;
        let lid_transparent = lid >= 0.999;
        let dt = 1.0 / self.sample_rate;
        let lid_fc = 1100.0 * 18.0_f32.powf(lid);
        let lid_rc = 1.0 / (2.0 * PI * lid_fc);
        let lid_lp_alpha = (dt / (lid_rc + dt)).clamp(0.01, 1.0);

        // Stereo width mid/side matrix
        let width = (params.output.stereo_width * 2.0).clamp(0.0, 2.0);

        // Binaural ITD + head-shadow low-pass crossfeed controlled by head_angle & head_diameter
        let binaural_enabled = params.output.binaural_mode;
        let bin_buf_len = self.binaural_buf_l.len();
        let max_bin_delay = (bin_buf_len.saturating_sub(2)) as f32;
        let head_diam = sanitize(params.output.head_diameter, 0.18).clamp(0.10, 0.35);
        let head_radius = head_diam * 0.5;
        let head_rad = sanitize(params.output.head_angle, 0.0).clamp(-180.0, 180.0).to_radians();
        let sin_theta = head_rad.sin();
        let itd_mag_s = (head_radius / SPEED_OF_SOUND_MPS)
            * (sin_theta.abs() + sin_theta.abs().asin().min(PI * 0.5));
        let (bin_delay_l_samps, bin_delay_r_samps, bin_gain_l, bin_gain_r) = if sin_theta >= 0.0 {
            (
                0.0,
                (itd_mag_s * self.sample_rate).clamp(0.0, max_bin_delay),
                1.0 + 0.08 * sin_theta,
                1.0 - 0.32 * sin_theta,
            )
        } else {
            let neg_s = -sin_theta;
            (
                (itd_mag_s * self.sample_rate).clamp(0.0, max_bin_delay),
                0.0,
                1.0 - 0.32 * neg_s,
                1.0 + 0.08 * neg_s,
            )
        };
        let bin_d_int_l = bin_delay_l_samps.floor() as usize;
        let bin_frac_l = bin_delay_l_samps - bin_d_int_l as f32;
        let bin_d_int_r = bin_delay_r_samps.floor() as usize;
        let bin_frac_r = bin_delay_r_samps - bin_d_int_r as f32;
        let cross_itd_samps = ((head_diam * 1.35 / SPEED_OF_SOUND_MPS) * self.sample_rate)
            .clamp(1.0, max_bin_delay);
        let cross_d_int = cross_itd_samps.floor() as usize;
        let cross_frac = cross_itd_samps - cross_d_int as f32;
        let shadow_fc = (1800.0 - (head_diam - 0.18) * 3500.0).clamp(900.0, 2600.0);
        let shadow_rc = 1.0 / (2.0 * PI * shadow_fc);
        let shadow_alpha = (dt / (shadow_rc + dt)).clamp(0.02, 0.95);

        // Precompute reverb parameters with environment acoustic profile
        let rev_profile = reverb_environment_profile(&params.reverb.preset_name);
        let rev_pre_len = self.reverb_pre_buf_l.len();
        let max_pre_samps = rev_pre_len.saturating_sub(2);
        let pre_delay_samps = ((params.reverb.pre_delay_ms.clamp(0.0, 220.0) * 0.001) * self.sample_rate).round() as usize;
        let pre_delay_samps = pre_delay_samps.min(max_pre_samps);

        let room_scale = (params.reverb.room_size_m.clamp(3.0, 60.0) / 18.0).clamp(0.25, 2.5);
        let er_level = (params.reverb.early_reflections.clamp(0.0, 1.5) * rev_profile.er_gain).clamp(0.0, 2.0);
        let mut er_taps_l = [0_usize; 6];
        let mut er_taps_r = [0_usize; 6];
        for t_idx in 0..6 {
            let tap_l = (REVERB_ER_TAPS_L_MS[t_idx] * room_scale * rev_profile.er_spread * 0.001 * self.sample_rate).round() as usize;
            let tap_r = (REVERB_ER_TAPS_R_MS[t_idx] * room_scale * rev_profile.er_spread * 0.001 * self.sample_rate).round() as usize;
            er_taps_l[t_idx] = (pre_delay_samps + tap_l).min(max_pre_samps);
            er_taps_r[t_idx] = (pre_delay_samps + tap_r).min(max_pre_samps);
        }

        let scale = self.sample_rate / 44100.0;
        let comb_room_scale = ((0.55 + 0.45 * room_scale) * rev_profile.comb_spread).clamp(0.40, 2.2);
        let mut active_comb_len_l = [256_usize; 4];
        let mut active_comb_len_r = [256_usize; 4];
        for c in 0..4 {
            active_comb_len_l[c] = ((REVERB_BASE_COMB_LENS[c] * scale * comb_room_scale) as usize)
                .clamp(64, self.reverb_combs_l[c].len());
            active_comb_len_r[c] = (((REVERB_BASE_COMB_LENS[c] * scale * comb_room_scale) as usize) + 23)
                .clamp(64, self.reverb_combs_r[c].len());
        }

        let reverb_fb = ((0.62 + (params.reverb.duration_s / 6.0).clamp(0.0, 0.34)) * rev_profile.fb_scale).clamp(0.35, 0.96);
        let reverb_damp = ((1.0 - params.reverb.tone) + rev_profile.damp_bias).clamp(0.05, 0.92);
        let reverb_dispersion = rev_profile.dispersion.clamp(0.0, 0.75);
        let hp_dt = 1.0 / self.sample_rate;
        let hp_rc = 1.0 / (2.0 * PI * params.reverb.highpass_hz.clamp(20.0, 2000.0));
        let hp_alpha = (hp_rc / (hp_rc + hp_dt)).clamp(0.5, 0.998);

        let post_gain = 10.0_f32.powf(params.post_effect_gain_db / 20.0);
        let lim_thresh = 10.0_f32.powf(params.limiter.threshold_db / 20.0).clamp(0.1, 1.0);
        let lim_sharpness = params.limiter.sharpness.clamp(0.0, 1.0);
        let lim_gain = 10.0_f32.powf(params.limiter.gain_db / 20.0);

        for i in 0..len {
            let raw_l = sanitize(buf_l[i], 0.0);
            let raw_r = sanitize(buf_r[i], 0.0);
            let (dry_in_l, dry_in_r) = if lid_transparent {
                self.lid_lp_l = raw_l;
                self.lid_lp_r = raw_r;
                (raw_l, raw_r)
            } else {
                self.lid_lp_l += lid_lp_alpha * (raw_l * lid_gain - self.lid_lp_l);
                self.lid_lp_r += lid_lp_alpha * (raw_r * lid_gain - self.lid_lp_r);
                (self.lid_lp_l, self.lid_lp_r)
            };

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

            if binaural_enabled {
                self.binaural_buf_l[self.binaural_pos] = l;
                self.binaural_buf_r[self.binaural_pos] = r;

                let idx_l0 = (self.binaural_pos + bin_buf_len - bin_d_int_l) % bin_buf_len;
                let idx_l1 = (self.binaural_pos + bin_buf_len - bin_d_int_l - 1) % bin_buf_len;
                let dir_l = self.binaural_buf_l[idx_l0] * (1.0 - bin_frac_l)
                    + self.binaural_buf_l[idx_l1] * bin_frac_l;

                let idx_r0 = (self.binaural_pos + bin_buf_len - bin_d_int_r) % bin_buf_len;
                let idx_r1 = (self.binaural_pos + bin_buf_len - bin_d_int_r - 1) % bin_buf_len;
                let dir_r = self.binaural_buf_r[idx_r0] * (1.0 - bin_frac_r)
                    + self.binaural_buf_r[idx_r1] * bin_frac_r;

                let c_idx0 = (self.binaural_pos + bin_buf_len - cross_d_int) % bin_buf_len;
                let c_idx1 = (self.binaural_pos + bin_buf_len - cross_d_int - 1) % bin_buf_len;
                let contra_l_raw = self.binaural_buf_r[c_idx0] * (1.0 - cross_frac)
                    + self.binaural_buf_r[c_idx1] * cross_frac;
                let contra_r_raw = self.binaural_buf_l[c_idx0] * (1.0 - cross_frac)
                    + self.binaural_buf_l[c_idx1] * cross_frac;

                self.binaural_shadow_l += shadow_alpha * (contra_l_raw - self.binaural_shadow_l);
                self.binaural_shadow_r += shadow_alpha * (contra_r_raw - self.binaural_shadow_r);
                self.binaural_pos = (self.binaural_pos + 1) % bin_buf_len;

                let cross_amt = 0.22_f32;
                l = dir_l * bin_gain_l * (1.0 - cross_amt * 0.5) + self.binaural_shadow_l * cross_amt;
                r = dir_r * bin_gain_r * (1.0 - cross_amt * 0.5) + self.binaural_shadow_r * cross_amt;
            }

            // 3-band EQ
            if params.eq_enabled {
                for f in &mut self.eq_filters {
                    let (el, er) = f.process_stereo(l, r);
                    l = el;
                    r = er;
                }
            }

            // FX Slots (all 10 Pianoteq 9 FX processors)
            for (slot_idx, slot) in params.fx.iter().enumerate() {
                if !slot.enabled {
                    continue;
                }
                let mix = slot.mix.clamp(0.0, 1.0);
                let p1 = sanitize(slot.param_1, 0.5).clamp(0.0, 1.0);
                let p2 = sanitize(slot.param_2, 0.5).clamp(0.0, 1.0);
                let p3 = sanitize(slot.param_3, 0.5).clamp(0.0, 1.0);
                let p4 = sanitize(slot.param_4, 0.5).clamp(0.0, 1.0);

                match slot.effect_type {
                    EffectType::Delay => {
                        let d_len = self.delay_buf_len(slot_idx);
                        let delay_samps = ((0.020 + p1 * 0.880) * self.sample_rate).clamp(1.0, (d_len - 2) as f32);
                        let d_int = delay_samps.floor() as usize;
                        let frac = delay_samps - d_int as f32;
                        let wpos = self.slot_delay_pos[slot_idx];
                        let r0 = (wpos + d_len - d_int) % d_len;
                        let r1 = (wpos + d_len - d_int - 1) % d_len;
                        let dl_raw = self.slot_delay_buf_l[slot_idx][r0] * (1.0 - frac)
                            + self.slot_delay_buf_l[slot_idx][r1] * frac;
                        let dr_raw = self.slot_delay_buf_r[slot_idx][r0] * (1.0 - frac)
                            + self.slot_delay_buf_r[slot_idx][r1] * frac;

                        let tone_fc = 500.0 * 28.0_f32.powf(p3);
                        let tone_rc = 1.0 / (2.0 * PI * tone_fc);
                        let tone_alpha = (dt / (tone_rc + dt)).clamp(0.02, 0.99);
                        self.slot_delay_lp_l[slot_idx] += tone_alpha * (dl_raw - self.slot_delay_lp_l[slot_idx]);
                        self.slot_delay_lp_r[slot_idx] += tone_alpha * (dr_raw - self.slot_delay_lp_r[slot_idx]);
                        let dl = self.slot_delay_lp_l[slot_idx];
                        let dr = self.slot_delay_lp_r[slot_idx];

                        let fb = p2 * 0.85;
                        let pp = p4;
                        let fb_l = dl * (1.0 - pp) + dr * pp;
                        let fb_r = dr * (1.0 - pp) + dl * pp;
                        self.slot_delay_buf_l[slot_idx][wpos] = (l + fb_l * fb).clamp(-4.0, 4.0);
                        self.slot_delay_buf_r[slot_idx][wpos] = (r + fb_r * fb).clamp(-4.0, 4.0);
                        self.slot_delay_pos[slot_idx] = (wpos + 1) % d_len;
                        l += dl * mix;
                        r += dr * mix;
                    }
                    EffectType::Chorus => {
                        let rate_hz = 0.1 + p1 * 4.9;
                        self.slot_lfo_phase[slot_idx] += 2.0 * PI * rate_hz / self.sample_rate;
                        if self.slot_lfo_phase[slot_idx] > 2.0 * PI {
                            self.slot_lfo_phase[slot_idx] -= 2.0 * PI;
                        }
                        let ph_l = self.slot_lfo_phase[slot_idx];
                        let ph_r = ph_l + p4 * PI;
                        let lfo_l = 0.5 + 0.5 * ph_l.sin();
                        let lfo_r = 0.5 + 0.5 * ph_r.sin();

                        let base_delay = 0.003 + p3 * 0.022;
                        let depth_s = 0.0005 + p2 * 0.0075;
                        let c_len = self.slot_chorus_buf_l[slot_idx].len();
                        let wpos = self.slot_chorus_pos[slot_idx];

                        let dl_samps = ((base_delay + lfo_l * depth_s) * self.sample_rate).clamp(1.0, (c_len - 2) as f32);
                        let dr_samps = ((base_delay + lfo_r * depth_s) * self.sample_rate).clamp(1.0, (c_len - 2) as f32);
                        let tl_i = dl_samps.floor() as usize;
                        let fl = dl_samps - tl_i as f32;
                        let tr_i = dr_samps.floor() as usize;
                        let fr = dr_samps - tr_i as f32;

                        let rl0 = (wpos + c_len - tl_i) % c_len;
                        let rl1 = (wpos + c_len - tl_i - 1) % c_len;
                        let rr0 = (wpos + c_len - tr_i) % c_len;
                        let rr1 = (wpos + c_len - tr_i - 1) % c_len;

                        let cl = self.slot_chorus_buf_l[slot_idx][rl0] * (1.0 - fl)
                            + self.slot_chorus_buf_l[slot_idx][rl1] * fl;
                        let cr = self.slot_chorus_buf_r[slot_idx][rr0] * (1.0 - fr)
                            + self.slot_chorus_buf_r[slot_idx][rr1] * fr;
                        self.slot_chorus_buf_l[slot_idx][wpos] = l;
                        self.slot_chorus_buf_r[slot_idx][wpos] = r;
                        self.slot_chorus_pos[slot_idx] = (wpos + 1) % c_len;
                        l = l * (1.0 - mix * 0.5) + cl * mix;
                        r = r * (1.0 - mix * 0.5) + cr * mix;
                    }
                    EffectType::Flanger => {
                        let rate_hz = 0.08 + p1 * 3.92;
                        self.slot_lfo_phase[slot_idx] += 2.0 * PI * rate_hz / self.sample_rate;
                        if self.slot_lfo_phase[slot_idx] > 2.0 * PI {
                            self.slot_lfo_phase[slot_idx] -= 2.0 * PI;
                        }
                        let ph_l = self.slot_lfo_phase[slot_idx];
                        let ph_r = ph_l + p4 * PI;
                        let lfo_l = 0.5 + 0.5 * ph_l.sin();
                        let lfo_r = 0.5 + 0.5 * ph_r.sin();

                        let depth_s = 0.0005 + p2 * 0.0075;
                        let f_len = self.slot_flanger_buf_l[slot_idx].len();
                        let wpos = self.slot_flanger_pos[slot_idx];
                        let dl_samps = ((0.0004 + lfo_l * depth_s) * self.sample_rate).clamp(1.0, (f_len - 2) as f32);
                        let dr_samps = ((0.0004 + lfo_r * depth_s) * self.sample_rate).clamp(1.0, (f_len - 2) as f32);

                        let tl_i = dl_samps.floor() as usize;
                        let fl = dl_samps - tl_i as f32;
                        let tr_i = dr_samps.floor() as usize;
                        let fr = dr_samps - tr_i as f32;

                        let rl0 = (wpos + f_len - tl_i) % f_len;
                        let rl1 = (wpos + f_len - tl_i - 1) % f_len;
                        let rr0 = (wpos + f_len - tr_i) % f_len;
                        let rr1 = (wpos + f_len - tr_i - 1) % f_len;

                        let fl_tap_l = self.slot_flanger_buf_l[slot_idx][rl0] * (1.0 - fl)
                            + self.slot_flanger_buf_l[slot_idx][rl1] * fl;
                        let fl_tap_r = self.slot_flanger_buf_r[slot_idx][rr0] * (1.0 - fr)
                            + self.slot_flanger_buf_r[slot_idx][rr1] * fr;

                        let fb = (p3 * 2.0 - 1.0) * 0.82;
                        self.slot_flanger_buf_l[slot_idx][wpos] = (l + fl_tap_l * fb).tanh();
                        self.slot_flanger_buf_r[slot_idx][wpos] = (r + fl_tap_r * fb).tanh();
                        self.slot_flanger_pos[slot_idx] = (wpos + 1) % f_len;

                        l = l * (1.0 - mix * 0.5) + fl_tap_l * mix;
                        r = r * (1.0 - mix * 0.5) + fl_tap_r * mix;
                    }
                    EffectType::Phaser => {
                        let rate_hz = 0.08 + p1 * 4.42;
                        self.slot_lfo_phase[slot_idx] += 2.0 * PI * rate_hz / self.sample_rate;
                        if self.slot_lfo_phase[slot_idx] > 2.0 * PI {
                            self.slot_lfo_phase[slot_idx] -= 2.0 * PI;
                        }
                        let ph_l = self.slot_lfo_phase[slot_idx];
                        let ph_r = ph_l + 0.65 * PI;
                        let center_hz = 200.0 * 13.0_f32.powf(p4);
                        let oct_span = 0.3 + p2 * 2.2;
                        let f_l = (center_hz * 2.0_f32.powf(ph_l.sin() * oct_span)).clamp(60.0, self.sample_rate * 0.42);
                        let f_r = (center_hz * 2.0_f32.powf(ph_r.sin() * oct_span)).clamp(60.0, self.sample_rate * 0.42);

                        let t_l = (PI * f_l / self.sample_rate).clamp(0.001, 1.3).tan();
                        let a_l = (t_l - 1.0) / (t_l + 1.0);
                        let t_r = (PI * f_r / self.sample_rate).clamp(0.001, 1.3).tan();
                        let a_r = (t_r - 1.0) / (t_r + 1.0);

                        let fb_res = p3 * 0.78;
                        let mut x_l = (l + fb_res * self.slot_phaser_fb_l[slot_idx]).clamp(-4.0, 4.0);
                        let mut x_r = (r + fb_res * self.slot_phaser_fb_r[slot_idx]).clamp(-4.0, 4.0);
                        for st in 0..4 {
                            let y_l = a_l * x_l + self.slot_phaser_ap_l[slot_idx][st];
                            self.slot_phaser_ap_l[slot_idx][st] = x_l - a_l * y_l;
                            x_l = y_l;

                            let y_r = a_r * x_r + self.slot_phaser_ap_r[slot_idx][st];
                            self.slot_phaser_ap_r[slot_idx][st] = x_r - a_r * y_r;
                            x_r = y_r;
                        }
                        self.slot_phaser_fb_l[slot_idx] = x_l;
                        self.slot_phaser_fb_r[slot_idx] = x_r;

                        l = l * (1.0 - 0.5 * mix) + x_l * (0.5 * mix);
                        r = r * (1.0 - 0.5 * mix) + x_r * (0.5 * mix);
                    }
                    EffectType::Tremolo => {
                        let rate_hz = 0.5 + p1 * 13.5;
                        self.slot_lfo_phase[slot_idx] += 2.0 * PI * rate_hz / self.sample_rate;
                        if self.slot_lfo_phase[slot_idx] > 2.0 * PI {
                            self.slot_lfo_phase[slot_idx] -= 2.0 * PI;
                        }
                        let ph_l = self.slot_lfo_phase[slot_idx];
                        let ph_r = ph_l + p3 * 2.0 * PI;
                        let shape_gain = 1.0 + p2 * 4.5;
                        let shape_norm = 1.0 / shape_gain.tanh();
                        let uni_l = 0.5 + 0.5 * ((ph_l.sin() * shape_gain).tanh() * shape_norm);
                        let uni_r = 0.5 + 0.5 * ((ph_r.sin() * shape_gain).tanh() * shape_norm);
                        let depth_exp = 0.5 + p4 * 1.5;
                        let mod_l = 1.0 - mix * uni_l.clamp(0.0, 1.0).powf(depth_exp);
                        let mod_r = 1.0 - mix * uni_r.clamp(0.0, 1.0).powf(depth_exp);
                        l *= mod_l;
                        r *= mod_r;
                    }
                    EffectType::Wah => {
                        let det = l.abs().max(r.abs());
                        let att_c = (-1.0 / (0.003 * self.sample_rate)).exp();
                        let rel_c = (-1.0 / (0.085 * self.sample_rate)).exp();
                        let env = &mut self.slot_wah_env[slot_idx];
                        if det > *env {
                            *env = att_c * *env + (1.0 - att_c) * det;
                        } else {
                            *env = rel_c * *env + (1.0 - rel_c) * det;
                        }
                        let base_hz = 240.0 * (2800.0_f32 / 240.0).powf(p1);
                        let sweep_oct = (*env * 3.0 * p2).min(1.0) * (2.8 * p3);
                        let fc = (base_hz * 2.0_f32.powf(sweep_oct)).clamp(180.0, self.sample_rate * 0.38);
                        let q = 1.2 + p4 * 7.8;
                        let f_coeff = (2.0 * (PI * fc / (self.sample_rate * 2.0)).sin()).min(0.45);
                        let q_damp = (1.0 / q).clamp(0.11, 1.2);

                        let lp_l = &mut self.slot_wah_lp_l[slot_idx];
                        let bp_l = &mut self.slot_wah_bp_l[slot_idx];
                        let lp_r = &mut self.slot_wah_lp_r[slot_idx];
                        let bp_r = &mut self.slot_wah_bp_r[slot_idx];
                        for _ in 0..2 {
                            let hp_l = l - *lp_l - q_damp * *bp_l;
                            *bp_l += f_coeff * hp_l;
                            *lp_l += f_coeff * *bp_l;

                            let hp_r = r - *lp_r - q_damp * *bp_r;
                            *bp_r += f_coeff * hp_r;
                            *lp_r += f_coeff * *bp_r;
                        }
                        let wah_l = (*bp_l * 0.78 + *lp_l * 0.22).clamp(-4.0, 4.0);
                        let wah_r = (*bp_r * 0.78 + *lp_r * 0.22).clamp(-4.0, 4.0);
                        l = l * (1.0 - mix) + wah_l * mix;
                        r = r * (1.0 - mix) + wah_r * mix;
                    }
                    EffectType::Fuzz => {
                        let drive = 1.0 + p1 * 27.0;
                        let neg_scale = 1.0 - p3 * 0.48;
                        let drv_l = l * drive;
                        let drv_r = r * drive;
                        let asym_l = if drv_l >= 0.0 {
                            drv_l.tanh()
                        } else {
                            -((-drv_l).tanh()) * neg_scale
                        };
                        let asym_r = if drv_r >= 0.0 {
                            drv_r.tanh()
                        } else {
                            -((-drv_r).tanh()) * neg_scale
                        };
                        let grit = p3 * 0.22;
                        let raw_l = asym_l + grit * asym_l * asym_l;
                        let raw_r = asym_r + grit * asym_r * asym_r;

                        let tone_fc = 700.0 * 16.0_f32.powf(p2);
                        let tone_rc = 1.0 / (2.0 * PI * tone_fc);
                        let tone_alpha = (dt / (tone_rc + dt)).clamp(0.02, 0.99);
                        self.slot_fuzz_lp_l[slot_idx] += tone_alpha * (raw_l - self.slot_fuzz_lp_l[slot_idx]);
                        self.slot_fuzz_lp_r[slot_idx] += tone_alpha * (raw_r - self.slot_fuzz_lp_r[slot_idx]);

                        let makeup = 0.25 + p4 * 1.0;
                        let fz_l = (self.slot_fuzz_lp_l[slot_idx] * makeup).clamp(-1.5, 1.5);
                        let fz_r = (self.slot_fuzz_lp_r[slot_idx] * makeup).clamp(-1.5, 1.5);
                        l = l * (1.0 - mix) + fz_l * mix;
                        r = r * (1.0 - mix) + fz_r * mix;
                    }
                    EffectType::Amp => {
                        let drive = 1.0 + p1 * 13.0;
                        let bias = 0.14_f32;
                        let bias_dc = bias.tanh();
                        let tube_l = (l * drive + bias).tanh() - bias_dc;
                        let tube_r = (r * drive + bias).tanh() - bias_dc;

                        // 2-band tone stack (warm vs bright tilt)
                        let tone_fc = 1600.0 + p2 * 5900.0;
                        let tone_rc = 1.0 / (2.0 * PI * tone_fc);
                        let tone_alpha = (dt / (tone_rc + dt)).clamp(0.05, 0.98);
                        self.slot_amp_lp_l[slot_idx] += tone_alpha * (tube_l - self.slot_amp_lp_l[slot_idx]);
                        self.slot_amp_lp_r[slot_idx] += tone_alpha * (tube_r - self.slot_amp_lp_r[slot_idx]);
                        let bright_tilt = 0.55 + p2 * 0.75;
                        let ts_l = self.slot_amp_lp_l[slot_idx] * (1.15 - p2 * 0.35) + (tube_l - self.slot_amp_lp_l[slot_idx]) * bright_tilt;
                        let ts_r = self.slot_amp_lp_r[slot_idx] * (1.15 - p2 * 0.35) + (tube_r - self.slot_amp_lp_r[slot_idx]) * bright_tilt;

                        // Cabinet resonant bandpass bump (~125 Hz) + speaker cone LPF (~4.4 kHz)
                        let cab_f = (2.0 * (PI * 125.0 / self.sample_rate).sin()).min(0.3);
                        let cab_q = 0.45_f32;
                        let hp_c_l = ts_l - self.slot_amp_cab_lp_l[slot_idx] - cab_q * self.slot_amp_cab_bp_l[slot_idx];
                        self.slot_amp_cab_bp_l[slot_idx] += cab_f * hp_c_l;
                        self.slot_amp_cab_lp_l[slot_idx] += cab_f * self.slot_amp_cab_bp_l[slot_idx];
                        let hp_c_r = ts_r - self.slot_amp_cab_lp_r[slot_idx] - cab_q * self.slot_amp_cab_bp_r[slot_idx];
                        self.slot_amp_cab_bp_r[slot_idx] += cab_f * hp_c_r;
                        self.slot_amp_cab_lp_r[slot_idx] += cab_f * self.slot_amp_cab_bp_r[slot_idx];

                        let cab_res = p3 * 0.45;
                        let body_l = ts_l + self.slot_amp_cab_bp_l[slot_idx] * cab_res;
                        let body_r = ts_r + self.slot_amp_cab_bp_r[slot_idx] * cab_res;

                        // Presence boost (> ~2.2 kHz)
                        let pres_rc = 1.0 / (2.0 * PI * 2200.0);
                        let pres_alpha = (dt / (pres_rc + dt)).clamp(0.05, 0.98);
                        self.slot_amp_pres_lp_l[slot_idx] += pres_alpha * (body_l - self.slot_amp_pres_lp_l[slot_idx]);
                        self.slot_amp_pres_lp_r[slot_idx] += pres_alpha * (body_r - self.slot_amp_pres_lp_r[slot_idx]);
                        let pres_boost = p4 * 1.1;
                        let amp_l = (body_l + (body_l - self.slot_amp_pres_lp_l[slot_idx]) * pres_boost) * 0.72;
                        let amp_r = (body_r + (body_r - self.slot_amp_pres_lp_r[slot_idx]) * pres_boost) * 0.72;

                        l = l * (1.0 - mix) + amp_l * mix;
                        r = r * (1.0 - mix) + amp_r * mix;
                    }
                    EffectType::ParametricEq => {
                        let (eq_l, eq_r) = self.slot_eq_filters[slot_idx].process_stereo(l, r);
                        l = l * (1.0 - mix) + eq_l * mix;
                        r = r * (1.0 - mix) + eq_r * mix;
                    }
                    EffectType::Compressor => {
                        let thresh_db = -36.0 + p1 * 32.0;
                        let ratio = 1.0 + p2 * 11.0;
                        let attack_s = 0.001 + p3 * 0.049;
                        let release_s = 0.02 + p4 * 0.45;
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

                let mut late_l = comb_sum_l * 0.85;
                let mut late_r = comb_sum_r * 0.85;

                // 2-stage Schroeder all-pass dispersion on late tail (Plate/Spring/Cathedral/Taj Mahal)
                if reverb_dispersion > 0.01 {
                    for ap in 0..2 {
                        let len_l = self.reverb_ap_l[ap].len();
                        let idx_l = self.reverb_ap_idx[ap] % len_l;
                        let buf_l_val = self.reverb_ap_l[ap][idx_l];
                        let out_l = -reverb_dispersion * late_l + buf_l_val;
                        self.reverb_ap_l[ap][idx_l] = late_l + reverb_dispersion * out_l;
                        late_l = out_l;

                        let len_r = self.reverb_ap_r[ap].len();
                        let idx_r = self.reverb_ap_idx[ap] % len_r;
                        let buf_r_val = self.reverb_ap_r[ap][idx_r];
                        let out_r = -reverb_dispersion * late_r + buf_r_val;
                        self.reverb_ap_r[ap][idx_r] = late_r + reverb_dispersion * out_r;
                        late_r = out_r;

                        self.reverb_ap_idx[ap] = self.reverb_ap_idx[ap].wrapping_add(1);
                    }
                }

                let wet_l = er_l * 0.45 + late_l;
                let wet_r = er_r * 0.45 + late_r;

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

    #[inline]
    fn delay_buf_len(&self, slot_idx: usize) -> usize {
        self.slot_delay_buf_l[slot_idx].len()
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
            param_1: 0.2, // (0.020 + 0.2*0.880) = 0.196s = 9408 samples
            param_2: 0.65,
            param_3: 0.9, // bright tone LPF
            param_4: 0.5, // 50% ping-pong
        };

        let mut fx = EffectsChain::new(sr);
        let total = 24000; // 0.5 seconds
        let mut buf_l = vec![0.0_f32; total];
        let mut buf_r = vec![0.0_f32; total];
        // Short burst at start
        buf_l[0] = 0.9;
        buf_r[0] = 0.6;

        fx.process_block(&mut buf_l, &mut buf_r, &params);

        let expected_echo = ((0.020 + 0.2 * 0.880) * sr) as usize;
        let first_echo_window = &buf_r[expected_echo - 8..expected_echo + 16];
        let second_echo_window = &buf_l[expected_echo * 2 - 8..expected_echo * 2 + 24];

        let max_e1 = first_echo_window.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
        let max_e2 = second_echo_window.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
        assert!(max_e1 > 0.15, "First delay echo must appear near {} samples (got {})", expected_echo, max_e1);
        assert!(max_e2 > 0.04 && max_e2 < max_e1, "Second feedback echo must appear and decay ({} vs {})", max_e2, max_e1);
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

    #[test]
    fn test_lid_position_spectral_attenuation() {
        let sr = 48000.0;
        let n = 4096;

        let mut p_open = make_clean_params();
        p_open.output.lid_position = 1.0;

        let mut p_closed = make_clean_params();
        p_closed.output.lid_position = 0.0;

        // 120 Hz bass signal
        let mut bass_open_l = vec![0.0; n];
        let mut bass_open_r = vec![0.0; n];
        fill_sine(&mut bass_open_l, &mut bass_open_r, sr, 120.0, 0.5);
        let mut bass_closed_l = bass_open_l.clone();
        let mut bass_closed_r = bass_open_r.clone();

        EffectsChain::new(sr).process_block(&mut bass_open_l, &mut bass_open_r, &p_open);
        EffectsChain::new(sr).process_block(&mut bass_closed_l, &mut bass_closed_r, &p_closed);

        // 6000 Hz treble signal
        let mut hi_open_l = vec![0.0; n];
        let mut hi_open_r = vec![0.0; n];
        fill_sine(&mut hi_open_l, &mut hi_open_r, sr, 6000.0, 0.5);
        let mut hi_closed_l = hi_open_l.clone();
        let mut hi_closed_r = hi_open_r.clone();

        EffectsChain::new(sr).process_block(&mut hi_open_l, &mut hi_open_r, &p_open);
        EffectsChain::new(sr).process_block(&mut hi_closed_l, &mut hi_closed_r, &p_closed);

        let bass_ratio = rms(&bass_closed_l[512..]) / rms(&bass_open_l[512..]);
        let hi_ratio = rms(&hi_closed_l[512..]) / rms(&hi_open_l[512..]);

        assert!(
            bass_ratio > 0.65,
            "Closed lid should preserve low-frequency fundamental energy (ratio {})",
            bass_ratio
        );
        assert!(
            hi_ratio < bass_ratio * 0.35,
            "Closed lid must attenuate 6kHz significantly more than 120Hz (hi ratio {} vs bass ratio {})",
            hi_ratio,
            bass_ratio
        );
    }

    #[test]
    fn test_stereo_width_mono_to_wide_separation() {
        let sr = 48000.0;
        let n = 4096;

        let mut p_mono = make_clean_params();
        p_mono.output.stereo_width = 0.0;

        let mut p_mid = make_clean_params();
        p_mid.output.stereo_width = 0.5;

        let mut p_wide = make_clean_params();
        p_wide.output.stereo_width = 1.0;

        let mut src_l = vec![0.0; n];
        let mut src_r = vec![0.0; n];
        for i in 0..n {
            let s = (2.0 * PI * 440.0 * (i as f32) / sr).sin();
            src_l[i] = 0.8 * s;
            src_r[i] = 0.1 * s;
        }

        let mut ml = src_l.clone();
        let mut mr = src_r.clone();
        let mut midl = src_l.clone();
        let mut midr = src_r.clone();
        let mut wl = src_l.clone();
        let mut wr = src_r.clone();

        EffectsChain::new(sr).process_block(&mut ml, &mut mr, &p_mono);
        EffectsChain::new(sr).process_block(&mut midl, &mut midr, &p_mid);
        EffectsChain::new(sr).process_block(&mut wl, &mut wr, &p_wide);

        let diff_rms = |a: &[f32], b: &[f32]| -> f32 {
            let sum_sq: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum();
            (sum_sq / a.len().max(1) as f32).sqrt()
        };

        let d_mono = diff_rms(&ml[512..], &mr[512..]);
        let d_mid = diff_rms(&midl[512..], &midr[512..]);
        let d_wide = diff_rms(&wl[512..], &wr[512..]);

        assert!(
            d_mono < 1e-6,
            "stereo_width = 0.0 must collapse L/R difference to near-zero (got {})",
            d_mono
        );
        assert!(
            d_wide > d_mid * 1.8 && d_mid > 0.02,
            "stereo_width = 1.0 ({}) must significantly widen separation over 0.5 ({})",
            d_wide,
            d_mid
        );
    }

    #[test]
    fn test_binaural_mode_interaural_crossfeed_and_head_angle() {
        let sr = 48000.0;
        let n = 512;

        // 1. Crossfeed + ITD check with hard-left-panned mic and left-only impulse
        let mut p_off = make_clean_params();
        p_off.output.binaural_mode = false;
        p_off.output.stereo_width = 0.5;
        p_off.output.mics[0].enabled = true;
        p_off.output.mics[0].x = 0.0;
        p_off.output.mics[0].pan = -1.0;
        p_off.output.mics[1].enabled = false;
        p_off.output.mics[2].enabled = false;

        let mut p_on = p_off.clone();
        p_on.output.binaural_mode = true;
        p_on.output.head_angle = 0.0;
        p_on.output.head_diameter = 0.18;

        let mut p_large_head = p_on.clone();
        p_large_head.output.head_diameter = 0.28;

        let mut l_off = vec![0.0_f32; n];
        let mut r_off = vec![0.0_f32; n];
        l_off[0] = 1.0;
        let mut l_on = l_off.clone();
        let mut r_on = r_off.clone();
        let mut l_lh = l_off.clone();
        let mut r_lh = r_off.clone();

        EffectsChain::new(sr).process_block(&mut l_off, &mut r_off, &p_off);
        EffectsChain::new(sr).process_block(&mut l_on, &mut r_on, &p_on);
        EffectsChain::new(sr).process_block(&mut l_lh, &mut r_lh, &p_large_head);

        assert!(
            rms(&r_off) < 1e-6,
            "Without binaural_mode, hard-left impulse must not crossfeed to right ear (got {})",
            rms(&r_off)
        );
        assert!(
            rms(&r_on) > 0.0008,
            "With binaural_mode=true, head-shadow crossfeed must reach right ear (got {})",
            rms(&r_on)
        );

        let peak_idx = |slice: &[f32]| -> usize {
            slice
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
                .unwrap()
                .0
        };
        let cross_peak_std = peak_idx(&r_on);
        let cross_peak_large = peak_idx(&r_lh);
        assert!(
            cross_peak_std >= 20 && cross_peak_large > cross_peak_std,
            "Contralateral crossfeed ITD must delay with head_diameter (std {} vs large {})",
            cross_peak_std,
            cross_peak_large
        );

        // 2. Head angle ITD and energy balance asymmetry
        let mut p_rot_right = make_clean_params();
        p_rot_right.output.binaural_mode = true;
        p_rot_right.output.head_diameter = 0.20;
        p_rot_right.output.head_angle = 45.0;

        let mut p_rot_left = p_rot_right.clone();
        p_rot_left.output.head_angle = -45.0;

        let mut rr_l = vec![0.0_f32; n];
        let mut rr_r = vec![0.0_f32; n];
        rr_l[0] = 1.0;
        rr_r[0] = 1.0;
        let mut rl_l = rr_l.clone();
        let mut rl_r = rr_r.clone();

        EffectsChain::new(sr).process_block(&mut rr_l, &mut rr_r, &p_rot_right);
        EffectsChain::new(sr).process_block(&mut rl_l, &mut rl_r, &p_rot_left);

        assert!(
            peak_idx(&rr_r) > peak_idx(&rr_l) + 10 && rms(&rr_l) > rms(&rr_r) * 1.15,
            "Turning head_angle=+45 deg must delay and shadow right ear (L peak {} RMS {}, R peak {} RMS {})",
            peak_idx(&rr_l),
            rms(&rr_l),
            peak_idx(&rr_r),
            rms(&rr_r)
        );
        assert!(
            peak_idx(&rl_l) > peak_idx(&rl_r) + 10 && rms(&rl_r) > rms(&rl_l) * 1.15,
            "Turning head_angle=-45 deg must delay and shadow left ear (L peak {} RMS {}, R peak {} RMS {})",
            peak_idx(&rl_l),
            rms(&rl_l),
            peak_idx(&rl_r),
            rms(&rl_r)
        );
    }

    #[test]
    fn test_flanger_and_phaser_modulation() {
        let sr = 48000.0;
        let n = 24000;

        // 1. Flanger jet modulation & regenerative comb sweeping
        let mut p_fl = make_clean_params();
        p_fl.fx[0] = EffectSlot {
            enabled: true,
            effect_type: EffectType::Flanger,
            mix: 0.9,
            param_1: 0.65, // ~2.6 Hz
            param_2: 0.75, // deep sweep
            param_3: 0.90, // strong positive feedback (+0.65)
            param_4: 0.50, // quadrature stereo phase
        };

        let mut fl_l = vec![0.0; n];
        let mut fl_r = vec![0.0; n];
        for i in 0..n {
            let t = i as f32 / sr;
            let s = 0.25 * (2.0 * PI * 330.0 * t).sin() + 0.20 * (2.0 * PI * 990.0 * t).sin();
            fl_l[i] = s;
            fl_r[i] = s;
        }
        let dry = fl_l.clone();
        EffectsChain::new(sr).process_block(&mut fl_l, &mut fl_r, &p_fl);

        assert!(fl_l.iter().all(|x| x.is_finite()) && fl_r.iter().all(|x| x.is_finite()));
        let mut min_w = f32::MAX;
        let mut max_w = 0.0_f32;
        for chunk in fl_l[2048..].chunks_exact(1024) {
            let r = rms(chunk);
            min_w = min_w.min(r);
            max_w = max_w.max(r);
        }
        let lr_diff = rms(&fl_l[2048..].iter().zip(&fl_r[2048..]).map(|(a, b)| a - b).collect::<Vec<_>>());
        assert!(
            max_w > min_w * 1.45,
            "Flanger comb sweep must modulate RMS over LFO cycle (max {} vs min {})",
            max_w,
            min_w
        );
        assert!(lr_diff > 0.03, "Flanger stereo phase must produce L/R spatial motion (diff {})", lr_diff);

        // 2. Phaser 4-stage all-pass notch sweep
        let mut p_ph = make_clean_params();
        p_ph.fx[1] = EffectSlot {
            enabled: true,
            effect_type: EffectType::Phaser,
            mix: 1.0,
            param_1: 0.55, // ~2.5 Hz
            param_2: 0.85, // wide sweep
            param_3: 0.70, // feedback resonance
            param_4: 0.45, // center ~630 Hz
        };

        let mut ph_l = dry.clone();
        let mut ph_r = dry.clone();
        EffectsChain::new(sr).process_block(&mut ph_l, &mut ph_r, &p_ph);
        assert!(ph_l.iter().all(|x| x.is_finite()) && ph_r.iter().all(|x| x.is_finite()));

        let ph_diff_from_dry = rms(&ph_l[2048..].iter().zip(&dry[2048..]).map(|(a, b)| a - b).collect::<Vec<_>>());
        let ph_lr_diff = rms(&ph_l[2048..].iter().zip(&ph_r[2048..]).map(|(a, b)| a - b).collect::<Vec<_>>());
        let mut ph_min = f32::MAX;
        let mut ph_max = 0.0_f32;
        for chunk in ph_l[2048..].chunks_exact(1024) {
            let r = rms(chunk);
            ph_min = ph_min.min(r);
            ph_max = ph_max.max(r);
        }
        assert!(
            ph_diff_from_dry > 0.05 && ph_lr_diff > 0.01 && ph_max > ph_min * 1.20,
            "Phaser must sweep stereo all-pass notches (dry diff {}, LR diff {}, sweep {}/{})",
            ph_diff_from_dry,
            ph_lr_diff,
            ph_max,
            ph_min
        );
    }

    #[test]
    fn test_wah_fuzz_and_amp_spectral_shaping() {
        let sr = 48000.0;
        let n = 8192;

        // 1. Wah bandpass resonance selectivity (manual pedal sweep at ~650 Hz vs 3500 Hz)
        let mut p_wah_low = make_clean_params();
        p_wah_low.fx[0] = EffectSlot {
            enabled: true,
            effect_type: EffectType::Wah,
            mix: 1.0,
            param_1: 0.40, // ~640 Hz
            param_2: 0.0,  // manual pedal only
            param_3: 0.0,
            param_4: 0.75, // high Q
        };

        let mut low_l = vec![0.0; n];
        let mut low_r = vec![0.0; n];
        fill_sine(&mut low_l, &mut low_r, sr, 640.0, 0.25);
        let mut hi_l = vec![0.0; n];
        let mut hi_r = vec![0.0; n];
        fill_sine(&mut hi_l, &mut hi_r, sr, 3500.0, 0.25);

        EffectsChain::new(sr).process_block(&mut low_l, &mut low_r, &p_wah_low);
        EffectsChain::new(sr).process_block(&mut hi_l, &mut hi_r, &p_wah_low);
        assert!(
            rms(&low_l[1024..]) > rms(&hi_l[1024..]) * 3.0,
            "Wah pedal tuned to 640 Hz must emphasize 640 Hz ({}) over 3500 Hz ({})",
            rms(&low_l[1024..]),
            rms(&hi_l[1024..])
        );

        // 2. Fuzz harmonic generation & tone shaping (pure sine -> rich odd+even harmonics)
        let mut p_fuzz = make_clean_params();
        p_fuzz.fx[0] = EffectSlot {
            enabled: true,
            effect_type: EffectType::Fuzz,
            mix: 1.0,
            param_1: 0.85, // high drive
            param_2: 0.85, // bright tone
            param_3: 0.70, // asymmetric grit
            param_4: 0.65,
        };
        let mut fz_l = vec![0.0; n];
        let mut fz_r = vec![0.0; n];
        fill_sine(&mut fz_l, &mut fz_r, sr, 220.0, 0.4);
        EffectsChain::new(sr).process_block(&mut fz_l, &mut fz_r, &p_fuzz);

        // Measure 2nd (440 Hz) + 3rd (660 Hz) harmonic energy via DFT projection
        let harmonic_energy = |sig: &[f32], freq_hz: f32| -> f32 {
            let mut re = 0.0_f32;
            let mut im = 0.0_f32;
            let m = sig.len() as f32;
            for (i, &s) in sig.iter().enumerate() {
                let ang = 2.0 * PI * freq_hz * (i as f32) / sr;
                re += s * ang.cos();
                im += s * ang.sin();
            }
            ((re * re + im * im).sqrt()) / m
        };
        let h2_fuzz = harmonic_energy(&fz_l[1024..], 440.0);
        let h3_fuzz = harmonic_energy(&fz_l[1024..], 660.0);
        assert!(
            h2_fuzz > 0.01 && h3_fuzz > 0.05,
            "Asymmetric Fuzz must generate 2nd ({}) and 3rd ({}) harmonics",
            h2_fuzz,
            h3_fuzz
        );

        // 3. Amp warm vs bright tone stack + cabinet resonance
        let mut p_amp_warm = make_clean_params();
        p_amp_warm.fx[2] = EffectSlot {
            enabled: true,
            effect_type: EffectType::Amp,
            mix: 1.0,
            param_1: 0.55,
            param_2: 0.05, // warm
            param_3: 0.75, // cabinet resonance
            param_4: 0.0,  // no presence boost
        };
        let mut p_amp_bright = p_amp_warm.clone();
        p_amp_bright.fx[2].param_2 = 0.95; // bright
        p_amp_bright.fx[2].param_4 = 0.90; // strong presence

        let mut aw_l = vec![0.0; n];
        let mut aw_r = vec![0.0; n];
        fill_sine(&mut aw_l, &mut aw_r, sr, 4200.0, 0.25);
        let mut ab_l = aw_l.clone();
        let mut ab_r = aw_r.clone();

        EffectsChain::new(sr).process_block(&mut aw_l, &mut aw_r, &p_amp_warm);
        EffectsChain::new(sr).process_block(&mut ab_l, &mut ab_r, &p_amp_bright);
        assert!(
            rms(&ab_l[1024..]) > rms(&aw_l[1024..]) * 1.4,
            "Amp bright tone+presence ({}) must boost 4.2 kHz over warm setting ({})",
            rms(&ab_l[1024..]),
            rms(&aw_l[1024..])
        );
    }

    #[test]
    fn test_reverb_environment_presets_differentiation() {
        let sr = 48000.0;
        let total = 24000;

        let render_env = |preset: &str| -> (f32, f32) {
            let mut p = make_clean_params();
            p.reverb.enabled = true;
            p.reverb.mix = 1.0;
            p.reverb.duration_s = 2.0;
            p.reverb.pre_delay_ms = 5.0;
            p.reverb.early_reflections = 0.6;
            p.reverb.tone = 0.5;
            p.reverb.preset_name = preset.to_string();

            let mut l = vec![0.0_f32; total];
            let mut r = vec![0.0_f32; total];
            fill_sine(&mut l[..256], &mut r[..256], sr, 600.0, 0.7);
            EffectsChain::new(sr).process_block(&mut l, &mut r, &p);
            let early = rms(&l[300..1100]);
            let late = rms(&l[8000..22000]);
            (early, late)
        };

        let envs = [
            "Concert Hall",
            "Cathedral",
            "Studio",
            "Chamber",
            "Taj Mahal",
            "Plate",
            "Spring",
            "Speakers",
            "Broceliande",
            "Club",
        ];

        let mut results = Vec::new();
        for env in envs {
            let (early, late) = render_env(env);
            assert!(
                early.is_finite() && late.is_finite() && (early > 0.001 || late > 0.001),
                "Reverb environment '{}' must produce finite non-zero response ({}, {})",
                env,
                early,
                late
            );
            results.push((env, early, late));
        }

        // Cathedral / Taj Mahal must have much longer/louder late decay than tight Studio / Speakers
        let (_, _, late_cathedral) = results.iter().find(|r| r.0 == "Cathedral").unwrap();
        let (_, _, late_taj) = results.iter().find(|r| r.0 == "Taj Mahal").unwrap();
        let (_, _, late_studio) = results.iter().find(|r| r.0 == "Studio").unwrap();
        let (_, _, late_speakers) = results.iter().find(|r| r.0 == "Speakers").unwrap();
        let (_, early_plate, _) = results.iter().find(|r| r.0 == "Plate").unwrap();
        let (_, early_chamber, _) = results.iter().find(|r| r.0 == "Chamber").unwrap();

        assert!(
            *late_cathedral > *late_studio * 2.0 && *late_taj > *late_speakers * 2.5,
            "Large spaces (Cathedral {}, Taj Mahal {}) must sustain longer tail than Studio {} / Speakers {}",
            late_cathedral,
            late_taj,
            late_studio,
            late_speakers
        );
        assert!(
            *early_chamber > *early_plate * 1.4,
            "Chamber ({}) must have denser early reflections than Plate ({})",
            early_chamber,
            early_plate
        );
    }
}
