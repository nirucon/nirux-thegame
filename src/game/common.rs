//! What the three kinds of level share: the random numbers, explosions and
//! particles, screen shake, floating score texts, power-ups, the HUD and
//! the stars.
use crate::fb::{Fb, H, W};
use crate::palette::*;
use crate::sfx::Sfx;

pub const HUD_H: i32 = 12;

#[derive(Clone, Debug)]
pub struct Rng(pub u32);

impl Rng {
    pub fn new(seed: u32) -> Rng {
        Rng(seed.max(1))
    }
    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    /// 0.0 to 1.0.
    pub fn f(&mut self) -> f32 {
        (self.next() >> 8) as f32 / (1u32 << 24) as f32
    }
    /// -1.0 to 1.0.
    pub fn s(&mut self) -> f32 {
        self.f() * 2.0 - 1.0
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            self.next() % n
        }
    }
    pub fn chance(&mut self, p: f32) -> bool {
        self.f() < p
    }
}

/// Stable per-position noise for drawing details.
pub fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9e37_79b1) ^ (y as u32).wrapping_mul(0x85eb_ca77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pk {
    Spark,
    Debris,
    Smoke,
    Ring,
    Flash,
}

#[derive(Clone, Debug)]
pub struct Part {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: u16,
    pub max: u16,
    pub kind: Pk,
    pub c: u8,
    pub r: f32,
}

#[derive(Clone, Debug)]
pub struct FloatText {
    pub x: f32,
    pub y: f32,
    pub t: u16,
    pub s: String,
    pub c: u8,
}

/// Particles, shake, the white flash and floating texts.
pub struct Fx {
    pub p: Vec<Part>,
    pub shake: f32,
    pub flash: u8,
    pub texts: Vec<FloatText>,
    pub rng: Rng,
    /// Gravity on debris (side views have none, top-down views a little).
    pub gravity: f32,
}

const MAX_PARTS: usize = 900;

impl Fx {
    pub fn new(seed: u32) -> Fx {
        Fx {
            p: Vec::with_capacity(MAX_PARTS),
            shake: 0.0,
            flash: 0,
            texts: Vec::new(),
            rng: Rng::new(seed),
            gravity: 0.0,
        }
    }

    fn push(&mut self, p: Part) {
        if self.p.len() < MAX_PARTS {
            self.p.push(p);
        }
    }

    /// An explosion: a flash, a shock ring, hot sparks, debris and smoke.
    /// `size` 1 is a small fighter, 3 a big target, 6 a boss.
    pub fn explode(&mut self, x: f32, y: f32, size: f32) {
        let n = (10.0 * size) as usize + 6;
        self.push(Part {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            life: 0,
            max: (6.0 + size * 2.0) as u16,
            kind: Pk::Flash,
            c: WHITE,
            r: 3.0 + size * 2.5,
        });
        self.push(Part {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            life: 0,
            max: (10.0 + size * 3.0) as u16,
            kind: Pk::Ring,
            c: YELLOW,
            r: 2.0 + size * 1.5,
        });
        for _ in 0..n {
            let a = self.rng.f() * std::f32::consts::TAU;
            let sp = self.rng.range(0.4, 1.6 + size * 0.5);
            let life = self.rng.range(14.0, 26.0 + size * 6.0) as u16;
            self.push(Part {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp,
                life: 0,
                max: life,
                kind: Pk::Spark,
                c: WHITE,
                r: 0.0,
            });
        }
        for _ in 0..(n / 3) {
            let a = self.rng.f() * std::f32::consts::TAU;
            let sp = self.rng.range(0.6, 1.2 + size * 0.4);
            let c = [GREY, LGREY, DGREY, BROWN][self.rng.below(4) as usize];
            let life = self.rng.range(30.0, 60.0 + size * 8.0) as u16;
            self.push(Part {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp - 0.4,
                life: 0,
                max: life,
                kind: Pk::Debris,
                c,
                r: 0.0,
            });
        }
        for _ in 0..(2 + n / 5) {
            let life = self.rng.range(30.0, 60.0) as u16;
            let (dx, dy) = (self.rng.s() * size * 2.0, self.rng.s() * size * 2.0);
            let (vx, vy) = (self.rng.s() * 0.2, -self.rng.range(0.05, 0.3));
            self.push(Part {
                x: x + dx,
                y: y + dy,
                vx,
                vy,
                life: 0,
                max: life,
                kind: Pk::Smoke,
                c: DGREY,
                r: 1.0 + size * 0.6,
            });
        }
        self.shake = self.shake.max(size * 1.2);
    }

    /// A few sparks where a shot hit something that did not die.
    pub fn sparks(&mut self, x: f32, y: f32, n: usize, dir: f32) {
        for _ in 0..n {
            let a = dir + self.rng.s() * 1.2;
            let sp = self.rng.range(0.8, 2.2);
            let max = self.rng.range(6.0, 14.0) as u16;
            self.push(Part {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp,
                life: 0,
                max,
                kind: Pk::Spark,
                c: WHITE,
                r: 0.0,
            });
        }
    }

    pub fn smoke(&mut self, x: f32, y: f32, r: f32) {
        let life = self.rng.range(25.0, 45.0) as u16;
        let (vx, vy) = (self.rng.s() * 0.15, -self.rng.range(0.1, 0.35));
        self.push(Part {
            x,
            y,
            vx,
            vy,
            life: 0,
            max: life,
            kind: Pk::Smoke,
            c: DGREY,
            r,
        });
    }

    pub fn text(&mut self, x: f32, y: f32, s: String, c: u8) {
        if self.texts.len() < 24 {
            self.texts.push(FloatText { x, y, t: 0, s, c });
        }
    }

    pub fn update(&mut self) {
        let g = self.gravity;
        self.p.retain_mut(|p| {
            p.life += 1;
            p.x += p.vx;
            p.y += p.vy;
            match p.kind {
                Pk::Spark => {
                    p.vx *= 0.94;
                    p.vy *= 0.94;
                }
                Pk::Debris => {
                    p.vy += g;
                    p.vx *= 0.985;
                }
                Pk::Smoke => {
                    p.r += 0.05;
                }
                _ => {}
            }
            p.life < p.max
        });
        self.texts.retain_mut(|t| {
            t.t += 1;
            t.y -= 0.35;
            t.t < 60
        });
        self.shake *= 0.88;
        if self.shake < 0.2 {
            self.shake = 0.0;
        }
        self.flash = self.flash.saturating_sub(1);
    }

    /// The shake offset for this frame.
    pub fn offset(&self, t: u32) -> (i32, i32) {
        if self.shake <= 0.0 {
            return (0, 0);
        }
        let h = hash(t as i32, 7);
        let a = self.shake.min(8.0);
        let dx = ((h & 0xff) as f32 / 127.5 - 1.0) * a;
        let dy = (((h >> 8) & 0xff) as f32 / 127.5 - 1.0) * a;
        (dx.round() as i32, dy.round() as i32)
    }

    pub fn draw(&self, fb: &mut Fb, ox: i32, oy: i32) {
        for p in &self.p {
            let k = p.life as f32 / p.max.max(1) as f32;
            let (x, y) = (p.x as i32 + ox, p.y as i32 + oy);
            if p.kind == Pk::Smoke {
                let c = if k < 0.4 { GREY } else { DGREY };
                let r = p.r as i32;
                // Dithered puffs read as smoke without blending.
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dy * dy <= r * r && (x + dx + y + dy) & 1 == 0 {
                            fb.set(x + dx, y + dy, c);
                        }
                    }
                }
            }
        }
        for p in &self.p {
            let k = p.life as f32 / p.max.max(1) as f32;
            let (x, y) = (p.x as i32 + ox, p.y as i32 + oy);
            match p.kind {
                Pk::Spark => {
                    let c = HEAT[((k * 6.0) as usize).min(6)];
                    fb.set(x, y, c);
                    if k < 0.35 {
                        fb.set(x - p.vx.signum() as i32, y - p.vy.signum() as i32, YELLOW);
                    }
                }
                Pk::Debris => {
                    fb.set(x, y, p.c);
                    if k < 0.5 {
                        fb.set(x + 1, y, p.c);
                    }
                }
                Pk::Ring => {
                    let r = (p.r + k * p.r * 3.0) as i32;
                    let c = HEAT[((k * 5.0) as usize + 1).min(6)];
                    fb.circle(x, y, r, c);
                    if k < 0.5 {
                        fb.circle(x, y, r - 1, HEAT[((k * 5.0) as usize).min(6)]);
                    }
                }
                Pk::Flash => {
                    let r = (p.r * (1.0 - k * 0.5)) as i32;
                    let c = HEAT[((k * 4.0) as usize).min(4)];
                    fb.disc(x, y, r, c);
                    if k < 0.5 {
                        fb.disc(x, y, r / 2, WHITE);
                    }
                }
                Pk::Smoke => {}
            }
        }
        for t in &self.texts {
            let c = if t.t % 8 < 4 { t.c } else { WHITE };
            let w = crate::fb::text_w(&t.s);
            fb.text(t.x as i32 - w / 2 + ox, t.y as i32 + oy, &t.s, c);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerUp {
    Weapon,
    Shield,
    Bomb,
    Life,
    Points,
}

impl PowerUp {
    pub fn letter(self) -> char {
        match self {
            PowerUp::Weapon => 'P',
            PowerUp::Shield => 'S',
            PowerUp::Bomb => 'B',
            PowerUp::Life => '1',
            PowerUp::Points => '*',
        }
    }
    pub fn colors(self) -> (u8, u8) {
        match self {
            PowerUp::Weapon => (RED, LRED),
            PowerUp::Shield => (BLUE, CYAN),
            PowerUp::Bomb => (ORANGE, YELLOW),
            PowerUp::Life => (GREEN, LGREEN),
            PowerUp::Points => (PURPLE, LRED),
        }
    }
}

/// A capsule with a letter, its highlight running around it.
pub fn draw_capsule(fb: &mut Fb, x: i32, y: i32, kind: PowerUp, t: u32) {
    let (dark, light) = kind.colors();
    let (w, h) = (14, 10);
    let (x0, y0) = (x - w / 2, y - h / 2);
    fb.rect(x0 + 1, y0, w - 2, h, dark);
    fb.rect(x0, y0 + 1, w, h - 2, dark);
    fb.hline(x0 + 2, x0 + w - 3, y0 + 1, light);
    let blink = (t / 4) % 2 == 0;
    let edge = if blink { WHITE } else { light };
    fb.hline(x0 + 1, x0 + w - 2, y0, edge);
    fb.hline(x0 + 1, x0 + w - 2, y0 + h - 1, edge);
    for yy in y0 + 1..y0 + h - 1 {
        fb.set(x0, yy, edge);
        fb.set(x0 + w - 1, yy, edge);
    }
    fb.char(
        x - 4,
        y0 + 1,
        kind.letter(),
        if blink { WHITE } else { YELLOW },
    );
}

#[derive(Clone, Debug)]
pub struct Stats {
    pub score: u32,
    pub lives: i32,
    pub weapon: u8,
    pub bombs: u8,
    pub shield: u8,
    pub kills: u32,
    pub next_life: u32,
    pub deaths_this_level: u32,
}

pub const LIFE_EVERY: u32 = 40000;

impl Stats {
    pub fn new(lives: i32) -> Stats {
        Stats {
            score: 0,
            lives,
            weapon: 1,
            bombs: 2,
            shield: 0,
            kills: 0,
            next_life: LIFE_EVERY,
            deaths_this_level: 0,
        }
    }

    /// Add points; returns true when that earned an extra life.
    pub fn add(&mut self, pts: u32) -> bool {
        self.score = self.score.saturating_add(pts);
        if self.score >= self.next_life {
            self.next_life += LIFE_EVERY;
            self.lives = (self.lives + 1).min(9);
            return true;
        }
        false
    }

    pub fn collect(&mut self, p: PowerUp, out: &mut Out) -> &'static str {
        match p {
            PowerUp::Weapon => {
                if self.weapon < 4 {
                    self.weapon += 1;
                    out.sfx.push(Sfx::PowerUp);
                    ["", "", "DOUBLE", "SPREAD", "MAXIMUM"][self.weapon as usize]
                } else {
                    self.add(5000);
                    out.sfx.push(Sfx::PowerUp);
                    "5000"
                }
            }
            PowerUp::Shield => {
                self.shield = 3;
                out.sfx.push(Sfx::Shield);
                "SHIELD"
            }
            PowerUp::Bomb => {
                self.bombs = (self.bombs + 1).min(5);
                out.sfx.push(Sfx::PowerUp);
                "BOMB"
            }
            PowerUp::Life => {
                self.lives = (self.lives + 1).min(9);
                out.sfx.push(Sfx::ExtraLife);
                "1UP"
            }
            PowerUp::Points => {
                self.add(2500);
                out.sfx.push(Sfx::PowerUp);
                "2500"
            }
        }
    }
}

/// What a level asks of the game this frame.
#[derive(Default)]
pub struct Out {
    pub sfx: Vec<Sfx>,
    pub music: Option<crate::songs::SongId>,
}

impl Out {
    pub fn play(&mut self, s: Sfx) {
        if !self.sfx.contains(&s) {
            self.sfx.push(s);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Playing,
    Cleared,
    Over,
}

/// Parallax stars: three layers, `speed` pixels per frame for the nearest.
pub struct Stars {
    s: Vec<(f32, f32, u8)>,
}

impl Stars {
    pub fn new(n: usize, seed: u32) -> Stars {
        let mut r = Rng::new(seed);
        let s = (0..n)
            .map(|i| {
                (
                    r.f() * W as f32,
                    HUD_H as f32 + r.f() * (H - HUD_H) as f32,
                    (i % 3) as u8,
                )
            })
            .collect();
        Stars { s }
    }

    pub fn update(&mut self, dx: f32, dy: f32) {
        for (x, y, l) in self.s.iter_mut() {
            let k = [0.2, 0.5, 1.0][*l as usize];
            *x -= dx * k;
            *y -= dy * k;
            if *x < 0.0 {
                *x += W as f32;
            }
            if *x >= W as f32 {
                *x -= W as f32;
            }
            if *y < HUD_H as f32 {
                *y += (H - HUD_H) as f32;
            }
            if *y >= H as f32 {
                *y -= (H - HUD_H) as f32;
            }
        }
    }

    pub fn draw(&self, fb: &mut Fb, t: u32) {
        for (i, (x, y, l)) in self.s.iter().enumerate() {
            let c = match l {
                0 => DGREY,
                1 => {
                    if (i as u32 + t / 9) % 11 == 0 {
                        LGREY
                    } else {
                        GREY
                    }
                }
                _ => {
                    if (i as u32 + t / 5) % 7 == 0 {
                        CYAN
                    } else {
                        WHITE
                    }
                }
            };
            fb.set(*x as i32, *y as i32, c);
            if *l == 2 && i % 4 == 0 {
                fb.set(*x as i32 + 1, *y as i32, LGREY);
            }
        }
    }
}

fn digits(n: u32, w: usize) -> String {
    format!("{n:0w$}")
}

/// The status line: score, high score, lives, bombs, weapon and shield.
pub fn draw_hud(fb: &mut Fb, st: &Stats, hi: u32, line: u8) {
    fb.rect(0, 0, W, HUD_H, BLACK);
    fb.hline(0, W - 1, HUD_H - 1, line);
    fb.text(2, 2, "1UP", LRED);
    fb.text(28, 2, &digits(st.score, 7), WHITE);
    fb.text(92, 2, "HI", LBLUE);
    fb.text(110, 2, &digits(hi.max(st.score), 7), LGREY);
    let mut x = 172;
    for i in 0..st.lives.clamp(0, 5) {
        fb.char(x + i * 9, 2, '~', CYAN);
    }
    x += 46;
    for i in 0..st.bombs.min(5) as i32 {
        fb.char(x + i * 7, 2, '&', YELLOW);
    }
    // Weapon power as four little bars.
    let wx = 268;
    fb.text(wx, 2, "P", LRED);
    for i in 0..4 {
        let c = if (i as u8) < st.weapon {
            [RED, LRED, ORANGE, YELLOW][i]
        } else {
            DGREY
        };
        fb.rect(
            wx + 9 + i as i32 * 4,
            3 + (3 - i as i32),
            3,
            4 + i as i32,
            c,
        );
    }
    if st.shield > 0 {
        for i in 0..st.shield as i32 {
            fb.rect(wx + 28 + i * 6, 3, 4, 6, CYAN);
            fb.set(wx + 28 + i * 6, 3, BLACK);
        }
    }
}

/// A boss health bar along the bottom of the screen.
pub fn draw_boss_bar(fb: &mut Fb, name: &str, hp: i32, max: i32, t: u32) {
    let y = H - 9;
    fb.rect(0, y - 1, W, 10, BLACK);
    fb.text(4, y, name, if t % 16 < 8 { LRED } else { RED });
    let x0 = 8 + crate::fb::text_w(name) + 4;
    let w = W - x0 - 6;
    fb.frame(x0, y + 1, w, 6, GREY);
    let f = (w - 2) * hp.max(0) / max.max(1);
    fb.rect(x0 + 1, y + 2, f, 4, if hp * 4 < max { RED } else { LRED });
    fb.hline(x0 + 1, x0 + f, y + 2, YELLOW);
}

/// Distance squared.
pub fn d2(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    (ax - bx) * (ax - bx) + (ay - by) * (ay - by)
}

/// A round enemy bullet, flickering between two colours.
pub fn draw_bullet(fb: &mut Fb, x: i32, y: i32, t: u32, big: bool) {
    let c = if t % 6 < 3 { LRED } else { PURPLE };
    if big {
        fb.disc(x, y, 3, c);
        fb.disc(x, y, 1, YELLOW);
        fb.set(x, y, WHITE);
    } else {
        fb.hline(x - 1, x + 1, y - 1, c);
        fb.hline(x - 2, x + 2, y, c);
        fb.hline(x - 1, x + 1, y + 1, c);
        fb.set(x, y, WHITE);
    }
}
