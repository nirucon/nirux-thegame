//! Drawing the screens around the action: the title with its raster bars,
//! logo and sine scroller, the menus, the sector intro, pause, sector clear,
//! game over, the high score table and name entry, and the ending.
use super::common::{Stars, HUD_H};
use super::{Game, Scr, INFO, LEVELS};
use crate::fb::{text_w, Fb, H, W};
use crate::palette::*;
use std::sync::OnceLock;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const SCROLL: &str = "WELCOME TO NIRUX ...  AN ORIGINAL SHOOTER FOR THE NIRUX DESKTOP, \
    WITH PIXELS, RASTER BARS AND A THREE VOICE SYNTHESISER OF ITS OWN ...  \
    ARROWS OR WASD TO MOVE, SPACE OR CTRL TO FIRE, X OR SHIFT FOR BOMBS AND GRENADES, \
    ESC PAUSES, F11 FULLSCREEN, M MUTES ...  THIS DEMO HOLDS THREE SECTORS: \
    THE OUTER RIM, THE GLASS DUNES AND THE IRON JUNGLE ...  CLEAR A SECTOR AND IT \
    STAYS OPEN IN LEVEL SELECT ...  GOOD LUCK, PILOT ...          ";

fn stars() -> &'static std::sync::Mutex<Stars> {
    static S: OnceLock<std::sync::Mutex<Stars>> = OnceLock::new();
    S.get_or_init(|| std::sync::Mutex::new(Stars::new(90, 99)))
}

/// Copper bars: horizontal bands with a bright middle.
pub fn raster_bar(fb: &mut Fb, y: i32, ramp: &[u8]) {
    for (i, &c) in ramp.iter().enumerate() {
        fb.hline(0, W - 1, y + i as i32, c);
    }
}

/// Chrome: sky above the horizon line, sunset below it.
const CHROME: [u8; 42] = [
    WHITE, WHITE, WHITE, LGREY, CYAN, CYAN, CYAN, LBLUE, LBLUE, LBLUE, LBLUE, BLUE, BLUE, BLUE,
    BLUE, BLUE, LBLUE, LBLUE, CYAN, WHITE, BLACK, PURPLE, BROWN, RED, RED, ORANGE, ORANGE, ORANGE,
    LRED, LRED, LRED, YELLOW, YELLOW, YELLOW, YELLOW, WHITE, WHITE, YELLOW, YELLOW, LRED, ORANGE,
    BROWN,
];

/// The NIRUX logo: chrome blocks with a bevel, a drop shadow, a gentle
/// wave and a glint that sweeps across now and then.
pub fn logo(fb: &mut Fb, y0: i32, t: u32) {
    let a = crate::art::get();
    let rows = &a.logo;
    let cols = rows[0].len() as i32;
    let (bw, bh) = (5, 6);
    let x0 = (W - cols * bw) / 2;
    let filled = |r: i32, c: i32| {
        r >= 0
            && c >= 0
            && (r as usize) < rows.len()
            && (c as usize) < rows[0].len()
            && rows[r as usize].as_bytes()[c as usize] == b'#'
    };
    let glint = (t % 300) as i32 * 2 - 40;
    for pass in 0..2 {
        for r in 0..rows.len() as i32 {
            for py in 0..bh {
                let ly = r * bh + py;
                let y = y0 + ly;
                let wave = ((r as f32 * 0.7 + t as f32 * 0.05).sin() * 2.0).round() as i32;
                for c in 0..cols {
                    if !filled(r, c) {
                        continue;
                    }
                    for px in 0..bw {
                        let x = x0 + c * bw + px + wave;
                        if pass == 0 {
                            fb.rect(x - 1, y - 1, 3, 3, BLACK);
                            fb.set(x + 3, y + 3, BLUE);
                            continue;
                        }
                        let mut col = CHROME[(ly as usize).min(CHROME.len() - 1)];
                        let lx = c * bw + px;
                        let g = lx - ly / 2 - glint;
                        if (0..4).contains(&g) {
                            col = WHITE;
                        }
                        if (px == bw - 1 && !filled(r, c + 1)) || (py == bh - 1 && !filled(r + 1, c)) {
                            col = BLUE;
                        } else if (px == 0 && !filled(r, c - 1)) || (py == 0 && !filled(r - 1, c)) {
                            if col != BLACK {
                                col = WHITE;
                            }
                        }
                        fb.set(x, y, col);
                    }
                }
            }
        }
    }
}

fn scroller(fb: &mut Fb, y: i32, t: u32) {
    let chars: Vec<char> = SCROLL.chars().collect();
    let total = chars.len() as i32 * 8;
    let off = (t as i32 * 2) % total;
    let rainbow = [WHITE, YELLOW, LGREEN, CYAN, LBLUE, PURPLE, LRED, ORANGE, YELLOW];
    let first = off / 8;
    for i in 0..42 {
        let ci = (first + i) as usize % chars.len();
        let x = i * 8 - off % 8;
        let wy = ((x as f32 * 0.045 + t as f32 * 0.07).sin() * 5.0) as i32;
        let c = rainbow[((x / 16 + t as i32 / 6).rem_euclid(rainbow.len() as i32)) as usize];
        fb.char(x + 1, y + wy + 1, chars[ci], BLUE);
        fb.char(x, y + wy, chars[ci], c);
    }
}

fn menu_item(fb: &mut Fb, y: i32, s: &str, selected: bool, t: u32) {
    if selected {
        let ramp = [BLUE, LBLUE, BLUE];
        for (i, &c) in ramp.iter().enumerate() {
            fb.hline(60, W - 61, y - 2 + i as i32 * 4, c);
        }
        fb.rect(60, y - 1, W - 120, 9, BLUE);
        fb.hline(60, W - 61, y - 2, LBLUE);
        fb.hline(60, W - 61, y + 8, LBLUE);
        let w = text_w(s);
        let x = (W - w) / 2;
        fb.text_sh(x, y, s, if t % 20 < 14 { THEME_ACCENT } else { WHITE }, BLACK);
        let bob = ((t / 6) % 3) as i32;
        fb.char(x - 14 - bob, y, '>', YELLOW);
        fb.char(x + w + 6 + bob, y, '<', YELLOW);
    } else {
        fb.text_center_sh(y, s, THEME_FG, DGREY);
    }
}

fn title(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    if let Ok(mut s) = stars().lock() {
        s.update(0.8, 0.0);
        s.draw(fb, t);
    }
    // Three copper bars swinging behind the logo.
    let ramps: [&[u8]; 3] = [&RAMP_PURPLE, &RAMP_FIRE, &RAMP_BLUE];
    for (i, r) in ramps.iter().enumerate() {
        let y = 36 + ((t as f32 * 0.035 + i as f32 * 2.1).sin() * 24.0) as i32;
        raster_bar(fb, y, r);
    }
    logo(fb, 18, t);
    fb.text_center_sh(70, "A NIRUX DESKTOP ORIGINAL", LGREY, DGREY);
    let items = [
        "START GAME".to_string(),
        "LEVEL SELECT".to_string(),
        format!("DIFFICULTY  {}", g.cfg.difficulty.name()),
        "OPTIONS".to_string(),
        "HIGH SCORES".to_string(),
        "QUIT".to_string(),
    ];
    for (i, s) in items.iter().enumerate() {
        menu_item(fb, 88 + i as i32 * 12, s, g.sel == i, t);
    }
    // A floor of raster lines under the scroller.
    for (i, &c) in [BLUE, BLACK, LBLUE, BLACK, BLUE, BLUE].iter().enumerate() {
        fb.hline(0, W - 1, 172 + i as i32 * 2, c);
    }
    scroller(fb, 183, t);
    fb.text(W - text_w(VERSION) - 16, 162, &format!("V{VERSION}"), DGREY);
    fb.text(8, 162, &format!("HI {:07}", g.hi()), DGREY);
}

fn select(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    if let Ok(mut s) = stars().lock() {
        s.update(0.5, 0.0);
        s.draw(fb, t);
    }
    raster_bar(fb, 14, &RAMP_BLUE);
    fb.text_big_center(26, "LEVEL SELECT", 2, WHITE, Some(BLUE));
    let reached = g.save.best_reached() as usize;
    for (i, info) in INFO.iter().enumerate() {
        let y = 62 + i as i32 * 26;
        let open = i < reached;
        let sel = g.sel == i;
        if sel {
            fb.rect(24, y - 4, W - 48, 22, if open { BLUE } else { DGREY });
            fb.frame(24, y - 4, W - 48, 22, if t % 16 < 8 { LBLUE } else { CYAN });
        }
        fb.rect(32, y - 1, 12, 15, info.border);
        fb.text(34, y + 3, &format!("{}", i + 1), WHITE);
        if open {
            fb.text_sh(52, y, &format!("SECTOR {}  {}", i + 1, info.name), if sel { THEME_ACCENT } else { WHITE }, BLACK);
            fb.text(52, y + 9, info.mode, LGREY);
        } else {
            fb.text(52, y, &format!("SECTOR {}  ??????", i + 1), GREY);
            fb.text(52, y + 9, "LOCKED", DGREY);
        }
    }
    let y = 62 + LEVELS as i32 * 26 + 4;
    menu_item(fb, y, "BACK", g.sel == LEVELS, t);
    fb.text_center(176, "FIVE MORE SECTORS IN THE FULL CAMPAIGN", DGREY);
    fb.text_center(188, &format!("DIFFICULTY {}", g.cfg.difficulty.name()), GREY);
}

fn options(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    raster_bar(fb, 14, &RAMP_GREEN);
    fb.text_big_center(26, "OPTIONS", 2, WHITE, Some(GREEN));
    let c = &g.cfg;
    let on = |b: bool| if b { "ON" } else { "OFF" };
    let bar = |v: u8| {
        let mut s = String::new();
        for i in 0..10 {
            s.push(if i < v { '#' } else { '-' });
        }
        s
    };
    let items = [
        format!("CRT SCANLINES   {:>3}", on(c.crt)),
        format!("FULLSCREEN      {:>3}", on(c.fullscreen)),
        format!("MUSIC  {}", bar(c.music)),
        format!("EFFECTS {}", bar(c.sfx)),
        format!("ECHO            {:>3}", on(c.echo)),
        "BACK".to_string(),
    ];
    for (i, s) in items.iter().enumerate() {
        menu_item(fb, 64 + i as i32 * 16, s, g.sel == i, t);
    }
    fb.text_center(170, "LEFT AND RIGHT CHANGE A SETTING", GREY);
    fb.text_center(182, "F11 FULLSCREEN   M MUTE", DGREY);
}

pub fn scores(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    if let Ok(mut s) = stars().lock() {
        s.update(0.0, -0.5);
        s.draw(fb, t);
    }
    raster_bar(fb, 10, &RAMP_FIRE);
    fb.text_big_center(22, "HALL OF FAME", 2, YELLOW, Some(RED));
    fb.text(40, 46, "RANK  NAME   SCORE  SECTOR", GREY);
    let ramp = [WHITE, YELLOW, LGREEN, CYAN, LBLUE, PURPLE, LRED, ORANGE, YELLOW, LGREEN];
    for (i, s) in g.save.scores.iter().enumerate() {
        let y = 58 + i as i32 * 11;
        let mine = g.last_place == Some(i) && t % 20 < 12;
        let c = if mine { WHITE } else { ramp[(i + t as usize / 8) % ramp.len()] };
        let line = format!(
            "{:>2}.   {:<3}  {:07}    {}  {}",
            i + 1,
            s.name,
            s.score,
            s.sector,
            &s.difficulty.name()[..1]
        );
        fb.text_sh(40, y, &line, c, DGREY);
    }
    fb.text_center(176, "E EASY  N NORMAL  H HARD", DGREY);
    if t % 40 < 26 {
        fb.text_center(188, "PRESS FIRE", WHITE);
    }
}

fn intro(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    let info = &INFO[g.lvl];
    let ramp: &[u8] = match g.lvl {
        0 => &RAMP_BLUE,
        1 => &RAMP_FIRE,
        _ => &RAMP_GREEN,
    };
    for i in 0..6 {
        let y = ((t as i32 * 2 + i * 40) % 260) - 30;
        raster_bar(fb, y, ramp);
    }
    fb.rect(0, 40, W, 120, BLACK);
    fb.hline(0, W - 1, 40, info.border);
    fb.hline(0, W - 1, 159, info.border);
    let show = t.min(60) as i32;
    let s = format!("SECTOR {}", g.lvl + 1);
    let x = W / 2 - text_w(&s) * 3 / 2;
    fb.text_big(x - (60 - show) * 6 + 3, 52 + 3, &s, 3, info.border);
    fb.text_big(x - (60 - show) * 6, 52, &s, 3, WHITE);
    if t > 30 {
        fb.text_big_center(86, info.name, 2, YELLOW, Some(ORANGE));
    }
    if t > 60 {
        fb.text_center(112, info.tip[0], LGREY);
        fb.text_center(122, info.tip[1], LGREY);
    }
    if t > 90 && t % 30 < 20 {
        fb.text_center_sh(142, "GET READY", WHITE, info.border);
    }
    fb.text_center(170, info.mode, GREY);
    fb.text_center(182, &format!("LIVES {}   {}", g.stats.lives, g.diff.name()), DGREY);
}

const DARK: [u8; 16] = [0, 12, 9, 6, 6, 9, 0, 8, 9, 0, 2, 0, 11, 5, 6, 11];

fn pause(g: &Game, fb: &mut Fb) {
    if let Some(l) = &g.level {
        l.draw(fb, &g.stats, g.hi());
    }
    fb.remap(0, HUD_H, W, H - HUD_H, &DARK);
    fb.remap(0, HUD_H, W, H - HUD_H, &DARK);
    fb.text_big_center(64, "PAUSED", 3, WHITE, Some(BLUE));
    menu_item(fb, 108, "CONTINUE", g.sel == 0, g.t);
    menu_item(fb, 122, "QUIT TO TITLE", g.sel == 1, g.t);
}

fn clear(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    let info = &INFO[g.lvl];
    for i in 0..3 {
        let y = 16 + ((t as f32 * 0.04 + i as f32 * 2.0).sin() * 10.0) as i32;
        raster_bar(fb, y, &RAMP_GREEN);
    }
    fb.text_big_center(34, &format!("SECTOR {}", g.lvl + 1), 2, WHITE, Some(info.border));
    fb.text_big_center(54, "CLEAR", 3, if t % 12 < 6 { YELLOW } else { WHITE }, Some(ORANGE));
    let (kills, bonus, nodeath) = g.tally;
    let mut y = 92;
    let line = |fb: &mut Fb, y: i32, a: &str, b: String, c: u8| {
        fb.text(40, y, a, LGREY);
        fb.text(W - 40 - text_w(&b), y, &b, c);
    };
    if t > 20 {
        line(fb, y, "ENEMIES DESTROYED", kills.to_string(), WHITE);
    }
    y += 14;
    let shown = g.tally_shown;
    if t > 40 {
        line(fb, y, "SECTOR BONUS", shown.min(bonus).to_string(), YELLOW);
    }
    y += 14;
    if t > 50 {
        let s = if nodeath > 0 {
            shown.saturating_sub(bonus).min(nodeath).to_string()
        } else {
            "-".to_string()
        };
        line(fb, y, "NO LOSS BONUS", s, LGREEN);
    }
    y += 20;
    if t > 60 {
        let total = g.stats.score - (bonus + nodeath) + shown;
        line(fb, y, "SCORE", format!("{total:07}"), WHITE);
    }
    if t > 150 && t % 30 < 20 {
        let s = if g.lvl + 1 < LEVELS { "PRESS FIRE FOR THE NEXT SECTOR" } else { "PRESS FIRE" };
        fb.text_center(176, s, CYAN);
    }
}

fn over(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    for i in 0..4 {
        let y = 60 + i * 22 + ((t as f32 * 0.05 + i as f32).sin() * 6.0) as i32;
        raster_bar(fb, y, &RAMP_FIRE);
    }
    fb.rect(0, 70, W, 50, BLACK);
    let k = t.min(50) as i32;
    fb.text_big_center(78 - (50 - k), "GAME", 3, if t % 8 < 4 { LRED } else { RED }, Some(BROWN));
    fb.text_big_center(78 + 26 + (50 - k), "OVER", 3, if t % 8 < 4 { LRED } else { RED }, Some(BROWN));
    if t > 60 {
        fb.text_center_sh(150, &format!("SCORE {:07}", g.stats.score), WHITE, DGREY);
        fb.text_center(164, &format!("SECTOR {}", g.lvl + 1), GREY);
    }
}

fn entry(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    raster_bar(fb, 18, &RAMP_PURPLE);
    fb.text_big_center(34, "NEW HIGH SCORE", 2, WHITE, Some(PURPLE));
    fb.text_center(62, &format!("{:07}", g.stats.score), YELLOW);
    fb.text_center(80, "ENTER YOUR NAME", LGREY);
    let x0 = W / 2 - 3 * 24 / 2 - 8;
    for i in 0..3 {
        let x = x0 + i as i32 * 32;
        let c = g.entry[i] as char;
        let sel = g.entry_pos == i;
        if sel {
            fb.rect(x - 2, 98, 28, 30, BLUE);
        }
        let col = if sel && t % 16 < 8 { YELLOW } else { WHITE };
        fb.text_big(x + 1, 102, &c.to_string(), 3, col);
        fb.hline(x, x + 23, 128, if sel { CYAN } else { GREY });
    }
    fb.text_center(150, "UP AND DOWN PICK A LETTER", GREY);
    fb.text_center(162, "FIRE NEXT   ENTER DONE", GREY);
}

fn victory(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    if let Ok(mut s) = stars().lock() {
        s.update(2.0, 0.0);
        s.draw(fb, t);
    }
    let ramps: [&[u8]; 3] = [&RAMP_FIRE, &RAMP_GREEN, &RAMP_BLUE];
    for (i, r) in ramps.iter().enumerate() {
        let y = 20 + ((t as f32 * 0.05 + i as f32 * 2.1).sin() * 12.0) as i32;
        raster_bar(fb, y, r);
    }
    logo(fb, 34, t);
    let lines = [
        "CONGRATULATIONS, PILOT!",
        "",
        "THE OUTER RIM, THE GLASS DUNES",
        "AND THE IRON JUNGLE ARE FREE.",
        "",
        "THE FULL CAMPAIGN OF EIGHT SECTORS",
        "IS ON ITS WAY.",
        "",
        &format!("FINAL SCORE {:07}", g.stats.score),
    ];
    for (i, l) in lines.iter().enumerate() {
        if t as usize > 30 + i * 15 {
            let c = if i == 0 { [YELLOW, WHITE, CYAN][(t as usize / 6) % 3] } else { LGREY };
            fb.text_center(92 + i as i32 * 10, l, c);
        }
    }
}

pub fn draw(g: &Game, fb: &mut Fb) {
    match g.scr {
        Scr::Title => title(g, fb),
        Scr::Select => select(g, fb),
        Scr::Options => options(g, fb),
        Scr::Scores => scores(g, fb),
        Scr::Intro => intro(g, fb),
        Scr::Play | Scr::Demo => {
            if let Some(l) = &g.level {
                l.draw(fb, &g.stats, g.hi());
            }
            if g.scr == Scr::Demo {
                fb.rect(0, H - 22, W, 12, BLACK);
                if g.t % 40 < 28 {
                    fb.text_center_sh(H - 20, "DEMO   PRESS FIRE TO PLAY", YELLOW, RED);
                }
            }
        }
        Scr::Pause => pause(g, fb),
        Scr::Clear => clear(g, fb),
        Scr::Over => over(g, fb),
        Scr::Entry => entry(g, fb),
        Scr::Victory => victory(g, fb),
    }
}
