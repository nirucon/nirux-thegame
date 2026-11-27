//! Sector 3, "Iron Jungle": a top-down run and gun on foot. Fight up the
//! jungle road past gun nests and riflemen, cross the river, survive the
//! tank at the outpost and break the Gatekeeper at the fortress gate.
use super::common::*;
use super::Level;
use crate::art;
use crate::config::Difficulty;
use crate::fb::{Fb, H, W};
use crate::input::Input;
use crate::palette::*;
use crate::sfx::Sfx;
use crate::songs::SongId;

const T: i32 = 16;
const COLS: usize = 20;
const VIEW: i32 = H - HUD_H;

/// The sector from the fortress (top) down to the landing zone (bottom).
const MAP: &[&str] = &[
    "%%%%%%%%%%%%%%%%%%%%",
    "%##################%",
    "%#................#%",
    "%#................#%",
    "%#.......X........#%",
    "%#................#%",
    "%#................#%",
    "%#.s............s.#%",
    "%#.s............s.#%",
    "%#................#%",
    "%##....r....r....##%",
    "%###.....==.....###%",
    "%%%%.....==.....%%%%",
    "TTT%.....==.....%TTT",
    "TT.:..r..==..r..:.TT",
    "T..o.....==.....o..T",
    "T.sss....==....sss.T",
    "T........==........T",
    "TT..n....==....n..TT",
    "T........==........T",
    "T.:...g..==..g...:.T",
    "TT...o...==...o...TT",
    "T...ss...==...ss...T",
    "T..r.....==.....r..T",
    "TT..c....==....c..TT",
    "T.....r..==..r.....T",
    "T.:......==......:.T",
    "TTT..p...==...p..TTT",
    "..%......==......%..",
    "~~~BB~~~~BB~~~~BB~~~",
    "~~~BB~~~~BB~~~~BB~~~",
    "~~~BB~~~~BB~~~~BB~~~",
    "~~~BB~~~~BB~~~~BB~~~",
    "%..r.....==.....r..%",
    "T..ss....==....ss..T",
    "T........k=........T",
    "TT..HH...==...HH..TT",
    "T...HH...==...HH...T",
    "T..r.....==.....r..T",
    "T.s.s.s..==..s.s.s.T",
    "T........==........T",
    "TT.n.....==.....n.TT",
    "T..:..g..==..g..:..T",
    "T....c...==...c....T",
    "TT.......==.......TT",
    "T..r..o..==..o..r..T",
    "T.ss.....==.....ss.T",
    "TT.......==.......TT",
    "TTT..p...==...p..TTT",
    "TT..:....==....:..TT",
    "T...r....==....r...T",
    "T..o..%..==..%..o..T",
    "T.c......==......c.T",
    "TT...:...==...:...TT",
    "TTT..r...==...r..TTT",
    "TT..p....==....p..TT",
    "T....:...==...:....T",
    "T..%.....==.....%..T",
    "T.....r..==..r.....T",
    "TT.:.....==.....:.TT",
    "TTT..o...==...o..TTT",
    "TT.......==.......TT",
    "T..p.....==.....p..T",
    "T....:...==...:....T",
    "T........==........T",
    "TT.......==.......TT",
    "TTT......==......TTT",
    "TTTT.....==.....TTTT",
];

/// Sector 6: the burning city, from the plaza (top) down to the outskirts.
const CITY: &[&str] = &[
    "HHHHHHHH....HHHHHHHH",
    "HHHHHHHH....HHHHHHHH",
    "#..................#",
    "#.........X........#",
    "#..................#",
    "#..................#",
    "#..s.....==.....s..#",
    "#..s.....==.....s..#",
    "#........==........#",
    "##..r....==....r..##",
    "HHHH.....==.....HHHH",
    "HHHq.....==.....qHHH",
    "HHHH..F..==..%..HHHH",
    "RRRRRRRRRRRRRRRRRRRR",
    "....r....==....r....",
    "..ss.....==.....ss..",
    "HHH......==......HHH",
    "HHq..g...==...g..qHH",
    "HHH..c...==...c..HHH",
    ".....%...==..F......",
    "..n......==......n..",
    ".........==.........",
    "HHHH...r.==.r...HHHH",
    "HHHq.....==.....qHHH",
    "HHHH..s..==..s..HHHH",
    "....F.......%.......",
    "RRRRRRRRRRRRRRRRRRRR",
    ".........==.........",
    "..%..r...==...r..%..",
    "HHHH.....k=.....HHHH",
    "HHHq.....==.....qHHH",
    "HHHH.....==.....HHHH",
    "..s.s.s..==..s.s.s..",
    ".........==.........",
    "T.n.....F==.......nT",
    "..:..g...==...g..:..",
    "HHHH.c...==...c.HHHH",
    "HHHH.....==.....HHHH",
    "...r..%..==..%..r...",
    ".ss......==......ss.",
    "RRRRRRRRRRRRRRRRRRRR",
    ".........==.........",
    "T..%..r..==..r..%..T",
    "HHHH.....==.....HHHH",
    "HHHq.....==.....qHHH",
    "HHHH..F..==..F..HHHH",
    "..:......==......:..",
    "T...r....==....r...T",
    "..c......==......c..",
    "HHHHH....==....HHHHH",
    "HHHHq....==....qHHHH",
    "HHHHH....==....HHHHH",
    "....%....==....%....",
    "T..r.....==.....r..T",
    "..s......==......s..",
    ".........==.........",
    "T...:....==....:...T",
    "HHHH.....==.....HHHH",
    "HHHH..r..==..r..HHHH",
    "HHHH.....==.....HHHH",
    "..F......==......%..",
    ".........==.........",
    "T.......:==:.......T",
    "TT.......==.......TT",
];

/// Sector 8, stage three: the heart of the NIRUX core.
const CORE: &[&str] = &[
    "####################",
    "#..................#",
    "#..................#",
    "#.........X........#",
    "#..................#",
    "#..................#",
    "#.%..............%.#",
    "#..................#",
    "#..s....====....s..#",
    "##.r............r.##",
    "###......==......###",
    "#F#......==......#F#",
    "#....r...==...r....#",
    "#..%.....==.....%..#",
    "#..n.....==.....n..#",
    "#........==........#",
    "##..g....==....g..##",
    "#...ss...==...ss...#",
    "#.c......==......c.#",
    "#....r...==...r....#",
    "#F.......==.......F#",
    "##..%....k=....%..##",
    "#........==........#",
    "#..r.....==.....r..#",
    "#.sss....==....sss.#",
    "#..n.....==.....n..#",
    "#........==........#",
    "##.r..%..==..%..r.##",
    "#........==........#",
    "#.c......==......c.#",
    "#F..g....==....g..F#",
    "#........==........#",
    "##..r....==....r..##",
    "#........==........#",
    "#..%.....==.....%..#",
    "#........==........#",
    "#........==........#",
    "##.......==.......##",
];

pub struct FootStyle {
    pub name: &'static str,
    /// Hull, plates, guns, gun light.
    pub cols: [u8; 4],
    pub guns: i32,
    pub core: i32,
    /// 0 the crawler, 1 a gunship with rotors, 2 the core.
    pub shape: u8,
}

pub const FOOT_STYLES: [FootStyle; 3] = [
    FootStyle {
        name: "GATEKEEPER",
        cols: [DGREY, GREY, BLUE, LBLUE],
        guns: 24,
        core: 50,
        shape: 0,
    },
    FootStyle {
        name: "HELI GUNSHIP",
        cols: [BLUE, LBLUE, DGREY, GREY],
        guns: 28,
        core: 60,
        shape: 1,
    },
    FootStyle {
        name: "NIRUX CORE",
        cols: [PURPLE, LRED, RED, YELLOW],
        guns: 34,
        core: 90,
        shape: 2,
    },
];

/// What makes one on-foot sector different from another.
pub struct FootSpec {
    pub map: &'static [&'static str],
    /// 0 jungle, 1 city, 2 core.
    pub theme: u8,
    pub style: usize,
    pub border: u8,
    pub hud: u8,
    pub song: SongId,
    pub boss_song: SongId,
    pub intro: &'static str,
}

pub fn spec(sector: usize) -> FootSpec {
    match sector {
        5 => FootSpec {
            map: CITY,
            theme: 1,
            style: 1,
            border: RED,
            hud: LRED,
            song: SongId::Level6,
            boss_song: SongId::Boss,
            intro: "WATCH THE TRAINS AND THE ROOFS",
        },
        7 => FootSpec {
            map: CORE,
            theme: 2,
            style: 2,
            border: PURPLE,
            hud: LRED,
            song: SongId::Level8,
            boss_song: SongId::FinalBoss,
            intro: "STAGE THREE: THE HEART OF THE CORE",
        },
        _ => FootSpec {
            map: MAP,
            theme: 0,
            style: 0,
            border: GREEN,
            hud: LGREEN,
            song: SongId::Level3,
            boss_song: SongId::Boss,
            intro: "CLEAR THE ROAD TO THE GATE",
        },
    }
}

fn solid_walk(c: u8) -> bool {
    matches!(
        c,
        b'T' | b'p' | b's' | b'#' | b'%' | b'H' | b'c' | b'n' | b'~' | b'F' | b'Q'
    )
}

fn solid_shot(c: u8) -> bool {
    matches!(
        c,
        b'T' | b'p' | b's' | b'#' | b'%' | b'H' | b'c' | b'n' | b'F'
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Fk {
    Rifle,
    Grenadier,
    Nest,
    Tank,
    Sniper,
}

#[derive(Clone, Debug)]
struct Foe {
    kind: Fk,
    x: f32,
    y: f32,
    hp: i32,
    t: u32,
    cd: i32,
    fx: f32,
    fy: f32,
    tx: f32,
    ty: f32,
    flash: u8,
    walking: bool,
}

#[derive(Clone, Debug)]
struct Bullet {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: u32,
    big: bool,
}

#[derive(Clone, Debug)]
struct Grenade {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    t: u32,
    dur: u32,
    mine: bool,
}

impl Grenade {
    fn pos(&self) -> (f32, f32, f32) {
        let k = self.t as f32 / self.dur as f32;
        let x = self.x0 + (self.x1 - self.x0) * k;
        let y = self.y0 + (self.y1 - self.y0) * k;
        let z = 4.0 * 18.0 * k * (1.0 - k);
        (x, y, z)
    }
}

#[derive(Clone, Debug)]
struct Pickup {
    x: f32,
    y: f32,
    kind: PowerUp,
    t: u32,
}

struct Boss {
    x: f32,
    y: f32,
    t: u32,
    guns: [i32; 2],
    core: i32,
    flash: [u8; 3],
    dying: u32,
}

impl Boss {
    fn gun(&self, i: usize) -> (f32, f32) {
        (self.x + if i == 0 { -36.0 } else { 36.0 }, self.y + 14.0)
    }
    fn core(&self) -> (f32, f32) {
        (self.x, self.y + 8.0)
    }
    fn open(&self) -> bool {
        self.guns.iter().all(|&g| g <= 0)
    }
}

pub struct Foot {
    t: u32,
    agg: f32,
    diff: Difficulty,
    rng: Rng,
    fx: Fx,
    map: Vec<Vec<u8>>,
    crates: Vec<((usize, usize), i32)>,
    markers: Vec<(usize, usize, u8)>,
    cy: f32,
    px: f32,
    py: f32,
    fxd: f32,
    fyd: f32,
    walk: u32,
    inv: u32,
    dead: u32,
    fire_cd: u32,
    gren_cd: u32,
    muzzle: u32,
    foes: Vec<Foe>,
    shots: Vec<Bullet>,
    eshots: Vec<Bullet>,
    grenades: Vec<Grenade>,
    pickups: Vec<Pickup>,
    boss: Option<Boss>,
    boss_row: usize,
    clear_t: Option<u32>,
    over_t: Option<u32>,
    banner: Option<(&'static str, u32)>,
    tiles: Vec<[u8; 256]>,
    spec: FootSpec,
    /// Rail rows: (row, train x, warning frames left, cool down).
    trains: Vec<(usize, f32, u32, u32)>,
}

fn tile_index(c: u8, theme: u8) -> usize {
    if theme == 1 {
        return match c {
            b':' => 14,
            b'=' | b'k' => 18,
            b'H' | b'Q' => 12,
            b'R' => 13,
            b'%' | b'F' => 20,
            b's' => 5,
            b'#' => 6,
            b'n' => 10,
            _ => 11,
        };
    }
    if theme == 2 {
        return match c {
            b'=' | b'k' => 16,
            b'#' => 19,
            b'%' | b'F' => 17,
            b's' => 6,
            b'n' => 10,
            _ => 15,
        };
    }
    match c {
        b'.' | b'r' | b'g' | b'X' => 0,
        b':' => 1,
        b'=' | b'k' => 2,
        b'~' => 3,
        b'B' => 4,
        b's' => 5,
        b'#' => 6,
        b'%' => 7,
        b'H' => 8,
        b'n' => 10,
        _ => 0,
    }
}

/// The ground tiles, drawn once.
fn make_tiles() -> Vec<[u8; 256]> {
    // Every theme's tiles; tile_index picks.
    let mut v = Vec::new();
    for kind in 0..21 {
        let mut t = [0u8; 256];
        for y in 0..16i32 {
            for x in 0..16i32 {
                let h = hash(x + kind * 31, y * 7 + kind);
                let c = match kind {
                    0 => {
                        if h % 13 == 0 {
                            LGREEN
                        } else if h % 17 == 0 {
                            DGREY
                        } else {
                            GREEN
                        }
                    }
                    1 => {
                        // Tall grass tufts.
                        let tuft = (x % 5 == 1 || x % 5 == 3) && (y + x / 5) % 6 < 3;
                        if tuft {
                            if y % 6 == 0 {
                                LGREEN
                            } else {
                                DGREY
                            }
                        } else if h % 11 == 0 {
                            LGREEN
                        } else {
                            GREEN
                        }
                    }
                    2 => {
                        if h % 7 == 0 {
                            ORANGE
                        } else if h % 19 == 0 {
                            DGREY
                        } else {
                            BROWN
                        }
                    }
                    3 => BLUE,
                    4 => {
                        if y % 4 == 3 {
                            BLACK
                        } else if x == 0 || x == 15 || h % 9 == 0 {
                            BROWN
                        } else {
                            ORANGE
                        }
                    }
                    5 => {
                        // Two rows of sandbags.
                        let row = y / 8;
                        let bx = (x + row * 4) % 8;
                        let by = y % 8;
                        if by == 7 || bx == 7 {
                            BROWN
                        } else if by == 0 || bx == 0 || by < 3 {
                            YELLOW
                        } else {
                            ORANGE
                        }
                    }
                    6 => {
                        if y % 8 == 7 || (x + (y / 8) * 8) % 16 == 15 {
                            DGREY
                        } else if y % 8 == 0 || h % 9 == 0 {
                            LGREY
                        } else {
                            GREY
                        }
                    }
                    7 => {
                        let d = (x - 8) * (x - 8) + (y - 8) * (y - 8);
                        if d < 36 {
                            if x + y < 13 {
                                LGREY
                            } else {
                                GREY
                            }
                        } else if d < 56 {
                            DGREY
                        } else if h % 11 == 0 {
                            LGREEN
                        } else {
                            GREEN
                        }
                    }
                    8 => {
                        // Hut roof: planks.
                        if x % 4 == 3 || y == 0 || y == 15 || h % 6 == 0 {
                            BROWN
                        } else {
                            ORANGE
                        }
                    }
                    9 => {
                        // A small bush on grass.
                        let d = (x - 8) * (x - 8) + (y - 9) * (y - 9);
                        if d < 30 {
                            if h % 3 == 0 {
                                LGREEN
                            } else {
                                GREEN
                            }
                        } else if d < 40 {
                            DGREY
                        } else {
                            GREEN
                        }
                    }
                    11 => {
                        if h % 23 == 0 {
                            BLACK
                        } else if h % 9 == 0 {
                            GREY
                        } else {
                            DGREY
                        }
                    }
                    12 => {
                        if x == 0 || y == 0 {
                            LGREY
                        } else if x == 15 || y == 15 {
                            DGREY
                        } else if (5..9).contains(&x) && (5..9).contains(&y) {
                            if (x + y) % 2 == 0 {
                                DGREY
                            } else {
                                BLACK
                            }
                        } else if h % 11 == 0 {
                            LGREY
                        } else {
                            GREY
                        }
                    }
                    13 => {
                        if y == 4 || y == 11 {
                            LGREY
                        } else if x % 4 == 0 && (3..13).contains(&y) {
                            BROWN
                        } else if h % 5 == 0 {
                            GREY
                        } else {
                            DGREY
                        }
                    }
                    14 => {
                        if h % 5 == 0 {
                            GREY
                        } else if h % 7 == 0 {
                            LGREY
                        } else if h % 13 == 0 {
                            BROWN
                        } else {
                            DGREY
                        }
                    }
                    15 => {
                        if x % 8 == 0 || y % 8 == 0 {
                            DGREY
                        } else if (x % 8 == 2 && y % 8 == 2) || h % 17 == 0 {
                            LGREY
                        } else {
                            GREY
                        }
                    }
                    16 => {
                        if (6..10).contains(&x) {
                            if (x + y) % 4 == 0 {
                                CYAN
                            } else {
                                LBLUE
                            }
                        } else if x == 5 || x == 10 {
                            BLUE
                        } else {
                            DGREY
                        }
                    }
                    17 => {
                        if x == 0 || y == 0 {
                            LGREY
                        } else if x == 15 || y == 15 {
                            BLACK
                        } else if (4..12).contains(&x) && (4..12).contains(&y) {
                            if (x + y) % 3 == 0 {
                                LRED
                            } else {
                                PURPLE
                            }
                        } else {
                            DGREY
                        }
                    }
                    18 => {
                        if x == 0 && y % 8 < 4 {
                            YELLOW
                        } else if h % 9 == 0 {
                            GREY
                        } else {
                            DGREY
                        }
                    }
                    19 => {
                        if y == 0 {
                            LBLUE
                        } else if y % 8 == 7 || x % 16 == 15 {
                            BLACK
                        } else if h % 13 == 0 {
                            PURPLE
                        } else {
                            BLUE
                        }
                    }
                    20 => {
                        let d = (x - 8) * (x - 8) + (y - 8) * (y - 8);
                        if d < 30 {
                            if (x / 3 + y / 3) % 2 == 0 {
                                GREY
                            } else {
                                LGREY
                            }
                        } else if d < 45 {
                            BLACK
                        } else {
                            DGREY
                        }
                    }
                    _ => {
                        // Gun nest: a ring of sandbags around a pit.
                        let d = (x - 8) * (x - 8) + (y - 8) * (y - 8);
                        if d < 16 {
                            BLACK
                        } else if d < 30 {
                            if (x + y) % 3 == 0 {
                                BROWN
                            } else {
                                YELLOW
                            }
                        } else if d < 46 {
                            ORANGE
                        } else {
                            GREEN
                        }
                    }
                };
                t[(y * 16 + x) as usize] = c;
            }
        }
        v.push(t);
    }
    v
}

impl Foot {
    pub fn new(diff: Difficulty, seed: u32, sector: usize) -> Foot {
        let spec = spec(sector);
        let mut map: Vec<Vec<u8>> = spec
            .map
            .iter()
            .map(|r| {
                let mut v: Vec<u8> = r.bytes().collect();
                v.resize(COLS, b'T');
                v
            })
            .collect();
        let mut markers = Vec::new();
        let mut crates = Vec::new();
        let mut boss_row = 4;
        for (r, row) in map.iter_mut().enumerate() {
            for (c, ch) in row.iter_mut().enumerate() {
                match *ch {
                    b'r' | b'g' | b'n' | b'k' | b'q' => markers.push((c, r, *ch)),
                    b'X' => boss_row = r,
                    b'c' => crates.push(((c, r), 3)),
                    _ => {}
                }
                if *ch == b'k' {
                    *ch = b'=';
                }
                if *ch == b'q' {
                    *ch = b'Q';
                }
            }
        }
        let trains = map
            .iter()
            .enumerate()
            .filter(|(_, r)| r.iter().all(|&c| c == b'R'))
            .map(|(i, _)| (i, -400.0, 0, 120))
            .collect();
        let rows = map.len() as i32;
        let start_y = (rows * T - 40) as f32;
        let mut fx = Fx::new(seed ^ 0xf007);
        fx.gravity = 0.0;
        Foot {
            t: 0,
            agg: diff.aggression() * (0.85 + sector as f32 * 0.04),
            diff,
            rng: Rng::new(seed),
            fx,
            map,
            crates,
            markers,
            cy: (rows * T - VIEW) as f32,
            px: 168.0,
            py: start_y,
            fxd: 0.0,
            fyd: -1.0,
            walk: 0,
            inv: 120,
            dead: 0,
            fire_cd: 0,
            gren_cd: 0,
            muzzle: 0,
            foes: Vec::new(),
            shots: Vec::new(),
            eshots: Vec::new(),
            grenades: Vec::new(),
            pickups: Vec::new(),
            boss: None,
            boss_row,
            clear_t: None,
            over_t: None,
            banner: Some((spec.intro, 30)),
            tiles: make_tiles(),
            spec,
            trains,
        }
    }

    fn at(&self, x: f32, y: f32) -> u8 {
        if x < 0.0 || y < 0.0 {
            return b'#';
        }
        let (c, r) = ((x as i32 / T) as usize, (y as i32 / T) as usize);
        self.map
            .get(r)
            .and_then(|row| row.get(c))
            .copied()
            .unwrap_or(b'#')
    }

    fn blocked(&self, x: f32, y: f32, r: f32) -> bool {
        for (dx, dy) in [(-r, -r), (r, -r), (-r, r), (r, r)] {
            if solid_walk(self.at(x + dx, y + dy)) {
                return true;
            }
        }
        if let Some(b) = &self.boss {
            if b.dying == 0 && (x - b.x).abs() < 56.0 && (y - b.y).abs() < 26.0 {
                return true;
            }
        }
        false
    }

    fn sy(&self, y: f32) -> i32 {
        (y - self.cy) as i32 + HUD_H
    }

    fn spawn(&mut self, kind: Fk, x: f32, y: f32) {
        let hp = match kind {
            Fk::Rifle => 1,
            Fk::Grenadier => 2,
            Fk::Nest => 8,
            Fk::Tank => 30,
            Fk::Sniper => 2,
        };
        self.foes.push(Foe {
            kind,
            x,
            y,
            hp,
            t: 0,
            cd: 60 + self.rng.below(90) as i32,
            fx: 0.0,
            fy: 1.0,
            tx: x,
            ty: y,
            flash: 0,
            walking: true,
        });
    }

    fn shoot_at(&mut self, x: f32, y: f32, speed: f32, spread: f32, big: bool) {
        let a = (self.py - y).atan2(self.px - x) + spread;
        let sp = speed * (0.7 + 0.3 * self.agg);
        if self.eshots.len() < 90 {
            self.eshots.push(Bullet {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp,
                life: 0,
                big,
            });
        }
    }

    fn lob(&mut self, x: f32, y: f32, tx: f32, ty: f32, mine: bool) {
        let d = ((tx - x).powi(2) + (ty - y).powi(2)).sqrt();
        self.grenades.push(Grenade {
            x0: x,
            y0: y,
            x1: tx,
            y1: ty,
            t: 0,
            dur: (d / 1.6).clamp(20.0, 60.0) as u32,
            mine,
        });
    }

    fn kill_player(&mut self, st: &mut Stats, out: &mut Out) {
        if self.inv > 0 || self.dead > 0 || self.clear_t.is_some() {
            return;
        }
        let (x, y) = (self.px, self.sy(self.py) as f32);
        if st.shield > 0 {
            st.shield -= 1;
            self.inv = 70;
            self.fx.sparks(x, y, 12, 0.0);
            out.play(Sfx::Clang);
            return;
        }
        self.fx.explode(x, y, 2.0);
        out.play(Sfx::PlayerDie);
        st.lives -= 1;
        st.deaths_this_level += 1;
        st.weapon = st.weapon.saturating_sub(1).max(1);
        self.dead = 1;
        self.eshots.clear();
    }

    /// An explosion on the ground that hurts everything near it.
    fn blast(&mut self, x: f32, y: f32, r: f32, mine: bool, st: &mut Stats, out: &mut Out) {
        let sy = self.sy(y) as f32;
        self.fx.explode(x, sy, 2.2);
        out.play(Sfx::ExplodeBig);
        if mine {
            if self.dead == 0 && d2(x, y, self.px, self.py) < r * r {
                self.kill_player(st, out);
            }
            return;
        }
        for f in self.foes.iter_mut() {
            if d2(x, y, f.x, f.y) < (r + 6.0) * (r + 6.0) {
                f.hp -= 8;
                f.flash = 6;
            }
        }
        let mut broke = Vec::new();
        for ((c, r0), hp) in self.crates.iter_mut() {
            let (cx, cy) = (*c as f32 * 16.0 + 8.0, *r0 as f32 * 16.0 + 8.0);
            if *hp > 0 && d2(x, y, cx, cy) < (r + 8.0) * (r + 8.0) {
                *hp = 0;
                broke.push((*c, *r0));
            }
        }
        for (c, r0) in broke {
            self.break_crate(c, r0, st, out);
        }
        if let Some(b) = self.boss.as_mut() {
            for i in 0..2 {
                let (gx, gy) = b.gun(i);
                if b.guns[i] > 0 && d2(x, y, gx, gy) < (r + 10.0) * (r + 10.0) {
                    b.guns[i] -= 6;
                    b.flash[i] = 6;
                }
            }
            let (cx, cy) = b.core();
            if b.open() && d2(x, y, cx, cy) < (r + 10.0) * (r + 10.0) {
                b.core -= 6;
                b.flash[2] = 6;
            }
        }
    }

    fn break_crate(&mut self, c: usize, r: usize, st: &mut Stats, out: &mut Out) {
        self.map[r][c] = b'.';
        let (x, y) = (c as f32 * 16.0 + 8.0, r as f32 * 16.0 + 8.0);
        let sy = self.sy(y) as f32;
        self.fx.explode(x, sy, 1.2);
        out.play(Sfx::ExplodeSmall);
        let kinds = [
            PowerUp::Weapon,
            PowerUp::Bomb,
            PowerUp::Shield,
            PowerUp::Weapon,
            PowerUp::Points,
            PowerUp::Life,
        ];
        let kind = if st.weapon < 2 {
            PowerUp::Weapon
        } else {
            kinds[self.rng.below(kinds.len() as u32) as usize]
        };
        self.pickups.push(Pickup { x, y, kind, t: 0 });
    }

    fn player_fire(&mut self, st: &Stats, out: &mut Out) {
        let (fx, fy) = (self.fxd, self.fyd);
        let (x, y) = (self.px + fx * 7.0, self.py + fy * 7.0 - 2.0);
        let a = fy.atan2(fx);
        let mut add = |off: f32, da: f32| {
            let (px, py) = (-fy * off, fx * off);
            self.shots.push(Bullet {
                x: x + px,
                y: y + py,
                vx: (a + da).cos() * 3.8,
                vy: (a + da).sin() * 3.8,
                life: 0,
                big: false,
            });
        };
        match st.weapon {
            1 => add(0.0, 0.0),
            2 => {
                add(-2.5, 0.0);
                add(2.5, 0.0);
            }
            3 => {
                add(0.0, 0.0);
                add(0.0, -0.22);
                add(0.0, 0.22);
            }
            _ => {
                add(-2.5, 0.0);
                add(2.5, 0.0);
                add(0.0, -0.25);
                add(0.0, 0.25);
            }
        }
        self.muzzle = 4;
        out.play(Sfx::Laser);
        self.fire_cd = match st.weapon {
            1 => 10,
            2 | 3 => 9,
            _ => 7,
        };
    }

    fn update_boss(&mut self, st: &mut Stats, out: &mut Out) {
        let Some(mut b) = self.boss.take() else {
            return;
        };
        b.t += 1;
        for f in b.flash.iter_mut() {
            *f = f.saturating_sub(1);
        }
        if b.dying > 0 {
            b.dying += 1;
            if b.dying % 6 == 0 {
                let (x, y) = (b.x + self.rng.s() * 50.0, b.y + self.rng.s() * 20.0);
                let sy = self.sy(y) as f32;
                self.fx.explode(x, sy, 2.5);
                out.play(Sfx::ExplodeBig);
            }
            if b.dying == 160 {
                let sy = self.sy(b.y) as f32;
                self.fx.explode(b.x, sy, 8.0);
                self.fx.flash = 10;
                st.add(30000);
                self.fx.text(b.x, sy, "30000".into(), YELLOW);
                self.clear_t = Some(self.t);
                // The gate is open.
                for r in 0..3.min(self.map.len()) {
                    for c in 8..12 {
                        self.map[r][c] = b'=';
                    }
                }
                return;
            }
            self.boss = Some(b);
            return;
        }
        b.x = 160.0 + (b.t as f32 * 0.008).sin() * 64.0;
        let period = (90.0 / self.agg) as u32;
        for i in 0..2 {
            if b.guns[i] > 0 && (b.t + i as u32 * period / 2) % period == 0 {
                let (gx, gy) = b.gun(i);
                self.shoot_at(gx, gy + 6.0, 1.6, 0.0, false);
                if self.diff != Difficulty::Easy {
                    self.shoot_at(gx, gy + 6.0, 1.6, 0.2, false);
                    self.shoot_at(gx, gy + 6.0, 1.6, -0.2, false);
                }
                out.play(Sfx::EnemyShot);
            }
        }
        if b.t % ((200.0 / self.agg) as u32) == 100 {
            let (px, py) = (self.px, self.py);
            self.lob(b.x, b.y, px, py, true);
            out.play(Sfx::Grenade);
        }
        if b.open() && b.t % ((80.0 / self.agg) as u32) == 0 {
            let (cx, cy) = b.core();
            for k in 0..9 {
                let a = 0.35 + k as f32 * 0.3;
                self.eshots.push(Bullet {
                    x: cx,
                    y: cy,
                    vx: a.cos() * 1.3,
                    vy: a.sin() * 1.3,
                    life: 0,
                    big: true,
                });
            }
            out.play(Sfx::EnemyShot);
        }
        if FOOT_STYLES[self.spec.style].shape == 2
            && b.open()
            && b.core * 2 < FOOT_STYLES[2].core
            && b.t % 6 == 0
        {
            // The core's last rage: a turning spiral.
            let a = b.t as f32 * 0.09;
            let (cx, cy) = b.core();
            for k in 0..2 {
                let a = a + k as f32 * std::f32::consts::PI;
                self.eshots.push(Bullet {
                    x: cx,
                    y: cy,
                    vx: a.cos() * 1.2,
                    vy: a.sin() * 1.2,
                    life: 0,
                    big: false,
                });
            }
        }
        if b.t % 360 == 180 && self.foes.len() < 8 {
            let y = b.y + 30.0;
            self.spawn(Fk::Rifle, 40.0, y);
            self.spawn(Fk::Rifle, 280.0, y);
        }
        self.boss = Some(b);
    }
}

impl Level for Foot {
    fn update(&mut self, inp: &Input, st: &mut Stats, out: &mut Out) -> Status {
        self.t += 1;
        let cy_before = self.cy;
        if self.t == 1 {
            out.music = Some(self.spec.song);
        }
        // Markers coming into view become enemies.
        let top = self.cy - 8.0;
        let mut i = 0;
        while i < self.markers.len() {
            let (c, r, k) = self.markers[i];
            if (r as f32 * 16.0) > top && (r as f32 * 16.0) < self.cy + VIEW as f32 {
                self.markers.swap_remove(i);
                let (x, y) = (c as f32 * 16.0 + 8.0, r as f32 * 16.0 + 8.0);
                match k {
                    b'r' => self.spawn(Fk::Rifle, x, y),
                    b'g' => self.spawn(Fk::Grenadier, x, y),
                    b'n' => self.spawn(Fk::Nest, x, y),
                    b'q' => self.spawn(Fk::Sniper, x, y + 4.0),
                    _ => {
                        self.spawn(Fk::Tank, x + 8.0, self.cy - 20.0);
                        if let Some(f) = self.foes.last_mut() {
                            f.ty = y;
                        }
                        self.banner = Some(("TANK!", self.t));
                        out.play(Sfx::Alarm);
                    }
                }
            } else {
                i += 1;
            }
        }
        // Reinforcements from the top edge now and then.
        let every = (300.0 / self.agg) as u32;
        if self.boss.is_none()
            && self.t % every == every / 2
            && self.foes.len() < 10
            && self.t > 200
        {
            let x = self.rng.range(40.0, 280.0);
            let y = self.cy - 6.0;
            if !solid_walk(self.at(x, y + 10.0)) {
                self.spawn(Fk::Rifle, x, y);
            }
        }
        // The fortress yard: the camera stops, the boss wakes.
        let boss_y = self.boss_row as f32 * 16.0 + 8.0;
        if self.boss.is_none() && self.clear_t.is_none() && self.cy <= 2.0 {
            let fs = &FOOT_STYLES[self.spec.style];
            self.boss = Some(Boss {
                x: 160.0,
                y: boss_y - 26.0,
                t: 0,
                guns: [fs.guns; 2],
                core: fs.core,
                flash: [0; 3],
                dying: 0,
            });
            out.music = Some(self.spec.boss_song);
            out.play(Sfx::Alarm);
            self.banner = Some((fs.name, self.t));
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
                    // Back on the road, at the bottom of the view.
                    self.px = 160.0;
                    self.py = self.cy + VIEW as f32 - 20.0;
                    while self.blocked(self.px, self.py, 4.0) && self.py > self.cy + 20.0 {
                        self.py -= 4.0;
                    }
                }
            }
        } else if self.clear_t.is_none() {
            let (dx, dy) = (inp.dx(), inp.dy());
            if dx != 0.0 || dy != 0.0 {
                let k = if dx != 0.0 && dy != 0.0 {
                    std::f32::consts::FRAC_1_SQRT_2
                } else {
                    1.0
                };
                self.fxd = dx * k;
                self.fyd = dy * k;
                let sp = 1.1;
                let nx = self.px + dx * k * sp;
                if !self.blocked(nx, self.py, 4.0) {
                    self.px = nx.clamp(6.0, W as f32 - 6.0);
                }
                let ny = self.py + dy * k * sp;
                if !self.blocked(self.px, ny, 4.0) {
                    self.py = ny.clamp(self.cy + 10.0, self.cy + VIEW as f32 - 8.0);
                }
                self.walk += 1;
            }
            self.fire_cd = self.fire_cd.saturating_sub(1);
            self.gren_cd = self.gren_cd.saturating_sub(1);
            if inp.fire && self.fire_cd == 0 {
                self.player_fire(st, out);
            }
            if inp.p_special && st.bombs > 0 && self.gren_cd == 0 {
                st.bombs -= 1;
                self.gren_cd = 30;
                let (tx, ty) = (self.px + self.fxd * 64.0, self.py + self.fyd * 64.0);
                let (px, py) = (self.px, self.py);
                self.lob(px, py, tx, ty, false);
                out.play(Sfx::Grenade);
            }
        }
        self.muzzle = self.muzzle.saturating_sub(1);
        self.inv = self.inv.saturating_sub(1);
        // The camera follows upwards only.
        let want = (self.py - 110.0).max(0.0);
        if want < self.cy {
            self.cy += (want - self.cy) * 0.08;
            if self.cy - want < 0.5 {
                self.cy = want;
            }
        }
        // Enemies.
        let (px, py) = (self.px, self.py);
        let mut acts: Vec<(u8, f32, f32)> = Vec::new();
        let mut foes = std::mem::take(&mut self.foes);
        for f in foes.iter_mut() {
            f.t += 1;
            f.flash = f.flash.saturating_sub(1);
            let (dx, dy) = (px - f.x, py - f.y);
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            match f.kind {
                Fk::Rifle | Fk::Grenadier => {
                    // Walk towards a spot near the player, stop to shoot.
                    if f.t % 90 == 1 {
                        let keep = if f.kind == Fk::Grenadier { 70.0 } else { 40.0 };
                        f.tx = px + self.rng.s() * 50.0;
                        f.ty = py - keep - self.rng.f() * 40.0;
                        f.walking = self.rng.chance(0.7);
                    }
                    if f.walking {
                        let (mx, my) = (f.tx - f.x, f.ty - f.y);
                        let md = (mx * mx + my * my).sqrt();
                        if md > 2.0 {
                            let sp = if f.kind == Fk::Rifle { 0.55 } else { 0.4 };
                            let (vx, vy) = (mx / md * sp, my / md * sp);
                            let nx = f.x + vx;
                            let ny = f.y + vy;
                            if !solid_walk(self.at(nx, f.y)) {
                                f.x = nx;
                            }
                            if !solid_walk(self.at(f.x, ny)) {
                                f.y = ny;
                            }
                            f.fx = vx / sp;
                            f.fy = vy / sp;
                        }
                    }
                    if !f.walking || f.t % 40 == 0 {
                        f.fx = dx / d;
                        f.fy = dy / d;
                    }
                    let on = f.y > self.cy && f.y < self.cy + VIEW as f32;
                    f.cd -= 1;
                    if f.cd <= 0 && on && d < 170.0 {
                        f.cd = if f.kind == Fk::Rifle {
                            (110.0 / self.agg) as i32
                        } else {
                            (170.0 / self.agg) as i32
                        } + self.rng.below(60) as i32;
                        f.fx = dx / d;
                        f.fy = dy / d;
                        acts.push((if f.kind == Fk::Rifle { 0 } else { 1 }, f.x, f.y));
                    }
                }
                Fk::Sniper => {
                    f.fx = dx / d;
                    f.fy = dy / d;
                    let on = f.y > self.cy + 4.0 && f.y < self.cy + VIEW as f32;
                    f.cd -= 1;
                    if on && f.cd <= 0 {
                        f.cd = (150.0 / self.agg) as i32 + self.rng.below(60) as i32;
                        acts.push((4, f.x, f.y));
                    }
                }
                Fk::Nest => {
                    f.fx = dx / d;
                    f.fy = dy / d;
                    let on = f.y > self.cy && f.y < self.cy + VIEW as f32;
                    f.cd -= 1;
                    if on && f.cd <= 0 && d < 220.0 {
                        let burst = f.t % 3;
                        acts.push((2, f.x, f.y));
                        f.cd = if burst < 2 {
                            10
                        } else {
                            (140.0 / self.agg) as i32
                        };
                    }
                }
                Fk::Tank => {
                    if f.y < f.ty {
                        f.y += 0.5;
                    } else {
                        f.x = 168.0 + (f.t as f32 * 0.01).sin() * 20.0;
                    }
                    f.fx = dx / d;
                    f.fy = dy / d;
                    f.cd -= 1;
                    if f.cd <= 0 && f.y > self.cy {
                        f.cd = (110.0 / self.agg) as i32;
                        acts.push((3, f.x + f.fx * 16.0, f.y + f.fy * 16.0));
                    }
                }
            }
        }
        self.foes = foes;
        for (k, x, y) in acts {
            match k {
                0 => {
                    self.shoot_at(x, y, 1.5, 0.0, false);
                    out.play(Sfx::EnemyShot);
                }
                1 => {
                    let (tx, ty) = (self.px + self.rng.s() * 12.0, self.py + self.rng.s() * 12.0);
                    self.lob(x, y, tx, ty, true);
                    out.play(Sfx::Grenade);
                }
                2 => {
                    let s = self.rng.s() * 0.12;
                    self.shoot_at(x, y, 1.8, s, false);
                    out.play(Sfx::EnemyShot);
                }
                4 => {
                    self.shoot_at(x, y, 2.1, 0.0, false);
                    out.play(Sfx::EnemyShot);
                }
                _ => {
                    self.shoot_at(x, y, 1.4, 0.0, true);
                    if self.diff == Difficulty::Hard {
                        self.shoot_at(x, y, 1.4, 0.3, true);
                        self.shoot_at(x, y, 1.4, -0.3, true);
                    }
                    self.fx.shake = self.fx.shake.max(1.5);
                    out.play(Sfx::ExplodeSmall);
                }
            }
        }
        self.update_boss(st, out);
        // Player bullets.
        let mut shots = std::mem::take(&mut self.shots);
        let mut crate_hits = Vec::new();
        shots.retain_mut(|s| {
            s.x += s.vx;
            s.y += s.vy;
            s.life += 1;
            if s.life > 48
                || s.x < 0.0
                || s.x > W as f32
                || s.y < self.cy - 8.0
                || s.y > self.cy + VIEW as f32
            {
                return false;
            }
            let c = self.at(s.x, s.y);
            if c == b'c' {
                crate_hits.push(((s.x as i32 / T) as usize, (s.y as i32 / T) as usize));
                return false;
            }
            for f in self.foes.iter_mut() {
                let r = match f.kind {
                    Fk::Tank => 13.0,
                    Fk::Nest | Fk::Sniper => 8.0,
                    _ => 5.0,
                };
                if f.hp > 0 && (s.x - f.x).abs() < r && (s.y - f.y).abs() < r {
                    f.hp -= 1;
                    f.flash = 4;
                    return false;
                }
            }
            if solid_shot(c) {
                let sy = (s.y - self.cy) + HUD_H as f32;
                self.fx
                    .sparks(s.x, sy, 2, s.vy.atan2(s.vx) + std::f32::consts::PI);
                return false;
            }
            if let Some(b) = self.boss.as_mut() {
                if b.dying == 0 && (s.x - b.x).abs() < 54.0 && (s.y - b.y).abs() < 24.0 {
                    for i in 0..2 {
                        let (gx, gy) = b.gun(i);
                        if b.guns[i] > 0 && (s.x - gx).abs() < 10.0 && (s.y - gy).abs() < 10.0 {
                            b.guns[i] -= 1;
                            b.flash[i] = 3;
                            return false;
                        }
                    }
                    let (cx, cy) = b.core();
                    if b.open() && (s.x - cx).abs() < 10.0 && s.y > cy - 10.0 && s.y < cy + 18.0 {
                        b.core -= 1;
                        b.flash[2] = 3;
                        return false;
                    }
                    let sy = (s.y - self.cy) + HUD_H as f32;
                    self.fx.sparks(s.x, sy, 2, -1.57);
                    return false;
                }
            }
            true
        });
        self.shots = shots;
        for (c, r) in crate_hits {
            if let Some(e) = self.crates.iter_mut().find(|e| e.0 == (c, r)) {
                e.1 -= 1;
                if e.1 <= 0 && self.map[r][c] == b'c' {
                    self.break_crate(c, r, st, out);
                } else {
                    out.play(Sfx::Clang);
                }
            }
        }
        // Boss parts that broke.
        if let Some(b) = self.boss.as_mut() {
            for i in 0..2 {
                if b.guns[i] <= 0 && b.guns[i] > -100 {
                    b.guns[i] = -100;
                    let (gx, gy) = b.gun(i);
                    let sy = (gy - self.cy) + HUD_H as f32;
                    self.fx.explode(gx, sy, 3.0);
                    out.play(Sfx::ExplodeBig);
                    st.add(5000);
                    self.fx.text(gx, sy - 10.0, "5000".into(), YELLOW);
                }
            }
            if b.core <= 0 && b.dying == 0 {
                b.dying = 1;
                self.eshots.clear();
            }
        }
        // Grenades in flight.
        let mut booms = Vec::new();
        self.grenades.retain_mut(|g| {
            g.t += 1;
            if g.t >= g.dur {
                booms.push((g.x1, g.y1, g.mine));
                return false;
            }
            true
        });
        for (x, y, mine) in booms {
            self.blast(x, y, if mine { 16.0 } else { 24.0 }, mine, st, out);
        }
        // Dead enemies.
        let mut dead = Vec::new();
        self.foes.retain(|f| {
            if f.hp <= 0 {
                dead.push((f.kind, f.x, f.y));
                return false;
            }
            f.y < self.cy + VIEW as f32 + 40.0 && f.y > self.cy - 200.0
        });
        for (k, x, y) in dead {
            let sy = self.sy(y) as f32;
            let (size, pts) = match k {
                Fk::Rifle => (1.0, 150),
                Fk::Grenadier => (1.0, 300),
                Fk::Nest => (2.5, 1500),
                Fk::Sniper => (1.2, 600),
                Fk::Tank => (5.0, 8000),
            };
            self.fx.explode(x, sy, size);
            out.play(if size > 2.0 {
                Sfx::ExplodeBig
            } else {
                Sfx::ExplodeSmall
            });
            if st.add(pts) {
                out.play(Sfx::ExtraLife);
            }
            st.kills += 1;
            self.fx.text(x, sy - 6.0, pts.to_string(), YELLOW);
            if k == Fk::Nest || k == Fk::Tank {
                let (c, r) = ((x as i32 / T) as usize, (y as i32 / T) as usize);
                if k == Fk::Nest && self.map.get(r).and_then(|row| row.get(c)) == Some(&b'n') {
                    self.map[r][c] = b'.';
                }
                let kind = if k == Fk::Tank {
                    PowerUp::Shield
                } else if st.weapon < 4 {
                    PowerUp::Weapon
                } else {
                    PowerUp::Points
                };
                self.pickups.push(Pickup { x, y, kind, t: 0 });
            }
        }
        // Trains: a warning, then a train along the rails.
        let view = (self.cy, self.cy + VIEW as f32);
        let prow = (self.py / 16.0) as usize;
        let mut train_hits = Vec::new();
        for tr in self.trains.iter_mut() {
            let ry = tr.0 as f32 * 16.0 + 8.0;
            let visible = ry > view.0 - 20.0 && ry < view.1 + 20.0;
            if tr.1 > -400.0 {
                tr.1 += 7.0;
                train_hits.push((tr.1, ry));
                if tr.1 > W as f32 + 20.0 {
                    tr.1 = -400.0;
                    tr.3 = 360 + (tr.0 as u32 * 37) % 200;
                }
            } else if tr.2 > 0 {
                tr.2 -= 1;
                if tr.2 % 30 == 0 {
                    out.play(Sfx::Alarm);
                }
                if tr.2 == 0 {
                    tr.1 = -300.0;
                }
            } else if visible && prow.abs_diff(tr.0) < 6 {
                tr.3 = tr.3.saturating_sub(1);
                if tr.3 == 0 {
                    tr.2 = 100;
                }
            }
        }
        for (tx, ty) in train_hits {
            // The train is 300 px long and runs from tx - 300 to tx.
            for f in self.foes.iter_mut() {
                if (f.y - ty).abs() < 9.0 && f.x < tx && f.x > tx - 300.0 && f.kind != Fk::Tank {
                    f.hp = 0;
                }
            }
            if self.dead == 0 && (self.py - ty).abs() < 10.0 && self.px < tx && self.px > tx - 300.0
            {
                self.kill_player(st, out);
            }
        }
        // Enemy bullets.
        let alive = self.dead == 0;
        let mut hit = false;
        let cy = self.cy;
        let map = &self.map;
        let at = |x: f32, y: f32| -> u8 {
            if x < 0.0 || y < 0.0 {
                return b'#';
            }
            map.get(y as usize / 16)
                .and_then(|r| r.get(x as usize / 16))
                .copied()
                .unwrap_or(b'#')
        };
        self.eshots.retain_mut(|s| {
            s.x += s.vx;
            s.y += s.vy;
            s.life += 1;
            if alive && d2(s.x, s.y, px, py - 2.0) < if s.big { 20.0 } else { 10.0 } {
                hit = true;
                return false;
            }
            if s.life > 6 && solid_shot(at(s.x, s.y)) && at(s.x, s.y) != b'n' {
                return false;
            }
            s.life < 400
                && s.x > -4.0
                && s.x < W as f32 + 4.0
                && s.y > cy - 10.0
                && s.y < cy + VIEW as f32 + 10.0
        });
        if alive {
            for f in &self.foes {
                if f.kind == Fk::Tank && (f.x - px).abs() < 14.0 && (f.y - py).abs() < 16.0 {
                    hit = true;
                }
            }
        }
        if hit {
            self.kill_player(st, out);
        }
        // Pickups.
        let mut got = Vec::new();
        self.pickups.retain_mut(|p| {
            p.t += 1;
            if alive && (p.x - px).abs() < 10.0 && (p.y - py).abs() < 10.0 {
                got.push((p.kind, p.x, p.y));
                return false;
            }
            p.t < 60 * 20 && p.y < cy + VIEW as f32 + 20.0
        });
        for (k, x, y) in got {
            let s = st.collect(k, out);
            let sy = self.sy(y) as f32;
            self.fx.text(x, sy - 8.0, s.to_string(), LGREEN);
            self.fx.sparks(x, sy, 10, 0.0);
        }
        // Keep the particles where they happened as the view scrolls.
        let dcy = cy_before - self.cy;
        for p in self.fx.p.iter_mut() {
            p.y += dcy;
        }
        for t in self.fx.texts.iter_mut() {
            t.y += dcy;
        }
        self.fx.update();
        if let Some(t) = self.over_t {
            if self.t - t > 30 {
                return Status::Over;
            }
        }
        if let Some(t) = self.clear_t {
            // March through the open gate.
            if self.t - t > 60 {
                self.fyd = -1.0;
                self.fxd = 0.0;
                self.px += (160.0 - self.px) * 0.05;
                self.py -= 0.8;
                self.walk += 1;
            }
            if self.t - t > 220 {
                return Status::Cleared;
            }
        }
        Status::Playing
    }

    fn draw(&self, fb: &mut Fb, st: &Stats, hi: u32) {
        let (ox, oy) = self.fx.offset(self.t);
        let cy = self.cy as i32 - oy;
        let r0 = (cy / T).max(0);
        let r1 = ((cy + VIEW) / T + 1).min(self.map.len() as i32 - 1);
        for r in r0..=r1 {
            for c in 0..COLS as i32 {
                let ch = self.map[r as usize][c as usize];
                let sx = c * T + ox;
                let sy = r * T - cy + HUD_H;
                let idx = tile_index(ch, self.spec.theme);
                let tile = &self.tiles[idx];
                for y in 0..T {
                    let yy = sy + y;
                    if !(HUD_H..H).contains(&yy) {
                        continue;
                    }
                    for x in 0..T {
                        let mut p = tile[(y * T + x) as usize];
                        if idx == 3 {
                            // Water: moving ripples.
                            let w = (x + c * T + (self.t / 4) as i32 + ((y + r * T) / 3) * 5) % 16;
                            if w == 0 && (y + r * T) % 3 == 0 {
                                p = LBLUE;
                            } else if w == 8 && (y + r * T) % 6 == 0 {
                                p = CYAN;
                            }
                        }
                        fb.set(sx + x, yy, p);
                    }
                }
                // Bank edges along the water.
                if ch == b'~' {
                    let above = if r > 0 {
                        self.map[(r - 1) as usize][c as usize]
                    } else {
                        b'~'
                    };
                    if above != b'~' && above != b'B' {
                        fb.hline(sx, sx + 15, sy, BROWN);
                        fb.hline(sx, sx + 15, sy + 1, BLACK);
                    }
                }
                if ch == b'c' {
                    fb.blit(&art::get().crate_, sx + 1, sy + 1);
                }
            }
        }
        // Ground things: grenades' shadows, pickups.
        for p in &self.pickups {
            draw_capsule(fb, p.x as i32 + ox, self.sy(p.y) + oy, p.kind, p.t);
        }
        for g in &self.grenades {
            let (x, y, _) = g.pos();
            let sy = self.sy(y) + oy;
            fb.hline(x as i32 - 1 + ox, x as i32 + 1 + ox, sy + 1, BLACK);
            if g.mine && g.dur - g.t < 20 && g.t % 4 < 2 {
                fb.circle(g.x1 as i32 + ox, self.sy(g.y1) + oy, 12, RED);
            }
        }
        // Units, sorted by height on the screen.
        let mut units: Vec<(f32, usize)> = self
            .foes
            .iter()
            .enumerate()
            .map(|(i, f)| (f.y, i))
            .collect();
        units.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, i) in &units {
            let f = &self.foes[*i];
            let (x, y) = (f.x as i32 + ox, self.sy(f.y) + oy);
            let fl = f.flash > 0 && f.flash % 2 == 0;
            match f.kind {
                Fk::Rifle | Fk::Grenadier => {
                    let (body, helmet) = if f.kind == Fk::Rifle {
                        (ORANGE, LRED)
                    } else {
                        (BROWN, YELLOW)
                    };
                    let walk = if f.walking { f.t / 6 } else { 0 };
                    soldier(
                        fb,
                        x,
                        y,
                        f.fx,
                        f.fy,
                        walk,
                        if fl { WHITE } else { body },
                        if fl { WHITE } else { helmet },
                        false,
                    );
                }
                Fk::Sniper => {
                    soldier(
                        fb,
                        x,
                        y,
                        f.fx,
                        f.fy,
                        0,
                        if fl { WHITE } else { GREY },
                        if fl { WHITE } else { RED },
                        f.cd < 6,
                    );
                }
                Fk::Nest => {
                    let (gx, gy) = ((f.fx * 9.0) as i32, (f.fy * 9.0) as i32);
                    fb.disc(x, y, 3, if fl { WHITE } else { DGREY });
                    fb.line(x, y, x + gx, y + gy, if fl { WHITE } else { LGREY });
                    fb.disc(x, y - 1, 2, if fl { WHITE } else { LRED });
                    if f.cd > 6 && f.cd < 10 {
                        fb.set(x + gx, y + gy, YELLOW);
                    }
                }
                Fk::Tank => tank(fb, x, y, f.fx, f.fy, f.t, fl),
            }
        }
        if let Some(b) = &self.boss {
            self.draw_boss(fb, b, ox, oy);
        }
        if self.dead == 0 && !(self.inv > 0 && self.inv < 140 && (self.t / 3) % 2 == 0) {
            let (x, y) = (self.px as i32 + ox, self.sy(self.py) + oy);
            soldier(
                fb,
                x,
                y,
                self.fxd,
                self.fyd,
                self.walk / 5,
                LBLUE,
                CYAN,
                self.muzzle > 0,
            );
            if st.shield > 0 {
                let r = 9 + (self.t % 8 < 4) as i32;
                for k in 0..16 {
                    let a = k as f32 * std::f32::consts::FRAC_PI_8 + self.t as f32 * 0.06;
                    fb.set(
                        x + (a.cos() * r as f32) as i32,
                        y + (a.sin() * r as f32) as i32,
                        if k % 2 == 0 { WHITE } else { CYAN },
                    );
                }
            }
        }
        // Trains and their warning lights.
        for tr in &self.trains {
            let y = tr.0 as i32 * T - cy + HUD_H;
            if !(HUD_H - 16..H).contains(&y) {
                continue;
            }
            if tr.2 > 0 && tr.2 % 20 < 10 {
                fb.disc(4 + ox, y + 8, 3, LRED);
                fb.disc(W - 5 + ox, y + 8, 3, LRED);
            }
            if tr.1 > -400.0 {
                let x1 = tr.1 as i32 + ox;
                for car in 0..5 {
                    let x0 = x1 - 300 + car * 60;
                    fb.rect(x0 + 2, y + 1, 56, 14, if car == 4 { RED } else { DGREY });
                    fb.frame(x0 + 2, y + 1, 56, 14, LGREY);
                    for w in 0..5 {
                        fb.rect(
                            x0 + 8 + w * 10,
                            y + 4,
                            5,
                            3,
                            if car == 4 { YELLOW } else { CYAN },
                        );
                    }
                }
                fb.rect(x1 - 4, y + 5, 4, 6, YELLOW);
            }
        }
        // Tree tops over everything on the ground.
        let a = art::get();
        let burnt = a.tree.recolor(&[(5, 9), (13, 8), (1, 7), (11, 0)]);
        for r in r0..=r1 {
            for c in 0..COLS as i32 {
                let ch = self.map[r as usize][c as usize];
                let sx = c * T + ox;
                let sy = r * T - cy + HUD_H;
                match ch {
                    b'T' => {
                        let v = (hash(c, r) % 3) as i32;
                        // A shadow on the ground, then the canopy.
                        for yy in 0..12 {
                            for xx in 0..14 {
                                if (xx + yy) & 1 == 0
                                    && (xx - 7) * (xx - 7) + (yy - 6) * (yy - 6) < 40
                                {
                                    fb.set(sx + v + xx + 1, sy + yy + 4, BLACK);
                                }
                            }
                        }
                        let tree = if self.spec.theme == 1 {
                            &burnt
                        } else {
                            &a.tree
                        };
                        fb.blit(tree, sx + v - 2, sy - 2 + (hash(r, c) % 3) as i32);
                    }
                    b'F' => {
                        let pal = if self.spec.theme == 2 {
                            [WHITE, CYAN, LBLUE, BLUE]
                        } else {
                            [YELLOW, ORANGE, RED, DGREY]
                        };
                        for k in 0..10 {
                            let h = hash(c * 7 + k, r) as i32;
                            let life = (self.t as i32 + h) % 18;
                            let fx = sx + 3 + (h % 10) + ((self.t as i32 / 3 + k) % 3) - 1;
                            let fy = sy + 12 - life;
                            fb.set(fx, fy, pal[(life / 5).min(3) as usize]);
                            fb.set(fx + 1, fy, pal[(life / 6).min(3) as usize]);
                        }
                    }
                    b'p' => {
                        fb.blit_solid(&a.palm, sx + 1, sy + 4, BLACK);
                        fb.blit(&a.palm, sx - 1, sy + 1);
                    }
                    b'o' => fb.blit(&a.bush, sx + 2, sy + 5),
                    b'H' if self.spec.theme == 0 && (c + r) % 2 == 0 => {
                        fb.hline(sx, sx + 31, sy, YELLOW)
                    }
                    _ => {}
                }
            }
        }
        for g in &self.grenades {
            let (x, y, z) = g.pos();
            let sy = self.sy(y) - z as i32 + oy;
            fb.disc(x as i32 + ox, sy, 1, if g.mine { RED } else { DGREY });
            fb.set(
                x as i32 + ox,
                sy - 1,
                if g.t % 4 < 2 { YELLOW } else { WHITE },
            );
        }
        for s in &self.shots {
            let (x, y) = (s.x as i32 + ox, self.sy(s.y) + oy);
            fb.rect(x - 1, y - 1, 2, 2, YELLOW);
            fb.set(x, y, WHITE);
        }
        for s in &self.eshots {
            draw_bullet(fb, s.x as i32 + ox, self.sy(s.y) + oy, self.t, s.big);
        }
        self.fx.draw(fb, ox, 0);
        if let Some((s, t0)) = self.banner {
            let k = self.t.saturating_sub(t0);
            if self.t >= t0 && k < 170 && (k > 20 || k % 4 < 2) {
                fb.text_center_sh(30, s, if k % 16 < 8 { WHITE } else { LGREEN }, BLACK);
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
            if b.dying == 0 {
                draw_boss_bar(
                    fb,
                    FOOT_STYLES[self.spec.style].name,
                    b.guns.iter().map(|g| (*g).max(0)).sum::<i32>() + b.core.max(0),
                    FOOT_STYLES[self.spec.style].guns * 2 + FOOT_STYLES[self.spec.style].core,
                    self.t,
                );
            }
        }
        for f in &self.foes {
            if f.kind == Fk::Tank {
                draw_boss_bar(fb, "TANK", f.hp, 30, self.t);
            }
        }
    }

    fn bot(&mut self, st: &Stats) -> Input {
        let mut i = Input {
            fire: true,
            ..Default::default()
        };
        let (px, py) = (self.px, self.py);
        // The nearest enemy close by.
        let mut target: Option<(f32, f32)> = None;
        let mut best = 95.0f32 * 95.0;
        for f in &self.foes {
            let d = d2(f.x, f.y, px, py);
            if d < best && f.y > self.cy {
                best = d;
                target = Some((f.x, f.y));
            }
        }
        if let Some(b) = &self.boss {
            target = Some(if b.guns[0] > 0 {
                b.gun(0)
            } else if b.guns[1] > 0 {
                b.gun(1)
            } else {
                b.core()
            });
        }
        // Default: up the road, swaying a little.
        let road = 160.0 + (self.t as f32 * 0.013).sin() * 30.0;
        let (mut wx, mut wy) = (road - px, -1.0);
        if let Some(p) = self.pickups.first() {
            if d2(p.x, p.y, px, py) < 100.0 * 100.0 {
                wx = p.x - px;
                wy = p.y - py;
            }
        }
        if let Some((x, y)) = target {
            let (dx, dy) = (x - px, y - py);
            let aligned = dx.abs() < 5.0 || dy.abs() < 5.0 || (dx.abs() - dy.abs()).abs() < 5.0;
            if aligned {
                // Face it (moving that way a step at a time).
                let d = (dx * dx + dy * dy).sqrt().max(1.0);
                wx = dx / d;
                wy = dy / d;
                if self.boss.is_some() && dy < -40.0 && self.t % 3 != 0 {
                    wy = 0.0;
                    wx = 0.0;
                }
            } else {
                wx = dx;
                wy = if dy < -30.0 { -0.5 } else { 0.0 };
            }
        }
        i.left = wx < -1.5;
        i.right = wx > 1.5;
        i.up = wy < -0.3;
        i.down = wy > 0.3;
        if let Some(b) = &self.boss {
            if py < b.y + 56.0 {
                i.up = false;
                i.down = true;
            }
        }
        // Stuck: go around.
        if (i.up || i.down || i.left || i.right)
            && self.blocked(px + i.dx() * 3.0, py + i.dy() * 3.0, 4.0)
        {
            let s = if (self.t / 90) % 2 == 0 { 1.0 } else { -1.0 };
            i.left = s < 0.0;
            i.right = s > 0.0;
        }
        // Dodge bullets.
        for s in &self.eshots {
            if d2(s.x, s.y, px, py) < 20.0 * 20.0 {
                if s.vx.abs() > s.vy.abs() {
                    i.up = s.y >= py;
                    i.down = s.y < py;
                } else {
                    i.left = s.x >= px;
                    i.right = s.x < px;
                }
            }
        }
        let near_big = self
            .foes
            .iter()
            .any(|f| matches!(f.kind, Fk::Nest | Fk::Tank) && d2(f.x, f.y, px, py) < 70.0 * 70.0);
        i.p_special = st.bombs > 0 && (near_big || self.boss.is_some()) && self.t % 50 == 0;
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
        let total = (self.map.len() as i32 * T - VIEW) as f32;
        1.0 - self.cy / total.max(1.0)
    }

    fn info(&self) -> String {
        match &self.boss {
            Some(b) => format!("boss guns {:?} core {} dying {}", b.guns, b.core, b.dying),
            None => format!("cy {:.0} foes {}", self.cy, self.foes.len()),
        }
    }
}

impl Foot {
    fn draw_boss(&self, fb: &mut Fb, b: &Boss, ox: i32, oy: i32) {
        let (x, y) = (b.x as i32 + ox, self.sy(b.y) + oy);
        let dying = b.dying > 0 && b.dying % 4 < 2;
        let fs = &FOOT_STYLES[self.spec.style];
        let (hull, plate) = if dying {
            (RED, LRED)
        } else {
            (fs.cols[0], fs.cols[1])
        };
        if fs.shape == 1 {
            // A gunship seen from above: tail, body, stub wings, rotors.
            fb.rect(x - 3, y - 46, 6, 30, hull);
            fb.rect(x - 10, y - 48, 20, 4, plate);
            fb.rect(x - 40, y + 10, 80, 8, hull);
            fb.disc(x, y, 22, hull);
            fb.disc(x, y - 2, 18, plate);
            fb.disc(x, y + 8, 8, BLACK);
            let a = b.t as f32 * 0.5;
            for k in 0..2 {
                let a = a + k as f32 * std::f32::consts::FRAC_PI_2;
                let (dx, dy) = (a.cos() * 48.0, a.sin() * 48.0);
                fb.line(
                    x - dx as i32,
                    y - dy as i32,
                    x + dx as i32,
                    y + dy as i32,
                    LGREY,
                );
            }
            fb.disc(x, y, 3, DGREY);
        } else {
            self.draw_crawler(fb, x, y, hull, plate, b.t);
        }
        self.draw_boss_parts(fb, b, ox, oy);
    }

    fn draw_crawler(&self, fb: &mut Fb, x: i32, y: i32, hull: u8, plate: u8, t: u32) {
        let b_t = t;
        // Treads.
        for side in [-54, 42] {
            fb.rect(x + side, y - 24, 12, 48, BLACK);
            for k in 0..8 {
                let ty = y - 24 + ((k * 6 + (b_t / 3) as i32) % 48);
                fb.hline(x + side + 1, x + side + 10, ty, DGREY);
            }
        }
        // Hull.
        fb.rect(x - 44, y - 22, 88, 44, hull);
        fb.rect(x - 40, y - 18, 80, 36, plate);
        fb.frame(x - 44, y - 22, 88, 44, LGREY);
        for k in 0..5 {
            fb.hline(x - 38, x + 38, y - 14 + k * 7, DGREY);
        }
        // Hazard stripes along the front.
        for k in 0..22 {
            fb.rect(
                x - 44 + k * 4,
                y + 18,
                2,
                4,
                if k % 2 == 0 { YELLOW } else { BLACK },
            );
        }
        // Mortar in the middle.
        fb.disc(x, y - 8, 6, BLACK);
        fb.circle(x, y - 8, 6, LGREY);
    }

    fn draw_boss_parts(&self, fb: &mut Fb, b: &Boss, ox: i32, oy: i32) {
        let (gun, gun_light) = (
            FOOT_STYLES[self.spec.style].cols[2],
            FOOT_STYLES[self.spec.style].cols[3],
        );
        // Guns.
        for i in 0..2 {
            let (gx, gy) = b.gun(i);
            let (gx, gy) = (gx as i32 + ox, self.sy(gy) + oy);
            if b.guns[i] > 0 {
                let fl = b.flash[i] > 0 && b.flash[i] % 2 == 1;
                fb.disc(gx, gy, 7, if fl { WHITE } else { gun });
                fb.disc(gx, gy, 4, if fl { WHITE } else { gun_light });
                let (dx, dy) = (
                    self.px - gx as f32,
                    self.py - (gy as f32 + self.cy - HUD_H as f32),
                );
                let d = (dx * dx + dy * dy).sqrt().max(1.0);
                fb.line(
                    gx,
                    gy,
                    gx + (dx / d * 11.0) as i32,
                    gy + (dy / d * 11.0) as i32,
                    LGREY,
                );
                fb.line(
                    gx + 1,
                    gy,
                    gx + 1 + (dx / d * 11.0) as i32,
                    gy + (dy / d * 11.0) as i32,
                    GREY,
                );
            } else {
                fb.disc(gx, gy, 6, BLACK);
                if b.t % 8 < 4 {
                    fb.set(gx + (b.t % 5) as i32 - 2, gy - 2, ORANGE);
                }
            }
        }
        let (cx, cy) = b.core();
        let (cx, cy) = (cx as i32 + ox, self.sy(cy) + oy);
        if b.open() {
            let c = if b.flash[2] > 0 {
                WHITE
            } else {
                [RED, LRED, ORANGE, YELLOW, WHITE, YELLOW, ORANGE, LRED][((b.t / 3) % 8) as usize]
            };
            fb.rect(cx - 9, cy - 5, 18, 11, BLACK);
            fb.disc(cx, cy, 5, c);
        } else {
            fb.rect(cx - 9, cy - 5, 18, 11, DGREY);
            fb.frame(cx - 9, cy - 5, 18, 11, LGREY);
            fb.line(cx - 9, cy, cx + 8, cy, BLACK);
        }
    }
}

/// A soldier seen from above: legs, body, gun and helmet, facing (fx, fy).
#[allow(clippy::too_many_arguments)]
fn soldier(
    fb: &mut Fb,
    x: i32,
    y: i32,
    fx: f32,
    fy: f32,
    walk: u32,
    body: u8,
    helmet: u8,
    firing: bool,
) {
    let (px, py) = (-fy, fx);
    // Shadow.
    fb.hline(x - 3, x + 3, y + 5, BLACK);
    fb.hline(x - 2, x + 2, y + 6, BLACK);
    // Legs swing along the facing direction.
    let swing = match walk % 4 {
        0 => 2.0,
        2 => -2.0,
        _ => 0.0,
    };
    for (side, s) in [(-1.0, swing), (1.0, -swing)] {
        let lx = x as f32 + px * side * 2.0 + fx * s;
        let ly = y as f32 + 2.0 + py * side * 2.0 + fy * s;
        fb.rect(lx as i32 - 1, ly as i32, 2, 3, DGREY);
    }
    // Body.
    fb.disc(x, y, 3, BLACK);
    fb.rect(x - 2, y - 2, 5, 5, body);
    fb.hline(x - 2, x + 2, y - 3, body);
    // The rifle.
    let (gx, gy) = (x as f32 + px * 2.0, y as f32 - 1.0 + py * 2.0);
    fb.line(
        gx as i32,
        gy as i32,
        (gx + fx * 7.0) as i32,
        (gy + fy * 7.0) as i32,
        BLACK,
    );
    fb.set((gx + fx * 7.0) as i32, (gy + fy * 7.0) as i32, LGREY);
    if firing {
        let (mx, my) = ((gx + fx * 9.0) as i32, (gy + fy * 9.0) as i32);
        fb.set(mx, my, WHITE);
        fb.set(mx + 1, my, YELLOW);
        fb.set(mx, my + 1, YELLOW);
        fb.set(mx - 1, my, YELLOW);
        fb.set(mx, my - 1, YELLOW);
    }
    // Helmet.
    fb.disc(x - fx.round() as i32 / 2, y - 2, 2, helmet);
    fb.set(x - 1, y - 3, WHITE);
}

fn tank(fb: &mut Fb, x: i32, y: i32, fx: f32, fy: f32, t: u32, flash: bool) {
    let (hull, light) = if flash {
        (WHITE, WHITE)
    } else {
        (GREEN, LGREEN)
    };
    for side in [-13, 8] {
        fb.rect(x + side, y - 15, 5, 30, BLACK);
        for k in 0..6 {
            let ty = y - 15 + ((k * 5 + (t / 4) as i32) % 30);
            fb.hline(x + side, x + side + 4, ty, DGREY);
        }
    }
    fb.rect(x - 9, y - 14, 18, 28, hull);
    fb.frame(x - 9, y - 14, 18, 28, light);
    fb.hline(x - 8, x + 7, y - 6, BROWN);
    fb.hline(x - 8, x + 7, y + 6, BROWN);
    fb.disc(x, y, 6, if flash { WHITE } else { DGREY });
    fb.disc(x, y, 4, light);
    fb.line(x, y, x + (fx * 15.0) as i32, y + (fy * 15.0) as i32, LGREY);
    fb.line(
        x + 1,
        y,
        x + 1 + (fx * 15.0) as i32,
        y + (fy * 15.0) as i32,
        GREY,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_map_is_rectangular_and_has_a_boss() {
        for (i, r) in MAP.iter().enumerate() {
            assert_eq!(r.len(), COLS, "row {i}");
        }
        assert!(MAP.iter().any(|r| r.contains('X')));
        let lv = Foot::new(Difficulty::Normal, 1);
        assert!(!lv.blocked(lv.px, lv.py, 4.0));
    }
}
