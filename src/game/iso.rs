//! Sector 2, "Glass Dunes": an isometric flying shooter. The ship flies up
//! the diagonal over a fortress deck that hangs above dunes of glass; up and
//! down change altitude and the shadow on the deck shows where the ship is.
//! Walls must be climbed over, energy fences flown over or under, towers
//! dodged, and at the end the Monolith and its orbiting shards wait.
use super::common::*;
use super::Level;
use crate::config::Difficulty;
use crate::fb::{Fb, H, W};
use crate::input::Input;
use crate::palette::*;
use crate::sfx::Sfx;
use crate::songs::SongId;

const OX: f32 = 22.0;
const OY: f32 = 146.0;
const LW: f32 = 96.0;
const ZMAX: f32 = 56.0;
const ZMIN: f32 = 3.0;
const CAM_SPEED: f32 = 0.8;
const PLAYER_DF: f32 = 40.0;
const DROP: i32 = 22;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Ok {
    Wall,
    Tower,
    Pylon,
    Fence { z0: f32, z1: f32 },
    Turret,
    Silo(PowerUp),
}

#[derive(Clone, Debug)]
struct Obj {
    kind: Ok,
    f0: f32,
    f1: f32,
    l0: f32,
    l1: f32,
    h: f32,
    hp: i32,
    flash: u8,
    cd: i32,
    /// Sideways speed of a moving wall.
    vl: f32,
    /// A fence that swaps between low and high every this many frames.
    switch: u32,
}

impl Obj {
    fn moving(mut self, vl: f32) -> Obj {
        self.vl = vl;
        self
    }
    fn switching(mut self, every: u32) -> Obj {
        self.switch = every;
        self
    }
    fn new(kind: Ok, f: f32, l: f32, df: f32, dl: f32, h: f32) -> Obj {
        let hp = match kind {
            Ok::Turret => 3,
            Ok::Silo(_) => 5,
            _ => 0,
        };
        Obj {
            kind,
            f0: f,
            f1: f + df,
            l0: l,
            l1: l + dl,
            h,
            hp,
            flash: 0,
            cd: 60,
            vl: 0.0,
            switch: 0,
        }
    }
    fn solid(&self) -> bool {
        !matches!(self.kind, Ok::Fence { .. })
    }
    fn z_range(&self) -> (f32, f32) {
        match self.kind {
            Ok::Fence { z0, z1 } => (z0, z1),
            _ => (0.0, self.h),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Ak {
    Kite,
    Saucer,
    Shard,
    Sentinel,
    Bomber,
}

#[derive(Clone, Debug)]
struct Air {
    kind: Ak,
    f: f32,
    l: f32,
    z: f32,
    base: f32,
    t: u32,
    hp: i32,
    flash: u8,
    cd: i32,
    dir: f32,
}

#[derive(Clone, Debug)]
struct Shot {
    f: f32,
    l: f32,
    z: f32,
    vf: f32,
    vl: f32,
    vz: f32,
    life: u32,
}

#[derive(Clone, Debug)]
struct Pickup {
    f: f32,
    l: f32,
    z: f32,
    kind: PowerUp,
    t: u32,
}

struct Boss {
    l: f32,
    t: u32,
    core_hp: i32,
    core_max: i32,
    flash: u8,
    dying: u32,
}

#[derive(Clone, Copy)]
enum Wave {
    Kites(u8),
    Saucers(u8, f32),
    Bombers(u8),
    Sentinel,
    Text(&'static str),
}

/// What makes one isometric sector different from another.
pub struct IsoSpec {
    course: fn(&mut Rng) -> (Vec<Obj>, Vec<(f32, Wave)>),
    deck: fn(i32, i32) -> bool,
    pub boss_at: f32,
    pub boss_name: &'static str,
    pub shards: u8,
    pub core: i32,
    pub crystal: [u8; 4],
    /// Colour map for the ground (deck and what lies below).
    pub ground: [u8; 16],
    pub border: u8,
    pub hud: u8,
    pub song: SongId,
    pub boss_song: SongId,
}

const SAME: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
const ICE: [u8; 16] = [0, 1, 4, 3, 4, 5, 6, 3, 14, 6, 10, 14, 15, 13, 14, 1];
const CORE: [u8; 16] = [0, 15, 2, 3, 4, 5, 6, 10, 4, 0, 10, 0, 11, 13, 14, 12];

pub fn spec(sector: usize) -> IsoSpec {
    match sector {
        4 => IsoSpec {
            course: course_ice,
            deck: deck_ice,
            boss_at: 7600.0,
            boss_name: "REACTOR",
            shards: 4,
            core: 80,
            crystal: [WHITE, LBLUE, BLUE, CYAN],
            ground: ICE,
            border: LBLUE,
            hud: CYAN,
            song: SongId::Level5,
            boss_song: SongId::Boss,
        },
        7 => IsoSpec {
            course: course_core,
            deck: deck_core,
            boss_at: 3600.0,
            boss_name: "CORE SPIRE",
            shards: 5,
            core: 90,
            crystal: [LRED, PURPLE, BLUE, WHITE],
            ground: CORE,
            border: PURPLE,
            hud: LRED,
            song: SongId::Level8,
            boss_song: SongId::FinalBoss,
        },
        _ => IsoSpec {
            course: build_course,
            deck,
            boss_at: 7300.0,
            boss_name: "MONOLITH",
            shards: 3,
            core: 60,
            crystal: [CYAN, LBLUE, BLUE, WHITE],
            ground: SAME,
            border: BROWN,
            hud: ORANGE,
            song: SongId::Level2,
            boss_song: SongId::Boss,
        },
    }
}

pub struct Iso {
    t: u32,
    cam: f32,
    agg: f32,
    diff: Difficulty,
    rng: Rng,
    fx: Fx,
    l: f32,
    z: f32,
    inv: u32,
    dead: u32,
    fire_cd: u32,
    bomb_cd: u32,
    pending: Vec<Obj>,
    objs: Vec<Obj>,
    waves: Vec<(f32, Wave)>,
    air: Vec<Air>,
    shots: Vec<Shot>,
    eshots: Vec<Shot>,
    pickups: Vec<Pickup>,
    boss: Option<Boss>,
    boss_started: bool,
    clear_t: Option<u32>,
    over_t: Option<u32>,
    banner: Option<(&'static str, u32)>,
    bank: f32,
    last_cause: &'static str,
    spec: IsoSpec,
    /// How far the course was held back while a mid-boss fought.
    off: f32,
}

/// Where the deck exists (tile coordinates of 16 units).
fn deck(fi: i32, li: i32) -> bool {
    if !(0..6).contains(&li) {
        return false;
    }
    let f = fi * 16;
    // Narrow bridges over the dunes.
    if (1900..2700).contains(&f) {
        return (2..4).contains(&li);
    }
    if (2700..3300).contains(&f) {
        return li == 0 || li == 5 || (f / 16) % 8 < 2;
    }
    if (5400..5800).contains(&f) {
        return (1..5).contains(&li);
    }
    true
}

fn build_course(rng: &mut Rng) -> (Vec<Obj>, Vec<(f32, Wave)>) {
    let mut o = Vec::new();
    let mut w = Vec::new();
    let pair = |o: &mut Vec<Obj>, f: f32| {
        o.push(Obj::new(Ok::Turret, f, 14.0, 10.0, 10.0, 7.0));
        o.push(Obj::new(Ok::Turret, f, 72.0, 10.0, 10.0, 7.0));
    };
    let fence = |o: &mut Vec<Obj>, f: f32, z0: f32, z1: f32| {
        o.push(Obj::new(Ok::Pylon, f - 2.0, -10.0, 8.0, 8.0, ZMAX + 6.0));
        o.push(Obj::new(Ok::Pylon, f - 2.0, LW + 2.0, 8.0, 8.0, ZMAX + 6.0));
        o.push(Obj::new(Ok::Fence { z0, z1 }, f, 0.0, 3.0, LW, z1));
    };
    w.push((60.0, Wave::Text("CLIMB AND DIVE: WATCH THE SHADOW")));
    // A: a gentle start.
    pair(&mut o, 380.0);
    w.push((300.0, Wave::Kites(3)));
    pair(&mut o, 620.0);
    o.push(Obj::new(
        Ok::Silo(PowerUp::Weapon),
        760.0,
        42.0,
        12.0,
        12.0,
        12.0,
    ));
    o.push(Obj::new(Ok::Wall, 950.0, 0.0, 10.0, LW, 14.0));
    w.push((820.0, Wave::Text("WALL AHEAD: CLIMB")));
    w.push((1050.0, Wave::Kites(4)));
    pair(&mut o, 1180.0);
    fence(&mut o, 1450.0, 0.0, 26.0);
    w.push((1320.0, Wave::Text("ENERGY FENCE: FLY OVER IT")));
    w.push((1550.0, Wave::Saucers(5, 30.0)));
    o.push(Obj::new(
        Ok::Silo(PowerUp::Shield),
        1700.0,
        70.0,
        12.0,
        12.0,
        12.0,
    ));
    // B: bridges and towers.
    w.push((1850.0, Wave::Text("THE BRIDGES")));
    for k in 0..8 {
        let f = 2000.0 + k as f32 * 130.0;
        let l = if k % 2 == 0 { 20.0 } else { 62.0 };
        o.push(Obj::new(
            Ok::Tower,
            f,
            l,
            12.0,
            12.0,
            44.0 + (k % 3) as f32 * 6.0,
        ));
    }
    w.push((2100.0, Wave::Kites(5)));
    fence(&mut o, 2600.0, 26.0, ZMAX + 4.0);
    w.push((2480.0, Wave::Text("HIGH FENCE: FLY UNDER IT")));
    w.push((2750.0, Wave::Saucers(6, 18.0)));
    o.push(Obj::new(
        Ok::Silo(PowerUp::Weapon),
        2900.0,
        2.0,
        12.0,
        12.0,
        12.0,
    ));
    w.push((3050.0, Wave::Kites(5)));
    // C: the fortress, walls with gaps.
    w.push((3350.0, Wave::Text("THE FORTRESS")));
    for k in 0..5 {
        let f = 3500.0 + k as f32 * 260.0;
        let gap = [36.0, 8.0, 60.0, 30.0, 50.0][k];
        o.push(Obj::new(Ok::Wall, f, 0.0, 12.0, gap, 34.0));
        o.push(Obj::new(
            Ok::Wall,
            f,
            gap + 30.0,
            12.0,
            LW - gap - 30.0,
            34.0,
        ));
        o.push(Obj::new(Ok::Turret, f + 60.0, gap + 10.0, 10.0, 10.0, 7.0));
    }
    w.push((3600.0, Wave::Saucers(6, 40.0)));
    w.push((4000.0, Wave::Kites(6)));
    o.push(Obj::new(
        Ok::Silo(PowerUp::Bomb),
        4300.0,
        44.0,
        12.0,
        12.0,
        12.0,
    ));
    w.push((4500.0, Wave::Saucers(7, 12.0)));
    for k in 0..6 {
        pair(&mut o, 4700.0 + k as f32 * 90.0);
    }
    // D: fences in quick succession.
    w.push((5150.0, Wave::Text("FENCE RUN")));
    for k in 0..5 {
        let f = 5300.0 + k as f32 * 210.0;
        if k % 2 == 0 {
            fence(&mut o, f, 0.0, 24.0);
        } else {
            fence(&mut o, f, 30.0, ZMAX + 4.0);
        }
    }
    w.push((5400.0, Wave::Kites(6)));
    w.push((5800.0, Wave::Saucers(8, 26.0)));
    o.push(Obj::new(
        Ok::Silo(PowerUp::Weapon),
        6400.0,
        30.0,
        12.0,
        12.0,
        12.0,
    ));
    o.push(Obj::new(
        Ok::Silo(PowerUp::Shield),
        6400.0,
        60.0,
        12.0,
        12.0,
        12.0,
    ));
    for k in 0..6 {
        let f = 6550.0 + k as f32 * 100.0;
        o.push(Obj::new(
            Ok::Tower,
            f,
            rng.range(4.0, 80.0),
            12.0,
            12.0,
            50.0,
        ));
        o.push(Obj::new(
            Ok::Turret,
            f + 50.0,
            rng.range(4.0, 80.0),
            10.0,
            10.0,
            7.0,
        ));
    }
    w.push((6020.0, Wave::Sentinel));
    w.push((6600.0, Wave::Kites(7)));
    w.push((7050.0, Wave::Text("WARNING: THE MONOLITH")));
    o.sort_by(|a, b| a.f0.total_cmp(&b.f0));
    (o, w)
}

/// Sector 5: an ice deck over a frozen sea.
fn deck_ice(fi: i32, li: i32) -> bool {
    if !(0..6).contains(&li) {
        return false;
    }
    let f = fi * 16;
    if (2300..2900).contains(&f) {
        return li != 2 && li != 3;
    }
    if (5000..5600).contains(&f) {
        return (1..5).contains(&li) && (f / 16) % 6 != 0;
    }
    true
}

fn course_ice(rng: &mut Rng) -> (Vec<Obj>, Vec<(f32, Wave)>) {
    let mut o = Vec::new();
    let mut w = Vec::new();
    let pair = |o: &mut Vec<Obj>, f: f32, a: f32, b: f32| {
        o.push(Obj::new(Ok::Turret, f, a, 10.0, 10.0, 7.0));
        o.push(Obj::new(Ok::Turret, f, b, 10.0, 10.0, 7.0));
    };
    let fence = |o: &mut Vec<Obj>, f: f32, sw: u32| {
        o.push(Obj::new(Ok::Pylon, f - 2.0, -10.0, 8.0, 8.0, ZMAX + 6.0));
        o.push(Obj::new(Ok::Pylon, f - 2.0, LW + 2.0, 8.0, 8.0, ZMAX + 6.0));
        o.push(Obj::new(Ok::Fence { z0: 0.0, z1: 24.0 }, f, 0.0, 3.0, LW, 24.0).switching(sw));
    };
    w.push((60.0, Wave::Text("ICE PYLONS: WALLS THAT MOVE")));
    pair(&mut o, 400.0, 10.0, 76.0);
    w.push((300.0, Wave::Kites(4)));
    for k in 0..4 {
        let f = 700.0 + k as f32 * 180.0;
        o.push(
            Obj::new(Ok::Wall, f, 8.0 + k as f32 * 14.0, 12.0, 30.0, 40.0).moving(if k % 2 == 0 {
                0.5
            } else {
                -0.5
            }),
        );
    }
    o.push(Obj::new(
        Ok::Silo(PowerUp::Weapon),
        1150.0,
        44.0,
        12.0,
        12.0,
        12.0,
    ));
    w.push((1200.0, Wave::Bombers(3)));
    w.push((1350.0, Wave::Text("FENCES THAT SWITCH: WATCH THEM")));
    fence(&mut o, 1550.0, 150);
    fence(&mut o, 1800.0, 110);
    w.push((1700.0, Wave::Saucers(6, 22.0)));
    for k in 0..6 {
        let f = 2000.0 + k as f32 * 70.0;
        o.push(Obj::new(
            Ok::Tower,
            f,
            rng.range(4.0, 80.0),
            12.0,
            12.0,
            52.0,
        ));
    }
    w.push((2200.0, Wave::Text("THE CRACKED ICE")));
    w.push((2350.0, Wave::Kites(6)));
    pair(&mut o, 2600.0, 4.0, 82.0);
    o.push(Obj::new(
        Ok::Silo(PowerUp::Shield),
        3000.0,
        44.0,
        12.0,
        12.0,
        12.0,
    ));
    w.push((3150.0, Wave::Bombers(4)));
    w.push((3500.0, Wave::Sentinel));
    for k in 0..5 {
        let f = 3700.0 + k as f32 * 220.0;
        let gap = [20.0, 56.0, 34.0, 8.0, 48.0][k];
        o.push(Obj::new(Ok::Wall, f, 0.0, 12.0, gap, 34.0));
        o.push(Obj::new(
            Ok::Wall,
            f,
            gap + 32.0,
            12.0,
            LW - gap - 32.0,
            34.0,
        ));
        pair(&mut o, f + 90.0, 4.0, 84.0);
    }
    w.push((3800.0, Wave::Saucers(7, 40.0)));
    w.push((4300.0, Wave::Kites(7)));
    o.push(Obj::new(
        Ok::Silo(PowerUp::Bomb),
        4800.0,
        30.0,
        12.0,
        12.0,
        12.0,
    ));
    w.push((4900.0, Wave::Text("FENCE STORM")));
    for k in 0..6 {
        fence(&mut o, 5100.0 + k as f32 * 170.0, 90 + k * 13);
    }
    w.push((5300.0, Wave::Bombers(5)));
    for k in 0..5 {
        let f = 6200.0 + k as f32 * 160.0;
        o.push(Obj::new(Ok::Wall, f, 10.0, 12.0, 34.0, 44.0).moving(0.8));
        o.push(Obj::new(
            Ok::Tower,
            f + 80.0,
            rng.range(4.0, 80.0),
            12.0,
            12.0,
            54.0,
        ));
    }
    o.push(Obj::new(
        Ok::Silo(PowerUp::Weapon),
        6100.0,
        70.0,
        12.0,
        12.0,
        12.0,
    ));
    w.push((6500.0, Wave::Kites(8)));
    w.push((7000.0, Wave::Saucers(8, 30.0)));
    w.push((7350.0, Wave::Text("WARNING: THE FROZEN REACTOR")));
    o.sort_by(|a, b| a.f0.total_cmp(&b.f0));
    (o, w)
}

/// Sector 8, stage two: the descent to the core.
fn deck_core(fi: i32, li: i32) -> bool {
    if !(0..6).contains(&li) {
        return false;
    }
    let f = fi * 16;
    if (1200..1800).contains(&f) {
        return li == 0 || li == 5 || (f / 32) % 3 == 0;
    }
    true
}

fn course_core(rng: &mut Rng) -> (Vec<Obj>, Vec<(f32, Wave)>) {
    let mut o = Vec::new();
    let mut w = Vec::new();
    w.push((40.0, Wave::Text("STAGE TWO: THE DESCENT")));
    w.push((200.0, Wave::Kites(6)));
    for k in 0..6 {
        let f = 400.0 + k as f32 * 120.0;
        o.push(Obj::new(Ok::Wall, f, rng.range(0.0, 40.0), 12.0, 40.0, 38.0).moving(0.7));
        o.push(Obj::new(
            Ok::Turret,
            f + 60.0,
            rng.range(4.0, 84.0),
            10.0,
            10.0,
            7.0,
        ));
    }
    w.push((900.0, Wave::Bombers(5)));
    o.push(Obj::new(
        Ok::Silo(PowerUp::Weapon),
        1100.0,
        44.0,
        12.0,
        12.0,
        12.0,
    ));
    for k in 0..4 {
        let f = 1250.0 + k as f32 * 150.0;
        o.push(Obj::new(Ok::Pylon, f - 2.0, -10.0, 8.0, 8.0, ZMAX + 6.0));
        o.push(Obj::new(Ok::Pylon, f - 2.0, LW + 2.0, 8.0, 8.0, ZMAX + 6.0));
        o.push(
            Obj::new(Ok::Fence { z0: 0.0, z1: 24.0 }, f, 0.0, 3.0, LW, 24.0).switching(80 + k * 20),
        );
    }
    w.push((1500.0, Wave::Saucers(8, 34.0)));
    w.push((1900.0, Wave::Sentinel));
    o.push(Obj::new(
        Ok::Silo(PowerUp::Shield),
        2300.0,
        44.0,
        12.0,
        12.0,
        12.0,
    ));
    for k in 0..6 {
        let f = 2500.0 + k as f32 * 120.0;
        o.push(Obj::new(
            Ok::Tower,
            f,
            rng.range(4.0, 80.0),
            12.0,
            12.0,
            56.0,
        ));
    }
    w.push((2700.0, Wave::Kites(8)));
    w.push((3200.0, Wave::Bombers(6)));
    w.push((3400.0, Wave::Text("THE CORE SPIRE")));
    o.sort_by(|a, b| a.f0.total_cmp(&b.f0));
    (o, w)
}

impl Iso {
    pub fn new(diff: Difficulty, seed: u32, sector: usize) -> Iso {
        let spec = spec(sector);
        let mut rng = Rng::new(seed);
        let (pending, waves) = (spec.course)(&mut rng);
        let mut fx = Fx::new(seed ^ 0x150);
        fx.gravity = 0.04;
        Iso {
            t: 0,
            cam: 0.0,
            agg: diff.aggression() * (0.85 + sector as f32 * 0.05),
            diff,
            rng,
            fx,
            l: 48.0,
            z: 22.0,
            inv: 120,
            dead: 0,
            fire_cd: 0,
            bomb_cd: 0,
            pending,
            objs: Vec::new(),
            waves,
            air: Vec::new(),
            shots: Vec::new(),
            eshots: Vec::new(),
            pickups: Vec::new(),
            boss: None,
            boss_started: false,
            clear_t: None,
            over_t: None,
            banner: None,
            bank: 0.0,
            last_cause: "",
            spec,
            off: 0.0,
        }
    }

    fn pf(&self) -> f32 {
        self.cam + PLAYER_DF
    }

    fn proj(&self, f: f32, l: f32, z: f32) -> (f32, f32) {
        let df = f - self.cam;
        (OX + df + l, OY - df * 0.5 + l * 0.5 - z)
    }

    /// Height of whatever is under a point (for shadows and landing).
    fn ground(&self, f: f32, l: f32) -> Option<f32> {
        let mut h = None;
        let fi = ((f - self.off) / 16.0).floor() as i32;
        let li = (l / 16.0).floor() as i32;
        if (self.spec.deck)(fi, li) {
            h = Some(0.0);
        }
        for o in &self.objs {
            if o.solid() && f >= o.f0 && f <= o.f1 && l >= o.l0 && l <= o.l1 {
                h = Some(h.unwrap_or(0.0f32).max(o.h));
            }
        }
        h
    }

    fn spawn_wave(&mut self, w: Wave) {
        match w {
            Wave::Kites(n) => {
                for i in 0..n {
                    let z = self.rng.range(12.0, 44.0);
                    self.air.push(Air {
                        kind: Ak::Kite,
                        f: self.cam + 300.0 + i as f32 * 30.0,
                        l: self.rng.range(10.0, 86.0),
                        z,
                        base: self.rng.range(20.0, 76.0),
                        t: i as u32 * 20,
                        hp: 2,
                        flash: 0,
                        cd: 50 + self.rng.below(80) as i32,
                        dir: 0.0,
                    });
                }
            }
            Wave::Saucers(n, z) => {
                let from_left = self.rng.chance(0.5);
                for i in 0..n {
                    self.air.push(Air {
                        kind: Ak::Saucer,
                        f: self.cam + 150.0 + i as f32 * 4.0,
                        l: if from_left {
                            -20.0 - i as f32 * 16.0
                        } else {
                            LW + 20.0 + i as f32 * 16.0
                        },
                        z,
                        base: self.cam,
                        t: 0,
                        hp: 1,
                        flash: 0,
                        cd: 80 + self.rng.below(120) as i32,
                        dir: if from_left { 1.0 } else { -1.0 },
                    });
                }
            }
            Wave::Bombers(n) => {
                for i in 0..n {
                    self.air.push(Air {
                        kind: Ak::Bomber,
                        f: self.cam + 320.0 + i as f32 * 26.0,
                        l: 16.0 + (i as f32 * 23.0) % 64.0,
                        z: 46.0,
                        base: 0.0,
                        t: 0,
                        hp: 3,
                        flash: 0,
                        cd: 40 + i as i32 * 9,
                        dir: 0.0,
                    });
                }
            }
            Wave::Sentinel => {
                self.banner = Some(("SENTINEL INCOMING", self.t));
                self.air.push(Air {
                    kind: Ak::Sentinel,
                    f: self.cam + 360.0,
                    l: 48.0,
                    z: 30.0,
                    base: 0.0,
                    t: 0,
                    hp: 50,
                    flash: 0,
                    cd: 120,
                    dir: 0.0,
                });
            }
            Wave::Text(s) => self.banner = Some((s, self.t)),
        }
    }

    fn fire_at_player(&mut self, f: f32, l: f32, z: f32, speed: f32, spread: f32) {
        let (pf, pl, pz) = (self.pf(), self.l, self.z);
        let (df, dl, dz) = (pf - f, pl - l, pz - z);
        let d = (df * df + dl * dl + dz * dz).sqrt().max(1.0);
        let sp = speed * (0.7 + 0.3 * self.agg);
        if self.eshots.len() < 100 {
            self.eshots.push(Shot {
                f,
                l,
                z,
                vf: df / d * sp + CAM_SPEED,
                vl: dl / d * sp + spread,
                vz: dz / d * sp,
                life: 0,
            });
        }
    }

    fn kill_player(&mut self, st: &mut Stats, out: &mut Out) {
        if self.inv > 0 || self.dead > 0 || self.clear_t.is_some() {
            return;
        }
        let (x, y) = self.proj(self.pf(), self.l, self.z);
        if st.shield > 0 {
            st.shield -= 1;
            self.inv = 70;
            self.fx.sparks(x, y, 14, 0.0);
            self.fx.shake = 3.0;
            out.play(Sfx::Clang);
            return;
        }
        self.fx.explode(x, y, 3.0);
        self.fx.flash = 3;
        out.play(Sfx::PlayerDie);
        st.lives -= 1;
        st.deaths_this_level += 1;
        st.weapon = st.weapon.saturating_sub(1).max(1);
        self.dead = 1;
        self.eshots.clear();
    }

    fn player_fire(&mut self, st: &Stats, out: &mut Out) {
        let (f, l, z) = (self.pf() + 8.0, self.l, self.z);
        let mut add = |dl: f32, vl: f32| {
            self.shots.push(Shot {
                f,
                l: l + dl,
                z,
                vf: 5.0 + CAM_SPEED,
                vl,
                vz: 0.0,
                life: 0,
            })
        };
        match st.weapon {
            1 => add(0.0, 0.0),
            2 => {
                add(-4.0, 0.0);
                add(4.0, 0.0);
            }
            3 => {
                add(-4.0, 0.0);
                add(4.0, 0.0);
                add(0.0, -0.8);
                add(0.0, 0.8);
            }
            _ => {
                add(0.0, 0.0);
                add(-6.0, 0.0);
                add(6.0, 0.0);
                add(-2.0, -1.0);
                add(2.0, 1.0);
            }
        }
        out.play(if st.weapon >= 4 {
            Sfx::Laser2
        } else {
            Sfx::Laser
        });
        self.fire_cd = if st.weapon >= 4 { 6 } else { 8 };
    }

    fn score(&mut self, st: &mut Stats, out: &mut Out, pts: u32, x: f32, y: f32) {
        if st.add(pts) {
            out.play(Sfx::ExtraLife);
            self.fx.text(x, y - 12.0, "1UP".into(), LGREEN);
        }
        st.kills += 1;
        self.fx.text(x, y - 6.0, pts.to_string(), YELLOW);
    }

    fn update_boss(&mut self, st: &mut Stats, out: &mut Out) {
        let Some(mut b) = self.boss.take() else {
            return;
        };
        b.t += 1;
        b.flash = b.flash.saturating_sub(1);
        let bf = self.cam + 210.0;
        if b.dying > 0 {
            b.dying += 1;
            if b.dying % 5 == 0 {
                let z = self.rng.range(0.0, 60.0);
                let (r1, r2) = (self.rng.s() * 12.0, self.rng.s() * 12.0);
                let (x, y) = self.proj(bf + r1, b.l + r2, z);
                self.fx.explode(x, y, 2.5);
                out.play(Sfx::ExplodeBig);
            }
            if b.dying == 160 {
                let (x, y) = self.proj(bf, b.l, 30.0);
                self.fx.explode(x, y, 8.0);
                self.fx.flash = 10;
                st.add(30000);
                self.fx.text(x, y, "30000".into(), YELLOW);
                self.clear_t = Some(self.t);
                return;
            }
            self.boss = Some(b);
            return;
        }
        b.l = 42.0 + (b.t as f32 * 0.012).sin() * 26.0;
        let shards = self.air.iter().filter(|a| a.kind == Ak::Shard).count();
        let period = (110.0 / self.agg) as u32;
        if shards == 0 && b.t % period == 0 {
            // Fans from the open core.
            let n = if self.diff == Difficulty::Hard { 7 } else { 5 };
            for k in 0..n {
                let s = (k as f32 - (n - 1) as f32 / 2.0) * 0.35;
                self.fire_at_player(bf - 14.0, b.l + 12.0, 30.0, 1.4, s);
            }
            out.play(Sfx::EnemyShot);
        }
        if shards == 0 && b.t % (period * 2) == period / 2 {
            // Mines rolling along the deck.
            for k in 0..3 {
                self.eshots.push(Shot {
                    f: bf - 10.0,
                    l: b.l - 12.0 + k as f32 * 12.0,
                    z: 3.0,
                    vf: CAM_SPEED - 1.2,
                    vl: 0.0,
                    vz: 0.0,
                    life: 0,
                });
            }
        }
        self.boss = Some(b);
    }
}

impl Level for Iso {
    fn update(&mut self, inp: &Input, st: &mut Stats, out: &mut Out) -> Status {
        self.t += 1;
        if self.boss.is_none() || self.boss.as_ref().is_some_and(|b| b.dying == 0) {
            self.cam += CAM_SPEED;
        } else {
            self.cam += CAM_SPEED * 0.5;
        }
        // A sentinel holds the course back while it fights.
        if self.air.iter().any(|a| a.kind == Ak::Sentinel) {
            self.off += CAM_SPEED;
            for o in self.pending.iter_mut() {
                o.f0 += CAM_SPEED;
                o.f1 += CAM_SPEED;
            }
        }
        let t = self.t;
        for o in self.objs.iter_mut() {
            if o.vl != 0.0 {
                o.l0 += o.vl;
                o.l1 += o.vl;
                if o.l0 < 0.0 || o.l1 > LW {
                    o.vl = -o.vl;
                }
            }
            if o.switch > 0 {
                let high = (t / o.switch) % 2 == 1;
                o.kind = if high {
                    Ok::Fence {
                        z0: 30.0,
                        z1: ZMAX + 4.0,
                    }
                } else {
                    Ok::Fence { z0: 0.0, z1: 24.0 }
                };
                o.h = if high { ZMAX + 4.0 } else { 24.0 };
            }
        }
        // Bring the course into view.
        while self
            .pending
            .first()
            .is_some_and(|o| o.f0 < self.cam + 340.0)
        {
            let o = self.pending.remove(0);
            self.objs.push(o);
        }
        let cam = self.cam;
        self.objs.retain(|o| o.f1 > cam - 140.0);
        while self
            .waves
            .first()
            .is_some_and(|w| w.0 < self.cam - self.off)
        {
            let w = self.waves.remove(0).1;
            self.spawn_wave(w);
        }
        if self.t == 1 {
            out.music = Some(self.spec.song);
        }
        if !self.boss_started && self.cam - self.off >= self.spec.boss_at {
            self.boss_started = true;
            out.music = Some(self.spec.boss_song);
            out.play(Sfx::Alarm);
            self.boss = Some(Boss {
                l: 42.0,
                t: 0,
                core_hp: self.spec.core,
                core_max: self.spec.core,
                flash: 0,
                dying: 0,
            });
            let n = self.spec.shards as i32;
            for k in 0..n {
                self.air.push(Air {
                    kind: Ak::Shard,
                    f: 0.0,
                    l: 0.0,
                    z: 0.0,
                    base: k as f32 * std::f32::consts::TAU / n as f32,
                    t: 0,
                    hp: 14,
                    flash: 0,
                    cd: 60 + k * 40,
                    dir: 0.0,
                });
            }
        }
        // The player.
        if self.dead > 0 {
            self.dead += 1;
            if self.dead > 100 {
                if st.lives <= 0 {
                    self.over_t.get_or_insert(self.t);
                } else {
                    self.dead = 0;
                    self.inv = 150;
                    self.z = 30.0;
                    self.l = 48.0;
                }
            }
        } else {
            let (dx, dy) = (inp.dx(), inp.dy());
            self.l = (self.l + dx * 1.3).clamp(2.0, LW - 2.0);
            self.z = (self.z - dy * 0.9).clamp(ZMIN, ZMAX);
            self.bank += (dx - self.bank) * 0.2;
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
                st.bombs -= 1;
                self.bomb_cd = 50;
                self.fx.flash = 8;
                self.fx.shake = 7.0;
                out.play(Sfx::Bomb);
                self.eshots.clear();
                let near = self.cam + 300.0;
                for a in self.air.iter_mut() {
                    if a.f < near {
                        a.hp -= 8;
                        a.flash = 6;
                    }
                }
                for o in self.objs.iter_mut() {
                    if o.hp > 0 && o.f0 < near {
                        o.hp -= 8;
                        o.flash = 6;
                    }
                }
            }
        }
        self.inv = self.inv.saturating_sub(1);
        let (pf, pl, pz) = (self.pf(), self.l, self.z);
        // Ground objects: turrets fire, destroyed things explode.
        let mut fire: Vec<(f32, f32, f32)> = Vec::new();
        for o in self.objs.iter_mut() {
            o.flash = o.flash.saturating_sub(1);
            if o.kind == Ok::Turret && o.hp > 0 {
                let df = o.f0 - cam;
                if (70.0..300.0).contains(&df) {
                    o.cd -= 1;
                    if o.cd <= 0 {
                        o.cd = (130.0 / self.agg) as i32 + self.rng.below(60) as i32;
                        fire.push((o.f0 + 5.0, o.l0 + 5.0, 9.0));
                    }
                }
            }
        }
        for (f, l, z) in fire {
            self.fire_at_player(f, l, z, 1.3, 0.0);
            out.play(Sfx::EnemyShot);
        }
        // Air enemies.
        let boss_l = self.boss.as_ref().map(|b| b.l);
        let mut afire: Vec<(f32, f32, f32)> = Vec::new();
        let mut sentinel_fire: Vec<(f32, f32, f32)> = Vec::new();
        let mut bombs: Vec<(f32, f32, f32)> = Vec::new();
        for a in self.air.iter_mut() {
            a.t += 1;
            a.flash = a.flash.saturating_sub(1);
            match a.kind {
                Ak::Kite => {
                    a.f -= 1.0;
                    a.l = a.base + (a.t as f32 * 0.04).sin() * 22.0;
                    a.z += (pz - a.z).clamp(-0.15, 0.15);
                }
                Ak::Saucer => {
                    a.l += a.dir * 1.1;
                    a.f += CAM_SPEED;
                    a.z += (a.t as f32 * 0.1).sin() * 0.3;
                }
                Ak::Sentinel => {
                    let df = a.f - cam;
                    if df > 175.0 {
                        a.f -= 0.6;
                    } else {
                        a.f += CAM_SPEED;
                    }
                    a.l = 48.0 + (a.t as f32 * 0.017).sin() * 32.0;
                    a.z = 30.0 + (a.t as f32 * 0.023).sin() * 12.0;
                }
                Ak::Bomber => {
                    a.f -= 0.4;
                    a.l += (pl - a.l).clamp(-0.3, 0.3);
                }
                Ak::Shard => {
                    let bl = boss_l.unwrap_or(48.0);
                    let ang = a.base + a.t as f32 * 0.02;
                    a.f = cam + 210.0 + ang.cos() * 34.0;
                    a.l = bl + ang.sin() * 34.0;
                    a.z = 26.0 + (ang * 2.0).sin() * 14.0;
                }
            }
            let df = a.f - cam;
            if (60.0..290.0).contains(&df) {
                a.cd -= 1;
                if a.cd <= 0 {
                    a.cd = match a.kind {
                        Ak::Kite => (150.0 / self.agg) as i32,
                        Ak::Saucer => (200.0 / self.agg) as i32,
                        Ak::Shard => (90.0 / self.agg) as i32,
                        Ak::Sentinel => (85.0 / self.agg) as i32,
                        Ak::Bomber => (70.0 / self.agg) as i32,
                    } + self.rng.below(50) as i32;
                    if a.kind == Ak::Sentinel {
                        sentinel_fire.push((a.f, a.l, a.z));
                    } else if a.kind == Ak::Bomber {
                        bombs.push((a.f, a.l, a.z));
                    } else if self.diff != Difficulty::Easy || a.kind != Ak::Saucer {
                        afire.push((a.f, a.l, a.z));
                    }
                }
            }
        }
        for (f, l, z) in afire {
            self.fire_at_player(f, l, z, 1.4, 0.0);
            out.play(Sfx::EnemyShot);
        }
        for (f, l, z) in sentinel_fire {
            let n = if self.diff == Difficulty::Hard { 7 } else { 5 };
            for k in 0..n {
                let s = (k as f32 - (n - 1) as f32 / 2.0) * 0.3;
                self.fire_at_player(f - 10.0, l, z, 1.3, s);
            }
            out.play(Sfx::EnemyShot);
        }
        for (f, l, z) in bombs {
            if self.eshots.len() < 100 {
                self.eshots.push(Shot {
                    f,
                    l,
                    z,
                    vf: CAM_SPEED * 0.6,
                    vl: 0.0,
                    vz: -0.7,
                    life: 0,
                });
            }
        }
        self.update_boss(st, out);
        // Player shots.
        let mut shots = std::mem::take(&mut self.shots);
        let mut kills: Vec<(f32, f32, f32, f32, u32)> = Vec::new();
        shots.retain_mut(|s| {
            s.f += s.vf;
            s.l += s.vl;
            s.life += 1;
            if s.f - cam > 320.0 || s.life > 80 {
                return false;
            }
            for o in self.objs.iter_mut() {
                let (z0, z1) = o.z_range();
                if s.f >= o.f0
                    && s.f <= o.f1 + 4.0
                    && s.l >= o.l0 - 2.0
                    && s.l <= o.l1 + 2.0
                    && s.z >= z0
                    && s.z <= z1 + 2.0
                {
                    if matches!(o.kind, Ok::Fence { .. }) {
                        return true;
                    }
                    if o.hp > 0 {
                        o.hp -= 1;
                        o.flash = 4;
                        if o.hp <= 0 {
                            let pts = if o.kind == Ok::Turret { 400 } else { 600 };
                            kills.push((
                                (o.f0 + o.f1) / 2.0,
                                (o.l0 + o.l1) / 2.0,
                                o.h / 2.0,
                                1.8,
                                pts,
                            ));
                        }
                    } else {
                        let df = s.f - cam;
                        let (x, y) = (OX + df + s.l, OY - df * 0.5 + s.l * 0.5 - s.z);
                        self.fx.sparks(x, y, 2, 3.5);
                    }
                    return false;
                }
            }
            for a in self.air.iter_mut() {
                if a.hp > 0
                    && (a.f - s.f).abs() < 8.0
                    && (a.l - s.l).abs() < 7.0
                    && (a.z - s.z).abs() < 6.0
                {
                    a.hp -= 1;
                    a.flash = 4;
                    return false;
                }
            }
            if let Some(b) = self.boss.as_mut() {
                let bf = cam + 210.0;
                if b.dying == 0
                    && (s.f - (bf - 12.0)).abs() < 8.0
                    && (s.l - b.l).abs() < 14.0
                    && s.z < 64.0
                {
                    let open = !self.air.iter().any(|a| a.kind == Ak::Shard);
                    if open && (s.z - 30.0).abs() < 9.0 {
                        b.core_hp -= 1;
                        b.flash = 3;
                        if b.core_hp <= 0 {
                            b.dying = 1;
                        }
                    } else {
                        let df = s.f - cam;
                        let (x, y) = (OX + df + s.l, OY - df * 0.5 + s.l * 0.5 - s.z);
                        self.fx.sparks(x, y, 2, 3.5);
                    }
                    return false;
                }
            }
            true
        });
        self.shots = shots;
        // Deaths.
        let mut drops = Vec::new();
        self.objs.retain(|o| {
            if o.hp <= 0 && matches!(o.kind, Ok::Turret | Ok::Silo(_)) {
                if let Ok::Silo(p) = o.kind {
                    drops.push(((o.f0 + o.f1) / 2.0, (o.l0 + o.l1) / 2.0, p));
                }
                if !kills
                    .iter()
                    .any(|k| (k.0 - (o.f0 + o.f1) / 2.0).abs() < 0.1)
                {
                    kills.push((
                        (o.f0 + o.f1) / 2.0,
                        (o.l0 + o.l1) / 2.0,
                        o.h / 2.0,
                        1.8,
                        400,
                    ));
                }
                return false;
            }
            true
        });
        self.air.retain(|a| {
            if a.hp <= 0 {
                let pts = match a.kind {
                    Ak::Kite => 300,
                    Ak::Saucer => 200,
                    Ak::Shard => 4000,
                    Ak::Sentinel => 8000,
                    Ak::Bomber => 500,
                };
                kills.push((
                    a.f,
                    a.l,
                    a.z,
                    match a.kind {
                        Ak::Sentinel => 6.0,
                        Ak::Shard => 3.0,
                        _ => 1.3,
                    },
                    pts,
                ));
                if a.kind == Ak::Sentinel {
                    drops.push((a.f, a.l, PowerUp::Weapon));
                    drops.push((a.f + 10.0, a.l + 12.0, PowerUp::Shield));
                }
                return false;
            }
            let df = a.f - cam;
            df > -60.0 && df < 420.0 && a.l > -160.0 && a.l < LW + 160.0
        });
        for (f, l, p) in drops {
            self.pickups.push(Pickup {
                f,
                l,
                z: 10.0,
                kind: p,
                t: 0,
            });
        }
        for (f, l, z, size, pts) in kills {
            let (x, y) = self.proj(f, l, z);
            self.fx.explode(x, y, size);
            out.play(if size >= 2.0 {
                Sfx::ExplodeBig
            } else {
                Sfx::ExplodeSmall
            });
            self.score(st, out, pts, x, y);
        }
        // Enemy shots.
        let alive = self.dead == 0;
        let mut hit = false;
        let mut cause = "";
        self.eshots.retain_mut(|s| {
            s.f += s.vf;
            s.l += s.vl;
            s.z += s.vz;
            s.life += 1;
            if alive && (s.f - pf).abs() < 5.0 && (s.l - pl).abs() < 5.0 && (s.z - pz).abs() < 5.0 {
                hit = true;
                cause = "shot";
                return false;
            }
            let df = s.f - cam;
            s.life < 400 && df > -40.0 && df < 360.0 && s.z > -2.0 && s.z < 90.0
        });
        if alive {
            // Crashing into the course.
            for o in &self.objs {
                let (z0, z1) = o.z_range();
                if pf + 4.0 >= o.f0
                    && pf - 4.0 <= o.f1
                    && pl + 3.0 >= o.l0
                    && pl - 3.0 <= o.l1
                    && pz + 1.0 >= z0
                    && pz - 1.0 <= z1
                {
                    hit = true;
                    cause = "course";
                }
            }
            for a in &self.air {
                if (a.f - pf).abs() < 7.0 && (a.l - pl).abs() < 7.0 && (a.z - pz).abs() < 6.0 {
                    hit = true;
                    cause = "air";
                }
            }
        }
        if hit {
            if self.inv == 0 && self.dead == 0 {
                self.last_cause = cause;
            }
            self.kill_player(st, out);
        }
        // Pickups.
        let mut got = Vec::new();
        self.pickups.retain_mut(|p| {
            p.t += 1;
            if alive
                && (p.f - pf).abs() < 12.0
                && (p.l - pl).abs() < 10.0
                && (p.z - pz).abs() < 14.0
            {
                got.push((p.kind, p.f, p.l, p.z));
                return false;
            }
            p.f > cam - 40.0
        });
        for (k, f, l, z) in got {
            let s = st.collect(k, out);
            let (x, y) = self.proj(f, l, z);
            self.fx.text(x, y - 8.0, s.to_string(), LGREEN);
            self.fx.sparks(x, y, 10, 0.0);
        }
        // Debris and sparks drift with the deck.
        for p in self.fx.p.iter_mut() {
            p.x -= CAM_SPEED;
            p.y += CAM_SPEED * 0.5;
        }
        for t in self.fx.texts.iter_mut() {
            t.x -= CAM_SPEED;
            t.y += CAM_SPEED * 0.5;
        }
        self.fx.update();
        if let Some(t) = self.over_t {
            if self.t - t > 30 {
                return Status::Over;
            }
        }
        if let Some(t) = self.clear_t {
            if self.t - t > 60 {
                self.z += 0.6;
            }
            if self.t - t > 200 {
                return Status::Cleared;
            }
        }
        Status::Playing
    }

    fn draw(&self, fb: &mut Fb, st: &Stats, hi: u32) {
        let (ox, oy) = self.fx.offset(self.t);
        self.draw_ground(fb, ox, oy);
        // Shadows first, on the deck.
        let pf = self.pf();
        if self.dead == 0 {
            if let Some(g) = self.ground(pf, self.l) {
                self.draw_plane(fb, pf, self.l, g, ox, oy, true, false, 1.35);
            }
        }
        for a in &self.air {
            if let Some(g) = self.ground(a.f, a.l) {
                let (x, y) = self.proj(a.f, a.l, g);
                fb.hline(x as i32 - 3 + ox, x as i32 + 3 + ox, y as i32 + oy, BLACK);
                fb.hline(
                    x as i32 - 2 + ox,
                    x as i32 + 2 + ox,
                    y as i32 + 1 + oy,
                    BLACK,
                );
            }
        }
        for s in &self.eshots {
            if let Some(g) = self.ground(s.f, s.l) {
                let (x, y) = self.proj(s.f, s.l, g);
                fb.set(x as i32 + ox, y as i32 + oy, BLACK);
                fb.set(x as i32 + 1 + ox, y as i32 + oy, BLACK);
            }
        }
        // Ground structures, far to near, split in narrow slices.
        let mut items: Vec<(f32, usize, f32, f32)> = Vec::new();
        for (i, o) in self.objs.iter().enumerate() {
            let mut l = o.l0;
            while l < o.l1 - 0.01 {
                let l1 = (l + 16.0).min(o.l1);
                items.push(((o.f0 + o.f1) / 2.0 - (l + l1) / 2.0, i, l, l1));
                l = l1;
            }
        }
        items.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (_, i, l0, l1) in items {
            self.draw_obj(fb, &self.objs[i], l0, l1, ox, oy);
        }
        // The boss tower stands on the deck.
        self.draw_boss(fb, ox, oy);
        // Air: sorted far to near.
        let mut air: Vec<(f32, u8, usize)> = Vec::new();
        for (i, a) in self.air.iter().enumerate() {
            air.push((a.f - a.l, 0, i));
        }
        for (i, p) in self.pickups.iter().enumerate() {
            air.push((p.f - p.l, 1, i));
        }
        if self.dead == 0 {
            air.push((pf - self.l, 2, 0));
        }
        air.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (_, k, i) in air {
            match k {
                0 => self.draw_air(fb, &self.air[i], ox, oy),
                1 => {
                    let p = &self.pickups[i];
                    let (x, y) = self.proj(p.f, p.l, p.z + (p.t as f32 * 0.1).sin() * 2.0);
                    draw_capsule(fb, x as i32 + ox, y as i32 + oy, p.kind, p.t);
                }
                _ => {
                    if !(self.inv > 0 && self.inv < 140 && (self.t / 3) % 2 == 0) {
                        self.draw_plane(fb, pf, self.l, self.z, ox, oy, false, false, 1.35);
                    }
                }
            }
        }
        for s in &self.shots {
            let (x, y) = self.proj(s.f, s.l, s.z);
            let (x, y) = (x as i32 + ox, y as i32 + oy);
            fb.line(x - 6, y + 3, x, y, ORANGE);
            fb.line(x - 6, y + 2, x, y - 1, YELLOW);
            fb.line(x - 3, y + 1, x + 1, y - 1, WHITE);
        }
        for s in &self.eshots {
            let (x, y) = self.proj(s.f, s.l, s.z);
            draw_bullet(fb, x as i32 + ox, y as i32 + oy, self.t, s.z < 4.0);
        }
        self.fx.draw(fb, ox, oy);
        if let Some((s, t0)) = self.banner {
            let k = self.t - t0;
            if k < 170 && (k > 20 || k % 4 < 2) {
                fb.text_center_sh(
                    30,
                    s,
                    if k % 16 < 8 { WHITE } else { YELLOW },
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
        // Altimeter.
        let (ax, ay0, ay1) = (4, 40, 150);
        fb.text(1, ay0 - 10, "A", LGREY);
        fb.rect(ax - 1, ay0 - 1, 6, ay1 - ay0 + 2, BLACK);
        fb.frame(ax - 1, ay0 - 1, 6, ay1 - ay0 + 2, GREY);
        let zy = ay1 - ((self.z - ZMIN) / (ZMAX - ZMIN) * (ay1 - ay0) as f32) as i32;
        for y in (ay0..ay1).step_by(8) {
            fb.set(ax + 3, y, DGREY);
        }
        fb.rect(ax, zy - 1, 4, 3, if self.z < 8.0 { LRED } else { YELLOW });
        draw_hud(fb, st, hi, self.spec.hud);
        for a in &self.air {
            if a.kind == Ak::Sentinel {
                draw_boss_bar(fb, "SENTINEL", a.hp, 50, self.t);
            }
        }
        if let Some(b) = &self.boss {
            if b.dying == 0 {
                let shard: i32 = self
                    .air
                    .iter()
                    .filter(|a| a.kind == Ak::Shard)
                    .map(|a| a.hp.max(0))
                    .sum();
                draw_boss_bar(
                    fb,
                    self.spec.boss_name,
                    b.core_hp + shard,
                    b.core_max + 14 * self.spec.shards as i32,
                    self.t,
                );
            }
        }
    }

    fn bot(&mut self, st: &Stats) -> Input {
        let mut i = Input {
            fire: true,
            ..Default::default()
        };
        let pf = self.pf();
        let mut tl = 48.0;
        let mut tz = 20.0;
        // The nearest target ahead.
        let mut best = f32::MAX;
        for a in &self.air {
            let df = a.f - pf;
            if df > 10.0 && df < best {
                best = df;
                tl = a.l;
                tz = a.z;
            }
        }
        for o in &self.objs {
            let df = o.f0 - pf;
            if o.hp > 0 && df > 10.0 && df < best {
                best = df;
                tl = (o.l0 + o.l1) / 2.0;
                tz = 6.0;
            }
        }
        if let Some(b) = &self.boss {
            if !self.air.iter().any(|a| a.kind == Ak::Shard) {
                tl = b.l;
                tz = 30.0;
            }
        }
        if let Some(p) = self.pickups.first() {
            if p.f > pf {
                tl = p.l;
                tz = p.z;
            }
        }
        // Pick the best place to be: aligned with the target, clear of
        // everything the course will put in the way (moving walls and
        // switching fences as they will be when they arrive), and away
        // from shots.
        let t = self.t;
        let mut best = (f32::MAX, self.l, self.z);
        for li in 0..14 {
            let l = 3.0 + li as f32 * 6.9;
            for zi in 0..11 {
                let z = ZMIN + 1.0 + zi as f32 * 5.0;
                let need = ((l - self.l).abs() / 1.3).max((z - self.z).abs() / 0.9);
                let mut cost = (l - tl).powi(2) * 0.4 + (z - tz).powi(2) * 0.3 + need * 1.5;
                for o in &self.objs {
                    let df = o.f0 - pf;
                    if !(-10.0..140.0).contains(&df) {
                        continue;
                    }
                    let arrive = ((df - 8.0) / CAM_SPEED).max(0.0);
                    let (mut l0, mut l1) = (o.l0, o.l1);
                    if o.vl != 0.0 {
                        // A moving wall could be anywhere by then: go over.
                        l0 = -20.0;
                        l1 = LW + 20.0;
                    }
                    let (z0, z1) = if o.switch > 0 {
                        let ta = t + arrive as u32;
                        let high = (ta / o.switch) % 2 == 1;
                        // Near a switch either state may hold: avoid both.
                        let edge = (ta % o.switch) < 25 || (ta % o.switch) > o.switch - 25;
                        if edge {
                            (0.0, ZMAX + 4.0)
                        } else if high {
                            (30.0, ZMAX + 4.0)
                        } else {
                            (0.0, 24.0)
                        }
                    } else {
                        o.z_range()
                    };
                    if l + 9.0 >= l0 && l - 9.0 <= l1 && z + 5.0 >= z0 && z - 4.0 <= z1 {
                        cost += 20000.0 / (1.0 + arrive * 0.05);
                    }
                    // Not reachable before it arrives, or the way there
                    // crosses it.
                    let hits_now = self.l + 5.0 >= l0
                        && self.l - 5.0 <= l1
                        && self.z + 3.0 >= z0
                        && self.z - 2.0 <= z1;
                    if hits_now && need + 6.0 > arrive {
                        cost += 8000.0;
                    }
                }
                for sh in &self.eshots {
                    for k in [5.0f32, 15.0, 30.0] {
                        let (sf, sl, sz) = (sh.f + sh.vf * k, sh.l + sh.vl * k, sh.z + sh.vz * k);
                        if (sf - (pf + CAM_SPEED * k)).abs() < 8.0
                            && (sl - l).abs() < 7.0
                            && (sz - z).abs() < 7.0
                        {
                            cost += 3000.0 / k;
                        }
                    }
                }
                if cost < best.0 {
                    best = (cost, l, z);
                }
            }
        }
        tl = best.1;
        tz = best.2;
        i.left = tl < self.l - 1.5;
        i.right = tl > self.l + 1.5;
        i.up = tz > self.z + 1.0;
        i.down = tz < self.z - 1.0;
        i.p_special = st.bombs > 0 && self.eshots.len() > 12 && self.t % 40 == 0;
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
        ((self.cam - self.off) / self.spec.boss_at).min(1.0)
    }

    fn info(&self) -> String {
        match &self.boss {
            Some(b) => format!(
                "boss core {} shards {} dying {}",
                b.core_hp,
                self.air.iter().filter(|a| a.kind == Ak::Shard).count(),
                b.dying
            ),
            None => format!(
                "last hit by {} cam {:.0} objs {} air {}",
                self.last_cause,
                self.cam,
                self.objs.len(),
                self.air.len()
            ),
        }
    }
}

impl Iso {
    /// The deck and the dunes, worked out for every pixel in fixed point
    /// from textures made once.
    fn draw_ground(&self, fb: &mut Fb, ox: i32, oy: i32) {
        let tex = textures();
        let gmap = self.spec.ground;
        let cam = self.cam;
        let t = self.t;
        let blink = (t / 20) as usize;
        let sparkle_on = t % 12 < 6;
        // Per column: how far below the last deck pixel we are, and where
        // that deck pixel was (for the face where the deck drops away).
        let mut since = [i32::MAX; W as usize];
        let mut edge = [0i32; W as usize];
        let mut tile = ((i32::MIN, 0), false, 0usize);
        const ONE: f32 = 65536.0;
        // Ground coordinates grow by half a unit per pixel along a row; the
        // dunes far below move at 0.45 of that.
        let step = (0.5 * ONE) as i32;
        let dstep = (0.5 * 0.45 * ONE) as i32;
        for y in HUD_H - DROP..H {
            let yy = (y - oy) as f32 + 0.5;
            let ya = OY - yy;
            let xa0 = (-ox) as f32 + 0.5 - OX;
            let df = (xa0 + 2.0 * ya) * 0.5;
            let l = (xa0 - 2.0 * ya) * 0.5;
            let mut ff = ((cam + df) * ONE) as i32;
            let offf = (self.off * ONE) as i32;
            let mut lf = (l * ONE) as i32;
            let mut pf = ((cam + df) * 0.45 * ONE) as i32;
            let mut pl = (l * 0.45 * ONE) as i32;
            let row = (y * W) as usize;
            for x in 0..W {
                let fi = ff >> 20;
                let li = lf >> 20;
                let xi = x as usize;
                if (fi, li) != tile.0 {
                    let h = hash(fi, li) as usize;
                    tile = (
                        (fi, li),
                        (self.spec.deck)(((ff - offf) >> 20) as i32, li),
                        (h % 11) * 2 + ((fi + li) & 1) as usize,
                    );
                }
                let c = if tile.1 {
                    since[xi] = 0;
                    edge[xi] = ((ff >> 18) + (lf >> 18)) & 1;
                    let fu = ((ff >> 16) & 15) as usize;
                    let lu = ((lf >> 16) & 15) as usize;
                    let mut c = tex.deck[tile.2][(lu << 4) | fu];
                    if c == RED && (blink + tile.2) % 4 == 0 {
                        c = LRED;
                    }
                    c
                } else {
                    since[xi] = since[xi].saturating_add(1);
                    let k = since[xi];
                    if k <= DROP {
                        if k == 1 {
                            LGREY
                        } else if k > DROP - 3 {
                            BLACK
                        } else if edge[xi] == 0 {
                            DGREY
                        } else if k % 6 == 3 {
                            BROWN
                        } else {
                            BLACK
                        }
                    } else {
                        let d =
                            tex.dunes[((((pl >> 16) & 255) << 8) | ((pf >> 16) & 255)) as usize];
                        if d & 0x80 != 0 && (t / 6 + (pf >> 16) as u32) % 5 == 0 {
                            if sparkle_on {
                                WHITE
                            } else {
                                CYAN
                            }
                        } else {
                            let band = d & 0x7f;
                            let odd = (x + y) & 1 == 0;
                            if band < 2 {
                                BROWN
                            } else if band < 4 {
                                if odd {
                                    BROWN
                                } else {
                                    ORANGE
                                }
                            } else if band < 9 || odd {
                                ORANGE
                            } else {
                                YELLOW
                            }
                        }
                    }
                };
                if y >= HUD_H {
                    fb.px[row + xi] = gmap[(c & 15) as usize];
                }
                ff += step;
                lf += step;
                pf += dstep;
                pl += dstep;
            }
        }
    }

    /// A box with its top and the two faces that turn towards the viewer.
    #[allow(clippy::too_many_arguments)]
    fn iso_box(
        &self,
        fb: &mut Fb,
        f0: f32,
        f1: f32,
        l0: f32,
        l1: f32,
        z0: f32,
        z1: f32,
        cols: [u8; 4],
        ox: i32,
        oy: i32,
    ) {
        let p = |f: f32, l: f32, z: f32| {
            let (x, y) = self.proj(f, l, z);
            (x + ox as f32, y + oy as f32)
        };
        let top = [p(f0, l0, z1), p(f1, l0, z1), p(f1, l1, z1), p(f0, l1, z1)];
        let left = [p(f0, l0, z0), p(f0, l1, z0), p(f0, l1, z1), p(f0, l0, z1)];
        let right = [p(f0, l1, z0), p(f1, l1, z0), p(f1, l1, z1), p(f0, l1, z1)];
        fb.poly(&left, cols[2]);
        fb.poly(&right, cols[1]);
        fb.poly(&top, cols[0]);
        let e = cols[3];
        let ln = |fb: &mut Fb, a: (f32, f32), b: (f32, f32), c: u8| {
            fb.line(a.0 as i32, a.1 as i32, b.0 as i32, b.1 as i32, c)
        };
        ln(fb, top[0], top[1], e);
        ln(fb, top[0], top[3], e);
        ln(fb, top[3], top[2], cols[1]);
        ln(fb, top[3], left[1], e);
    }

    fn draw_obj(&self, fb: &mut Fb, o: &Obj, l0: f32, l1: f32, ox: i32, oy: i32) {
        let fl = o.flash > 0 && o.flash % 2 == 0;
        let w = |c: [u8; 4]| if fl { [WHITE; 4] } else { c };
        match o.kind {
            Ok::Wall => {
                self.iso_box(
                    fb,
                    o.f0,
                    o.f1,
                    l0,
                    l1,
                    0.0,
                    o.h,
                    w([LGREY, GREY, DGREY, WHITE]),
                    ox,
                    oy,
                );
                // Warning stripes along the top.
                for k in 0..((l1 - l0) as i32 / 4) {
                    if k % 2 == 0 {
                        let l = l0 + k as f32 * 4.0 + 2.0;
                        let (x, y) = self.proj(o.f0, l, o.h - 3.0);
                        fb.set(x as i32 + ox, y as i32 + oy, YELLOW);
                        fb.set(x as i32 + ox + 1, y as i32 + oy, YELLOW);
                    }
                }
            }
            Ok::Tower => {
                self.iso_box(
                    fb,
                    o.f0,
                    o.f1,
                    l0,
                    l1,
                    0.0,
                    o.h,
                    w([LBLUE, BLUE, BLACK, CYAN]),
                    ox,
                    oy,
                );
                for z in (6..o.h as i32).step_by(8) {
                    let (x, y) = self.proj(o.f0, (l0 + l1) / 2.0, z as f32);
                    let on = (self.t / 10 + z as u32) % 4 != 0;
                    fb.set(
                        x as i32 + ox,
                        y as i32 + oy,
                        if on { YELLOW } else { DGREY },
                    );
                }
                let (x, y) = self.proj((o.f0 + o.f1) / 2.0, (l0 + l1) / 2.0, o.h + 2.0);
                if self.t % 40 < 20 {
                    fb.disc(x as i32 + ox, y as i32 + oy, 1, LRED);
                }
            }
            Ok::Pylon => {
                self.iso_box(
                    fb,
                    o.f0,
                    o.f1,
                    l0,
                    l1,
                    -(DROP as f32) * 0.5,
                    o.h,
                    [PURPLE, BLUE, BLACK, LRED],
                    ox,
                    oy,
                );
                let (x, y) = self.proj((o.f0 + o.f1) / 2.0, (l0 + l1) / 2.0, o.h + 3.0);
                fb.disc(
                    x as i32 + ox,
                    y as i32 + oy,
                    2,
                    if self.t % 16 < 8 { WHITE } else { CYAN },
                );
            }
            Ok::Fence { z0, z1 } => {
                // Crackling energy lines between the pylons.
                let mut z = z0 + 1.0;
                let mut k = 0;
                while z < z1 {
                    let (xa, ya) = self.proj(o.f0, l0, z);
                    let (xb, yb) = self.proj(o.f0, l1, z);
                    let c = if (self.t / 2 + k) % 5 == 0 {
                        WHITE
                    } else if k % 2 == 0 {
                        CYAN
                    } else {
                        LBLUE
                    };
                    let jit = if (self.t + k * 7) % 9 == 0 { 1 } else { 0 };
                    fb.line(
                        xa as i32 + ox,
                        ya as i32 + oy + jit,
                        xb as i32 + ox,
                        yb as i32 + oy,
                        c,
                    );
                    z += 3.0;
                    k += 1;
                }
                // The edges glow.
                let (xa, ya) = self.proj(o.f0, l0, z0);
                let (xb, yb) = self.proj(o.f0, l1, z0);
                fb.line(
                    xa as i32 + ox,
                    ya as i32 + oy,
                    xb as i32 + ox,
                    yb as i32 + oy,
                    WHITE,
                );
                let (xa, ya) = self.proj(o.f0, l0, z1);
                let (xb, yb) = self.proj(o.f0, l1, z1);
                fb.line(
                    xa as i32 + ox,
                    ya as i32 + oy,
                    xb as i32 + ox,
                    yb as i32 + oy,
                    WHITE,
                );
            }
            Ok::Turret => {
                self.iso_box(
                    fb,
                    o.f0,
                    o.f1,
                    o.l0,
                    o.l1,
                    0.0,
                    o.h - 3.0,
                    w([GREY, DGREY, BLACK, LGREY]),
                    ox,
                    oy,
                );
                let (cf, cl) = ((o.f0 + o.f1) / 2.0, (o.l0 + o.l1) / 2.0);
                let (x, y) = self.proj(cf, cl, o.h - 1.0);
                let (x, y) = (x as i32 + ox, y as i32 + oy);
                fb.disc(x, y, 4, if fl { WHITE } else { RED });
                fb.disc(x - 1, y - 1, 1, LRED);
                // The barrel points at the player.
                let (px, py) = self.proj(self.pf(), self.l, self.z);
                let (dx, dy) = (px - x as f32, py - y as f32);
                let d = (dx * dx + dy * dy).sqrt().max(1.0);
                fb.line(
                    x,
                    y,
                    x + (dx / d * 7.0) as i32,
                    y + (dy / d * 7.0) as i32,
                    LGREY,
                );
            }
            Ok::Silo(p) => {
                let (dark, light) = p.colors();
                self.iso_box(
                    fb,
                    o.f0,
                    o.f1,
                    o.l0,
                    o.l1,
                    0.0,
                    o.h,
                    w([light, dark, BLACK, WHITE]),
                    ox,
                    oy,
                );
                let (x, y) = self.proj((o.f0 + o.f1) / 2.0, (o.l0 + o.l1) / 2.0, o.h);
                fb.char(
                    x as i32 - 4 + ox,
                    y as i32 - 6 + oy,
                    p.letter(),
                    if self.t % 20 < 10 { WHITE } else { YELLOW },
                );
            }
        }
    }

    /// A delta-winged ship (or its shadow) seen from above at an angle.
    #[allow(clippy::too_many_arguments)]
    fn draw_plane(
        &self,
        fb: &mut Fb,
        f: f32,
        l: f32,
        z: f32,
        ox: i32,
        oy: i32,
        shadow: bool,
        enemy: bool,
        scale: f32,
    ) {
        let p = |df: f32, dl: f32, dz: f32| {
            let (x, y) = self.proj(f + df, l + dl, z + dz);
            (x + ox as f32, y + oy as f32)
        };
        let dir = if enemy { -1.0 } else { 1.0 };
        let bank = if enemy { 0.0 } else { self.bank * 2.0 };
        let k = scale;
        let nose = p(10.0 * dir * k, 0.0, 0.0);
        let wl = p(-6.0 * dir * k, -9.0 * k, bank);
        let wr = p(-6.0 * dir * k, 9.0 * k, -bank);
        let tail = p(-3.0 * dir * k, 0.0, 0.0);
        if shadow {
            fb.poly(&[nose, wl, tail, wr], BLACK);
            return;
        }
        let (c1, c2, c3) = if enemy {
            (LRED, RED, BROWN)
        } else {
            (WHITE, LGREY, GREY)
        };
        // Thickness: the underside one pixel lower.
        let d = |q: (f32, f32)| (q.0, q.1 + 1.0);
        fb.poly(&[d(nose), d(wl), d(tail), d(wr)], DGREY);
        fb.poly(&[nose, wl, tail], c1);
        fb.poly(&[nose, tail, wr], c2);
        let fin0 = p(-5.0 * dir, 0.0, 0.0);
        let fin1 = p(-3.0 * dir, 0.0, 5.0);
        fb.line(
            fin0.0 as i32,
            fin0.1 as i32,
            fin1.0 as i32,
            fin1.1 as i32,
            c3,
        );
        let cock = p(3.0 * dir, 0.0, 1.0);
        fb.set(
            cock.0 as i32,
            cock.1 as i32,
            if enemy { YELLOW } else { CYAN },
        );
        fb.set(
            cock.0 as i32 + 1,
            cock.1 as i32,
            if enemy { YELLOW } else { LBLUE },
        );
        if !enemy {
            let e = p(-5.0, 0.0, 0.0);
            let c = [YELLOW, ORANGE, WHITE][(self.t % 3) as usize];
            fb.set(e.0 as i32 - 1, e.1 as i32 + 1, c);
            fb.set(e.0 as i32 - 2, e.1 as i32 + 1, ORANGE);
        }
    }

    fn draw_air(&self, fb: &mut Fb, a: &Air, ox: i32, oy: i32) {
        let fl = a.flash > 0 && a.flash % 2 == 0;
        match a.kind {
            Ak::Kite => {
                if fl {
                    let (x, y) = self.proj(a.f, a.l, a.z);
                    fb.disc(x as i32 + ox, y as i32 + oy, 5, WHITE);
                } else {
                    self.draw_plane(fb, a.f, a.l, a.z, ox, oy, false, true, 1.0);
                }
            }
            Ak::Saucer => {
                let (x, y) = self.proj(a.f, a.l, a.z);
                let (x, y) = (x as i32 + ox, y as i32 + oy);
                for dy in -2i32..=2 {
                    let w = 7 - dy.abs() * 2;
                    fb.hline(
                        x - w,
                        x + w,
                        y + dy,
                        if fl {
                            WHITE
                        } else if dy < 0 {
                            LGREEN
                        } else {
                            GREEN
                        },
                    );
                }
                fb.disc(x, y - 3, 2, if fl { WHITE } else { CYAN });
                let k = (self.t / 4 + a.base as u32) % 4;
                fb.set(x - 6 + k as i32 * 4, y, YELLOW);
            }
            Ak::Sentinel | Ak::Bomber => {
                let k = if a.kind == Ak::Sentinel { 2.4 } else { 1.5 };
                if fl {
                    let (x, y) = self.proj(a.f, a.l, a.z);
                    fb.disc(x as i32 + ox, y as i32 + oy, (5.0 * k) as i32, WHITE);
                } else {
                    self.draw_plane(fb, a.f, a.l, a.z, ox, oy, false, true, k);
                }
            }
            Ak::Shard => {
                let (x, y) = self.proj(a.f, a.l, a.z);
                let (x, y) = (x as i32 + ox, y as i32 + oy);
                let c = if fl { WHITE } else { CYAN };
                fb.poly(
                    &[
                        (x as f32, y as f32 - 9.0),
                        (x as f32 + 5.0, y as f32),
                        (x as f32, y as f32 + 7.0),
                        (x as f32 - 5.0, y as f32),
                    ],
                    c,
                );
                fb.poly(
                    &[
                        (x as f32, y as f32 - 9.0),
                        (x as f32 + 5.0, y as f32),
                        (x as f32, y as f32 + 7.0),
                    ],
                    if fl { WHITE } else { LBLUE },
                );
                fb.line(x, y - 9, x, y + 7, WHITE);
            }
        }
    }

    fn draw_boss(&self, fb: &mut Fb, ox: i32, oy: i32) {
        let Some(b) = &self.boss else {
            return;
        };
        let bf = self.cam + 210.0;
        let (f0, l0) = (bf - 12.0, b.l - 12.0);
        let shake = if b.dying > 0 {
            ((b.dying % 4) as i32 - 2) as f32
        } else {
            0.0
        };
        let fl = b.flash > 0 && b.flash % 2 == 0;
        let cols = if fl { [WHITE; 4] } else { self.spec.crystal };
        // A tall crystal: three stacked boxes tapering upwards.
        self.iso_box(
            fb,
            f0,
            f0 + 24.0,
            l0 + shake,
            l0 + 24.0 + shake,
            0.0,
            22.0,
            [LGREY, GREY, DGREY, WHITE],
            ox,
            oy,
        );
        self.iso_box(
            fb,
            f0 + 3.0,
            f0 + 21.0,
            l0 + 3.0,
            l0 + 21.0,
            22.0,
            52.0,
            cols,
            ox,
            oy,
        );
        self.iso_box(
            fb,
            f0 + 7.0,
            f0 + 17.0,
            l0 + 7.0,
            l0 + 17.0,
            52.0,
            66.0,
            [WHITE, CYAN, LBLUE, WHITE],
            ox,
            oy,
        );
        // The core window on the front face.
        let open = !self.air.iter().any(|a| a.kind == Ak::Shard);
        let (x, y) = self.proj(f0 + 3.0, b.l + 2.0, 30.0);
        let (x, y) = (x as i32 + ox, y as i32 + oy);
        if open {
            let c = [RED, LRED, ORANGE, YELLOW, WHITE, YELLOW, ORANGE, LRED]
                [((self.t / 3) % 8) as usize];
            fb.disc(x, y, 6, BLACK);
            fb.disc(x, y, 4, c);
        } else {
            fb.disc(x, y, 5, DGREY);
            fb.circle(x, y, 5, LGREY);
        }
    }
}

struct Textures {
    /// 16x16 deck tiles: (variant * 2 + light or dark) by (lu * 16 + fu).
    deck: Vec<[u8; 256]>,
    /// 256x256 dunes: the band (0 to 11), bit 7 for a glint of glass.
    dunes: Vec<u8>,
}

fn textures() -> &'static Textures {
    use std::sync::OnceLock;
    static T: OnceLock<Textures> = OnceLock::new();
    T.get_or_init(|| {
        let mut deck = Vec::new();
        for variant in 0..11 {
            for light in 0..2 {
                let base = if light == 0 { GREY } else { LGREY };
                let mut tx = [0u8; 256];
                for lu in 0..16usize {
                    for fu in 0..16usize {
                        let (f, l) = (fu as f32 + 0.5, lu as f32 + 0.5);
                        let c = if fu == 0 || lu == 15 {
                            DGREY
                        } else if lu == 0 || fu == 15 {
                            if base == LGREY {
                                WHITE
                            } else {
                                LGREY
                            }
                        } else {
                            match variant {
                                0 if (4..12).contains(&fu) && (4..12).contains(&lu) => {
                                    if fu % 2 == 0 {
                                        DGREY
                                    } else {
                                        base
                                    }
                                }
                                1 if (6..10).contains(&lu) => {
                                    let k = (f - (l - 8.0).abs()) as i32;
                                    if (6..9).contains(&k) {
                                        YELLOW
                                    } else {
                                        base
                                    }
                                }
                                2 if ((f - 8.0).powi(2) + (l - 8.0).powi(2)) < 4.0 => RED,
                                _ => {
                                    if base == GREY && (fu + lu) % 2 == 0 && variant % 3 == 0 {
                                        LGREY
                                    } else {
                                        base
                                    }
                                }
                            }
                        };
                        tx[(lu << 4) | fu] = c;
                    }
                }
                deck.push(tx);
            }
        }
        let mut dunes = vec![0u8; 65536];
        let tau = std::f32::consts::TAU;
        for v in 0..256usize {
            for u in 0..256usize {
                let (uf, vf) = (u as f32, v as f32);
                let d = (tau * uf / 128.0).sin() * 6.0
                    + (tau * (vf / 64.0 + uf / 256.0)).sin() * 9.0
                    + vf * (72.0 / 256.0);
                let band = (d as i32).rem_euclid(12) as u8;
                let glint = hash(u as i32, v as i32) % 97 == 0;
                dunes[(v << 8) | u] = band | if glint { 0x80 } else { 0 };
            }
        }
        Textures { deck, dunes }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_round_trips() {
        let lv = Iso::new(Difficulty::Normal, 3);
        let (x, y) = lv.proj(40.0, 48.0, 0.0);
        let (xa, ya) = (x - OX, OY - y);
        let df = (xa + 2.0 * ya) * 0.5;
        let l = (xa - 2.0 * ya) * 0.5;
        assert!((df - 40.0).abs() < 0.01 && (l - 48.0).abs() < 0.01);
        assert!(deck(0, 3));
        assert!(!deck(0, 7));
    }
}
