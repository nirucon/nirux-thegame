//! NIRUX: an original Commodore 64 style shooter for the NIRUX desktop.
//! A 320x200 picture in the 16 C64 colours, scaled by whole numbers to any
//! screen with the C64 border around it, a three voice SID-style
//! synthesiser for the music and effects, and a 60 Hz fixed time step.
// Synth voices and tile rows are indexed in step with other arrays.
#![allow(clippy::needless_range_loop)]
mod art;
mod audio;
mod config;
mod fb;
mod font;
mod game;
mod hero;
mod input;
mod music;
mod pad;
mod palette;
mod scale;
mod sfx;
mod sink;
mod songs;
mod synth;
mod video;

use std::time::{Duration, Instant};

const HELP: &str = "NIRUX: an original C64 style shooter for the NIRUX desktop

Usage: nirux-thegame [OPTIONS]

  --windowed            start in a window (default: fullscreen, see config)
  --fullscreen          start fullscreen
  --demo                start in the attract mode demo
  --level N             start a game at sector N (1 to 8)
  --snap SCREEN FRAMES FILE
                        write a picture of a screen after FRAMES steps
                        (title, storyN, introN, levelN, ending, controls)
  --selftest [FRAMES]   play every sector with the autopilot, no window or
                        sound, and check that everything holds together
  --render-wav SONG FILE [SECONDS]
                        render a song to a WAV file (SONG: title, level1,
                        level2, level3, boss, gameover, clear)
  --list-songs          list the songs
  --version             print the version
  -h, --help            this help

Keys: arrows or WASD move, Space, Ctrl or Z fire, X or Shift bomb or
grenade, Esc or P pause, Enter start, F11 fullscreen, M mute.
Settings: $XDG_CONFIG_HOME/nirux-thegame/config.toml
Progress and high scores: $XDG_DATA_HOME/nirux-thegame/save.toml";

struct Opts {
    windowed: Option<bool>,
    demo: bool,
    level: Option<usize>,
    bot: bool,
    quit_after: Option<f32>,
    show: Option<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut o = Opts {
        windowed: None,
        demo: false,
        level: None,
        bot: false,
        quit_after: None,
        show: None,
    };
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "-h" | "--help" => {
                println!("{HELP}");
                return;
            }
            "--version" | "-V" => {
                println!("NIRUX {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--list-songs" => {
                for s in songs::ALL {
                    let song = songs::build(*s);
                    println!("{:<9} {:>5.1} s  {}", s.name(), song.seconds(), s.title());
                }
                return;
            }
            "--render-wav" => {
                let (Some(name), Some(file)) = (args.get(i + 1), args.get(i + 2)) else {
                    eprintln!("usage: --render-wav SONG FILE [SECONDS]");
                    std::process::exit(2);
                };
                let Some(id) = songs::SongId::from_name(name) else {
                    eprintln!("unknown song {name} (try --list-songs)");
                    std::process::exit(2);
                };
                let secs = args.get(i + 3).and_then(|s| s.parse::<f32>().ok());
                match audio::render_wav(id, file, secs) {
                    Ok((peak, rms, s)) => {
                        let db = |x: f32| 20.0 * x.max(1e-9).log10();
                        println!(
                            "{file}: {s:.1} s, peak {:.1} dBFS, rms {:.1} dBFS",
                            db(peak),
                            db(rms)
                        );
                    }
                    Err(e) => {
                        eprintln!("{file}: {e}");
                        std::process::exit(1);
                    }
                }
                return;
            }
            "--snap" => {
                // For screenshots and art checks without a window: run a
                // screen for some frames and write the picture as a PPM.
                let (Some(scr), Some(n), Some(file)) =
                    (args.get(i + 1), args.get(i + 2), args.get(i + 3))
                else {
                    eprintln!("usage: --snap SCREEN FRAMES FILE.ppm");
                    std::process::exit(2);
                };
                let n: u32 = n.parse().unwrap_or(60);
                std::process::exit(match snap(scr, n, file) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("{e}");
                        1
                    }
                });
            }
            "--selftest" => {
                let frames = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(9000);
                std::process::exit(if selftest(frames, true) { 0 } else { 1 });
            }
            "--windowed" => o.windowed = Some(true),
            "--fullscreen" => o.windowed = Some(false),
            "--demo" => o.demo = true,
            "--bot" => o.bot = true,
            "--level" => {
                i += 1;
                o.level = args
                    .get(i)
                    .and_then(|s| s.parse::<usize>().ok())
                    .filter(|n| (1..=game::LEVELS).contains(n));
                if o.level.is_none() {
                    eprintln!("--level needs a sector from 1 to {}", game::LEVELS);
                    std::process::exit(2);
                }
            }
            "--show" => {
                // For screenshots: open a screen directly (controls, ending).
                i += 1;
                o.show = args.get(i).cloned();
            }
            "--quit-after" => {
                i += 1;
                o.quit_after = args.get(i).and_then(|s| s.parse().ok());
            }
            _ => {
                eprintln!("unknown option {a} (try --help)");
                std::process::exit(2);
            }
        }
        i += 1;
    }
    if let Err(e) = run(o) {
        eprintln!("nirux-thegame: {e}");
        std::process::exit(1);
    }
}

fn run(o: Opts) -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = config::Config::load();
    if let Some(w) = o.windowed {
        cfg.fullscreen = !w;
    }
    let save = config::Save::load();
    let mut vid = video::Video::open(cfg.fullscreen)?;
    let theme = config::theme_colors(vid.root_theme().as_deref());
    let lut = palette::Lut::new(theme);
    let snd = audio::Audio::start(&cfg.audio);
    snd.send(audio::Cmd::Volume {
        music: cfg.music as f32 / 10.0,
        sfx: cfg.sfx as f32 / 10.0,
    });
    snd.send(audio::Cmd::Echo(cfg.echo));
    let mut g = game::Game::new(cfg.clone(), save, true);
    if o.demo {
        g.start_attract_demo();
    } else if let Some(l) = o.level {
        g.start_run(l - 1);
    }
    match o.show.as_deref() {
        Some("controls") => g.go(game::Scr::Controls),
        Some("ending") => {
            g.lvl = game::LEVELS - 1;
            g.go(game::Scr::Victory);
        }
        Some("select") => g.go(game::Scr::Select),
        _ => {}
    }
    let mut ctl = input::Controls::new();
    let mut fb = fb::Fb::new();
    let frame = Duration::from_nanos(1_000_000_000 / 60);
    let start = Instant::now();
    let mut next = Instant::now();
    let mut muted = false;
    let mut visible = true;
    let debug = std::env::var_os("NIRUX_DEBUG").is_some();
    let mut prof = [Duration::ZERO; 3];
    let mut prof_n = 0u32;
    loop {
        let p0 = Instant::now();
        for ev in vid.poll()? {
            match ev {
                video::Ev::Key { sym, down } => {
                    if down && sym == input::K_F11 {
                        let fs = !vid.fullscreen;
                        vid.set_fullscreen(fs)?;
                        g.cfg.fullscreen = fs;
                        g.cfg_dirty = true;
                    } else if down && sym == input::K_M {
                        muted = !muted;
                        snd.send(audio::Cmd::Mute(muted));
                    } else if let Some(k) = input::map_key(sym) {
                        ctl.key(k, down);
                    }
                }
                video::Ev::Close => g.quit = true,
                video::Ev::Focus(false) => {
                    ctl.release_all();
                    if g.scr == game::Scr::Play {
                        g.scr = game::Scr::Pause;
                        g.sel = 0;
                    }
                }
                video::Ev::Focus(true) => {}
                video::Ev::Visible(v) => {
                    visible = v;
                    if !v && g.scr == game::Scr::Play {
                        g.scr = game::Scr::Pause;
                        g.sel = 0;
                    }
                }
            }
        }
        let mut inp = ctl.frame();
        if g.pad != ctl.pads.status {
            g.pad = ctl.pads.status.clone();
        }
        if o.bot && g.scr == game::Scr::Play {
            if let Some(l) = g.level.as_mut() {
                inp = l.bot(&g.stats);
            }
        }
        g.update(inp);
        for c in g.take_cmds() {
            snd.send(c);
        }
        if let Some(fs) = g.fullscreen_req.take() {
            vid.set_fullscreen(fs)?;
        }
        if g.quit
            || o.quit_after
                .is_some_and(|s| start.elapsed().as_secs_f32() > s)
        {
            break;
        }
        let p1 = Instant::now();
        if visible {
            g.draw(&mut fb);
        }
        let p2 = Instant::now();
        if visible {
            vid.present(&fb, &lut, g.border(), g.cfg.crt)?;
        }
        if debug {
            prof[0] += p1 - p0;
            prof[1] += p2 - p1;
            prof[2] += p2.elapsed();
            prof_n += 1;
            if prof_n == 300 {
                let ms = |d: Duration| d.as_secs_f64() * 1000.0 / prof_n as f64;
                eprintln!(
                    "frame: events+update {:.3} ms, draw {:.3} ms, present {:.3} ms (window {}x{})",
                    ms(prof[0]),
                    ms(prof[1]),
                    ms(prof[2]),
                    vid.w,
                    vid.h
                );
                prof = [Duration::ZERO; 3];
                prof_n = 0;
            }
        }
        // A fixed 60 Hz step; sleep until the next frame is due.
        next += frame;
        let now = Instant::now();
        if next > now {
            std::thread::sleep(next - now);
        } else if now - next > Duration::from_millis(100) {
            next = now;
        }
        if !visible {
            std::thread::sleep(Duration::from_millis(30));
        }
    }
    if g.cfg_dirty {
        g.cfg.save();
    }
    Ok(())
}

/// Play each sector with the autopilot on every difficulty, drawing every
/// frame, then run the menus with scripted keys and render the songs.
/// Opens a screen ("title", "story", "introN", "ending", "levelN" played
/// by the autopilot), steps it `frames` times and writes a PPM, 2x scaled.
fn snap(scr: &str, frames: u32, file: &str) -> std::io::Result<()> {
    let mut g = game::Game::new(config::Config::default(), config::Save::default(), false);
    let num = |p: &str| {
        scr.strip_prefix(p)
            .and_then(|n| n.parse::<usize>().ok())
            .filter(|n| (1..=game::LEVELS).contains(n))
            .map(|n| n - 1)
    };
    let mut bot = false;
    match scr {
        "title" => {}
        "story" => g.go(game::Scr::Story),
        "ending" => {
            g.lvl = game::LEVELS - 1;
            g.go(game::Scr::Victory);
        }
        "controls" => g.go(game::Scr::Controls),
        _ => {
            if let Some(p) = scr
                .strip_prefix("story")
                .and_then(|n| n.parse::<usize>().ok())
            {
                g.go(game::Scr::Story);
                g.page = p.saturating_sub(1);
            } else if let Some(l) = num("intro") {
                g.start_run(l);
            } else if let Some(l) = num("level") {
                g.start_run(l);
                g.t = 1000;
                g.update(input::Input::default());
                bot = true;
            } else {
                return Err(std::io::Error::other(format!("unknown screen {scr}")));
            }
        }
    }
    let mut fb = fb::Fb::new();
    for _ in 0..frames {
        let inp = if bot {
            g.bot_input()
        } else {
            input::Input::default()
        };
        g.update(inp);
        let _ = g.take_cmds();
    }
    g.draw(&mut fb);
    let lut = palette::Lut::new([0xc0c0c0, 0x9ad284, 0x6c6c6c]);
    let (w, h) = (fb::W as usize * 2, fb::H as usize * 2);
    let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
    for y in 0..h {
        for x in 0..w {
            let c = lut.rgb(fb.px[(y / 2) * fb::W as usize + x / 2]);
            out.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
        }
    }
    std::fs::write(file, out)
}

fn selftest(frames: u32, verbose: bool) -> bool {
    use game::common::{Out, Stats, Status};
    let mut ok = true;
    let mut fb = fb::Fb::new();
    let only: Option<usize> = std::env::var("NIRUX_ONLY")
        .ok()
        .and_then(|v| v.parse().ok());
    for lvl in 0..game::LEVELS {
        if only.is_some_and(|o| o != lvl + 1) {
            continue;
        }
        for d in [
            config::Difficulty::Easy,
            config::Difficulty::Normal,
            config::Difficulty::Hard,
        ] {
            let mut l = game::make_level(lvl, d, 0xC64 + lvl as u32);
            let mut st = Stats::new(9);
            st.next_life = u32::MAX;
            let t0 = Instant::now();
            let mut status = Status::Playing;
            let mut n = 0;
            let mut worst = Duration::ZERO;
            while n < frames && status == Status::Playing {
                let f0 = Instant::now();
                let inp = l.bot(&st);
                let mut out = Out::default();
                let before = st.lives;
                status = l.update(&inp, &mut st, &mut out);
                // The autopilot plays on however often it dies.
                st.lives = st.lives.max(3);
                if std::env::var_os("NIRUX_DEBUG").is_some() && st.lives < before {
                    println!("  frame {n}: life lost, {}", l.info());
                }
                l.draw(&mut fb, &st, 0);
                worst = worst.max(f0.elapsed());
                n += 1;
            }
            let per = t0.elapsed().as_secs_f64() * 1000.0 / n.max(1) as f64;
            if verbose {
                println!(
                    "sector {} {:<6} {:>6} frames  {:?}  score {:>7}  lives lost {:>2}  {:.3} ms/frame (worst {:.2} ms) {}",
                    lvl + 1,
                    d.name(),
                    n,
                    status,
                    st.score,
                    st.deaths_this_level,
                    per,
                    worst.as_secs_f64() * 1000.0,
                    l.info()
                );
            }
            if per > 4.0 && !cfg!(debug_assertions) {
                println!("  too slow");
                ok = false;
            }
        }
    }
    // The menus: title, options, level select, a game, pause, quit.
    let mut g = game::Game::new(config::Config::default(), config::Save::default(), false);
    let script: &[(u32, input::Key)] = &[
        (20, input::Key::Down),
        (24, input::Key::Down),
        (28, input::Key::Down),
        (32, input::Key::Fire),
        (45, input::Key::Fire),
        (50, input::Key::Down),
        (60, input::Key::Fire),
        (80, input::Key::Right),
        (90, input::Key::Back),
        (120, input::Key::Fire),
        (150, input::Key::Fire),
        (170, input::Key::Back),
        (230, input::Key::Fire),
        (400, input::Key::Back),
        (420, input::Key::Down),
        (430, input::Key::Fire),
    ];
    let mut ctl = input::Controls::keyboard_only();
    let mut seen = std::collections::HashSet::new();
    for f in 0..600u32 {
        for (at, k) in script {
            if *at == f {
                ctl.key(*k, true);
            }
            if *at + 2 == f {
                ctl.key(*k, false);
            }
        }
        let inp = ctl.frame();
        g.update(inp);
        g.draw(&mut fb);
        let _ = g.take_cmds();
        seen.insert(format!("{:?}", g.scr));
    }
    if verbose {
        let mut v: Vec<_> = seen.iter().cloned().collect();
        v.sort();
        println!("screens visited: {}", v.join(" "));
    }
    for s in [
        "Controls", "Options", "Story", "Intro", "Play", "Pause", "Title",
    ] {
        if !seen.contains(s) {
            println!("menu script never reached {s}");
            ok = false;
        }
    }
    // Music.
    for &id in songs::ALL {
        let mut eng = audio::Engine::new();
        eng.handle(audio::Cmd::Music(Some(id)));
        let mut buf = vec![0f32; 44100 * 2];
        let mut peak = 0f32;
        for _ in 0..5 {
            eng.render(&mut buf);
            peak = buf.iter().fold(peak, |p, x| p.max(x.abs()));
        }
        if verbose {
            println!("song {:<9} peak {:.2}", id.name(), peak);
        }
        if !(0.05..0.97).contains(&peak) {
            ok = false;
        }
    }
    println!("{}", if ok { "selftest OK" } else { "selftest FAILED" });
    ok
}

#[cfg(test)]
mod tests {
    #[test]
    fn selftest_passes_briefly() {
        assert!(super::selftest(1500, false));
    }
}
