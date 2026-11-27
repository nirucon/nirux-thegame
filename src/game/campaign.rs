//! The campaign: the side-on sector scripts and the finale, which runs
//! three stages (side-on, isometric, on foot) as one sector.
use super::common::*;
use super::side::{Ev, SideSpec};
use super::Level;
use crate::config::Difficulty;
use crate::fb::{Fb, W};
use crate::input::Input;
use crate::palette::*;
use crate::songs::SongId;

fn waves(v: &mut Vec<(u32, Ev)>, from: u32, to: u32, every: u32, k0: u32, fire: bool) {
    let mut t = from;
    let mut k = k0;
    while t < to {
        let ev = match k % 6 {
            0 => Ev::Wisps {
                y: 60.0 + (k % 3) as f32 * 40.0,
                n: 6,
                amp: 26.0,
                fire,
            },
            1 => Ev::Hornets {
                top: k % 2 == 0,
                n: 4,
            },
            2 => Ev::Darts { n: 4 },
            3 => Ev::Wisps {
                y: 100.0,
                n: 8,
                amp: 50.0,
                fire,
            },
            4 => Ev::Hornets {
                top: k % 2 == 1,
                n: 5,
            },
            _ => Ev::Wisps {
                y: 150.0 - (k % 2) as f32 * 80.0,
                n: 7,
                amp: 30.0,
                fire,
            },
        };
        v.push((t, ev));
        t += every;
        k += 1;
    }
}

pub fn side_spec(sector: usize) -> SideSpec {
    match sector {
        3 => {
            // The Shipyards: gantries, two capital ships, the Foundry.
            let mut v = vec![(60, Ev::Text("THE SHIPYARDS: FLY THROUGH THE GANTRIES"))];
            waves(&mut v, 150, 600, 160, 0, true);
            v.push((650, Ev::Gantries { n: 3, gap: 64.0 }));
            v.push((
                1100,
                Ev::Pod {
                    y: 90.0,
                    drop: PowerUp::Weapon,
                },
            ));
            v.push((1250, Ev::Text("CAPITAL SHIP IN DRY DOCK")));
            v.push((1350, Ev::Capital));
            waves(&mut v, 1600, 3000, 260, 2, true);
            v.push((3100, Ev::Gantries { n: 4, gap: 56.0 }));
            v.push((3600, Ev::Gunship(1)));
            v.push((
                3700,
                Ev::Pod {
                    y: 140.0,
                    drop: PowerUp::Shield,
                },
            ));
            waves(&mut v, 3800, 4300, 170, 4, true);
            v.push((4350, Ev::Text("A SECOND SHIP LEAVES THE DOCK")));
            v.push((4450, Ev::Capital));
            waves(&mut v, 4700, 6200, 240, 1, true);
            v.push((6300, Ev::Gantries { n: 5, gap: 50.0 }));
            v.push((
                6900,
                Ev::Pod {
                    y: 70.0,
                    drop: PowerUp::Bomb,
                },
            ));
            waves(&mut v, 7000, 7900, 160, 3, true);
            v.push((8000, Ev::Text("WARNING: THE FOUNDRY")));
            v.push((8200, Ev::Boss(1)));
            SideSpec {
                script: v,
                border: ORANGE,
                hud: YELLOW,
                song: SongId::Level4,
                boss_song: SongId::Boss,
                planet: Some([LGREY, GREY, DGREY]),
                speed: 1.2,
            }
        }
        6 => {
            // The Gauntlet: fast, tunnels, old foes in harder forms.
            let mut v = vec![(60, Ev::Text("THE GAUNTLET: FULL SPEED"))];
            v.push((150, Ev::Rocks { frames: 1000 }));
            waves(&mut v, 300, 1150, 200, 2, true);
            v.push((1250, Ev::Text("TUNNEL AHEAD")));
            v.push((1350, Ev::Tunnel { frames: 1200 }));
            waves(&mut v, 1500, 2500, 220, 3, true);
            v.push((
                2600,
                Ev::Pod {
                    y: 105.0,
                    drop: PowerUp::Weapon,
                },
            ));
            v.push((2800, Ev::Text("AN OLD FOE RETURNS")));
            v.push((2900, Ev::MidBoss(2)));
            v.push((3100, Ev::Gantries { n: 5, gap: 50.0 }));
            waves(&mut v, 3700, 4300, 150, 0, true);
            v.push((4400, Ev::Tunnel { frames: 1500 }));
            waves(&mut v, 4500, 5800, 200, 1, true);
            v.push((
                5900,
                Ev::Pod {
                    y: 105.0,
                    drop: PowerUp::Shield,
                },
            ));
            v.push((6000, Ev::Gunship(2)));
            v.push((6100, Ev::Rocks { frames: 1000 }));
            waves(&mut v, 6200, 7300, 180, 4, true);
            v.push((7400, Ev::Text("WARNING: THE EXECUTOR")));
            v.push((7600, Ev::Boss(3)));
            SideSpec {
                script: v,
                border: DGREY,
                hud: LRED,
                song: SongId::Level7,
                boss_song: SongId::Boss,
                planet: Some([ORANGE, RED, BROWN]),
                speed: 2.0,
            }
        }
        7 => {
            // The finale, stage one.
            let mut v = vec![(60, Ev::Text("STAGE ONE: THE OUTER SHELL"))];
            waves(&mut v, 150, 1300, 170, 0, true);
            v.push((1350, Ev::Gantries { n: 4, gap: 54.0 }));
            v.push((
                1800,
                Ev::Pod {
                    y: 105.0,
                    drop: PowerUp::Weapon,
                },
            ));
            v.push((1900, Ev::Tunnel { frames: 900 }));
            waves(&mut v, 2000, 2900, 200, 3, true);
            v.push((3000, Ev::Gunship(2)));
            waves(&mut v, 3100, 3900, 160, 1, true);
            v.push((4000, Ev::Text("THE CORE GUARDIAN WAKES")));
            v.push((4200, Ev::Boss(4)));
            SideSpec {
                script: v,
                border: PURPLE,
                hud: LRED,
                song: SongId::Level8,
                boss_song: SongId::FinalBoss,
                planet: Some([LRED, RED, PURPLE]),
                speed: 1.5,
            }
        }
        _ => {
            let mut v = vec![
                (90, Ev::Text("WELCOME TO THE OUTER RIM")),
                (
                    150,
                    Ev::Wisps {
                        y: 70.0,
                        n: 5,
                        amp: 18.0,
                        fire: false,
                    },
                ),
                (
                    400,
                    Ev::Wisps {
                        y: 140.0,
                        n: 5,
                        amp: 18.0,
                        fire: false,
                    },
                ),
                (
                    640,
                    Ev::Pod {
                        y: 105.0,
                        drop: PowerUp::Weapon,
                    },
                ),
                (
                    820,
                    Ev::Wisps {
                        y: 100.0,
                        n: 7,
                        amp: 40.0,
                        fire: false,
                    },
                ),
                (1080, Ev::Hornets { top: true, n: 3 }),
                (1260, Ev::Hornets { top: false, n: 3 }),
                (
                    1450,
                    Ev::Wisps {
                        y: 60.0,
                        n: 6,
                        amp: 25.0,
                        fire: true,
                    },
                ),
                (
                    1500,
                    Ev::Wisps {
                        y: 150.0,
                        n: 6,
                        amp: 25.0,
                        fire: true,
                    },
                ),
                (1750, Ev::Darts { n: 3 }),
                (1950, Ev::Text("ASTEROID FIELD AHEAD")),
                (2050, Ev::Rocks { frames: 1100 }),
                (
                    2500,
                    Ev::Pod {
                        y: 70.0,
                        drop: PowerUp::Shield,
                    },
                ),
                (
                    2900,
                    Ev::Pod {
                        y: 140.0,
                        drop: PowerUp::Weapon,
                    },
                ),
                (3250, Ev::Hornets { top: true, n: 4 }),
                (
                    3420,
                    Ev::Wisps {
                        y: 90.0,
                        n: 8,
                        amp: 50.0,
                        fire: true,
                    },
                ),
                (3700, Ev::Text("CAPITAL SHIP ON RADAR")),
                (3800, Ev::Capital),
                (
                    4300,
                    Ev::Wisps {
                        y: 50.0,
                        n: 5,
                        amp: 12.0,
                        fire: true,
                    },
                ),
                (4900, Ev::Darts { n: 3 }),
                (
                    5400,
                    Ev::Pod {
                        y: 60.0,
                        drop: PowerUp::Bomb,
                    },
                ),
                (5500, Ev::Hornets { top: true, n: 4 }),
                (
                    6300,
                    Ev::Wisps {
                        y: 70.0,
                        n: 7,
                        amp: 30.0,
                        fire: true,
                    },
                ),
                (
                    6450,
                    Ev::Wisps {
                        y: 130.0,
                        n: 7,
                        amp: 30.0,
                        fire: true,
                    },
                ),
                (6650, Ev::Darts { n: 5 }),
                (
                    6900,
                    Ev::Pod {
                        y: 100.0,
                        drop: PowerUp::Weapon,
                    },
                ),
                (7000, Ev::Hornets { top: false, n: 5 }),
                (7300, Ev::Text("WARNING: THE WARDEN APPROACHES")),
            ];
            v.sort_by_key(|e| e.0);
            v.push((3150, Ev::Gunship(0)));
            v.push((7500, Ev::Boss(0)));
            v.sort_by_key(|e| e.0);
            SideSpec {
                script: v,
                border: BLUE,
                hud: LBLUE,
                song: SongId::Level1,
                boss_song: SongId::Boss,
                planet: Some([LBLUE, BLUE, PURPLE]),
                speed: 1.2,
            }
        }
    }
}

/// The finale: three stages played one after the other.
pub struct Chain {
    stages: Vec<Box<dyn Level>>,
    i: usize,
    between: u32,
    t: u32,
}

impl Chain {
    pub fn new(d: Difficulty, seed: u32) -> Chain {
        Chain {
            stages: vec![
                Box::new(super::side::Side::new(d, seed, 7, side_spec(7))),
                Box::new(super::iso::Iso::new(d, seed ^ 0x2222, 7)),
                Box::new(super::foot::Foot::new(d, seed ^ 0x3333, 7)),
            ],
            i: 0,
            between: 0,
            t: 0,
        }
    }
}

impl Level for Chain {
    fn update(&mut self, inp: &Input, st: &mut Stats, out: &mut Out) -> Status {
        self.t += 1;
        if self.between > 0 {
            self.between -= 1;
            if self.between == 0 {
                self.i += 1;
            }
            return Status::Playing;
        }
        match self.stages[self.i].update(inp, st, out) {
            Status::Cleared if self.i + 1 < self.stages.len() => {
                self.between = 180;
                out.play(crate::sfx::Sfx::PowerUp);
                Status::Playing
            }
            s => s,
        }
    }

    fn draw(&self, fb: &mut Fb, st: &Stats, hi: u32) {
        if self.between == 0 {
            self.stages[self.i].draw(fb, st, hi);
            return;
        }
        fb.clear(BLACK);
        for k in 0..6 {
            let y = ((self.t as i32 * 3 + k * 36) % 230) - 20;
            super::screens::raster_bar(fb, y, &RAMP_PURPLE);
        }
        fb.rect(0, 70, W, 60, BLACK);
        let s = format!("STAGE {} CLEAR", self.i + 1);
        fb.text_big_center(78, &s, 2, WHITE, Some(PURPLE));
        let next = ["", "DESCENDING TO THE CORE", "INTO THE HEART ON FOOT"][(self.i + 1).min(2)];
        if self.t % 30 < 20 {
            fb.text_center(108, next, LRED);
        }
        draw_hud(fb, st, hi, LRED);
    }

    fn bot(&mut self, st: &Stats) -> Input {
        if self.between > 0 {
            return Input::default();
        }
        self.stages[self.i].bot(st)
    }

    fn border(&self) -> u8 {
        if self.between > 0 {
            PURPLE
        } else {
            self.stages[self.i].border()
        }
    }

    fn progress(&self) -> f32 {
        (self.i as f32 + self.stages[self.i].progress()) / self.stages.len() as f32
    }

    fn info(&self) -> String {
        format!("stage {} {}", self.i + 1, self.stages[self.i].info())
    }
}
