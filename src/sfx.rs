//! Sound effects for the same synthesiser: short programs of sweeps on the
//! two effect voices. Each segment glides its frequency smoothly (per
//! sample) and its volume per 5 ms step; a busy voice is taken over only by
//! an effect of the same or higher priority, so a laser never cuts off an
//! explosion.
use crate::synth::{Synth, Wave, MUSIC_VOICES, VOICES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sfx {
    Laser,
    Laser2,
    EnemyShot,
    Hit,
    ExplodeSmall,
    ExplodeBig,
    PowerUp,
    Shield,
    ExtraLife,
    PlayerDie,
    Bomb,
    MenuMove,
    MenuSelect,
    Alarm,
    Tally,
    Grenade,
    Clang,
}

#[derive(Clone, Copy, Debug)]
pub struct Seg {
    pub wave: Wave,
    pub f0: f32,
    pub f1: f32,
    pub ms: f32,
    pub v0: f32,
    pub v1: f32,
    pub pw: f32,
}

const fn s(wave: Wave, f0: f32, f1: f32, ms: f32, v0: f32, v1: f32, pw: f32) -> Seg {
    Seg {
        wave,
        f0,
        f1,
        ms,
        v0,
        v1,
        pw,
    }
}

use Wave::{Noise as N, Pulse as P, Saw as W, Tri as T};

macro_rules! segs {
    ($($e:expr),* $(,)?) => {{
        const S: &[Seg] = &[$($e),*];
        S
    }};
}

pub fn def(x: Sfx) -> (u8, &'static [Seg]) {
    match x {
        Sfx::Laser => (1, segs![s(P, 1900.0, 520.0, 55.0, 0.55, 0.0, 0.25)]),
        Sfx::Laser2 => (
            1,
            segs![
                s(P, 2600.0, 700.0, 40.0, 0.5, 0.2, 0.15),
                s(W, 900.0, 300.0, 40.0, 0.3, 0.0, 0.5),
            ],
        ),
        Sfx::EnemyShot => (1, segs![s(T, 1100.0, 260.0, 90.0, 0.55, 0.0, 0.5)]),
        Sfx::Hit => (2, segs![s(N, 6000.0, 2500.0, 35.0, 0.5, 0.0, 0.5)]),
        Sfx::ExplodeSmall => (
            3,
            segs![
                s(N, 3500.0, 1400.0, 40.0, 1.0, 0.8, 0.5),
                s(N, 1400.0, 180.0, 300.0, 0.8, 0.0, 0.5),
            ],
        ),
        Sfx::ExplodeBig => (
            5,
            segs![
                s(N, 4000.0, 900.0, 60.0, 1.0, 1.0, 0.5),
                s(N, 900.0, 60.0, 1100.0, 1.0, 0.0, 0.5),
            ],
        ),
        Sfx::PowerUp => (
            4,
            segs![
                s(P, 523.0, 523.0, 45.0, 0.6, 0.6, 0.5),
                s(P, 659.0, 659.0, 45.0, 0.6, 0.6, 0.5),
                s(P, 784.0, 784.0, 45.0, 0.6, 0.6, 0.5),
                s(P, 1047.0, 1047.0, 45.0, 0.6, 0.6, 0.4),
                s(P, 1319.0, 1319.0, 45.0, 0.6, 0.5, 0.3),
                s(P, 1568.0, 1568.0, 45.0, 0.5, 0.4, 0.25),
                s(P, 2093.0, 2093.0, 160.0, 0.5, 0.0, 0.2),
            ],
        ),
        Sfx::Shield => (
            4,
            segs![
                s(T, 300.0, 1800.0, 220.0, 0.7, 0.7, 0.5),
                s(T, 1800.0, 2400.0, 120.0, 0.7, 0.0, 0.5),
            ],
        ),
        Sfx::ExtraLife => (
            4,
            segs![
                s(P, 784.0, 784.0, 70.0, 0.6, 0.6, 0.5),
                s(P, 988.0, 988.0, 70.0, 0.6, 0.6, 0.5),
                s(P, 1175.0, 1175.0, 70.0, 0.6, 0.6, 0.5),
                s(P, 1568.0, 1568.0, 300.0, 0.6, 0.0, 0.5),
            ],
        ),
        Sfx::PlayerDie => (
            6,
            segs![
                s(W, 1200.0, 900.0, 60.0, 0.8, 0.8, 0.5),
                s(N, 3000.0, 400.0, 250.0, 1.0, 0.9, 0.5),
                s(N, 400.0, 40.0, 1000.0, 0.9, 0.0, 0.5),
            ],
        ),
        Sfx::Bomb => (
            6,
            segs![
                s(N, 200.0, 5000.0, 200.0, 0.6, 1.0, 0.5),
                s(N, 5000.0, 50.0, 1300.0, 1.0, 0.0, 0.5),
            ],
        ),
        Sfx::MenuMove => (1, segs![s(T, 1400.0, 1300.0, 35.0, 0.6, 0.0, 0.5)]),
        Sfx::MenuSelect => (
            2,
            segs![
                s(P, 880.0, 880.0, 50.0, 0.6, 0.6, 0.5),
                s(P, 1760.0, 1760.0, 130.0, 0.6, 0.0, 0.5),
            ],
        ),
        Sfx::Alarm => (
            5,
            segs![
                s(P, 620.0, 620.0, 180.0, 0.6, 0.6, 0.5),
                s(P, 470.0, 470.0, 180.0, 0.6, 0.6, 0.5),
                s(P, 620.0, 620.0, 180.0, 0.6, 0.6, 0.5),
                s(P, 470.0, 470.0, 180.0, 0.6, 0.6, 0.5),
                s(P, 620.0, 620.0, 180.0, 0.6, 0.6, 0.5),
                s(P, 470.0, 470.0, 240.0, 0.6, 0.0, 0.5),
            ],
        ),
        Sfx::Tally => (1, segs![s(P, 2200.0, 2200.0, 25.0, 0.4, 0.0, 0.3)]),
        Sfx::Grenade => (2, segs![s(N, 800.0, 2500.0, 150.0, 0.0, 0.4, 0.5)]),
        Sfx::Clang => (
            2,
            segs![
                s(P, 1500.0, 1400.0, 20.0, 0.5, 0.4, 0.1),
                s(T, 700.0, 650.0, 90.0, 0.4, 0.0, 0.5),
            ],
        ),
    }
}

#[derive(Clone, Debug, Default)]
pub struct Slot {
    segs: Option<&'static [Seg]>,
    seg: usize,
    t: f32,
    prio: u8,
}

/// 5 ms steps.
pub const STEP_MS: f32 = 5.0;

pub fn start(slots: &mut [Slot; 2], synth: &mut Synth, x: Sfx) {
    let (prio, segs) = def(x);
    // A free voice, else the lower priority one, else nothing.
    let pick = (0..2)
        .find(|&i| slots[i].segs.is_none())
        .or_else(|| {
            let i = if slots[0].prio <= slots[1].prio { 0 } else { 1 };
            (slots[i].prio <= prio).then_some(i)
        });
    let Some(i) = pick else {
        return;
    };
    slots[i] = Slot {
        segs: Some(segs),
        seg: 0,
        t: 0.0,
        prio,
    };
    let v = &mut synth.v[MUSIC_VOICES + i];
    v.env.set(0, 0, 15, 3);
    v.retrigger();
    apply(&slots[i], v, true);
}

fn apply(slot: &Slot, v: &mut crate::synth::Voice, new_seg: bool) {
    let Some(segs) = slot.segs else {
        return;
    };
    let g = segs[slot.seg];
    let k = (slot.t / g.ms.max(1.0)).min(1.0);
    v.vol = g.v0 + (g.v1 - g.v0) * k;
    if new_seg {
        v.wave = g.wave;
        v.pw = g.pw;
        v.freq = g.f0;
        let samples = (g.ms / 1000.0 * crate::synth::RATE as f32).max(1.0);
        v.fmul = (g.f1 / g.f0).powf(1.0 / samples);
    }
}

/// Advance every effect by one 5 ms step.
pub fn step(slots: &mut [Slot; 2], synth: &mut Synth) {
    for i in 0..2 {
        let v = &mut synth.v[MUSIC_VOICES + i];
        let slot = &mut slots[i];
        let Some(segs) = slot.segs else {
            continue;
        };
        slot.t += STEP_MS;
        let mut new_seg = false;
        if slot.t >= segs[slot.seg].ms {
            slot.t = 0.0;
            slot.seg += 1;
            new_seg = true;
            if slot.seg >= segs.len() {
                slot.segs = None;
                slot.prio = 0;
                v.set_gate(false);
                v.fmul = 1.0;
                continue;
            }
        }
        apply(slot, v, new_seg);
    }
    debug_assert!(VOICES == MUSIC_VOICES + 2);
}
