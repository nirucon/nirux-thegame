//! Sector 1, "The Outer Rim": a horizontally scrolling space shooter. Waves
//! of fighters, an asteroid field, a capital ship a kilometre long that you
//! fly over and take apart turret by turret and generator by generator, and
//! the Warden guarding the jump gate at the end.
use super::common::*;
use super::Level;
use crate::art;
use crate::config::Difficulty;
use crate::fb::{Fb, H, W};
use crate::input::Input;
use crate::palette::*;
use crate::sfx::Sfx;
use crate::songs::SongId;

const TOP: f32 = HUD_H as f32 + 6.0;
const BOTTOM: f32 = H as f32 - 6.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ev {
    Wisps {
        y: f32,
        n: u8,
        amp: f32,
        fire: bool,
    },
    Darts {
        n: u8,
    },
    Hornets {
        top: bool,
        n: u8,
    },
    Pod {
        y: f32,
        drop: PowerUp,
    },
    Rocks {
        frames: u32,
    },
    Capital,
    /// A gunship that holds up the sector until it is shot down.
    Gunship(u8),
    /// A boss of style N that ends the sector.
    Boss(u8),
    /// A boss of style N that has to be beaten on the way.
    MidBoss(u8),
    /// Shipyard gantries: N frames of steel with a gap to fly through.
    Gantries {
        n: u8,
        gap: f32,
    },
    /// A tunnel of rock for this many frames.
    Tunnel {
        frames: u32,
    },
    Text(&'static str),
}

/// What makes one side-on sector different from another.
#[derive(Clone)]
pub struct SideSpec {
    pub script: Vec<(u32, Ev)>,
    pub border: u8,
    pub hud: u8,
    pub song: SongId,
    pub boss_song: SongId,
    /// Planet colours: light, mid, band (or none).
    pub planet: Option<[u8; 3]>,
    /// How fast the world scrolls (stars, gantries, tunnels).
    pub speed: f32,
}

pub struct BossStyle {
    pub name: &'static str,
    /// Hull, plates, edges.
    pub cols: [u8; 3],
    pub cannon: i32,
    pub core: i32,
    pub pattern: u8,
}

pub const STYLES: [BossStyle; 5] = [
    BossStyle {
        name: "WARDEN",
        cols: [DGREY, GREY, LGREY],
        cannon: 40,
        core: 75,
        pattern: 0,
    },
    BossStyle {
        name: "FOUNDRY",
        cols: [BROWN, ORANGE, YELLOW],
        cannon: 48,
        core: 95,
        pattern: 1,
    },
    BossStyle {
        name: "WARDEN MK II",
        cols: [BLUE, PURPLE, LRED],
        cannon: 45,
        core: 80,
        pattern: 1,
    },
    BossStyle {
        name: "EXECUTOR",
        cols: [BLUE, LBLUE, CYAN],
        cannon: 55,
        core: 115,
        pattern: 2,
    },
    BossStyle {
        name: "CORE GUARDIAN",
        cols: [RED, LRED, WHITE],
        cannon: 55,
        core: 110,
        pattern: 2,
    },
];

#[derive(Clone, Debug)]
struct Gantry {
    x: f32,
    gap_y: f32,
    gap: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Ek {
    Wisp,
    Dart,
    Hornet,
    Pod,
    Rock { big: bool },
    Turret { u: f32 },
    Generator { u: f32 },
    Gunship { v: u8 },
}

#[derive(Clone, Debug)]
struct Enemy {
    kind: Ek,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    hp: i32,
    t: u32,
    base: f32,
    flash: u8,
    group: u16,
    drop: Option<PowerUp>,
    cd: i32,
    fire: bool,
    aim: f32,
}

impl Enemy {
    fn new(kind: Ek, x: f32, y: f32, hp: i32) -> Enemy {
        Enemy {
            kind,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            hp,
            t: 0,
            base: y,
            flash: 0,
            group: 0,
            drop: None,
            cd: 60,
            fire: false,
            aim: std::f32::consts::PI,
        }
    }
    fn half(&self) -> (f32, f32) {
        match self.kind {
            Ek::Wisp => (7.0, 4.0),
            Ek::Dart => (7.0, 3.0),
            Ek::Hornet => (7.0, 5.0),
            Ek::Pod => (9.0, 9.0),
            Ek::Rock { big: true } => (11.0, 10.0),
            Ek::Rock { big: false } => (5.0, 4.0),
            Ek::Turret { .. } => (7.0, 5.0),
            Ek::Generator { .. } => (13.0, 8.0),
            Ek::Gunship { .. } => (16.0, 8.0),
        }
    }
    fn points(&self) -> u32 {
        match self.kind {
            Ek::Wisp => 100,
            Ek::Dart => 250,
            Ek::Hornet => 300,
            Ek::Pod => 500,
            Ek::Rock { big: true } => 200,
            Ek::Rock { big: false } => 50,
            Ek::Turret { .. } => 750,
            Ek::Generator { .. } => 3000,
            Ek::Gunship { .. } => 6000,
        }
    }
}

#[derive(Clone, Debug)]
struct Shot {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    big: bool,
}

#[derive(Clone, Debug)]
struct Pickup {
    x: f32,
    y: f32,
    kind: PowerUp,
    t: u32,
}

const CAP_LEN: f32 = 1500.0;

struct Capital {
    x: f32,
    generators: u8,
    burning: Vec<f32>,
    crippled: bool,
}

fn cap_top(u: f32) -> Option<f32> {
    if !(0.0..CAP_LEN).contains(&u) {
        return None;
    }
    let base = 128.0;
    let t = if u < 200.0 {
        196.0 - u * (196.0 - base) / 200.0
    } else if (560.0..700.0).contains(&u) {
        98.0
    } else if (540.0..560.0).contains(&u) {
        base - (u - 540.0) * 1.5
    } else if (960.0..1040.0).contains(&u) {
        108.0
    } else if (1280.0..1400.0).contains(&u) {
        if u < 1300.0 {
            base - (u - 1280.0) * 2.0
        } else {
            86.0
        }
    } else if u > CAP_LEN - 30.0 {
        base + 6.0
    } else {
        base
    };
    Some(t)
}

#[derive(Clone, Copy, Debug)]
struct BossPart {
    hp: i32,
    max: i32,
    flash: u8,
}

struct Boss {
    x: f32,
    y: f32,
    t: u32,
    parts: [BossPart; 3],
    dying: u32,
    spin: f32,
    style: u8,
    last: bool,
}

impl Boss {
    fn cannon_pos(&self, i: usize) -> (f32, f32) {
        if i == 0 {
            (self.x - 34.0, self.y - 25.0)
        } else {
            (self.x - 34.0, self.y + 25.0)
        }
    }
    fn core_pos(&self) -> (f32, f32) {
        (self.x - 22.0, self.y)
    }
    fn core_open(&self) -> bool {
        self.parts[0].hp <= 0 && self.parts[1].hp <= 0
    }
    fn hp(&self) -> (i32, i32) {
        let hp: i32 = self.parts.iter().map(|p| p.hp.max(0)).sum();
        let max: i32 = self.parts.iter().map(|p| p.max).sum();
        (hp, max)
    }
}

pub struct Side {
    t: u32,
    agg: f32,
    diff: Difficulty,
    rng: Rng,
    fx: Fx,
    stars: Stars,
    px: f32,
    py: f32,
    inv: u32,
    dead: u32,
    fire_cd: u32,
    bomb_cd: u32,
    shots: Vec<Shot>,
    eshots: Vec<Shot>,
    enemies: Vec<Enemy>,
    pickups: Vec<Pickup>,
    script: usize,
    groups: Vec<(u16, u8, u8)>,
    next_group: u16,
    rocks_until: u32,
    capital: Option<Capital>,
    boss: Option<Boss>,
    boss_started: bool,
    clear_t: Option<u32>,
    over_t: Option<u32>,
    banner: Option<(&'static str, u32)>,
    planet_x: f32,
    thrust: f32,
    spec: SideSpec,
    st: u32,
    dist: f32,
    tunnel: Option<(f32, f32)>,
    gantries: Vec<Gantry>,
}

impl Side {
    pub fn new(diff: Difficulty, seed: u32, sector: usize, spec: SideSpec) -> Side {
        let mut fx = Fx::new(seed ^ 0x51de);
        fx.gravity = 0.0;
        Side {
            t: 0,
            agg: diff.aggression() * (0.8 + sector as f32 * 0.07),
            diff,
            rng: Rng::new(seed),
            fx,
            stars: Stars::new(110, seed ^ 77),
            px: -20.0,
            py: 105.0,
            inv: 150,
            dead: 0,
            fire_cd: 0,
            bomb_cd: 0,
            shots: Vec::new(),
            eshots: Vec::new(),
            enemies: Vec::new(),
            pickups: Vec::new(),
            script: 0,
            groups: Vec::new(),
            next_group: 1,
            rocks_until: 0,
            capital: None,
            boss: None,
            boss_started: false,
            clear_t: None,
            over_t: None,
            banner: None,
            planet_x: 250.0,
            thrust: 0.0,
            spec,
            st: 0,
            dist: 0.0,
            tunnel: None,
            gantries: Vec::new(),
        }
    }

    fn spawn(&mut self, ev: Ev) {
        match ev {
            Ev::Wisps { y, n, amp, fire } => {
                let g = self.next_group;
                self.next_group += 1;
                self.groups.push((g, n, 0));
                for i in 0..n {
                    let mut e = Enemy::new(Ek::Wisp, W as f32 + 12.0 + i as f32 * 20.0, y, 1);
                    e.base = y;
                    e.vx = -1.5;
                    e.vy = amp;
                    e.t = i as u32 * 12;
                    e.group = g;
                    e.fire = fire && self.diff != Difficulty::Easy;
                    e.cd = 60 + self.rng.below(120) as i32;
                    self.enemies.push(e);
                }
            }
            Ev::Darts { n } => {
                for i in 0..n {
                    let y = self.rng.range(TOP + 20.0, BOTTOM - 20.0);
                    let mut e = Enemy::new(Ek::Dart, W as f32 + 10.0 + i as f32 * 36.0, y, 2);
                    e.vx = -2.2;
                    self.enemies.push(e);
                }
            }
            Ev::Hornets { top, n } => {
                for i in 0..n {
                    let y = if top { TOP - 14.0 } else { BOTTOM + 14.0 };
                    let mut e = Enemy::new(Ek::Hornet, 250.0 - i as f32 * 26.0, y, 2);
                    e.vy = if top { 1.6 } else { -1.6 };
                    e.vx = -0.3;
                    e.t = 0;
                    e.cd = 40 + i as i32 * 25;
                    e.fire = true;
                    self.enemies.push(e);
                }
            }
            Ev::Pod { y, drop } => {
                let mut e = Enemy::new(Ek::Pod, W as f32 + 14.0, y, 8);
                e.vx = -0.6;
                e.drop = Some(drop);
                self.enemies.push(e);
            }
            Ev::Rocks { frames } => self.rocks_until = self.st + frames,
            Ev::Capital => {
                let x = W as f32 + 10.0;
                for u in [
                    240.0, 380.0, 610.0, 660.0, 760.0, 900.0, 1000.0, 1130.0, 1340.0, 1450.0,
                ] {
                    let top = cap_top(u).unwrap_or(128.0);
                    let mut e = Enemy::new(Ek::Turret { u }, x + u, top - 4.0, 5);
                    e.cd = 60 + self.rng.below(100) as i32;
                    self.enemies.push(e);
                }
                for u in [470.0, 850.0, 1220.0] {
                    let top = cap_top(u).unwrap_or(128.0);
                    self.enemies
                        .push(Enemy::new(Ek::Generator { u }, x + u, top - 7.0, 22));
                }
                self.capital = Some(Capital {
                    x,
                    generators: 3,
                    burning: Vec::new(),
                    crippled: false,
                });
            }
            Ev::Boss(style) | Ev::MidBoss(style) => {
                self.boss_started = true;
                let bs = &STYLES[(style as usize).min(STYLES.len() - 1)];
                let part = |hp: i32| BossPart {
                    hp,
                    max: hp,
                    flash: 0,
                };
                self.boss = Some(Boss {
                    x: W as f32 + 70.0,
                    y: 105.0,
                    t: 0,
                    parts: [part(bs.cannon), part(bs.cannon), part(bs.core)],
                    dying: 0,
                    spin: 0.0,
                    style,
                    last: matches!(ev, Ev::Boss(_)),
                });
            }
            Ev::Gunship(v) => {
                let mut e = Enemy::new(
                    Ek::Gunship { v },
                    W as f32 + 30.0,
                    100.0,
                    45 + v as i32 * 10,
                );
                e.base = 100.0;
                e.cd = 90;
                self.banner = Some(("GUNSHIP INCOMING", self.t));
                self.enemies.push(e);
            }
            Ev::Gantries { n, gap } => {
                for k in 0..n {
                    let gap_y = self
                        .rng
                        .range(TOP + gap / 2.0 + 6.0, BOTTOM - gap / 2.0 - 6.0);
                    self.gantries.push(Gantry {
                        x: W as f32 + 10.0 + k as f32 * 110.0,
                        gap_y,
                        gap,
                    });
                }
            }
            Ev::Tunnel { frames } => {
                let start = self.dist + W as f32 + 20.0;
                self.tunnel = Some((start, start + frames as f32 * self.spec.speed));
            }
            Ev::Text(s) => self.banner = Some((s, self.t)),
        }
    }

    fn aimed(&mut self, x: f32, y: f32, speed: f32, spread: f32) {
        let a = (self.py - y).atan2(self.px - x) + spread;
        let sp = speed * (0.75 + 0.25 * self.agg);
        if self.eshots.len() < 120 {
            self.eshots.push(Shot {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp,
                big: false,
            });
        }
    }

    fn ring(&mut self, x: f32, y: f32, n: usize, speed: f32, rot: f32) {
        for i in 0..n {
            let a = rot + i as f32 * std::f32::consts::TAU / n as f32;
            if self.eshots.len() < 160 {
                self.eshots.push(Shot {
                    x,
                    y,
                    vx: a.cos() * speed,
                    vy: a.sin() * speed,
                    big: true,
                });
            }
        }
    }

    fn player_fire(&mut self, st: &Stats, out: &mut Out) {
        let (x, y) = (self.px + 12.0, self.py);
        let mut add = |dy: f32, vy: f32, big: bool| {
            self.shots.push(Shot {
                x,
                y: y + dy,
                vx: 6.5,
                vy,
                big,
            })
        };
        match st.weapon {
            1 => add(0.0, 0.0, false),
            2 => {
                add(-3.0, 0.0, false);
                add(3.0, 0.0, false);
            }
            3 => {
                add(-3.0, 0.0, false);
                add(3.0, 0.0, false);
                add(-2.0, -0.9, false);
                add(2.0, 0.9, false);
            }
            _ => {
                add(0.0, 0.0, true);
                add(-5.0, 0.0, false);
                add(5.0, 0.0, false);
                add(-2.0, -1.1, false);
                add(2.0, 1.1, false);
            }
        }
        out.play(if st.weapon >= 4 {
            Sfx::Laser2
        } else {
            Sfx::Laser
        });
        self.fire_cd = match st.weapon {
            1 => 9,
            2 | 3 => 8,
            _ => 6,
        };
    }

    fn kill_player(&mut self, st: &mut Stats, out: &mut Out) {
        if self.inv > 0 || self.dead > 0 || self.clear_t.is_some() {
            return;
        }
        if st.shield > 0 {
            st.shield -= 1;
            self.inv = 60;
            self.fx.sparks(self.px, self.py, 14, 0.0);
            self.fx.shake = 3.0;
            out.play(Sfx::Clang);
            return;
        }
        self.fx.explode(self.px, self.py, 3.0);
        self.fx.flash = 3;
        out.play(Sfx::PlayerDie);
        st.lives -= 1;
        st.deaths_this_level += 1;
        st.weapon = st.weapon.saturating_sub(1).max(1);
        self.dead = 1;
        self.eshots.clear();
    }

    fn damage(&mut self, i: usize, dmg: i32, st: &mut Stats, out: &mut Out) {
        let e = &mut self.enemies[i];
        e.hp -= dmg;
        e.flash = 4;
        if e.hp > 0 {
            out.play(Sfx::Hit);
            return;
        }
        let (x, y) = (e.x, e.y);
        let size = match e.kind {
            Ek::Generator { .. } => 4.0,
            Ek::Pod | Ek::Rock { big: true } | Ek::Turret { .. } => 2.0,
            _ => 1.2,
        };
        let pts = e.points();
        let kind = e.kind;
        let drop = e.drop;
        let group = e.group;
        self.fx.explode(x, y, size);
        out.play(if size >= 2.0 {
            Sfx::ExplodeBig
        } else {
            Sfx::ExplodeSmall
        });
        if st.add(pts) {
            out.play(Sfx::ExtraLife);
            self.fx.text(x, y - 12.0, "1UP".into(), LGREEN);
        }
        st.kills += 1;
        self.fx.text(x, y - 4.0, pts.to_string(), YELLOW);
        if let Some(d) = drop {
            self.pickups.push(Pickup {
                x,
                y,
                kind: d,
                t: 0,
            });
        }
        if let Ek::Rock { big: true } = kind {
            for k in 0..2 {
                let mut r = Enemy::new(Ek::Rock { big: false }, x, y, 2);
                r.vx = -1.2 - self.rng.f();
                r.vy = if k == 0 { -0.7 } else { 0.7 };
                self.enemies.push(r);
            }
        }
        if let Ek::Generator { .. } = kind {
            self.fx.flash = 4;
            if let Some(c) = self.capital.as_mut() {
                c.generators = c.generators.saturating_sub(1);
                c.burning.push(x - c.x);
                if c.generators == 0 && !c.crippled {
                    c.crippled = true;
                    st.add(10000);
                    self.banner = Some(("CAPITAL SHIP CRIPPLED  +10000", self.t));
                    out.play(Sfx::ExplodeBig);
                }
            }
        }
        if group > 0 {
            if let Some(g) = self.groups.iter_mut().find(|g| g.0 == group) {
                g.2 += 1;
                if g.2 == g.1 {
                    let kind = if st.weapon < 4 {
                        PowerUp::Weapon
                    } else {
                        PowerUp::Points
                    };
                    self.pickups.push(Pickup { x, y, kind, t: 0 });
                    self.fx.text(x, y - 14.0, "WAVE BONUS".into(), LGREEN);
                }
            }
        }
        self.enemies[i].hp = -999;
    }

    fn bomb(&mut self, st: &mut Stats, out: &mut Out) {
        st.bombs -= 1;
        self.bomb_cd = 50;
        self.fx.flash = 8;
        self.fx.shake = 7.0;
        out.play(Sfx::Bomb);
        for s in self.eshots.drain(..) {
            self.fx.sparks(s.x, s.y, 2, 0.0);
        }
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if e.x > -10.0 && e.x < W as f32 + 10.0 && e.hp > 0 {
                self.damage(i, 12, st, out);
            }
        }
        if let Some(b) = self.boss.as_mut() {
            for p in b.parts.iter_mut().take(2) {
                p.hp -= 8;
                p.flash = 6;
            }
            if b.core_open() {
                b.parts[2].hp -= 8;
            }
        }
    }

    fn update_boss(&mut self, st: &mut Stats, out: &mut Out) {
        let Some(mut b) = self.boss.take() else {
            return;
        };
        b.t += 1;
        let agg = self.agg;
        if b.dying > 0 {
            b.dying += 1;
            if b.dying % 6 == 0 {
                let (x, y) = (b.x + self.rng.s() * 45.0, b.y + self.rng.s() * 35.0);
                self.fx.explode(x, y, 2.5);
                out.play(Sfx::ExplodeBig);
            }
            b.x += 0.3;
            b.y += 0.15;
            if b.dying == 150 {
                self.fx.explode(b.x, b.y, 8.0);
                self.fx.flash = 10;
                self.fx.shake = 9.0;
                let pts = if b.last { 25000 } else { 12000 };
                st.add(pts);
                self.fx.text(b.x - 30.0, b.y, pts.to_string(), YELLOW);
                if b.last {
                    self.clear_t = Some(self.t);
                } else {
                    out.music = Some(self.spec.song);
                    self.pickups.push(Pickup {
                        x: b.x - 30.0,
                        y: b.y,
                        kind: PowerUp::Weapon,
                        t: 0,
                    });
                    self.pickups.push(Pickup {
                        x: b.x - 10.0,
                        y: b.y + 20.0,
                        kind: PowerUp::Bomb,
                        t: 0,
                    });
                }
                return;
            }
            self.boss = Some(b);
            return;
        }
        // Enter, then hover.
        if b.x > 248.0 {
            b.x -= 0.7;
        } else {
            let k = b.t as f32 * 0.018;
            b.y = 105.0 + k.sin() * 48.0;
            b.x = 248.0 + (k * 0.5).cos() * 10.0;
        }
        for p in b.parts.iter_mut() {
            p.flash = p.flash.saturating_sub(1);
        }
        let entered = b.x <= 250.0;
        let pattern = STYLES[(b.style as usize).min(STYLES.len() - 1)].pattern;
        if entered && pattern >= 1 && !b.core_open() && b.t % ((220.0 / agg) as u32).max(60) == 110
        {
            // Heavier styles add a fan from the hull.
            for k in -2..=2 {
                self.aimed(b.x - 40.0, b.y, 1.7, k as f32 * 0.18);
            }
        }
        if entered && pattern == 2 && b.core_open() {
            b.spin += 0.23;
            if b.t % 5 == 0 {
                let (cx, cy) = b.core_pos();
                let a = b.spin;
                if self.eshots.len() < 160 {
                    self.eshots.push(Shot {
                        x: cx,
                        y: cy,
                        vx: a.cos() * 1.4,
                        vy: a.sin() * 1.4,
                        big: false,
                    });
                }
            }
        }
        if entered {
            if !b.core_open() {
                let period = (80.0 / agg) as u32;
                for i in 0..2 {
                    if b.parts[i].hp > 0 && (b.t + i as u32 * period / 2) % period == 0 {
                        let (cx, cy) = b.cannon_pos(i);
                        let spread = if self.diff == Difficulty::Easy {
                            0.0
                        } else {
                            0.25
                        };
                        self.aimed(cx - 12.0, cy, 2.2, 0.0);
                        if spread > 0.0 {
                            self.aimed(cx - 12.0, cy, 2.2, spread);
                            self.aimed(cx - 12.0, cy, 2.2, -spread);
                        }
                        out.play(Sfx::EnemyShot);
                    }
                }
                if b.t % 300 == 150 {
                    for k in 0..2 {
                        let mut e =
                            Enemy::new(Ek::Dart, b.x, b.y + if k == 0 { -40.0 } else { 40.0 }, 2);
                        e.vx = -2.4;
                        self.enemies.push(e);
                    }
                }
            } else {
                b.spin += 0.11;
                let period = (95.0 / agg) as u32;
                let (cx, cy) = b.core_pos();
                if b.t % period == 0 {
                    let n = if self.diff == Difficulty::Hard {
                        14
                    } else {
                        10
                    };
                    self.ring(cx, cy, n, 1.5 * (0.8 + 0.2 * agg), b.spin);
                    out.play(Sfx::EnemyShot);
                }
                if b.t % (period / 4).max(6) == 0 && (b.t / 120) % 2 == 1 {
                    self.aimed(cx, cy, 2.4, 0.0);
                }
            }
        }
        self.boss = Some(b);
    }

    fn hit_boss(&mut self, x: f32, y: f32, st: &mut Stats, out: &mut Out) -> bool {
        let Some(b) = self.boss.as_mut() else {
            return false;
        };
        if b.dying > 0 {
            return false;
        }
        for i in 0..2 {
            let (cx, cy) = b.cannon_pos(i);
            if b.parts[i].hp > 0 && (x - cx).abs() < 12.0 && (y - cy).abs() < 7.0 {
                b.parts[i].hp -= 1;
                b.parts[i].flash = 3;
                if b.parts[i].hp <= 0 {
                    self.fx.explode(cx, cy, 3.0);
                    out.play(Sfx::ExplodeBig);
                    st.add(5000);
                    self.fx.text(cx, cy - 8.0, "5000".into(), YELLOW);
                } else {
                    out.play(Sfx::Hit);
                }
                return true;
            }
        }
        let (cx, cy) = b.core_pos();
        if b.core_open() && d2(x, y, cx, cy) < 110.0 {
            b.parts[2].hp -= 1;
            b.parts[2].flash = 3;
            out.play(Sfx::Hit);
            if b.parts[2].hp <= 0 {
                b.dying = 1;
                self.eshots.clear();
                out.play(Sfx::ExplodeBig);
            }
            return true;
        }
        // The hull: the nose in front of the core only blocks while the
        // core is shut.
        let dy = (y - b.y).abs();
        let hull = (x > b.x - 8.0 && x < b.x + 52.0 && dy < 36.0)
            || (x > b.x - 44.0 && dy < 32.0 + (x - b.x) * 0.1 && (dy > 12.0 || !b.core_open()));
        if hull {
            self.fx.sparks(x, y, 2, std::f32::consts::PI);
            if self.t % 4 == 0 {
                out.play(Sfx::Clang);
            }
            return true;
        }
        false
    }

    /// The rock walls of a tunnel at a screen column: (top, bottom).
    fn tunnel_at(&self, sx: f32) -> Option<(f32, f32)> {
        let (a, b) = self.tunnel?;
        let xw = self.dist + sx;
        if xw < a || xw > b {
            return None;
        }
        let env = ((xw - a).min(b - xw) / 160.0).clamp(0.0, 1.0);
        let top =
            TOP - 8.0 + env * (26.0 + 18.0 * (xw * 0.011).sin() + 9.0 * (xw * 0.037 + 1.0).sin());
        let mut bot = BOTTOM + 8.0
            - env * (26.0 + 18.0 * (xw * 0.009 + 2.0).sin() + 9.0 * (xw * 0.029).sin());
        if bot - top < 70.0 {
            bot = top + 70.0;
        }
        Some((top, bot))
    }

    /// Is there rock or steel at this point?
    fn solid_at(&self, x: f32, y: f32) -> bool {
        if let Some((t, b)) = self.tunnel_at(x) {
            if y < t || y > b {
                return true;
            }
        }
        self.gantries
            .iter()
            .any(|g| (x - g.x - 5.0).abs() < 6.0 && (y - g.gap_y).abs() > g.gap / 2.0)
    }

    fn draw_world(&self, fb: &mut Fb, ox: i32, oy: i32) {
        if self.tunnel.is_some() {
            for sx in 0..W {
                let Some((t, b)) = self.tunnel_at((sx - ox) as f32) else {
                    continue;
                };
                let xw = (self.dist as i32 + sx - ox).max(0);
                let (t, b) = (t as i32 + oy, b as i32 + oy);
                for y in HUD_H..H {
                    let d = if y < t {
                        t - y
                    } else if y > b {
                        y - b
                    } else {
                        continue;
                    };
                    let n = hash(xw / 3, y / 3) % 7;
                    let c = if d < 2 {
                        LGREY
                    } else if d < 5 {
                        if n < 3 {
                            ORANGE
                        } else {
                            BROWN
                        }
                    } else if n == 0 {
                        DGREY
                    } else if (xw + y) % 11 == 0 {
                        ORANGE
                    } else {
                        BROWN
                    };
                    fb.set(sx, y, c);
                }
            }
        }
        for g in &self.gantries {
            let x = g.x as i32 + ox;
            let (g0, g1) = (
                (g.gap_y - g.gap / 2.0) as i32 + oy,
                (g.gap_y + g.gap / 2.0) as i32 + oy,
            );
            for y in HUD_H..H {
                if y >= g0 && y <= g1 {
                    continue;
                }
                fb.set(x, y, LGREY);
                fb.set(x + 1, y, GREY);
                fb.set(x + 9, y, GREY);
                fb.set(x + 10, y, DGREY);
                let k = (y - oy).rem_euclid(10);
                fb.set(x + 1 + k, y, DGREY);
                fb.set(x + 9 - k, y, DGREY);
            }
            let blink = if self.t % 20 < 10 { LRED } else { RED };
            fb.rect(x - 1, g0 - 3, 13, 3, YELLOW);
            fb.rect(x - 1, g1 + 1, 13, 3, YELLOW);
            fb.set(x + 5, g0 - 4, blink);
            fb.set(x + 5, g1 + 4, blink);
        }
    }

    fn draw_ship(&self, fb: &mut Fb) {
        if self.dead > 0 || (self.inv > 0 && (self.t / 3) % 2 == 0 && self.inv < 140) {
            return;
        }
        let a = art::get();
        let (x, y) = (self.px as i32 - 13, self.py as i32 - 6);
        // The engine flame, longer with thrust.
        let len = 3 + (self.t % 3) as i32 + self.thrust as i32 * 2;
        for i in 0..len {
            let c = [WHITE, YELLOW, YELLOW, ORANGE, LRED, RED, BROWN][(i as usize).min(6)];
            fb.set(x - i, y + 6, c);
            if i < len - 2 {
                fb.set(x - i + 1, y + 5, if i < 2 { YELLOW } else { ORANGE });
                fb.set(x - i + 1, y + 7, if i < 2 { YELLOW } else { ORANGE });
            }
        }
        fb.blit(&a.ship, x, y);
    }

    fn draw_capital(&self, fb: &mut Fb, ox: i32, oy: i32) {
        let Some(c) = &self.capital else {
            return;
        };
        let x0 = c.x as i32;
        for sx in 0..W {
            let u = (sx - ox - x0) as f32;
            let Some(top) = cap_top(u) else {
                continue;
            };
            let ui = u as i32;
            let top = top as i32 + oy;
            let tower = top < 120 + oy;
            for y in top..H {
                let r = y - top;
                let mut col = if r == 0 {
                    LGREY
                } else if r < 3 {
                    GREY
                } else if r < 5 {
                    DGREY
                } else {
                    let band = (y - oy) / 12;
                    if band % 2 == 0 || (sx + y) & 1 == 0 {
                        DGREY
                    } else {
                        GREY
                    }
                };
                if r > 4 {
                    if ui.rem_euclid(48) == 0 || (y - oy) % 12 == 0 {
                        col = BLACK;
                    } else if ui.rem_euclid(48) == 24 && (y - oy) % 12 == 6 {
                        col = LGREY;
                    }
                    // Windows on the towers and along the hull.
                    if tower && r > 6 && ui % 6 < 2 && (y - oy) % 8 == 3 {
                        let on = hash(ui / 6, (y - oy) / 8) % 5 != 0;
                        col = if on {
                            if (hash(ui, y) + self.t / 40) % 17 == 0 {
                                WHITE
                            } else {
                                YELLOW
                            }
                        } else {
                            BLACK
                        };
                    }
                    if !tower && (y - oy) == 160 && ui % 10 < 6 {
                        col = if (ui / 10 + self.t as i32 / 8) % 6 == 0 {
                            LRED
                        } else {
                            RED
                        };
                    }
                    if (y - oy) > 176 && (y - oy) < 182 && ui % 40 < 30 {
                        col = if ((ui / 4) + (y - oy)) % 2 == 0 {
                            YELLOW
                        } else {
                            BLACK
                        };
                    }
                }
                // Scorched hull around destroyed generators.
                for &bu in &c.burning {
                    let dx = (u - bu).abs();
                    if dx < 40.0 && r > 1 && r < 30 && hash(ui, y) % 3 == 0 && dx < 40.0 - r as f32
                    {
                        col = if hash(ui, y) % 7 == 0 { RED } else { BLACK };
                    }
                }
                fb.set(sx, y, col);
            }
        }
        // Engines at the stern.
        let ex = x0 + CAP_LEN as i32 + ox;
        if ex > -40 && ex < W + 40 {
            for k in 0..4 {
                let ey = 142 + k * 14 + oy;
                fb.rect(ex - 4, ey - 4, 6, 9, GREY);
                let len = 10 + ((self.t as i32 + k * 3) % 5) * 2;
                for i in 0..len {
                    let c = [WHITE, CYAN, LBLUE, LBLUE, BLUE, BLUE][((i * 6) / len) as usize];
                    fb.hline(ex + 2 + i, ex + 2 + i, ey - 2 + (i % 2), c);
                    fb.set(ex + 2 + i, ey + 1, c);
                    if i < len / 2 {
                        fb.set(ex + 2 + i, ey, WHITE);
                    }
                }
            }
        }
    }

    fn draw_enemy(&self, fb: &mut Fb, e: &Enemy, ox: i32, oy: i32) {
        let a = art::get();
        let (x, y) = (e.x as i32 + ox, e.y as i32 + oy);
        let flash = e.flash > 0 && e.flash % 2 == 0;
        let spr = |fb: &mut Fb, s: &crate::fb::Sprite| {
            if flash {
                fb.blit_solid(s, x - s.w / 2, y - s.h / 2, WHITE);
            } else {
                fb.blit_c(s, x, y);
            }
        };
        match e.kind {
            Ek::Wisp => spr(fb, &a.wisp[((e.t / 8) % 2) as usize]),
            Ek::Dart => spr(fb, &a.dart),
            Ek::Hornet => spr(fb, &a.hornet[((e.t / 5) % 2) as usize]),
            Ek::Pod => {
                let c = if flash { WHITE } else { PURPLE };
                fb.disc(x, y, 9, c);
                fb.circle(x, y, 9, if flash { WHITE } else { LRED });
                let k = e.t as f32 * 0.08;
                for i in 0..4 {
                    let an = k + i as f32 * std::f32::consts::FRAC_PI_2;
                    fb.disc(
                        x + (an.cos() * 6.0) as i32,
                        y + (an.sin() * 6.0) as i32,
                        1,
                        LGREY,
                    );
                }
                let core = [RED, LRED, YELLOW, WHITE, YELLOW, LRED][((e.t / 4) % 6) as usize];
                fb.disc(x, y, 3, core);
            }
            Ek::Rock { big } => {
                if big {
                    let s = a.rock.doubled();
                    spr(fb, &s);
                } else {
                    spr(fb, &a.rock);
                }
            }
            Ek::Turret { .. } => {
                // A dome with a barrel that tracks the player.
                let c = if flash { WHITE } else { GREY };
                fb.disc(x, y + 2, 6, c);
                fb.rect(x - 7, y + 2, 15, 3, if flash { WHITE } else { DGREY });
                fb.hline(x - 4, x + 2, y - 2, LGREY);
                let (dx, dy) = (e.aim.cos(), e.aim.sin());
                for i in 3..11 {
                    fb.set(
                        x + (dx * i as f32) as i32,
                        y + (dy * i as f32) as i32,
                        LGREY,
                    );
                    fb.set(
                        x + (dx * i as f32) as i32,
                        y + 1 + (dy * i as f32) as i32,
                        DGREY,
                    );
                }
                if e.cd < 12 {
                    fb.set(x + (dx * 11.0) as i32, y + (dy * 11.0) as i32, YELLOW);
                }
            }
            Ek::Gunship { v } => {
                let pal: [(u8, u8); 4] = match v {
                    0 => [(2, 6), (10, 14), (12, 11), (15, 12)],
                    1 => [(2, 8), (10, 7), (12, 9), (15, 8)],
                    _ => [(2, 4), (10, 10), (12, 6), (15, 14)],
                };
                let s = a.dart.doubled().recolor(&pal);
                spr(fb, &s);
                let glow = [YELLOW, ORANGE, WHITE][(e.t % 3) as usize];
                fb.rect(x + 15, y - 1, 3, 3, glow);
            }
            Ek::Generator { .. } => {
                let c = if flash { WHITE } else { DGREY };
                fb.rect(x - 14, y + 2, 29, 7, c);
                fb.hline(x - 14, x + 14, y + 2, LGREY);
                let glow = [BLUE, LBLUE, CYAN, WHITE, CYAN, LBLUE][((e.t / 5) % 6) as usize];
                fb.disc(x, y, 8, if flash { WHITE } else { BLUE });
                fb.disc(x, y, 5, glow);
                fb.disc(x - 2, y - 2, 1, WHITE);
                // Energy arcs.
                if e.t % 20 < 10 {
                    fb.line(x - 12, y + 1, x - 6, y - 3, CYAN);
                    fb.line(x + 12, y + 1, x + 6, y - 3, CYAN);
                }
            }
        }
    }

    fn draw_boss(&self, fb: &mut Fb, ox: i32, oy: i32) {
        let Some(b) = &self.boss else {
            return;
        };
        let (x, y) = (b.x + ox as f32, b.y + oy as f32);
        let dying = b.dying > 0 && b.dying % 4 < 2;
        let cols = STYLES[(b.style as usize).min(STYLES.len() - 1)].cols;
        let body = if dying { RED } else { cols[0] };
        let pts = [
            (x - 46.0, y),
            (x - 20.0, y - 34.0),
            (x + 44.0, y - 40.0),
            (x + 56.0, y),
            (x + 44.0, y + 40.0),
            (x - 20.0, y + 34.0),
        ];
        fb.poly(&pts, body);
        let inner = [
            (x - 34.0, y),
            (x - 14.0, y - 24.0),
            (x + 36.0, y - 30.0),
            (x + 44.0, y),
            (x + 36.0, y + 30.0),
            (x - 14.0, y + 24.0),
        ];
        fb.poly(&inner, if dying { LRED } else { cols[1] });
        for i in 0..6 {
            let (a, c) = (pts[i], pts[(i + 1) % 6]);
            fb.line(a.0 as i32, a.1 as i32, c.0 as i32, c.1 as i32, cols[2]);
        }
        // Armour plates and lights.
        for k in 0..5 {
            let yy = y as i32 - 20 + k * 10;
            fb.hline(x as i32 - 6, x as i32 + 34, yy, DGREY);
            if (b.t / 10 + k as u32) % 5 == 0 {
                fb.set(x as i32 + 30, yy - 3, LRED);
            }
        }
        fb.rect(x as i32 + 8, y as i32 - 12, 20, 24, DGREY);
        fb.frame(x as i32 + 8, y as i32 - 12, 20, 24, LGREY);
        // Engines.
        for k in [-26, -8, 8, 26] {
            let len = 6 + ((b.t as i32 + k) % 4);
            for i in 0..len {
                let c = [
                    WHITE, YELLOW, ORANGE, RED, BROWN, BROWN, BROWN, BROWN, BROWN, BROWN,
                ][i as usize];
                fb.set(x as i32 + 52 + i, y as i32 + k, c);
                fb.set(x as i32 + 52 + i, y as i32 + k + 1, c);
            }
        }
        let a = art::get();
        for i in 0..2 {
            let p = b.parts[i];
            let (cx, cy) = b.cannon_pos(i);
            let (cx, cy) = (cx as i32 + ox, cy as i32 + oy);
            if p.hp > 0 {
                if p.flash > 0 && p.flash % 2 == 1 {
                    fb.blit_solid(&a.cannon, cx - 8, cy - 3, WHITE);
                } else {
                    fb.blit(&a.cannon, cx - 8, cy - 3);
                }
                fb.rect(cx - 18, cy - 1, 10, 3, GREY);
                fb.hline(cx - 18, cx - 9, cy - 1, LGREY);
            } else {
                fb.rect(cx - 6, cy - 2, 12, 5, BLACK);
                if b.t % 6 < 3 {
                    fb.set(cx + (b.t % 5) as i32 - 2, cy, ORANGE);
                }
            }
        }
        let (cx, cy) = b.core_pos();
        let (cx, cy) = (cx as i32 + ox, cy as i32 + oy);
        if b.core_open() {
            let p = b.parts[2];
            let c = if p.flash > 0 {
                WHITE
            } else {
                [RED, LRED, ORANGE, YELLOW, WHITE, YELLOW, ORANGE, LRED][((b.t / 3) % 8) as usize]
            };
            fb.disc(cx, cy, 10, BLACK);
            fb.disc(cx, cy, 8, RED);
            fb.disc(cx, cy, 6, c);
            fb.disc(cx - 2, cy - 2, 2, WHITE);
        } else {
            fb.disc(cx, cy, 9, GREY);
            fb.hline(cx - 9, cx + 9, cy, DGREY);
            fb.line(cx, cy - 9, cx, cy + 9, DGREY);
            fb.circle(cx, cy, 9, LGREY);
        }
    }
}

impl Level for Side {
    fn update(&mut self, inp: &Input, st: &mut Stats, out: &mut Out) -> Status {
        self.t += 1;
        if self.t == 1 {
            out.music = Some(self.spec.song);
        }
        // The script waits while a gunship or a mid-boss is in the way.
        let hold = self
            .enemies
            .iter()
            .any(|e| matches!(e.kind, Ek::Gunship { .. }))
            || self.boss.as_ref().is_some_and(|b| !b.last);
        if !hold {
            self.st += 1;
        }
        while self.script < self.spec.script.len() && self.spec.script[self.script].0 <= self.st {
            let ev = self.spec.script[self.script].1;
            self.script += 1;
            self.spawn(ev);
        }
        if self.boss_started && out.music.is_none() {
            if let Some(b) = &self.boss {
                if b.t == 1 {
                    out.music = Some(self.spec.boss_song);
                    out.play(Sfx::Alarm);
                }
            }
        }
        let spd = self.spec.speed;
        self.dist += spd;
        for g in self.gantries.iter_mut() {
            g.x -= spd;
        }
        self.gantries.retain(|g| g.x > -20.0);
        if self.tunnel.is_some_and(|(_, e)| self.dist > e + 60.0) {
            self.tunnel = None;
        }
        // Rocks drift in during the asteroid field.
        if self.st < self.rocks_until && self.t % 26 == 0 && !hold {
            let big = self.rng.chance(0.35);
            let y = self.rng.range(TOP + 10.0, BOTTOM - 10.0);
            let mut r = Enemy::new(
                Ek::Rock { big },
                W as f32 + 20.0,
                y,
                if big { 6 } else { 2 },
            );
            r.vx = -self.rng.range(0.8, 1.8);
            r.vy = self.rng.s() * 0.4;
            self.enemies.push(r);
        }
        self.stars
            .update(if self.boss.is_some() { spd + 0.4 } else { spd }, 0.0);
        self.planet_x -= 0.04;
        // The player.
        if self.dead > 0 {
            self.dead += 1;
            if self.dead > 100 {
                if st.lives <= 0 {
                    if self.over_t.is_none() {
                        self.over_t = Some(self.t);
                    }
                } else {
                    self.dead = 0;
                    self.inv = 150;
                    self.px = -20.0;
                    self.py = 105.0;
                }
            }
        } else if self.px < 40.0 && self.inv > 120 {
            self.px += 1.5;
            self.thrust = 1.0;
        } else {
            let sp = 1.7;
            let (dx, dy) = (inp.dx(), inp.dy());
            let k = if dx != 0.0 && dy != 0.0 {
                std::f32::consts::FRAC_1_SQRT_2
            } else {
                1.0
            };
            self.px = (self.px + dx * sp * k).clamp(14.0, W as f32 - 20.0);
            self.py = (self.py + dy * sp * k).clamp(TOP, BOTTOM);
            self.thrust = if dx > 0.0 {
                2.0
            } else if dx < 0.0 {
                0.0
            } else {
                1.0
            };
            if self.fire_cd > 0 {
                self.fire_cd -= 1;
            }
            if inp.fire && self.fire_cd == 0 && self.clear_t.is_none() {
                self.player_fire(st, out);
            }
            if self.bomb_cd > 0 {
                self.bomb_cd -= 1;
            }
            if inp.p_special && st.bombs > 0 && self.bomb_cd == 0 {
                self.bomb(st, out);
            }
        }
        self.inv = self.inv.saturating_sub(1);
        // Capital ship.
        if let Some(c) = self.capital.as_mut() {
            c.x -= 1.0;
            for &bu in &c.burning {
                let x = c.x + bu + self.rng.s() * 30.0;
                if x > -10.0 && x < W as f32 + 10.0 && self.rng.chance(0.25) {
                    let y = cap_top(bu).unwrap_or(128.0) + self.rng.range(-2.0, 10.0);
                    if self.rng.chance(0.3) {
                        self.fx.explode(x, y, 0.7);
                    } else {
                        self.fx.smoke(x, y, 2.0);
                    }
                }
            }
            if c.crippled && c.x + CAP_LEN > 0.0 && self.t % 7 == 0 {
                let x = self.rng.range(0.0, W as f32);
                let u = x - c.x;
                if let Some(top) = cap_top(u) {
                    self.fx.explode(x, top + self.rng.range(0.0, 40.0), 1.5);
                    if self.t % 21 == 0 {
                        out.play(Sfx::ExplodeSmall);
                    }
                }
            }
            if c.x + CAP_LEN < -60.0 {
                self.capital = None;
            }
        }
        // Enemies.
        let (px, py) = (self.px, self.py);
        let mut fire_from: Vec<(f32, f32, f32)> = Vec::new();
        let cap_x = self.capital.as_ref().map(|c| c.x);
        for e in self.enemies.iter_mut() {
            e.t += 1;
            e.flash = e.flash.saturating_sub(1);
            match e.kind {
                Ek::Wisp => {
                    e.x += e.vx;
                    e.y = e.base + (e.t as f32 * 0.045).sin() * e.vy;
                }
                Ek::Dart => {
                    e.x += e.vx;
                    if e.x > px + 30.0 {
                        e.y += (py - e.y).clamp(-0.7, 0.7);
                    }
                }
                Ek::Hornet => {
                    // Swoop in, track the player's height, then leave.
                    if e.t < 50 {
                        e.y += e.vy;
                    } else if e.t < 170 {
                        e.y += (py - e.y) * 0.025;
                        e.x += -0.5;
                    } else {
                        e.x += 1.8;
                        e.y += e.vy * 0.5;
                    }
                }
                Ek::Pod => {
                    e.x += e.vx;
                    e.y += (e.t as f32 * 0.03).sin() * 0.4;
                }
                Ek::Gunship { .. } => {
                    if e.x > 250.0 && e.t < 200 {
                        e.x -= 0.8;
                    } else {
                        let k = e.t as f32;
                        e.y = 105.0 + (k * 0.021).sin() * 55.0;
                        e.x = 245.0 + (k * 0.013).cos() * 25.0;
                    }
                    e.cd -= 1;
                    if e.cd <= 0 {
                        e.cd = (75.0 / self.agg) as i32;
                        // 20 marks a fan, 30 a ring.
                        fire_from.push((e.x - 14.0, e.y, if e.t % 4 == 0 { 30.0 } else { 20.0 }));
                    }
                }
                Ek::Rock { .. } => {
                    e.x += e.vx;
                    e.y += e.vy;
                    if e.y < TOP || e.y > BOTTOM {
                        e.vy = -e.vy;
                    }
                }
                Ek::Turret { u } | Ek::Generator { u } => {
                    if let Some(cx) = cap_x {
                        e.x = cx + u;
                    } else {
                        e.hp = -999;
                    }
                    if let Ek::Turret { .. } = e.kind {
                        let want = (py - e.y).atan2(px - e.x);
                        let want = if want > 0.2 {
                            if px < e.x {
                                -2.9
                            } else {
                                -0.2
                            }
                        } else {
                            want
                        };
                        e.aim += (want - e.aim).clamp(-0.05, 0.05);
                    }
                }
            }
            let on = e.x > 8.0 && e.x < W as f32 - 8.0;
            let shooter = match e.kind {
                Ek::Turret { .. } => true,
                Ek::Wisp | Ek::Hornet => e.fire,
                _ => false,
            };
            if shooter && on {
                e.cd -= 1;
                if e.cd <= 0 {
                    let base = match e.kind {
                        Ek::Turret { .. } => 110.0,
                        Ek::Hornet => 90.0,
                        _ => 170.0,
                    };
                    e.cd = (base / self.agg) as i32 + self.rng.below(40) as i32;
                    if e.x > px + 20.0 || matches!(e.kind, Ek::Turret { .. }) {
                        fire_from.push((
                            e.x,
                            e.y,
                            if let Ek::Turret { .. } = e.kind {
                                e.aim
                            } else {
                                9.0
                            },
                        ));
                    }
                }
            }
        }
        for (x, y, aim) in fire_from {
            if aim > 25.0 {
                let r = self.rng.f();
                self.ring(
                    x,
                    y,
                    if self.diff == Difficulty::Hard { 12 } else { 8 },
                    1.3,
                    r,
                );
            } else if aim > 15.0 {
                let n = if self.diff == Difficulty::Easy { 1 } else { 2 };
                for k in -n..=n {
                    self.aimed(x, y, 1.9, k as f32 * 0.2);
                }
            } else if aim < 8.0 {
                let sp = 1.6 * (0.7 + 0.3 * self.agg);
                self.eshots.push(Shot {
                    x: x + aim.cos() * 11.0,
                    y: y + aim.sin() * 11.0,
                    vx: aim.cos() * sp,
                    vy: aim.sin() * sp,
                    big: false,
                });
            } else {
                self.aimed(x, y, 1.6, 0.0);
            }
            out.play(Sfx::EnemyShot);
        }
        self.update_boss(st, out);
        // Player shots.
        let mut shots = std::mem::take(&mut self.shots);
        shots.retain_mut(|s| {
            s.x += s.vx;
            s.y += s.vy;
            if s.x > W as f32 + 8.0 || s.y < TOP - 10.0 || s.y > H as f32 + 4.0 {
                return false;
            }
            if self.solid_at(s.x, s.y) {
                self.fx.sparks(s.x, s.y, 2, std::f32::consts::PI);
                return false;
            }
            for i in 0..self.enemies.len() {
                let e = &self.enemies[i];
                if e.hp <= 0 {
                    continue;
                }
                let (hw, hh) = e.half();
                if (s.x - e.x).abs() < hw + 3.0 && (s.y - e.y).abs() < hh + 1.0 {
                    let dmg = if s.big { 2 } else { 1 };
                    self.fx.sparks(s.x, s.y, 2, std::f32::consts::PI);
                    self.damage(i, dmg, st, out);
                    return false;
                }
            }
            !self.hit_boss(s.x, s.y, st, out)
        });
        self.shots = shots;
        self.enemies.retain(|e| {
            e.hp > 0
                && e.x > -60.0
                && e.x < W as f32 + 1600.0
                && e.y > -40.0
                && e.y < H as f32 + 40.0
        });
        for g in self.groups.iter_mut() {
            let _ = g;
        }
        // Enemy shots and collisions with the player.
        let alive = self.dead == 0;
        let mut hit = false;
        self.eshots.retain_mut(|s| {
            s.x += s.vx;
            s.y += s.vy;
            if alive && d2(s.x, s.y, px, py) < if s.big { 12.0 } else { 7.0 } {
                hit = true;
                return false;
            }
            s.x > -6.0 && s.x < W as f32 + 6.0 && s.y > TOP - 8.0 && s.y < H as f32 + 6.0
        });
        if alive {
            for e in &self.enemies {
                let (hw, hh) = e.half();
                let body = !matches!(e.kind, Ek::Turret { .. } | Ek::Generator { .. });
                if body && (e.x - px).abs() < hw + 4.0 && (e.y - py).abs() < hh + 2.0 {
                    hit = true;
                }
            }
            if let Some(b) = &self.boss {
                if b.dying == 0 && px > b.x - 40.0 && (py - b.y).abs() < 30.0 {
                    hit = true;
                }
            }
            if self.inv == 0
                && (self.solid_at(px + 8.0, py)
                    || self.solid_at(px - 6.0, py - 3.0)
                    || self.solid_at(px - 6.0, py + 3.0))
            {
                hit = true;
            }
        }
        if hit {
            self.kill_player(st, out);
        }
        // Pickups.
        let mut got = Vec::new();
        self.pickups.retain_mut(|p| {
            p.t += 1;
            p.x -= 0.5;
            p.y += (p.t as f32 * 0.08).sin() * 0.3;
            if alive && (p.x - px).abs() < 14.0 && (p.y - py).abs() < 10.0 {
                got.push((p.kind, p.x, p.y));
                return false;
            }
            p.x > -10.0
        });
        for (k, x, y) in got {
            let s = st.collect(k, out);
            self.fx.text(x, y - 8.0, s.to_string(), LGREEN);
            self.fx.sparks(x, y, 10, 0.0);
        }
        self.fx.update();
        if let Some(t) = self.over_t {
            if self.t - t > 30 {
                return Status::Over;
            }
        }
        if let Some(t) = self.clear_t {
            // Fly off to the right.
            if self.t - t > 60 {
                self.px += (self.t - t - 60) as f32 * 0.08;
                self.thrust = 3.0;
            }
            if self.t - t > 210 {
                return Status::Cleared;
            }
        }
        Status::Playing
    }

    fn draw(&self, fb: &mut Fb, st: &Stats, hi: u32) {
        fb.clear(BLACK);
        let (ox, oy) = self.fx.offset(self.t);
        self.stars.draw(fb, self.t);
        if let Some(pc) = self.spec.planet {
            draw_planet(fb, self.planet_x as i32, 62, 26, pc);
        }
        self.draw_capital(fb, ox, oy);
        self.draw_world(fb, ox, oy);
        for e in &self.enemies {
            if matches!(e.kind, Ek::Turret { .. } | Ek::Generator { .. }) {
                self.draw_enemy(fb, e, ox, oy);
            }
        }
        self.draw_boss(fb, ox, oy);
        for e in &self.enemies {
            if !matches!(e.kind, Ek::Turret { .. } | Ek::Generator { .. }) {
                self.draw_enemy(fb, e, ox, oy);
            }
        }
        for p in &self.pickups {
            draw_capsule(fb, p.x as i32 + ox, p.y as i32 + oy, p.kind, p.t);
        }
        for s in &self.shots {
            let (x, y) = (s.x as i32, s.y as i32);
            if s.big {
                fb.hline(x - 7, x + 2, y - 1, CYAN);
                fb.hline(x - 9, x + 3, y, WHITE);
                fb.hline(x - 7, x + 2, y + 1, CYAN);
            } else {
                fb.hline(x - 5, x + 1, y, YELLOW);
                fb.hline(x - 2, x + 1, y, WHITE);
            }
        }
        self.draw_ship(fb);
        if st.shield > 0 && self.dead == 0 {
            let r = 13 + (self.t % 8 < 4) as i32;
            for k in 0..24 {
                let a = k as f32 * 0.2618 + self.t as f32 * 0.05;
                let c = if k % 3 == 0 { WHITE } else { CYAN };
                fb.set(
                    self.px as i32 + (a.cos() * r as f32) as i32,
                    self.py as i32 + (a.sin() * r as f32 * 0.7) as i32,
                    c,
                );
            }
        }
        for s in &self.eshots {
            draw_bullet(fb, s.x as i32 + ox, s.y as i32 + oy, self.t, s.big);
        }
        self.fx.draw(fb, ox, oy);
        if let Some((s, t0)) = self.banner {
            let k = self.t - t0;
            if k < 180 && (k > 20 || k % 4 < 2) {
                fb.text_center_sh(
                    40,
                    s,
                    if k % 16 < 8 { WHITE } else { CYAN },
                    self.spec.border,
                );
            }
        }
        if self.fx.flash > 0 {
            let map = if self.fx.flash > 4 {
                [1u8; 16]
            } else {
                [1, 1, 7, 1, 1, 7, 15, 1, 7, 7, 1, 15, 1, 1, 1, 1]
            };
            fb.remap(0, HUD_H, W, H - HUD_H, &map);
        }
        draw_hud(fb, st, hi, self.spec.hud);
        if let Some(b) = &self.boss {
            let (hp, max) = b.hp();
            if b.dying == 0 {
                draw_boss_bar(fb, STYLES[(b.style as usize).min(4)].name, hp, max, self.t);
            }
        }
        for e in &self.enemies {
            if let Ek::Gunship { v } = e.kind {
                draw_boss_bar(fb, "GUNSHIP", e.hp, 45 + v as i32 * 10, self.t);
            }
        }
    }

    fn bot(&mut self, st: &Stats) -> Input {
        let mut i = Input {
            fire: true,
            ..Default::default()
        };
        let (px, py) = (self.px, self.py);
        // Aim at the nearest target ahead, keep away from bullets.
        let mut ty = 105.0;
        let mut best = f32::MAX;
        for e in &self.enemies {
            if e.x > px && e.x < W as f32 {
                let d = e.x - px;
                if d < best {
                    best = d;
                    ty = e.y;
                }
            }
        }
        if let Some(b) = &self.boss {
            ty = if b.core_open() {
                b.core_pos().1
            } else if b.parts[0].hp > 0 {
                b.cannon_pos(0).1
            } else {
                b.cannon_pos(1).1
            };
        }
        let mut dodge = 0.0;
        let mut near = 0;
        let radius = if self.boss.is_some() { 360.0 } else { 900.0 };
        for s in &self.eshots {
            let d = d2(s.x, s.y, px, py);
            if d < radius {
                near += 1;
                dodge += if s.y > py { -1.0 } else { 1.0 };
            }
        }
        for e in &self.enemies {
            if !matches!(e.kind, Ek::Turret { .. } | Ek::Generator { .. })
                && d2(e.x, e.y, px, py) < 900.0
            {
                dodge += if e.y > py { -1.5 } else { 1.5 };
            }
        }
        let mut tx = if self.boss.is_some() { 90.0 } else { 70.0 };
        if let Some(p) = self.pickups.first() {
            ty = p.y;
            tx = p.x.clamp(30.0, 200.0);
        }
        let mut want = if dodge != 0.0 { py + dodge * 20.0 } else { ty };
        // Steer through gantry gaps and along tunnels.
        if let Some(g) = self
            .gantries
            .iter()
            .find(|g| g.x > px - 12.0 && g.x < px + 90.0)
        {
            want = g.gap_y;
            tx = tx.min(60.0);
        }
        let mut lo = TOP;
        let mut hi = BOTTOM;
        for k in [-10.0f32, 10.0, 30.0, 50.0] {
            if let Some((t, b)) = self.tunnel_at(px + k) {
                lo = lo.max(t + 12.0);
                hi = hi.min(b - 12.0);
            }
        }
        if lo < hi {
            want = want.clamp(lo, hi);
        } else {
            want = (lo + hi) / 2.0;
        }
        i.up = want < py - 3.0;
        i.down = want > py + 3.0;
        i.left = px > tx + 10.0;
        i.right = px < tx - 10.0;
        i.p_special = near > 6 && st.bombs > 0 && self.t % 30 == 0;
        i
    }

    fn border(&self) -> u8 {
        if self.fx.flash > 2 {
            WHITE
        } else {
            self.spec.border
        }
    }

    fn progress(&self) -> f32 {
        let end = self.spec.script.last().map(|e| e.0).unwrap_or(1).max(1);
        (self.st as f32 / end as f32).min(1.0)
    }

    fn info(&self) -> String {
        match &self.boss {
            Some(b) => format!(
                "boss {:?} at {:.0},{:.0} dying {}",
                b.parts.iter().map(|p| p.hp).collect::<Vec<_>>(),
                b.x,
                b.y,
                b.dying
            ),
            None => format!(
                "enemies {} shots {} player {:.0},{:.0} pickups {:?}",
                self.enemies.len(),
                self.eshots.len(),
                self.px,
                self.py,
                self.pickups
                    .iter()
                    .map(|p| (p.x as i32, p.y as i32))
                    .collect::<Vec<_>>()
            ),
        }
    }
}

/// A ringed planet lit from the upper left, with dithered shading.
fn draw_planet(fb: &mut Fb, cx: i32, cy: i32, r: i32, pc: [u8; 3]) {
    let (light, mid, band_c) = (pc[0], pc[1], pc[2]);
    let ring = |fb: &mut Fb, front: bool| {
        for k in 0..360 {
            let a = (k as f32).to_radians();
            let (x, y) = (a.cos() * r as f32 * 1.75, a.sin() * r as f32 * 0.3);
            if (y > 0.0) != front {
                continue;
            }
            let (px, py) = (cx + x as i32, cy + (y + x * 0.18) as i32);
            fb.set(
                px,
                py,
                if x.abs() > r as f32 * 1.4 {
                    DGREY
                } else {
                    GREY
                },
            );
            if k % 2 == 0 {
                fb.set(px, py + 1, DGREY);
            }
        }
    };
    ring(fb, false);
    for y in -r..=r {
        for x in -r..=r {
            let d2 = x * x + y * y;
            if d2 > r * r {
                continue;
            }
            // Light from the upper left: 0 is lit, 1 is dark.
            let l = ((x + y) as f32 / (r as f32 * 1.5) + 0.35 + d2 as f32 / (r * r * 4) as f32)
                .clamp(0.0, 1.2);
            let dith = (x + y) & 1 == 0;
            let c = if l < 0.25 {
                light
            } else if l < 0.4 {
                if dith {
                    light
                } else {
                    mid
                }
            } else if l < 0.7 {
                mid
            } else if l < 0.85 {
                if dith {
                    mid
                } else {
                    BLACK
                }
            } else {
                BLACK
            };
            // Cloud bands.
            let band = ((y + r) / 5) % 3 == 1 && c == mid && (x / 3 + y) % 4 != 0;
            fb.set(cx + x, cy + y, if band { band_c } else { c });
        }
    }
    ring(fb, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capital_profile_is_continuous_enough() {
        let mut last = cap_top(0.0).unwrap();
        for u in 1..CAP_LEN as i32 {
            let t = cap_top(u as f32).unwrap();
            assert!((60.0..=200.0).contains(&t));
            last = t.min(last + 50.0);
        }
        assert!(cap_top(-1.0).is_none());
    }
}
