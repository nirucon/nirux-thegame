//! The sound thread: the synthesiser, the music player and the effects,
//! rendered in small blocks and written to the audio sink, which paces the
//! thread. The game talks to it through a channel and never waits for it.
//! The same engine renders songs to WAV files for checking without sound.
use crate::music::Player;
use crate::sfx::{self, Sfx, Slot};
use crate::sink::Sink;
use crate::songs::{self, SongId};
use crate::synth::{Synth, RATE};
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cmd {
    Music(Option<SongId>),
    Sfx(Sfx),
    Volume { music: f32, sfx: f32 },
    Mute(bool),
    Echo(bool),
    Quit,
}

pub struct Engine {
    pub synth: Synth,
    player: Option<Player>,
    pub current: Option<SongId>,
    slots: [Slot; 2],
    acc: f32,
    sub: u32,
    muted: bool,
    vols: (f32, f32),
}

/// 200 Hz steps: effects every step, the music every fourth (50 Hz).
const STEP: f32 = RATE as f32 / 200.0;

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            synth: Synth::new(),
            player: None,
            current: None,
            slots: Default::default(),
            acc: 0.0,
            sub: 0,
            muted: false,
            vols: (0.8, 0.8),
        }
    }

    fn apply_vol(&mut self) {
        let m = if self.muted { 0.0 } else { 1.0 };
        self.synth.music_vol = self.vols.0 * m;
        self.synth.sfx_vol = self.vols.1 * m;
    }

    pub fn handle(&mut self, c: Cmd) {
        match c {
            Cmd::Music(id) => {
                if id == self.current && self.player.as_ref().is_some_and(|p| !p.done) {
                    return;
                }
                self.current = id;
                for v in 0..crate::synth::MUSIC_VOICES {
                    self.synth.v[v].set_gate(false);
                    self.synth.v[v].env.level = 0.0;
                }
                self.player = id.map(|i| Player::new(songs::build(i)));
            }
            Cmd::Sfx(x) => {
                if !self.muted {
                    sfx::start(&mut self.slots, &mut self.synth, x)
                }
            }
            Cmd::Volume { music, sfx } => {
                self.vols = (music.clamp(0.0, 1.0), sfx.clamp(0.0, 1.0));
                self.apply_vol();
            }
            Cmd::Mute(m) => {
                self.muted = m;
                self.apply_vol();
            }
            Cmd::Echo(e) => self.synth.echo = e,
            Cmd::Quit => {}
        }
    }

    /// Interleaved stereo, any length.
    pub fn render(&mut self, out: &mut [f32]) {
        let mut pos = 0;
        while pos < out.len() {
            if self.acc <= 0.0 {
                self.acc += STEP;
                sfx::step(&mut self.slots, &mut self.synth);
                if self.sub % 4 == 0 {
                    if let Some(p) = self.player.as_mut() {
                        p.tick(&mut self.synth);
                    }
                }
                self.sub = self.sub.wrapping_add(1);
            }
            let frames = (self.acc.ceil() as usize).max(1);
            let n = (frames * 2).min(out.len() - pos);
            self.synth.render(&mut out[pos..pos + n]);
            pos += n;
            self.acc -= (n / 2) as f32;
        }
    }
}

pub struct Audio {
    tx: Option<Sender<Cmd>>,
}

impl Audio {
    /// No sound at all (tests, `--selftest`, no audio program).
    pub fn none() -> Audio {
        Audio { tx: None }
    }

    pub fn start(custom: &str) -> Audio {
        let probe = Sink::open(custom, RATE, 0);
        let sink = match probe {
            Ok(s) => s,
            Err(e) => {
                eprintln!("nirux-thegame: {e}; playing without sound");
                return Audio::none();
            }
        };
        if std::env::var_os("NIRUX_DEBUG").is_some() {
            eprintln!("nirux-thegame: sound through {}", sink.name);
        }
        let (tx, rx) = channel();
        let custom = custom.to_string();
        let spawned = std::thread::Builder::new()
            .name("sound".into())
            .spawn(move || run(sink, rx, &custom));
        if spawned.is_err() {
            return Audio::none();
        }
        Audio { tx: Some(tx) }
    }

    pub fn send(&self, c: Cmd) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(c);
        }
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        self.send(Cmd::Quit);
    }
}

fn run(mut sink: Sink, rx: Receiver<Cmd>, custom: &str) {
    let mut eng = Engine::new();
    let mut f = vec![0f32; 256 * 2];
    let mut pcm = vec![0i16; 256 * 2];
    let mut skip = 0;
    // The sink normally paces this thread; a sink that swallows anything
    // (a file, a big buffer) must not make it spin, so it never runs more
    // than a little ahead of the clock.
    let start = std::time::Instant::now();
    let mut written = 0u64;
    loop {
        let ahead = written as f64 / RATE as f64 - start.elapsed().as_secs_f64();
        if ahead > 0.12 {
            std::thread::sleep(std::time::Duration::from_secs_f64(ahead - 0.08));
        }
        loop {
            match rx.try_recv() {
                Ok(Cmd::Quit) => return,
                Ok(c) => eng.handle(c),
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(_) => return,
            }
        }
        eng.render(&mut f);
        for (o, x) in pcm.iter_mut().zip(&f) {
            *o = (x * 32767.0).round().clamp(-32768.0, 32767.0) as i16;
        }
        written += (pcm.len() / 2) as u64;
        if sink.write(&pcm).is_err() {
            // The player went away: try the next program once or twice.
            skip += 1;
            match Sink::open(custom, RATE, skip) {
                Ok(s) if skip < 4 => sink = s,
                _ => return,
            }
        }
    }
}

/// Render a song to a 16-bit stereo WAV file. Returns (peak, rms, seconds).
pub fn render_wav(
    id: SongId,
    path: &str,
    seconds: Option<f32>,
) -> std::io::Result<(f32, f32, f32)> {
    let mut eng = Engine::new();
    eng.handle(Cmd::Music(Some(id)));
    let song_len = songs::build(id).seconds();
    let secs = seconds.unwrap_or_else(|| (song_len + 2.0).min(240.0));
    let frames = (secs * RATE as f32) as usize;
    let mut f = vec![0f32; 1024 * 2];
    let mut data: Vec<u8> = Vec::with_capacity(frames * 4);
    let (mut peak, mut sum) = (0f32, 0f64);
    let mut done = 0;
    while done < frames {
        let n = (frames - done).min(1024);
        eng.render(&mut f[..n * 2]);
        for x in &f[..n * 2] {
            peak = peak.max(x.abs());
            sum += (*x as f64) * (*x as f64);
            let v = (x * 32767.0).round().clamp(-32768.0, 32767.0) as i16;
            data.extend_from_slice(&v.to_le_bytes());
        }
        done += n;
    }
    let mut out = Vec::with_capacity(44 + data.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 4).to_le_bytes());
    out.extend_from_slice(&4u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&data);
    std::fs::write(path, out)?;
    let rms = (sum / (frames * 2).max(1) as f64).sqrt() as f32;
    Ok((peak, rms, secs))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_song_renders_without_clipping() {
        for &id in songs::ALL {
            let s = songs::build(id);
            assert!(s.problems.is_empty(), "{:?}: {:?}", id, s.problems);
            let mut eng = Engine::new();
            eng.handle(Cmd::Music(Some(id)));
            let mut f = vec![0f32; 4410 * 2];
            let (mut peak, mut sum, mut n) = (0f32, 0f64, 0usize);
            for k in 0..((s.seconds().min(40.0) * 10.0) as usize) {
                if k % 7 == 0 {
                    eng.handle(Cmd::Sfx(Sfx::ExplodeBig));
                }
                eng.render(&mut f);
                for x in &f {
                    assert!(x.is_finite());
                    peak = peak.max(x.abs());
                    sum += (*x as f64).powi(2);
                    n += 1;
                }
            }
            let rms = (sum / n as f64).sqrt();
            assert!(peak < 0.96, "{id:?} peak {peak}");
            assert!(rms > 0.02, "{id:?} too quiet: {rms}");
        }
    }
}

#[cfg(test)]
mod balance {
    use super::*;
    /// `cargo test --release balance -- --ignored --nocapture` prints the
    /// level of each voice alone, to check the mix without speakers.
    #[test]
    #[ignore]
    fn voice_levels() {
        for &id in songs::ALL {
            let mut line = format!("{:9}", id.name());
            for solo in 0..3 {
                let mut eng = Engine::new();
                eng.synth.echo = false;
                eng.handle(Cmd::Music(Some(id)));
                let mut f = vec![0f32; 4410 * 2];
                let mut sum = 0f64;
                let mut n = 0;
                for _ in 0..300 {
                    for v in 0..3 {
                        if v != solo {
                            eng.synth.v[v].vol = 0.0;
                        }
                    }
                    eng.render(&mut f);
                    for x in &f {
                        sum += (*x as f64).powi(2);
                        n += 1;
                    }
                }
                let db = 10.0 * (sum / n as f64).max(1e-12).log10();
                line += &format!("  v{} {:6.1} dB", solo + 1, db);
            }
            println!("{line}");
        }
    }
}
