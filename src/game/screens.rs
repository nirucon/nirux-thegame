//! Drawing the screens around the action: the title with its raster bars,
//! logo and sine scroller, the menus, the sector intro, pause, sector clear,
//! game over, the high score table and name entry, and the ending.
use super::common::{Stars, HUD_H};
use super::{Game, Scr, INFO, LEVELS};
use crate::fb::{text_w, Fb, H, W};
use crate::palette::*;
use std::sync::OnceLock;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const SCROLL: &str = "NIRUX ...  THE CORE HAS FALLEN AND NULL HOLDS ALL EIGHT SECTORS ...  \
    ONE GUARDIAN IS LEFT ...  ARROWS OR WASD TO MOVE, SPACE OR CTRL TO FIRE, \
    X OR SHIFT FOR BOMBS AND GRENADES, ESC PAUSES, F11 FULLSCREEN, M MUTES ...  \
    GAMEPADS WORK TOO, PLUG ONE IN ANY TIME ...  CLEAR A SECTOR AND IT STAYS OPEN \
    IN LEVEL SELECT ...  AN ORIGINAL GAME FOR THE NIRUX DESKTOP, WITH PIXELS, \
    RASTER BARS AND A THREE VOICE SYNTHESISER OF ITS OWN ...  \
    THE CORE IS WAITING, NIRUX ...          ";

/// The prologue, page by page. Lines fit the screen (38 characters).
const STORY: [&[&str]; 4] = [
    &[
        "LONG AGO THE NIRUX CORE WAS BUILT",
        "TO KEEP EIGHT SECTORS ALIVE.",
        "",
        "DEEP INSIDE IT SLEPT A GUARDIAN,",
        "MADE FOR ONE DAY ONLY:",
        "THE DAY THE CORE WOULD FALL.",
    ],
    &[
        "THAT DAY CAME OUT OF THE STATIC.",
        "ITS NAME WAS NULL.",
        "",
        "NULL TOOK THE CORE AND SEALED",
        "THE SECTORS ONE BY ONE, UNTIL",
        "ONLY SILENCE WAS LEFT.",
    ],
    &[
        "WITH ITS LAST POWER THE CORE",
        "WOKE ITS GUARDIAN AND THREW IT",
        "TO THE OUTER RIM, FAR FROM NULL.",
        "",
        "NO FACE. NO VOICE. ONE NAME.",
    ],
    &[
        "",
        "",
        "",
        "EIGHT SECTORS STAND BETWEEN YOU",
        "AND THE CORE.",
        "TAKE THEM BACK.",
    ],
];

pub const STORY_PAGES: usize = STORY.len();

/// How many characters a page types out.
pub fn story_page_len(p: usize) -> usize {
    STORY
        .get(p)
        .map(|l| l.iter().map(|s| s.len() + 4).sum())
        .unwrap_or(0)
}

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
                        if (px == bw - 1 && !filled(r, c + 1))
                            || (py == bh - 1 && !filled(r + 1, c))
                        {
                            col = BLUE;
                        } else if ((px == 0 && !filled(r, c - 1)) || (py == 0 && !filled(r - 1, c)))
                            && col != BLACK
                        {
                            col = WHITE;
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
    let rainbow = [
        WHITE, YELLOW, LGREEN, CYAN, LBLUE, PURPLE, LRED, ORANGE, YELLOW,
    ];
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
        fb.text_sh(
            x,
            y,
            s,
            if t % 20 < 14 { THEME_ACCENT } else { WHITE },
            BLACK,
        );
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
        "CONTROLS".to_string(),
        "OPTIONS".to_string(),
        "HIGH SCORES".to_string(),
        "QUIT".to_string(),
    ];
    for (i, s) in items.iter().enumerate() {
        menu_item(fb, 82 + i as i32 * 11, s, g.sel == i, t);
    }
    // A floor of raster lines under the scroller.
    for (i, &c) in [BLUE, BLACK, LBLUE, BLACK, BLUE, BLUE].iter().enumerate() {
        fb.hline(0, W - 1, 172 + i as i32 * 2, c);
    }
    // The hero keeps watch beside the menu.
    let glow = if t % 400 < 6 { 0.5 } else { 1.0 };
    crate::hero::draw(fb, 282, 118, 11.0, t, glow);
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
    raster_bar(fb, 8, &RAMP_BLUE);
    fb.text_big_center(18, "LEVEL SELECT", 2, WHITE, Some(BLUE));
    let reached = g.save.best_reached() as usize;
    for (i, info) in INFO.iter().enumerate() {
        let y = 42 + i as i32 * 14;
        let open = i < reached;
        let sel = g.sel == i;
        if sel {
            fb.rect(20, y - 3, W - 40, 13, if open { BLUE } else { DGREY });
            fb.frame(20, y - 3, W - 40, 13, if t % 16 < 8 { LBLUE } else { CYAN });
        }
        fb.rect(26, y - 2, 11, 11, info.border);
        fb.text(28, y, &format!("{}", i + 1), WHITE);
        if open {
            let c = if sel { THEME_ACCENT } else { WHITE };
            fb.text_sh(46, y, info.name, c, BLACK);
        } else {
            fb.text(46, y, "LOCKED", DGREY);
        }
    }
    menu_item(fb, 42 + LEVELS as i32 * 14 + 2, "BACK", g.sel == LEVELS, t);
    if let Some(info) = INFO.get(g.sel) {
        if g.sel < reached {
            fb.text_center(170, info.mode, CYAN);
        } else {
            fb.text_center(170, "CLEAR THE SECTOR BEFORE TO OPEN IT", GREY);
        }
    }
    fb.text_center(
        186,
        &format!("DIFFICULTY {}", g.cfg.difficulty.name()),
        GREY,
    );
}

/// Keep only what the 8x8 font can draw.
fn printable(s: &str, max: usize) -> String {
    s.to_ascii_uppercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || " .,:-+/()'".contains(c) {
                c
            } else {
                ' '
            }
        })
        .take(max)
        .collect()
}

/// A gamepad, drawn in C64 colours, with the fire and special buttons lit.
fn pad_picture(fb: &mut Fb, cx: i32, cy: i32, t: u32) {
    // Body and grips.
    fb.disc(cx - 34, cy + 6, 14, GREY);
    fb.disc(cx + 34, cy + 6, 14, GREY);
    fb.rect(cx - 36, cy - 12, 72, 22, GREY);
    fb.disc(cx - 34, cy - 2, 12, GREY);
    fb.disc(cx + 34, cy - 2, 12, GREY);
    fb.hline(cx - 34, cx + 34, cy - 13, LGREY);
    // D-pad.
    fb.rect(cx - 41, cy - 7, 14, 4, BLACK);
    fb.rect(cx - 36, cy - 12, 4, 14, BLACK);
    // Sticks.
    fb.disc(cx - 16, cy + 6, 5, DGREY);
    fb.disc(cx + 16, cy + 6, 5, DGREY);
    fb.disc(cx - 16, cy + 6, 3, BLACK);
    fb.disc(cx + 16, cy + 6, 3, BLACK);
    // Start and select.
    fb.rect(cx - 9, cy - 8, 6, 2, DGREY);
    fb.rect(cx + 4, cy - 8, 6, 2, DGREY);
    // Face buttons: south fires, east is special.
    let blink = t % 30 < 15;
    fb.disc(cx + 34, cy + 1, 3, if blink { LGREEN } else { GREEN });
    fb.disc(cx + 41, cy - 6, 3, if !blink { LRED } else { RED });
    fb.disc(cx + 27, cy - 6, 3, LBLUE);
    fb.disc(cx + 34, cy - 13, 3, YELLOW);
}

fn controls(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    raster_bar(fb, 4, &RAMP_GREEN);
    fb.text_big_center(14, "CONTROLS", 2, WHITE, Some(GREEN));
    pad_picture(fb, 160, 50, t);
    fb.text(198, 62, "FIRE", LGREEN);
    fb.text(210, 36, "SPECIAL", LRED);
    fb.line(195, 64, 194, 52, GREEN);
    fb.line(208, 40, 202, 44, RED);
    let rows: [(&str, &str, &str); 8] = [
        ("MOVE", "ARROWS WASD", "STICK D-PAD"),
        ("FIRE", "SPACE CTRL Z", "A CROSS RT"),
        ("SPECIAL", "X SHIFT", "B CIRCLE LT"),
        ("PAUSE", "ESC P", "START"),
        ("BACK", "ESC", "SELECT"),
        ("CONFIRM", "ENTER", "A START"),
        ("SCREEN", "F11", ""),
        ("SOUND", "M MUTES", ""),
    ];
    fb.text(4, 76, "ACTION", GREY);
    fb.text(84, 76, "KEYBOARD", GREY);
    fb.text(196, 76, "GAMEPAD", GREY);
    fb.hline(4, W - 5, 85, DGREY);
    for (i, (a, k, p)) in rows.iter().enumerate() {
        let y = 89 + i as i32 * 9;
        fb.text(4, y, a, YELLOW);
        fb.text(84, y, k, WHITE);
        fb.text(196, y, p, CYAN);
    }
    let pad = match &g.pad {
        crate::pad::Status::Connected(n) => (format!("PAD: {}", printable(n, 34)), LGREEN),
        crate::pad::Status::NotReadable(_) => {
            ("PAD FOUND, NOT READABLE: CHECK PERMISSIONS".into(), LRED)
        }
        crate::pad::Status::None => ("NO GAMEPAD FOUND (PLUG ONE IN ANY TIME)".into(), GREY),
    };
    fb.text_center(164, &printable(&pad.0, 40), pad.1);
    let hint = if (t / 180) % 2 == 0 {
        "FLIGHT: UP, DOWN CHANGE ALTITUDE"
    } else {
        "ON FOOT: HOLD FIRE TO KEEP YOUR AIM"
    };
    fb.text_center(176, hint, LGREY);
    if t % 40 < 28 {
        fb.text_center(188, "PRESS FIRE TO GO BACK", WHITE);
    }
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
    fb.text_center(170, "LEFT AND RIGHT CHANGE A SETTING", THEME_MUTED);
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
    let ramp = [
        WHITE, YELLOW, LGREEN, CYAN, LBLUE, PURPLE, LRED, ORANGE, YELLOW, LGREEN,
    ];
    for (i, s) in g.save.scores.iter().enumerate() {
        let y = 58 + i as i32 * 11;
        let mine = g.last_place == Some(i) && t % 20 < 12;
        let c = if mine {
            WHITE
        } else {
            ramp[(i + t as usize / 8) % ramp.len()]
        };
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
    fb.rect(0, 30, W, 143, BLACK);
    fb.hline(0, W - 1, 30, info.border);
    fb.hline(0, W - 1, 172, info.border);
    let show = t.min(60) as i32;
    let s = format!("SECTOR {}", g.lvl + 1);
    let x = W / 2 - text_w(&s) * 3 / 2;
    fb.text_big(x - (60 - show) * 6 + 3, 38 + 3, &s, 3, info.border);
    fb.text_big(x - (60 - show) * 6, 38, &s, 3, WHITE);
    if t > 30 {
        fb.text_big_center(68, info.name, 2, YELLOW, Some(ORANGE));
    }
    if t > 50 {
        fb.text_center(88, info.mode, CYAN);
    }
    // ECHO speaks: the Core's voice, typed out over the static.
    if t > 56 {
        let k = (t as usize - 56) / 2;
        let tag = "ECHO:";
        fb.text(10, 104, tag, if t % 24 < 16 { LGREEN } else { GREEN });
        let a = info.brief[0];
        let b = info.brief[1];
        let x0 = 10 + text_w(tag) + 8;
        fb.text(x0, 104, &a[..k.min(a.len())], LGREEN);
        if k > a.len() {
            fb.text(x0, 114, &b[..(k - a.len()).min(b.len())], LGREEN);
        }
    }
    if t > 110 {
        fb.text_center(130, info.tip[0], LGREY);
        fb.text_center(140, info.tip[1], LGREY);
    }
    if t > 130 && t % 30 < 20 {
        fb.text_center_sh(154, "GET READY", WHITE, info.border);
    }
    fb.text_center(
        164,
        &format!("LIVES {}   {}", g.stats.lives, g.diff.name()),
        GREY,
    );
}

/// The prologue: the hero's portrait over a slow raster sky, the story
/// typed out page by page. NULL's page tears the picture with static; on
/// the third page the visor lights for the first time.
fn story(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    let p = g.page.min(STORY_PAGES - 1);
    let pt = g.page_t;
    if let Ok(mut s) = stars().lock() {
        s.update(0.3, 0.0);
        s.draw(fb, t);
    }
    for i in 0..3 {
        let y = 70 + ((t as f32 * 0.02 + i as f32 * 2.1).sin() * 16.0) as i32;
        let ramp: &[u8] = if p == 1 { &RAMP_FIRE } else { &RAMP_PURPLE };
        raster_bar(fb, y, ramp);
    }
    let glow = match p {
        0 | 1 => 0.0,
        2 => {
            // The visor flickers on as the line about waking types out.
            let k = pt as f32 / 90.0;
            if k < 1.0 && (pt / 3) % 2 == 0 {
                0.3
            } else {
                k.min(1.0)
            }
        }
        _ => 1.0,
    };
    if p == 3 {
        logo(fb, 18, t);
        crate::hero::draw(fb, 160, 76, 13.0, t, glow);
    } else {
        crate::hero::draw(fb, 160, 46, 24.0, t, glow);
    }
    if p == 1 {
        // NULL: rows of static that tear across the picture.
        let mut r = t.wrapping_mul(2_654_435_761);
        for _ in 0..(6 + (pt / 20).min(14)) {
            r = r.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let y = (r >> 8) as i32 % 96 + 4;
            let len = (r >> 20) as i32 % 120 + 20;
            let x = (r >> 4) as i32 % W;
            for k in 0..len {
                let c = if (k + y) % 3 == 0 { WHITE } else { GREY };
                if (r >> (k % 16)) & 1 == 1 {
                    fb.set(x + k - 60, y, c);
                }
            }
        }
    }
    // The text, typed two frames a character.
    let lines = STORY[p];
    let mut left = pt as usize / 2;
    let y0 = 106;
    fb.rect(0, y0 - 6, W, 76, BLACK);
    fb.hline(0, W - 1, y0 - 6, DGREY);
    fb.hline(0, W - 1, y0 + 69, DGREY);
    for (i, l) in lines.iter().enumerate() {
        let n = left.min(l.len());
        let y = y0 + i as i32 * 11;
        let x = (W - text_w(l)) / 2;
        let c = if p == 1 && i == 1 { LRED } else { LGREY };
        fb.text(x, y, &l[..n], c);
        if n < l.len() {
            if n > 0 || left > 0 {
                fb.rect(
                    x + text_w(&l[..n]),
                    y,
                    7,
                    7,
                    if t % 16 < 8 { LGREEN } else { BLACK },
                );
            }
            break;
        }
        left -= n + 4.min(left - n);
    }
    if p == 2 && pt as usize / 2 >= story_page_len(2) {
        fb.text_big_center(
            y0 + 53,
            "NIRUX",
            2,
            if t % 12 < 6 { LGREEN } else { WHITE },
            Some(GREEN),
        );
    }
    if t % 60 < 44 {
        let s = if g.story_attract {
            "PRESS FIRE"
        } else {
            "FIRE: NEXT    ESC: SKIP"
        };
        fb.text_center(188, s, DGREY);
    }
}

const DARK: [u8; 16] = [0, 12, 9, 6, 6, 9, 0, 8, 9, 0, 2, 0, 11, 5, 6, 11];

fn pause(g: &Game, fb: &mut Fb) {
    if let Some(l) = &g.level {
        l.draw(fb, &g.stats, g.hi());
    }
    fb.remap(0, HUD_H, W, H - HUD_H, &DARK);
    fb.remap(0, HUD_H, W, H - HUD_H, &DARK);
    raster_bar(fb, 54, &RAMP_GREY);
    fb.rect(0, 57, W, 1, WHITE);
    fb.text_big_center(64, "PAUSED", 3, WHITE, Some(BLUE));
    if let Some(l) = &g.level {
        let p = (l.progress() * 100.0) as u32;
        fb.text_center(144, &format!("SECTOR {}  {}% DONE", g.lvl + 1, p), LGREY);
    }
    menu_item(fb, 104, "CONTINUE", g.sel == 0, g.t);
    menu_item(fb, 117, "CONTROLS", g.sel == 1, g.t);
    menu_item(fb, 130, "QUIT TO TITLE", g.sel == 2, g.t);
}

fn clear(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    let info = &INFO[g.lvl];
    if let Ok(mut s) = stars().lock() {
        s.update(1.5, 0.0);
        s.draw(fb, t);
    }
    for i in 0..3 {
        let y = 16 + ((t as f32 * 0.04 + i as f32 * 2.0).sin() * 10.0) as i32;
        raster_bar(fb, y, &RAMP_GREEN);
        let y = 187 + ((t as f32 * 0.05 + i as f32 * 2.0).sin() * 4.0) as i32;
        raster_bar(fb, y, &RAMP_BLUE);
    }
    // The ship crosses the screen.
    let sx = (t as i32 * 3) % (W + 120) - 60;
    let ship = &crate::art::get().ship;
    fb.blit(ship, sx, 164);
    for i in 0..6 {
        fb.set(
            sx - 1 - i,
            170,
            [WHITE, YELLOW, YELLOW, ORANGE, RED, BROWN][i as usize],
        );
    }
    fb.text_big_center(
        34,
        &format!("SECTOR {}", g.lvl + 1),
        2,
        WHITE,
        Some(info.border),
    );
    fb.text_big_center(
        54,
        "CLEAR",
        3,
        if t % 12 < 6 { YELLOW } else { WHITE },
        Some(ORANGE),
    );
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
        let s = if g.lvl + 1 < LEVELS {
            "PRESS FIRE FOR THE NEXT SECTOR"
        } else {
            "PRESS FIRE"
        };
        fb.text_center(153, s, CYAN);
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
    fb.text_big_center(
        78 - (50 - k),
        "GAME",
        3,
        if t % 8 < 4 { LRED } else { RED },
        Some(BROWN),
    );
    fb.text_big_center(
        78 + 26 + (50 - k),
        "OVER",
        3,
        if t % 8 < 4 { LRED } else { RED },
        Some(BROWN),
    );
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

const CREDITS: &[&str] = &[
    "NIRUX",
    "",
    "AN ORIGINAL GAME",
    "FOR THE NIRUX DESKTOP",
    "",
    "* THE SECTORS *",
    "THE OUTER RIM",
    "GLASS DUNES",
    "IRON JUNGLE",
    "THE SHIPYARDS",
    "ICE PYLONS",
    "BURNING CITY",
    "THE GAUNTLET",
    "THE NIRUX CORE",
    "",
    "* STARRING *",
    "NIRUX, THE GUARDIAN",
    "ECHO, THE VOICE OF THE CORE",
    "AND NULL",
    "",
    "* THE SOUNDTRACK *",
    "NIRUX  LEGEND OF NIRUX",
    "OUTER RIM  GLASS DUNES",
    "IRON JUNGLE  SHIPYARDS",
    "ICE PYLONS  BURNING CITY",
    "THE GAUNTLET  THE CORE",
    "WARDEN  HEART OF NIRUX",
    "FREEDOM",
    "",
    "PLAYED LIVE BY A",
    "THREE VOICE SYNTHESISER",
    "WRITTEN FOR THIS GAME",
    "",
    "* MADE WITH *",
    "RUST, X11 AND",
    "THE 16 COLOURS OF 1982",
    "",
    "PIXELS, MUSIC AND CODE",
    "MADE FOR THE NIRUX SUITE",
    "",
    "",
    "THANK YOU FOR PLAYING",
    "",
    "",
    "NIRUX WILL RETURN IN",
    "",
    "NIRUX II",
    "NIGHT OF THE DAEMONS",
];

/// How far the credits scroll before they stop.
pub fn credits_stop() -> i32 {
    H + (CREDITS.len() as i32 - 2) * 14 - 92
}

fn victory(g: &Game, fb: &mut Fb) {
    fb.clear(BLACK);
    let t = g.t;
    if let Ok(mut s) = stars().lock() {
        s.update(1.0, 0.0);
        s.draw(fb, t);
    }
    if t < 600 {
        let ramps: [&[u8]; 3] = [&RAMP_FIRE, &RAMP_GREEN, &RAMP_BLUE];
        for (i, r) in ramps.iter().enumerate() {
            let y = 20 + ((t as f32 * 0.05 + i as f32 * 2.1).sin() * 12.0) as i32;
            raster_bar(fb, y, r);
        }
        crate::hero::draw(fb, 160, 40, 17.0, t, 1.0);
        let score = format!("FINAL SCORE {:07}", g.stats.score);
        let lines = [
            "THE CORE IS FREE, NIRUX.",
            "THE EIGHT SECTORS WAKE AGAIN.",
            "",
            "BUT NULL WAS NOT DESTROYED.",
            "HE FLED DOWN INTO THE DARK,",
            "WHERE THE DELETED THINGS GO.",
            "",
            &score,
        ];
        for (i, l) in lines.iter().enumerate() {
            if t as usize > 40 + i * 30 {
                let c = if i == 0 {
                    [YELLOW, WHITE, CYAN][(t as usize / 6) % 3]
                } else if (3..6).contains(&i) {
                    LRED
                } else {
                    LGREY
                };
                fb.text_center(88 + i as i32 * 10, l, c);
            }
        }
        // The ship flies home.
        let x = (t as i32 * 2) % (W + 80) - 40;
        fb.blit(&crate::art::get().ship, x, 176);
        return;
    }
    // The credits roll.
    // They stop when the teaser for NIRUX II reaches the middle.
    let k = ((t - 600) as i32 / 2).min(credits_stop());
    let last = CREDITS.len() - 2;
    for (i, l) in CREDITS.iter().enumerate() {
        let y = H + i as i32 * 14 - k;
        if !(-8..H).contains(&y) {
            continue;
        }
        if i == last {
            fb.text_big_center(y - 4, l, 2, LRED, Some(RED));
        } else if i == last + 1 {
            fb.text_center_sh(y + 2, l, if t % 40 < 30 { PURPLE } else { LRED }, BLACK);
        } else if i == 0 {
            fb.text_big_center(
                y - 4,
                l,
                2,
                [YELLOW, WHITE, CYAN, LGREEN][(t as usize / 8) % 4],
                Some(BLUE),
            );
        } else {
            let c = if l.starts_with('*') { LRED } else { LGREY };
            fb.text_center(y, l, c);
        }
    }
    for i in 0..3 {
        let y = 4 + ((t as f32 * 0.04 + i as f32 * 2.0).sin() * 3.0) as i32;
        raster_bar(fb, y, &RAMP_PURPLE);
        raster_bar(fb, H - 11 + i % 2, &RAMP_BLUE);
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
        Scr::Controls => controls(g, fb),
        Scr::Story => story(g, fb),
    }
}
