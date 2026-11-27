//! A small tracker: instruments, patterns of rows for the three music
//! voices, and the player that runs them at 50 ticks a second (the PAL
//! frame rate the old music drivers ran at). Songs are written in a compact
//! text notation (see `SongB`): melodies as notes with lengths, bass and
//! drums as groove templates over a chord progression, and arpeggio chords
//! as the classic fast-cycling SID chords.
use crate::synth::{note_hz, Synth, Wave, LP, MUSIC_VOICES};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pitch {
    /// Relative to the note (for drum programs that follow the melody).
    #[allow(dead_code)]
    Rel(f32),
    Abs(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Step {
    pub wave: Wave,
    pub pitch: Pitch,
    pub ticks: u8,
}

#[derive(Clone, Debug)]
pub struct Instr {
    pub wave: Wave,
    pub adsr: [u8; 4],
    pub pw: f32,
    /// Pulse width change per tick (ping-pong between 0.1 and 0.9).
    pub pwm: f32,
    pub filter: bool,
    /// Filter envelope as multipliers of the song cutoff: start, end, ticks.
    pub fenv: Option<(f32, f32, u16)>,
    /// Vibrato: delay in ticks, depth in semitones, ticks per cycle.
    pub vib: (u16, f32, f32),
    /// A wave program for drums and attacks; the last step stays.
    pub prog: Vec<Step>,
    /// Semitones per tick (drum pitch drops, zaps).
    pub slide: f32,
    /// Release the gate after this many ticks (0: hold until the next row
    /// event).
    pub gate: u16,
    pub ring: bool,
    pub sync: bool,
    pub vol: f32,
    pub detune: f32,
    /// Ticks per arpeggio step.
    pub arp_speed: u8,
}

impl Default for Instr {
    fn default() -> Self {
        Instr {
            wave: Wave::Pulse,
            adsr: [0, 9, 8, 6],
            pw: 0.5,
            pwm: 0.0,
            filter: false,
            fenv: None,
            vib: (0, 0.0, 6.0),
            prog: Vec::new(),
            slide: 0.0,
            gate: 0,
            ring: false,
            sync: false,
            vol: 1.0,
            detune: 0.0,
            arp_speed: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Note {
        note: u8,
        ins: u8,
        arp: u8,
        slide: bool,
    },
    Off,
}

pub const ARPS: &[&[i8]] = &[
    &[0],
    &[0, 4, 7],
    &[0, 3, 7],
    &[0, 4, 7, 10],
    &[0, 5, 7],
    &[0, 12],
    &[0, 7, 12],
    &[0, 3, 6],
    &[0, 4, 7, 11],
    &[0, 3, 7, 10],
    &[0, 4, 7, 14],
    &[0, 2, 7],
];

/// Arpeggio index for a chord suffix.
fn arp_of(q: &str) -> Option<u8> {
    Some(match q {
        "" | "M" => 1,
        "m" => 2,
        "7" => 3,
        "sus" | "s" => 4,
        "o" => 5,
        "5" => 6,
        "dim" | "d" => 7,
        "maj7" | "j" => 8,
        "m7" | "n" => 9,
        "add9" | "9" => 10,
        "sus2" | "2" => 11,
        "-" | "x" => 0,
        _ => return None,
    })
}

#[derive(Clone, Debug)]
pub struct Song {
    pub name: &'static str,
    pub speed: u8,
    pub instrs: Vec<Instr>,
    pub v: [Vec<Cell>; MUSIC_VOICES],
    pub loop_row: Option<usize>,
    /// Song cutoff automation: (row, Hz), linear between points.
    pub filter: Vec<(usize, f32)>,
    pub res: f32,
    pub mode: u8,
    pub problems: Vec<String>,
}

impl Song {
    pub fn rows(&self) -> usize {
        self.v[0].len()
    }
    /// Seconds for one pass through the song.
    pub fn seconds(&self) -> f32 {
        self.rows() as f32 * self.speed as f32 / 50.0
    }
    fn base_cutoff(&self, row: f32) -> f32 {
        let f = &self.filter;
        if f.is_empty() {
            return 2400.0;
        }
        if row <= f[0].0 as f32 {
            return f[0].1;
        }
        for w in f.windows(2) {
            let (a, b) = (w[0], w[1]);
            if row < b.0 as f32 {
                let t = (row - a.0 as f32) / (b.0 - a.0).max(1) as f32;
                return a.1 + (b.1 - a.1) * t;
            }
        }
        f[f.len() - 1].1
    }
}

/// Parse a note name like `a4`, `c#5` or `bb3` into a MIDI note number.
pub fn parse_note(s: &str) -> Option<u8> {
    let b = s.as_bytes();
    if b.is_empty() {
        return None;
    }
    let base = match b[0].to_ascii_lowercase() {
        b'c' => 0,
        b'd' => 2,
        b'e' => 4,
        b'f' => 5,
        b'g' => 7,
        b'a' => 9,
        b'b' => 11,
        _ => return None,
    };
    let mut i = 1;
    let mut acc = 0i32;
    while i < b.len() && (b[i] == b'#' || (b[i] == b'b' && i + 1 < b.len())) {
        acc += if b[i] == b'#' { 1 } else { -1 };
        i += 1;
    }
    let oct: i32 = std::str::from_utf8(&b[i..]).ok()?.parse().ok()?;
    let n = (oct + 1) * 12 + base + acc;
    (0..=120).contains(&n).then_some(n as u8)
}

/// A chord like `Am`, `F#7`, `Bbmaj7`, `Esus`: root pitch class and arp.
pub fn parse_chord(s: &str) -> Option<(u8, u8)> {
    let b = s.as_bytes();
    let base: i32 = match b.first()? {
        b'C' => 0,
        b'D' => 2,
        b'E' => 4,
        b'F' => 5,
        b'G' => 7,
        b'A' => 9,
        b'B' => 11,
        _ => return None,
    };
    let mut i = 1;
    let mut acc = 0;
    if i < b.len() && (b[i] == b'#' || b[i] == b'b') {
        acc = if b[i] == b'#' { 1 } else { -1 };
        i += 1;
    }
    Some((((base + acc).rem_euclid(12)) as u8, arp_of(&s[i..])?))
}

/// Builds a song voice by voice, section by section.
pub struct SongB {
    pub song: Song,
}

impl SongB {
    pub fn new(name: &'static str, speed: u8, instrs: Vec<Instr>) -> SongB {
        SongB {
            song: Song {
                name,
                speed,
                instrs,
                v: [Vec::new(), Vec::new(), Vec::new()],
                loop_row: None,
                filter: Vec::new(),
                res: 0.4,
                mode: LP,
                problems: Vec::new(),
            },
        }
    }

    pub fn len(&self, v: usize) -> usize {
        self.song.v[v].len()
    }

    fn problem(&mut self, s: String) {
        self.song.problems.push(s);
    }

    /// Melody tokens: `NOTE[chord][:len]` (`a4:4`, `c#5m:8`, `~e5:2` slides
    /// into the note), `r[:len]` releases, `-[:len]` holds, `@N` picks
    /// instrument N for what follows.
    pub fn mel(&mut self, v: usize, ins: u8, text: &str) -> &mut Self {
        let mut ins = ins;
        for tok in text.split_whitespace() {
            if tok == "|" {
                continue;
            }
            if let Some(n) = tok.strip_prefix('@') {
                match n.parse() {
                    Ok(i) => ins = i,
                    Err(_) => self.problem(format!("bad instrument {tok}")),
                }
                continue;
            }
            let (head, len) = match tok.split_once(':') {
                Some((h, l)) => (h, l.parse::<usize>().unwrap_or(0)),
                None => (tok, 1),
            };
            if len == 0 {
                self.problem(format!("bad length in {tok}"));
                continue;
            }
            let cell = if head == "r" {
                Cell::Off
            } else if head == "-" {
                Cell::Empty
            } else {
                let (slide, h) = match head.strip_prefix('~') {
                    Some(h) => (true, h),
                    None => (false, head),
                };
                // Note name then an optional chord suffix.
                let digit = h.find(|c: char| c.is_ascii_digit());
                let Some(d) = digit else {
                    self.problem(format!("bad note {tok}"));
                    continue;
                };
                let (nn, q) = h.split_at(d + 1);
                let Some(note) = parse_note(nn) else {
                    self.problem(format!("bad note {tok}"));
                    continue;
                };
                let arp = if q.is_empty() {
                    0
                } else {
                    match arp_of(q) {
                        Some(a) => a,
                        None => {
                            self.problem(format!("bad chord {tok}"));
                            0
                        }
                    }
                };
                Cell::Note {
                    note,
                    ins,
                    arp,
                    slide,
                }
            };
            self.song.v[v].push(cell);
            for _ in 1..len {
                self.song.v[v].push(Cell::Empty);
            }
        }
        self
    }

    /// A groove over a chord progression ("Am:16 F:16 ..."). Template
    /// characters, one per row, repeating: `b` root, `o` root an octave up,
    /// `f` fifth, `F` fifth up, `t` third, `c` the chord as an arpeggio,
    /// `C` the chord an octave up, `K` kick, `S` snare, `H` hi-hat, `.`
    /// nothing, `_` release.
    #[allow(clippy::too_many_arguments)]
    pub fn groove(
        &mut self,
        v: usize,
        chords: &str,
        tpl: &str,
        oct: i32,
        ins: u8,
        drums: [u8; 3],
    ) -> &mut Self {
        let t: Vec<u8> = tpl.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
        if t.is_empty() {
            return self;
        }
        let mut row = 0usize;
        for ch in chords.split_whitespace().filter(|t| *t != "|") {
            let (name, len) = match ch.split_once(':') {
                Some((n, l)) => (n, l.parse::<usize>().unwrap_or(16)),
                None => (ch, 16),
            };
            let Some((root, arp)) = parse_chord(name) else {
                self.problem(format!("bad chord {ch}"));
                continue;
            };
            let third = if matches!(arp, 2 | 7 | 9) { 3 } else { 4 };
            let rootn = (oct + 1) * 12 + root as i32;
            for _ in 0..len {
                let c = t[row % t.len()];
                let note = |n: i32, arp: u8| Cell::Note {
                    note: n.clamp(0, 120) as u8,
                    ins,
                    arp,
                    slide: false,
                };
                let drum = |i: u8| Cell::Note {
                    note: 60,
                    ins: i,
                    arp: 0,
                    slide: false,
                };
                let cell = match c {
                    b'b' => note(rootn, 0),
                    b'o' => note(rootn + 12, 0),
                    b'f' => note(rootn + 7, 0),
                    b'F' => note(rootn + 19, 0),
                    b't' => note(rootn + third, 0),
                    b'c' => note(rootn, arp),
                    b'C' => note(rootn + 12, arp),
                    b'K' => drum(drums[0]),
                    b'S' => drum(drums[1]),
                    b'H' => drum(drums[2]),
                    b'_' => Cell::Off,
                    _ => Cell::Empty,
                };
                self.song.v[v].push(cell);
                row += 1;
            }
        }
        self
    }

    pub fn rest(&mut self, v: usize, rows: usize) -> &mut Self {
        if rows > 0 {
            self.song.v[v].push(Cell::Off);
            for _ in 1..rows {
                self.song.v[v].push(Cell::Empty);
            }
        }
        self
    }

    /// Copy the notes of voice `src` from `from` onwards into `dst`,
    /// `delay` rows later, with instrument `ins`: the classic echo voice.
    pub fn echo(
        &mut self,
        src: usize,
        dst: usize,
        from: usize,
        rows: usize,
        delay: usize,
        ins: u8,
    ) {
        let start = self.song.v[dst].len();
        for i in 0..rows {
            let cell = if i < delay {
                Cell::Empty
            } else {
                match self.song.v[src].get(from + i - delay).copied() {
                    Some(Cell::Note {
                        note, arp, slide, ..
                    }) => Cell::Note {
                        note,
                        ins,
                        arp,
                        slide,
                    },
                    Some(c) => c,
                    None => Cell::Empty,
                }
            };
            self.song.v[dst].push(cell);
        }
        debug_assert_eq!(self.song.v[dst].len(), start + rows);
    }

    pub fn cutoff(&mut self, row: usize, hz: f32) -> &mut Self {
        self.song.filter.push((row, hz));
        self
    }

    /// Check that every voice has reached the same row.
    pub fn sync(&mut self, what: &str) -> usize {
        let l: Vec<usize> = (0..MUSIC_VOICES).map(|v| self.len(v)).collect();
        if l.iter().any(|&x| x != l[0]) {
            self.problem(format!(
                "{}: voices out of step at {what}: {l:?}",
                self.song.name
            ));
        }
        l.into_iter().max().unwrap_or(0)
    }

    pub fn mark_loop(&mut self) {
        let r = self.sync("loop");
        self.song.loop_row = Some(r);
    }

    pub fn finish(mut self) -> Song {
        let n = self.sync("end");
        for v in self.song.v.iter_mut() {
            v.resize(n, Cell::Empty);
        }
        self.song.filter.sort_by_key(|p| p.0);
        self.song
    }
}

#[derive(Clone, Debug, Default)]
struct Ch {
    ins: usize,
    note: f32,
    target: f32,
    arp: u8,
    t: u32,
    step: usize,
    step_t: u32,
    pw: f32,
    pw_dir: f32,
    slide: f32,
    on: bool,
}

pub struct Player {
    pub song: Song,
    row: usize,
    tick: u8,
    ch: [Ch; MUSIC_VOICES],
    fenv: Option<(usize, u32)>,
    pub done: bool,
    pub ticks: u64,
}

impl Player {
    pub fn new(song: Song) -> Player {
        Player {
            song,
            row: 0,
            tick: 0,
            ch: Default::default(),
            fenv: None,
            done: false,
            ticks: 0,
        }
    }

    fn trigger(&mut self, v: usize, cell: Cell, synth: &mut Synth) {
        match cell {
            Cell::Empty => {}
            Cell::Off => {
                synth.v[v].set_gate(false);
                self.ch[v].on = false;
            }
            Cell::Note {
                note,
                ins,
                arp,
                slide,
            } => {
                let insn = (ins as usize).min(self.song.instrs.len().saturating_sub(1));
                let Some(ins) = self.song.instrs.get(insn) else {
                    return;
                };
                let ch = &mut self.ch[v];
                let legato = slide && ch.on;
                ch.target = note as f32;
                if !legato {
                    ch.note = note as f32;
                    ch.t = 0;
                    ch.step = 0;
                    ch.step_t = 0;
                    ch.pw = ins.pw;
                    ch.pw_dir = 1.0;
                    ch.slide = 0.0;
                    let sv = &mut synth.v[v];
                    sv.env
                        .set(ins.adsr[0], ins.adsr[1], ins.adsr[2], ins.adsr[3]);
                    sv.filter = ins.filter;
                    sv.ring = ins.ring;
                    sv.sync = ins.sync;
                    sv.vol = ins.vol;
                    sv.fmul = 1.0;
                    sv.retrigger();
                    if ins.filter && ins.fenv.is_some() {
                        self.fenv = Some((insn, 0));
                    }
                }
                ch.ins = insn;
                ch.arp = arp;
                ch.on = true;
            }
        }
    }

    /// One 50 Hz tick.
    pub fn tick(&mut self, synth: &mut Synth) {
        if self.done {
            return;
        }
        self.ticks += 1;
        if self.tick == 0 {
            for v in 0..MUSIC_VOICES {
                let cell = self.song.v[v].get(self.row).copied().unwrap_or(Cell::Empty);
                self.trigger(v, cell, synth);
            }
        }
        // The filter: the song's cutoff, shaped by the last filtered note.
        let rowf = self.row as f32 + self.tick as f32 / self.song.speed.max(1) as f32;
        let mut cut = self.song.base_cutoff(rowf);
        if let Some((i, t)) = self.fenv {
            if let Some((a, b, n)) = self.song.instrs[i].fenv {
                let k = (t as f32 / n.max(1) as f32).min(1.0);
                cut *= a * (b / a).powf(k);
                self.fenv = Some((i, t + 1));
            }
        }
        synth.cutoff = cut;
        synth.res = self.song.res;
        synth.mode = self.song.mode;
        for v in 0..MUSIC_VOICES {
            let ch = &mut self.ch[v];
            let ins = &self.song.instrs[ch.ins];
            let sv = &mut synth.v[v];
            // Portamento toward the target note.
            if (ch.note - ch.target).abs() > 0.01 {
                let d = ch.target - ch.note;
                ch.note += d.signum() * d.abs().min(0.6);
            }
            let mut wave = ins.wave;
            let mut pitch = ch.note + ins.detune;
            if !ins.prog.is_empty() {
                let st = ins.prog[ch.step.min(ins.prog.len() - 1)];
                wave = st.wave;
                match st.pitch {
                    Pitch::Rel(r) => pitch += r,
                    Pitch::Abs(a) => pitch = a,
                }
                ch.step_t += 1;
                if ch.step_t >= st.ticks as u32 && ch.step + 1 < ins.prog.len() {
                    ch.step += 1;
                    ch.step_t = 0;
                }
            }
            let arp = ARPS[(ch.arp as usize).min(ARPS.len() - 1)];
            if arp.len() > 1 {
                let k = (ch.t / ins.arp_speed.max(1) as u32) as usize % arp.len();
                pitch += arp[k] as f32;
            }
            let (vd, depth, per) = ins.vib;
            if depth > 0.0 && ch.t >= vd as u32 {
                let ph = (ch.t - vd as u32) as f32 / per.max(1.0);
                let ramp = (((ch.t - vd as u32) as f32) / 20.0).min(1.0);
                pitch += depth * ramp * (ph * std::f32::consts::TAU).sin();
            }
            ch.slide += ins.slide;
            pitch += ch.slide;
            if ins.pwm != 0.0 {
                ch.pw += ins.pwm * ch.pw_dir;
                if ch.pw > 0.9 {
                    ch.pw = 0.9;
                    ch.pw_dir = -1.0;
                } else if ch.pw < 0.1 {
                    ch.pw = 0.1;
                    ch.pw_dir = 1.0;
                }
            }
            sv.wave = wave;
            sv.pw = ch.pw;
            sv.freq = if wave == Wave::Noise {
                // Noise pitch: higher notes, brighter noise.
                note_hz(pitch.clamp(0.0, 135.0))
            } else {
                note_hz(pitch.clamp(0.0, 127.0))
            };
            if ins.gate > 0 && ch.t == ins.gate as u32 {
                sv.set_gate(false);
            }
            ch.t += 1;
        }
        self.tick += 1;
        if self.tick >= self.song.speed.max(1) {
            self.tick = 0;
            self.row += 1;
            if self.row >= self.song.rows() {
                match self.song.loop_row {
                    Some(r) if r < self.song.rows() => self.row = r,
                    _ => {
                        self.done = true;
                        for v in 0..MUSIC_VOICES {
                            synth.v[v].set_gate(false);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notes_and_chords_parse() {
        assert_eq!(parse_note("a4"), Some(69));
        assert_eq!(parse_note("c4"), Some(60));
        assert_eq!(parse_note("c#5"), Some(73));
        assert_eq!(parse_note("bb3"), Some(58));
        assert_eq!(parse_note("x4"), None);
        assert_eq!(parse_chord("Am"), Some((9, 2)));
        assert_eq!(parse_chord("Bb"), Some((10, 1)));
        assert_eq!(parse_chord("F#7"), Some((6, 3)));
        assert_eq!(parse_chord("Q"), None);
    }

    #[test]
    fn builder_keeps_voices_in_step() {
        let mut b = SongB::new("t", 6, vec![Instr::default()]);
        b.mel(0, 0, "a4:4 c5:4 r:8");
        b.groove(1, "Am:16", "b.o.", 2, 0, [0, 0, 0]);
        b.rest(2, 16);
        b.mark_loop();
        let s = b.finish();
        assert!(s.problems.is_empty(), "{:?}", s.problems);
        assert_eq!(s.rows(), 16);
    }
}
