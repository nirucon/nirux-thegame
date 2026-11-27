//! An original SID-style synthesiser: three music voices and two sound
//! effect voices with triangle, sawtooth, pulse and noise waveforms, SID
//! timed ADSR envelopes, pulse width, hard sync and ring modulation, and a
//! resonant multimode filter (a zero-delay state variable filter, stable at
//! any setting) shared by the music voices the way the SID shares its one
//! filter. Band limited oscillators (polyBLEP) keep the top end clean, a
//! light stereo spread and echo give the mix some room, and a soft limiter
//! keeps the output from ever clipping.

pub const RATE: u32 = 44100;
const FS: f32 = RATE as f32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wave {
    Off,
    Tri,
    Saw,
    Pulse,
    Noise,
}

pub const LP: u8 = 1;
pub const BP: u8 = 2;
pub const HP: u8 = 4;

const ATTACK_MS: [f32; 16] = [
    2.0, 8.0, 16.0, 24.0, 38.0, 56.0, 68.0, 80.0, 100.0, 250.0, 500.0, 800.0, 1000.0, 3000.0,
    5000.0, 8000.0,
];
const DECAY_MS: [f32; 16] = [
    6.0, 24.0, 48.0, 72.0, 114.0, 168.0, 204.0, 240.0, 300.0, 750.0, 1500.0, 2400.0, 3000.0,
    9000.0, 15000.0, 24000.0,
];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Stage {
    Attack,
    Decay,
    Release,
}

#[derive(Clone, Debug)]
pub struct Env {
    stage: Stage,
    pub level: f32,
    att: f32,
    dec: f32,
    sus: f32,
    rel: f32,
}

impl Env {
    fn new() -> Env {
        let mut e = Env {
            stage: Stage::Release,
            level: 0.0,
            att: 0.0,
            dec: 0.0,
            sus: 0.0,
            rel: 0.0,
        };
        e.set(0, 9, 0, 6);
        e
    }

    /// SID style values 0 to 15 for attack, decay, sustain and release.
    pub fn set(&mut self, a: u8, d: u8, s: u8, r: u8) {
        let ms = |t: f32| (t * FS / 1000.0).max(1.0);
        self.att = 1.0 / ms(ATTACK_MS[(a & 15) as usize]);
        self.dec = (-4.6 / ms(DECAY_MS[(d & 15) as usize])).exp();
        self.sus = (s & 15) as f32 / 15.0;
        self.rel = (-4.6 / ms(DECAY_MS[(r & 15) as usize])).exp();
    }

    fn gate(&mut self, on: bool) {
        self.stage = if on { Stage::Attack } else { Stage::Release };
    }

    #[inline]
    fn next(&mut self) -> f32 {
        match self.stage {
            Stage::Attack => {
                self.level += self.att;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = Stage::Decay;
                }
            }
            Stage::Decay => self.level = self.sus + (self.level - self.sus) * self.dec,
            Stage::Release => {
                self.level *= self.rel;
                if self.level < 1e-5 {
                    self.level = 0.0;
                }
            }
        }
        self.level
    }
}

/// Zero-delay-feedback state variable filter (one per voice, shared
/// coefficients).
#[derive(Clone, Debug, Default)]
struct Svf {
    ic1: f32,
    ic2: f32,
}

#[derive(Clone, Copy, Debug)]
struct SvfCoef {
    a1: f32,
    a2: f32,
    a3: f32,
    k: f32,
    mode: u8,
}

impl Svf {
    #[inline]
    fn run(&mut self, x: f32, c: &SvfCoef) -> f32 {
        // A little drive into the filter, like the SID's grit when
        // resonant.
        let v0 = x;
        let v3 = v0 - self.ic2;
        let v1 = c.a1 * self.ic1 + c.a2 * v3;
        let v2 = self.ic2 + c.a2 * self.ic1 + c.a3 * v3;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        if !self.ic1.is_finite() || !self.ic2.is_finite() {
            self.ic1 = 0.0;
            self.ic2 = 0.0;
        }
        let mut y = 0.0;
        if c.mode & LP != 0 {
            y += v2;
        }
        if c.mode & BP != 0 {
            y += v1;
        }
        if c.mode & HP != 0 {
            y += v0 - c.k * v1 - v2;
        }
        y
    }
}

#[derive(Clone, Debug)]
pub struct Voice {
    pub wave: Wave,
    pub freq: f32,
    pub pw: f32,
    pub ring: bool,
    pub sync: bool,
    pub filter: bool,
    pub vol: f32,
    pub pan: f32,
    /// Multiplied into the frequency every sample (smooth sweeps).
    pub fmul: f32,
    gate: bool,
    phase: f32,
    wrapped: bool,
    lfsr: u32,
    noise: f32,
    nphase: f32,
    pub env: Env,
    svf: Svf,
}

impl Voice {
    fn new(pan: f32, seed: u32) -> Voice {
        Voice {
            wave: Wave::Off,
            freq: 440.0,
            pw: 0.5,
            ring: false,
            sync: false,
            filter: false,
            vol: 1.0,
            pan,
            fmul: 1.0,
            gate: false,
            phase: 0.0,
            wrapped: false,
            lfsr: 0x7ffff8 ^ seed,
            noise: 0.0,
            nphase: 0.0,
            env: Env::new(),
            svf: Svf::default(),
        }
    }

    pub fn set_gate(&mut self, on: bool) {
        if on != self.gate {
            self.gate = on;
            self.env.gate(on);
        }
    }

    /// Start the envelope again even when the gate is already on.
    pub fn retrigger(&mut self) {
        self.gate = true;
        self.env.gate(true);
    }

    #[inline]
    fn advance(&mut self) {
        if self.fmul != 1.0 {
            self.freq = (self.freq * self.fmul).clamp(10.0, 18000.0);
        }
        let dt = (self.freq / FS).clamp(0.0, 0.5);
        self.phase += dt;
        self.wrapped = false;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
            self.wrapped = true;
        }
        if self.wave == Wave::Noise {
            self.nphase += dt * 16.0;
            while self.nphase >= 1.0 {
                self.nphase -= 1.0;
                // 23-bit Galois LFSR, the SID's length.
                let bit = self.lfsr & 1;
                self.lfsr >>= 1;
                if bit != 0 {
                    self.lfsr ^= 0x420000;
                }
                self.noise = (self.lfsr & 0xff) as f32 / 127.5 - 1.0;
            }
        }
    }

    #[inline]
    fn osc(&self, mod_phase: f32) -> f32 {
        let t = self.phase;
        let dt = (self.freq / FS).clamp(1e-6, 0.5);
        match self.wave {
            Wave::Off => 0.0,
            Wave::Tri => {
                let v = 1.0 - 4.0 * (t - 0.5).abs();
                if self.ring && mod_phase >= 0.5 {
                    -v
                } else {
                    v
                }
            }
            Wave::Saw => 2.0 * t - 1.0 - blep(t, dt),
            Wave::Pulse => {
                let pw = self.pw.clamp(0.02, 0.98);
                let naive = if t < pw { 1.0 } else { -1.0 };
                let mut t2 = t - pw;
                if t2 < 0.0 {
                    t2 += 1.0;
                }
                // Centre the pulse so narrow pulses do not shift the level.
                naive + blep(t, dt) - blep(t2, dt) - (2.0 * pw - 1.0)
            }
            Wave::Noise => self.noise,
        }
    }
}

#[inline]
fn blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        t + t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + t + t + 1.0
    } else {
        0.0
    }
}

pub const MUSIC_VOICES: usize = 3;
pub const VOICES: usize = 5;

pub struct Synth {
    pub v: [Voice; VOICES],
    pub cutoff: f32,
    pub res: f32,
    pub mode: u8,
    coef: SvfCoef,
    coef_key: (f32, f32, u8),
    pub music_vol: f32,
    pub sfx_vol: f32,
    pub echo: bool,
    dl: Vec<f32>,
    dr: Vec<f32>,
    dpos: usize,
    lp_l: f32,
    lp_r: f32,
    dc: [f32; 4],
}

impl Default for Synth {
    fn default() -> Self {
        Self::new()
    }
}

impl Synth {
    pub fn new() -> Synth {
        let delay = (FS * 0.29) as usize;
        let mut s = Synth {
            v: [
                Voice::new(0.0, 1),
                Voice::new(-0.32, 2),
                Voice::new(0.3, 3),
                Voice::new(-0.1, 4),
                Voice::new(0.1, 5),
            ],
            cutoff: 2000.0,
            res: 0.3,
            mode: LP,
            coef: SvfCoef {
                a1: 0.0,
                a2: 0.0,
                a3: 0.0,
                k: 2.0,
                mode: LP,
            },
            coef_key: (0.0, 0.0, 0),
            music_vol: 0.8,
            sfx_vol: 0.8,
            echo: true,
            dl: vec![0.0; delay],
            dr: vec![0.0; delay],
            dpos: 0,
            lp_l: 0.0,
            lp_r: 0.0,
            dc: [0.0; 4],
        };
        s.update_coef();
        s
    }

    fn update_coef(&mut self) {
        let key = (self.cutoff, self.res, self.mode);
        if key == self.coef_key {
            return;
        }
        self.coef_key = key;
        let fc = self.cutoff.clamp(30.0, 16000.0);
        let g = (std::f32::consts::PI * fc / FS).tan();
        let k = 2.0 - 1.9 * self.res.clamp(0.0, 1.0);
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        self.coef = SvfCoef {
            a1,
            a2,
            a3,
            k,
            mode: self.mode,
        };
    }

    /// Render interleaved stereo samples.
    pub fn render(&mut self, out: &mut [f32]) {
        self.update_coef();
        let c = self.coef;
        let n = self.dl.len();
        for frame in out.chunks_exact_mut(2) {
            for v in self.v.iter_mut() {
                v.advance();
            }
            // Sync and ring modulation take the voice before (as on the SID,
            // where voice 1 follows voice 3).
            let mut phases = [0f32; MUSIC_VOICES];
            let mut wraps = [false; MUSIC_VOICES];
            for i in 0..MUSIC_VOICES {
                phases[i] = self.v[i].phase;
                wraps[i] = self.v[i].wrapped;
            }
            let (mut l, mut r) = (0f32, 0f32);
            let (mut fl, mut fr) = (0f32, 0f32);
            for i in 0..VOICES {
                let m = if i < MUSIC_VOICES { (i + 2) % 3 } else { i };
                if i < MUSIC_VOICES && self.v[i].sync && wraps[m] {
                    self.v[i].phase = 0.0;
                }
                let mp = if i < MUSIC_VOICES { phases[m] } else { 0.0 };
                let v = &mut self.v[i];
                let e = v.env.next();
                if e == 0.0 && v.wave != Wave::Noise {
                    continue;
                }
                let mut x = v.osc(mp) * e * v.vol;
                if i < MUSIC_VOICES {
                    if v.filter {
                        x = v.svf.run(x, &c);
                    }
                    x *= 0.27 * self.music_vol;
                } else {
                    x *= 0.33 * self.sfx_vol;
                }
                let pl = 0.5 - v.pan * 0.5;
                let pr = 0.5 + v.pan * 0.5;
                l += x * pl * 2.0;
                r += x * pr * 2.0;
                if i >= MUSIC_VOICES {
                    fl += x * pl;
                    fr += x * pr;
                }
            }
            // Ping-pong echo, darker on every repeat.
            if self.echo {
                let el = self.dl[self.dpos];
                let er = self.dr[self.dpos];
                self.lp_l += (er - self.lp_l) * 0.35;
                self.lp_r += (el - self.lp_r) * 0.35;
                let send_l = l - fl * 0.5;
                let send_r = r - fr * 0.5;
                self.dl[self.dpos] = send_l * 0.5 + self.lp_l * 0.34;
                self.dr[self.dpos] = send_r * 0.5 + self.lp_r * 0.34;
                self.dpos = (self.dpos + 1) % n;
                l += el * 0.26;
                r += er * 0.26;
            }
            // DC blocker, then a soft limiter above -2 dBFS.
            let yl = l - self.dc[0] + 0.995 * self.dc[1];
            self.dc[0] = l;
            self.dc[1] = yl;
            let yr = r - self.dc[2] + 0.995 * self.dc[3];
            self.dc[2] = r;
            self.dc[3] = yr;
            frame[0] = limit(yl);
            frame[1] = limit(yr);
        }
    }
}

#[inline]
pub fn limit(x: f32) -> f32 {
    let a = x.abs();
    if a < 0.75 {
        x
    } else {
        let y = 0.75 + 0.2 * ((a - 0.75) / 0.2).tanh();
        y.copysign(x)
    }
}

/// Note number (MIDI, 69 = A4) to Hz.
pub fn note_hz(n: f32) -> f32 {
    440.0 * 2f32.powf((n - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn voices_sound_and_the_filter_is_stable() {
        let mut s = Synth::new();
        for (i, w) in [Wave::Saw, Wave::Pulse, Wave::Noise]
            .into_iter()
            .enumerate()
        {
            s.v[i].wave = w;
            s.v[i].freq = 110.0 * (i + 1) as f32;
            s.v[i].filter = true;
            s.v[i].env.set(0, 0, 15, 0);
            s.v[i].set_gate(true);
        }
        s.res = 1.0;
        let mut buf = vec![0f32; 8820];
        let mut peak = 0f32;
        for k in 0..40 {
            s.cutoff = 50.0 + k as f32 * 400.0;
            s.mode = [LP, BP, HP, LP | HP][k % 4];
            s.render(&mut buf);
            for x in &buf {
                assert!(x.is_finite());
                peak = peak.max(x.abs());
            }
        }
        assert!(peak > 0.05 && peak <= 0.951, "peak {peak}");
        assert!((note_hz(69.0) - 440.0).abs() < 0.01);
    }
}
