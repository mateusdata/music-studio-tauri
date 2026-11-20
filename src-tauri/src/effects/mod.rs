use crate::contracts::{EffectType, EngineParams};
use std::f32::consts::PI;

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
    fn update_peaking(&mut self, sr: f32, freq: f32, gain_db: f32, q: f32) {
        if gain_db.abs() < 0.02 {
            self.b0 = 1.0;
            self.b1 = 0.0;
            self.b2 = 0.0;
            self.a1 = 0.0;
            self.a2 = 0.0;
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

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
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
    eq_filters: [BiquadState; 3],
    delay_buf_l: Vec<f32>,
    delay_buf_r: Vec<f32>,
    delay_write_pos: usize,
    chorus_buf_l: Vec<f32>,
    chorus_buf_r: Vec<f32>,
    chorus_pos: usize,
    chorus_phase: f32,
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
        let max_delay = (sr * 1.5) as usize;
        let max_chorus = (sr * 0.08) as usize;
        let scale = sr / 44100.0;
        let c_lens = [
            (1557.0 * scale) as usize,
            (1617.0 * scale) as usize,
            (1491.0 * scale) as usize,
            (1422.0 * scale) as usize,
        ];
        Self {
            sample_rate: sr,
            eq_filters: [BiquadState::default(), BiquadState::default(), BiquadState::default()],
            delay_buf_l: vec![0.0; max_delay.max(4096)],
            delay_buf_r: vec![0.0; max_delay.max(4096)],
            delay_write_pos: 0,
            chorus_buf_l: vec![0.0; max_chorus.max(1024)],
            chorus_buf_r: vec![0.0; max_chorus.max(1024)],
            chorus_pos: 0,
            chorus_phase: 0.0,
            reverb_combs_l: [
                vec![0.0; c_lens[0]],
                vec![0.0; c_lens[1]],
                vec![0.0; c_lens[2]],
                vec![0.0; c_lens[3]],
            ],
            reverb_combs_r: [
                vec![0.0; c_lens[0] + 23],
                vec![0.0; c_lens[1] + 23],
                vec![0.0; c_lens[2] + 23],
                vec![0.0; c_lens[3] + 23],
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

        // Lid position gentle darkening when closed
        let lid_gain = 0.72 + params.output.lid_position.clamp(0.0, 1.0) * 0.28;
        // Stereo width mid/side matrix
        let width = (params.output.stereo_width * 2.0).clamp(0.0, 2.0);
        // Binaural slight crossfeed
        let crossfeed = if params.output.binaural_mode { 0.14 } else { 0.0 };

        let reverb_fb = (0.62 + (params.reverb.duration_s / 6.0).clamp(0.0, 0.34)).min(0.95);
        let reverb_damp = (1.0 - params.reverb.tone).clamp(0.1, 0.85);
        let post_gain = 10.0_f32.powf(params.post_effect_gain_db / 20.0);
        let lim_thresh = 10.0_f32.powf(params.limiter.threshold_db / 20.0).clamp(0.1, 1.0);
        let lim_gain = 10.0_f32.powf(params.limiter.gain_db / 20.0);

        for i in 0..len {
            let mut l = buf_l[i] * lid_gain;
            let mut r = buf_r[i] * lid_gain;

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

            // FX Slots (Delay & Chorus/Tremolo)
            for slot in &params.fx {
                if !slot.enabled {
                    continue;
                }
                match slot.effect_type {
                    EffectType::Delay => {
                        let delay_samps = ((0.06 + slot.param_1 * 0.55) * self.sample_rate) as usize;
                        let delay_samps = delay_samps.clamp(1, self.delay_buf_l.len() - 1);
                        let read_pos = (self.delay_write_pos + self.delay_buf_l.len() - delay_samps)
                            % self.delay_buf_l.len();
                        let dl = self.delay_buf_l[read_pos];
                        let dr = self.delay_buf_r[read_pos];
                        let fb = (slot.param_2 * 0.72).clamp(0.0, 0.82);
                        self.delay_buf_l[self.delay_write_pos] = l + dr * fb;
                        self.delay_buf_r[self.delay_write_pos] = r + dl * fb;
                        self.delay_write_pos = (self.delay_write_pos + 1) % self.delay_buf_l.len();
                        l += dl * slot.mix;
                        r += dr * slot.mix;
                    }
                    EffectType::Chorus | EffectType::Tremolo => {
                        self.chorus_phase += 2.0 * PI * (0.4 + slot.param_1 * 3.5) / self.sample_rate;
                        if self.chorus_phase > 2.0 * PI {
                            self.chorus_phase -= 2.0 * PI;
                        }
                        let lfo = 0.5 + 0.5 * self.chorus_phase.sin();
                        if slot.effect_type == EffectType::Tremolo {
                            let mod_l = 1.0 - slot.mix * lfo;
                            let mod_r = 1.0 - slot.mix * (1.0 - lfo);
                            l *= mod_l;
                            r *= mod_r;
                        } else {
                            let mod_delay = (0.004 + lfo * 0.012) * self.sample_rate;
                            let tap = mod_delay as usize;
                            let c_len = self.chorus_buf_l.len();
                            let rpos = (self.chorus_pos + c_len - tap.clamp(1, c_len - 1)) % c_len;
                            let cl = self.chorus_buf_l[rpos];
                            let cr = self.chorus_buf_r[rpos];
                            self.chorus_buf_l[self.chorus_pos] = l;
                            self.chorus_buf_r[self.chorus_pos] = r;
                            self.chorus_pos = (self.chorus_pos + 1) % c_len;
                            l = l * (1.0 - slot.mix * 0.5) + cl * slot.mix;
                            r = r * (1.0 - slot.mix * 0.5) + cr * slot.mix;
                        }
                    }
                    _ => {}
                }
            }

            // Algorithmic Stereo Reverb
            if params.reverb.enabled && params.reverb.mix > 0.001 {
                let mut wet_l = 0.0_f32;
                let mut wet_r = 0.0_f32;
                let rev_in_l = l * 0.25;
                let rev_in_r = r * 0.25;
                for c in 0..4 {
                    let idx_l = self.reverb_idx[c] % self.reverb_combs_l[c].len();
                    let out_cl = self.reverb_combs_l[c][idx_l];
                    self.reverb_lp_l[c] = out_cl * (1.0 - reverb_damp) + self.reverb_lp_l[c] * reverb_damp;
                    self.reverb_combs_l[c][idx_l] = rev_in_l + self.reverb_lp_l[c] * reverb_fb;
                    wet_l += out_cl;

                    let idx_r = self.reverb_idx[c] % self.reverb_combs_r[c].len();
                    let out_cr = self.reverb_combs_r[c][idx_r];
                    self.reverb_lp_r[c] = out_cr * (1.0 - reverb_damp) + self.reverb_lp_r[c] * reverb_damp;
                    self.reverb_combs_r[c][idx_r] = rev_in_r + self.reverb_lp_r[c] * reverb_fb;
                    wet_r += out_cr;

                    self.reverb_idx[c] = self.reverb_idx[c].wrapping_add(1);
                }
                // Highpass on wet tail
                let hp_l = 0.985 * (self.hp_prev_out_l + wet_l - self.hp_prev_in_l);
                self.hp_prev_in_l = wet_l;
                self.hp_prev_out_l = hp_l;

                let hp_r = 0.985 * (self.hp_prev_out_r + wet_r - self.hp_prev_in_r);
                self.hp_prev_in_r = wet_r;
                self.hp_prev_out_r = hp_r;

                let mix = params.reverb.mix.clamp(0.0, 1.0);
                l = l * (1.0 - mix * 0.3) + hp_l * mix;
                r = r * (1.0 - mix * 0.3) + hp_r * mix;
            }

            l *= post_gain;
            r *= post_gain;

            // Brickwall Soft Limiter
            if params.limiter.enabled {
                l = soft_limit(l * lim_gain, lim_thresh);
                r = soft_limit(r * lim_gain, lim_thresh);
            }

            buf_l[i] = if l.is_finite() { l.clamp(-1.0, 1.0) } else { 0.0 };
            buf_r[i] = if r.is_finite() { r.clamp(-1.0, 1.0) } else { 0.0 };
        }
    }
}

#[inline]
fn soft_limit(x: f32, thresh: f32) -> f32 {
    let abs = x.abs();
    if abs <= thresh {
        x
    } else {
        let excess = (abs - thresh) / (1.0 - thresh + 1e-4);
        let compressed = thresh + (1.0 - thresh) * excess.tanh();
        x.signum() * compressed.min(0.995)
    }
}
