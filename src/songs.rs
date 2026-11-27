//! The NIRUX soundtrack, composed for this game: a title theme, one theme
//! per sector, the boss theme, and the short game over and sector clear
//! pieces. Voice 1 plays bass and drums (the drums cut into the bass line
//! the way three-voice music always did it), voice 2 the arpeggio chords or
//! an echo of the lead, voice 3 the lead.
use crate::music::{Instr, Pitch, Song, SongB, Step};
use crate::synth::{Wave, BP, LP};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SongId {
    Title,
    Level1,
    Level2,
    Level3,
    Boss,
    GameOver,
    Clear,
}

pub const ALL: &[SongId] = &[
    SongId::Title,
    SongId::Level1,
    SongId::Level2,
    SongId::Level3,
    SongId::Boss,
    SongId::GameOver,
    SongId::Clear,
];

impl SongId {
    pub fn name(self) -> &'static str {
        match self {
            SongId::Title => "title",
            SongId::Level1 => "level1",
            SongId::Level2 => "level2",
            SongId::Level3 => "level3",
            SongId::Boss => "boss",
            SongId::GameOver => "gameover",
            SongId::Clear => "clear",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            SongId::Title => "Nirux (title theme)",
            SongId::Level1 => "Outer Rim (sector 1)",
            SongId::Level2 => "Glass Dunes (sector 2)",
            SongId::Level3 => "Iron Jungle (sector 3)",
            SongId::Boss => "Warden (boss theme)",
            SongId::GameOver => "Signal Lost (game over)",
            SongId::Clear => "Sector Clear (jingle)",
        }
    }
    pub fn from_name(s: &str) -> Option<SongId> {
        ALL.iter().copied().find(|i| i.name() == s)
    }
}

// Instrument numbers used by every song.
const BASS: u8 = 0;
const KICK: u8 = 1;
const SNARE: u8 = 2;
const HAT: u8 = 3;
const ARP: u8 = 4;
const LEAD: u8 = 5;
const ECHO: u8 = 6;
const PAD: u8 = 7;
const SAW: u8 = 8;
const PLUCK: u8 = 9;
const DR: [u8; 3] = [KICK, SNARE, HAT];

fn st(wave: Wave, pitch: Pitch, ticks: u8) -> Step {
    Step { wave, pitch, ticks }
}

/// The instrument set; `bright` opens the bass filter envelope wider.
fn kit(bright: f32) -> Vec<Instr> {
    use Wave::*;
    vec![
        // 0 bass: pulse through the low-pass filter, a snappy envelope.
        Instr {
            wave: Pulse,
            adsr: [0, 8, 9, 4],
            pw: 0.28,
            pwm: 0.012,
            filter: true,
            fenv: Some((3.2 * bright, 1.0, 9)),
            vol: 1.0,
            ..Default::default()
        },
        // 1 kick: a noise click, then a falling triangle.
        Instr {
            wave: Tri,
            adsr: [0, 6, 0, 2],
            prog: vec![st(Noise, Pitch::Abs(96.0), 1), st(Tri, Pitch::Abs(57.0), 1)],
            slide: -3.2,
            vol: 1.25,
            ..Default::default()
        },
        // 2 snare: noise, a pulse body, then darker noise.
        Instr {
            wave: Noise,
            adsr: [0, 7, 0, 3],
            prog: vec![
                st(Noise, Pitch::Abs(104.0), 1),
                st(Pulse, Pitch::Abs(57.0), 1),
                st(Noise, Pitch::Abs(94.0), 1),
            ],
            slide: -0.35,
            pw: 0.4,
            vol: 1.0,
            ..Default::default()
        },
        // 3 hi-hat.
        Instr {
            wave: Noise,
            adsr: [0, 3, 0, 1],
            prog: vec![st(Noise, Pitch::Abs(124.0), 1)],
            vol: 0.5,
            gate: 1,
            ..Default::default()
        },
        // 4 arpeggio chord: the shimmering SID chord.
        Instr {
            wave: Pulse,
            adsr: [0, 9, 7, 7],
            pw: 0.22,
            pwm: 0.006,
            vol: 0.62,
            ..Default::default()
        },
        // 5 lead: pulse with a slowly turning width and delayed vibrato.
        Instr {
            wave: Pulse,
            adsr: [1, 9, 10, 8],
            pw: 0.42,
            pwm: 0.004,
            vib: (14, 0.3, 8.0),
            vol: 1.1,
            ..Default::default()
        },
        // 6 echo of the lead, softer.
        Instr {
            wave: Pulse,
            adsr: [1, 9, 8, 8],
            pw: 0.3,
            pwm: 0.004,
            vib: (14, 0.3, 8.0),
            vol: 0.42,
            ..Default::default()
        },
        // 7 pad: a slow triangle.
        Instr {
            wave: Tri,
            adsr: [9, 10, 12, 10],
            vib: (30, 0.15, 10.0),
            vol: 1.1,
            ..Default::default()
        },
        // 8 saw lead for the big moments.
        Instr {
            wave: Saw,
            adsr: [1, 9, 11, 8],
            vib: (12, 0.32, 8.0),
            vol: 0.85,
            ..Default::default()
        },
        // 9 pluck: short bright pulse for runs.
        Instr {
            wave: Pulse,
            adsr: [0, 6, 4, 5],
            pw: 0.5,
            pwm: 0.01,
            vol: 0.8,
            ..Default::default()
        },
    ]
}

pub fn build(id: SongId) -> Song {
    match id {
        SongId::Title => title(),
        SongId::Level1 => level1(),
        SongId::Level2 => level2(),
        SongId::Level3 => level3(),
        SongId::Boss => boss(),
        SongId::GameOver => game_over(),
        SongId::Clear => clear(),
    }
}

/// "Nirux": A minor, a slow rise into a soaring chorus.
fn title() -> Song {
    let mut b = SongB::new("title", 6, kit(1.0));
    b.song.res = 0.55;
    // Intro: the filter opens over a pulsing bass, the chords shimmer.
    let intro1 = "Am:16 F:16 C:16 G:16";
    let intro2 = "Am:16 F:16 Dm:16 E:16";
    b.cutoff(0, 260.0).cutoff(64, 2600.0).cutoff(128, 1700.0);
    b.groove(0, intro1, "b.b.b.b.b.b.b.bo", 2, BASS, DR);
    b.groove(0, "Am:16 F:16 Dm:16", "K.b.b.b.K.b.S.b.", 2, BASS, DR);
    b.groove(0, "E:16", "K.b.S.b.K.S.SSSS", 2, BASS, DR);
    b.groove(1, intro1, "c...............", 4, ARP, DR);
    b.groove(1, intro2, "c.......c.......", 4, ARP, DR);
    b.rest(2, 64);
    b.mel(2, PAD, "e5:16 | a5:16 | f5:16 | g#5:16");
    b.mark_loop();
    let at = b.sync("verse");
    // Verse.
    let verse = "Am:16 F:16 C:16 G:16 Am:16 F:16 G:16 E:16";
    b.cutoff(at, 1500.0).cutoff(at + 128, 2200.0);
    b.groove(0, verse, "K.obS.obK.obS.of", 2, BASS, DR);
    b.groove(1, verse, "c.....c.....c.c.", 4, ARP, DR);
    b.mel(
        2,
        LEAD,
        "e5:6 a5:6 b5:4 | c6:8 b5:4 a5:4 | g5:6 e5:6 c5:4 | d5:12 g5:4 |
         e5:6 a5:6 c6:4 | d6:6 c6:2 a5:8 | b5:4 c6:4 d6:4 b5:4 | g#5:12 e5:4",
    );
    // Chorus: the saw lead with its echo on voice 2.
    let at = b.sync("chorus");
    let chorus = "F:16 G:16 Em:16 Am:16 Dm:16 G:16 C:16 E:16";
    b.cutoff(at, 2400.0).cutoff(at + 127, 3000.0);
    b.groove(0, chorus, "K.ooS.ooK.ooS.oS", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "a5:4 c6:4 f6:8 | e6:4 d6:4 b5:8 | g5:4 b5:4 e6:8 | c6:6 b5:2 a5:8 |
         f5:4 a5:4 d6:8 | d6:2 e6:2 d6:2 b5:2 g5:8 | e5:2 g5:2 c6:4 e6:8 | d6:4 b5:4 g#5:8",
    );
    b.echo(2, 1, at, 128, 3, ECHO);
    // Bridge: half time, runs over held chords.
    let at = b.sync("bridge");
    let bridge = "Dm:16 Am:16 Bb:16 E:16";
    b.cutoff(at, 900.0).cutoff(at + 63, 2600.0);
    b.groove(0, bridge, "K...b...S...b..o", 2, BASS, DR);
    b.groove(1, bridge, "C...............", 4, ARP, DR);
    b.mel(
        2,
        PLUCK,
        "d5 f5 a5 d6 f6 d6 a5 f5 d5 f5 a5 d6 f6 a6 f6 d6 |
         @5 e6:2 c6:2 a5:2 e5:2 a5:2 c6:2 e6:4 |
         @9 f6 d6 bb5 f5 d5 f5 bb5 d6 f6 d6 bb5 f5 bb5 d6 f6 bb6 |
         @8 g#6:4 e6:4 b5:4 g#5:4",
    );
    b.finish()
}

/// "Outer Rim": D major, bright and heroic.
fn level1() -> Song {
    let mut b = SongB::new("level1", 5, kit(1.2));
    b.song.res = 0.5;
    let intro = "D:16 C:16 G:16 A:16";
    b.cutoff(0, 500.0).cutoff(63, 2600.0);
    b.groove(0, "D:16 C:16 G:16", "K.boS.boK.boS.bo", 2, BASS, DR);
    b.groove(0, "A:16", "K.boS.boK.S.SSSS", 2, BASS, DR);
    b.groove(1, intro, "c...c.c.c...c.c.", 4, ARP, DR);
    b.rest(2, 64);
    b.mark_loop();
    let at = b.sync("verse");
    let verse = "D:16 C:16 G:16 D:16 D:16 C:16 G:16 A:16";
    b.cutoff(at, 1800.0);
    b.groove(0, verse, "K.boS.boK.boS.bo", 2, BASS, DR);
    b.groove(1, verse, "c...c.c.c...c.c.", 4, ARP, DR);
    b.mel(
        2,
        LEAD,
        "d5:4 f#5:4 a5:6 g5:2 | e5:4 g5:4 c6:8 | b5:6 a5:2 g5:4 d5:4 | a5:12 f#5:4 |
         d5:4 f#5:4 a5:4 d6:4 | e6:6 d6:2 c6:4 g5:4 | b5:4 d6:4 g6:4 f#6:2 e6:2 | e6:8 c#6:4 a5:4",
    );
    let at = b.sync("chorus");
    let chorus = "G:16 A:16 D:16 Bm:16 G:16 A:16 D:16 D:16";
    b.cutoff(at, 2600.0);
    b.groove(0, chorus, "K.obS.obK.obSSob", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "g5:2 a5:2 b5:4 d6:8 | c#6:2 d6:2 e6:4 a5:8 | f#6:6 e6:2 d6:4 a5:4 | b5:8 f#5:4 d6:4 |
         g6:6 f#6:2 e6:4 d6:4 | e6:6 c#6:2 a5:4 e6:4 | f#6:8 a6:8 | d6:12 r:4",
    );
    b.echo(2, 1, at, 128, 3, ECHO);
    // A short breakdown before it starts again.
    let at = b.sync("break");
    b.cutoff(at, 600.0).cutoff(at + 63, 2400.0);
    let brk = "Bm:16 G:16 A:16 A:16";
    b.groove(0, brk, "b.b.b.b.b.b.b.bo", 2, BASS, DR);
    b.groove(1, brk, "C.......C.......", 4, ARP, DR);
    b.mel(2, PAD, "f#5:16 | g5:16 | a5:16 | c#6:16");
    b.finish()
}

/// "Glass Dunes": E minor, mysterious and driving, syncopated.
fn level2() -> Song {
    let mut b = SongB::new("level2", 6, kit(1.4));
    b.song.res = 0.7;
    let intro = "Em:16 D:16 C:16 D:16";
    b.cutoff(0, 300.0).cutoff(127, 2000.0);
    b.groove(0, intro, "bbobbbobbbobbbob", 2, BASS, DR);
    b.groove(0, intro, "K.obS.obK.obS.ob", 2, BASS, DR);
    b.groove(1, intro, "c.H.c.H.c.H.c.HH", 4, ARP, DR);
    b.groove(1, intro, "c.H.c.H.c.H.c.HH", 4, ARP, DR);
    b.rest(2, 64);
    b.mel(2, PAD, "b4:16 | a4:16 | g4:16 | f#4:16");
    b.mark_loop();
    let at = b.sync("a");
    let a = "Em:16 D:16 C:16 D:16 Em:16 D:16 C:16 B7:16";
    b.cutoff(at, 1300.0);
    b.groove(0, a, "K..bS.b.K.bbS.ob", 2, BASS, DR);
    b.groove(1, a, "c..c..c.c..c..c.", 4, ARP, DR);
    b.mel(
        2,
        LEAD,
        "e5:3 g5:3 b5:2 a5:4 g5:4 | f#5:6 d5:2 a5:8 | g5:3 e5:3 c6:2 b5:4 g5:4 | a5:12 f#5:4 |
         b5:3 e6:3 d6:2 b5:4 a5:4 | a5:3 f#5:3 d5:2 f#5:4 a5:4 | g5:4 e6:4 d6:4 c6:4 | b5:8 d#6:4 f#6:4",
    );
    let at = b.sync("b");
    let bb = "C:16 D:16 Em:16 Em:16 C:16 D:16 B7:16 B7:16";
    b.cutoff(at, 2200.0);
    b.groove(0, bb, "K.obS.obK.obS.oS", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "e6:6 d6:2 c6:8 | d6:6 e6:2 f#6:8 | g6:4 f#6:4 e6:4 b5:4 | e6:16 |
         c6:4 b5:4 a5:4 g5:4 | a5:4 b5:4 c6:4 d6:4 | d#6:8 f#6:8 | b5:4 a5:4 f#5:4 d#5:4",
    );
    b.echo(2, 1, at, 128, 3, ECHO);
    b.finish()
}

/// "Iron Jungle": C minor, a march with snare rolls.
fn level3() -> Song {
    let mut b = SongB::new("level3", 6, kit(1.0));
    b.song.res = 0.45;
    let intro = "Cm:16 Cm:16 Ab:16 Bb:16";
    b.cutoff(0, 700.0).cutoff(63, 1800.0);
    b.groove(0, intro, "K.b.K.b.K.b.K.bb", 2, BASS, DR);
    b.groove(1, "Cm:16 Cm:16 Ab:16", "S.SSS.S.S.SSS.SS", 4, ARP, DR);
    b.groove(1, "Bb:16", "S.S.SSSSSSSSSSSS", 4, ARP, DR);
    b.rest(2, 64);
    b.mark_loop();
    let at = b.sync("a");
    let a = "Cm:16 Cm:16 Ab:16 Bb:16 Cm:16 Fm:16 G:16 G7:16";
    b.cutoff(at, 1600.0);
    b.groove(0, a, "K.bbS.bbK.bbSSbb", 2, BASS, DR);
    b.groove(1, a, "c...c...c...c.c.", 4, ARP, DR);
    b.mel(
        2,
        LEAD,
        "c5:4 c5:2 c5:2 eb5:4 g5:4 | g5:6 f5:2 eb5:4 d5:4 | c5:4 eb5:4 ab5:8 | bb5:6 ab5:2 g5:4 f5:4 |
         g5:4 c6:4 eb6:6 d6:2 | c6:6 ab5:2 f5:8 | b5:4 d6:4 g6:8 | f6:6 d6:2 b5:4 g5:4",
    );
    let at = b.sync("b");
    let bb = "Ab:16 Bb:16 Gm:16 Cm:16 Ab:16 Bb:16 G:16 G:16";
    b.cutoff(at, 2300.0);
    b.groove(0, bb, "K.ooS.ooK.ooSSoS", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "eb6:8 c6:8 | d6:8 bb5:8 | d6:4 bb5:4 g5:4 d6:4 | eb6:12 c6:4 |
         c6:4 eb6:4 ab6:8 | g6:4 f6:4 d6:8 | d6:4 b5:4 g5:4 b5:4 | d6:16",
    );
    b.echo(2, 1, at, 128, 3, ECHO);
    b.finish()
}

/// "Warden": the boss, fast and tense.
fn boss() -> Song {
    let mut b = SongB::new("boss", 4, kit(1.5));
    b.song.res = 0.75;
    b.song.mode = LP | BP;
    let ch = "Am:16 Am:16 Bb:16 Am:16 F:16 E:16 Am:16 E7:16";
    b.cutoff(0, 1200.0).cutoff(64, 2400.0).cutoff(127, 1400.0);
    b.groove(0, ch, "KbbbSbobKbbbSbob", 2, BASS, DR);
    b.groove(1, ch, "c.c.c.c.c.c.c.c.", 4, ARP, DR);
    b.mel(
        2,
        LEAD,
        "a5:2 r:2 a5:2 r:2 c6:2 b5:2 a5:2 e5:2 | a5:2 r:2 a5:2 r:2 e6:4 d6:2 c6:2 |
         bb5:2 r:2 bb5:2 r:2 d6:2 c6:2 bb5:2 f5:2 | a5:8 e5:8 |
         f5:2 a5:2 c6:2 f6:2 e6:4 c6:4 | e6:4 g#5:4 b5:4 d6:4 |
         c6:2 b5:2 a5:2 g#5:2 a5:2 b5:2 c6:2 d6:2 | e6:8 r:4 e5:4",
    );
    b.mark_loop();
    b.groove(0, ch, "KbobSbobKbobSSSS", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "e6:4 a6:4 g6:2 f6:2 e6:4 | d6:4 c6:4 b5:4 c6:4 | d6:4 f6:4 bb6:8 | a6:12 e6:4 |
         f6:4 e6:4 c6:4 a5:4 | g#5:4 b5:4 e6:8 | a6:4 g#6:4 e6:4 c6:4 | b5:8 g#5:8",
    );
    let at = b.len(0) - 128;
    b.echo(2, 1, at, 128, 2, ECHO);
    let mut s = b.finish();
    s.loop_row = Some(0);
    s
}

/// "Signal Lost": slow and sad, plays once.
fn game_over() -> Song {
    let mut b = SongB::new("gameover", 8, kit(0.8));
    b.song.res = 0.6;
    b.song.mode = LP;
    b.cutoff(0, 1600.0).cutoff(80, 400.0);
    let ch = "Am:16 F:16 Dm:16 E:16 Am:32";
    b.groove(0, ch, "b.......b.......", 2, BASS, DR);
    b.groove(1, ch, "c...............", 4, ARP, DR);
    b.mel(
        2,
        LEAD,
        "e5:8 c5:8 | c5:8 a4:8 | d5:8 f5:8 | e5:12 g#4:4 | a4:28 r:4",
    );
    b.finish()
}

/// "Sector Clear": a short fanfare, plays once.
fn clear() -> Song {
    let mut b = SongB::new("clear", 5, kit(1.3));
    b.song.res = 0.4;
    b.song.mode = LP | BP;
    b.cutoff(0, 2600.0);
    let ch = "C:8 F:8 G:8 C:24";
    b.groove(0, "C:8 F:8 G:8", "K.o.S.o.", 2, BASS, DR);
    b.groove(0, "C:24", "K...............o.......", 2, BASS, DR);
    b.groove(1, ch, "C.......", 4, ARP, DR);
    b.mel(
        2,
        SAW,
        "c5:2 e5:2 g5:2 c6:2 | a5:2 c6:2 f6:4 | g5:2 b5:2 d6:2 g6:2 | e6:4 c7:16 r:4",
    );
    b.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn songs_are_well_formed() {
        for &id in ALL {
            let s = build(id);
            assert!(s.problems.is_empty(), "{id:?}: {:?}", s.problems);
            assert!(s.rows() >= 32, "{id:?}");
            assert!(s.seconds() > 4.0, "{id:?}");
            assert_eq!(SongId::from_name(id.name()), Some(id));
        }
        assert!(build(SongId::Title).seconds() > 40.0);
        assert!(build(SongId::Title).loop_row.is_some());
        assert!(build(SongId::GameOver).loop_row.is_none());
        let _ = BP;
    }
}
