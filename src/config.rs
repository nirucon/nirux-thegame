//! Settings in `$XDG_CONFIG_HOME/nirux-thegame/config.toml`, progress and
//! high scores in `$XDG_DATA_HOME/nirux-thegame/`, and the NIRUX theme
//! colours (the same lookup as the other NIRUX programs: the theme NIRUWM
//! names, a user theme file, then the appearance overrides). The files are
//! simple `key = value` TOML, read by a small parser of our own.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn xdg(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home().join(fallback))
}

pub fn config_home() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config")
}

pub fn data_dir() -> PathBuf {
    xdg("XDG_DATA_HOME", ".local/share").join("nirux-thegame")
}

pub fn config_path() -> PathBuf {
    config_home().join("nirux-thegame/config.toml")
}

/// `[section]` headers and `key = value` lines; keys inside a section
/// come back as `section.key`. Quotes around strings are removed.
pub fn parse_toml(text: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut section = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(s) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = s.trim().to_string();
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let mut v = v.trim();
        if let Some(q) = v.strip_prefix('"') {
            v = q.split('"').next().unwrap_or("");
        } else if let Some(i) = v.find('#') {
            v = v[..i].trim();
        }
        let key = if section.is_empty() {
            k.trim().to_string()
        } else {
            format!("{section}.{}", k.trim())
        };
        out.insert(key, v.to_string());
    }
    out
}

fn read_map(p: &Path) -> HashMap<String, String> {
    std::fs::read_to_string(p)
        .map(|t| parse_toml(&t))
        .unwrap_or_default()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Easy => "EASY",
            Difficulty::Normal => "NORMAL",
            Difficulty::Hard => "HARD",
        }
    }
    pub fn parse(s: &str) -> Option<Difficulty> {
        match s.to_ascii_lowercase().as_str() {
            "easy" => Some(Difficulty::Easy),
            "normal" => Some(Difficulty::Normal),
            "hard" => Some(Difficulty::Hard),
            _ => None,
        }
    }
    pub fn next(self) -> Difficulty {
        match self {
            Difficulty::Easy => Difficulty::Normal,
            Difficulty::Normal => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Easy,
        }
    }
    pub fn prev(self) -> Difficulty {
        self.next().next()
    }
    /// Enemy fire rate and bullet speed factor.
    pub fn aggression(self) -> f32 {
        match self {
            Difficulty::Easy => 0.6,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.4,
        }
    }
    pub fn lives(self) -> i32 {
        match self {
            Difficulty::Easy => 5,
            Difficulty::Normal => 3,
            Difficulty::Hard => 3,
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub fullscreen: bool,
    pub crt: bool,
    pub music: u8,
    pub sfx: u8,
    pub echo: bool,
    pub difficulty: Difficulty,
    pub audio: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            fullscreen: true,
            crt: true,
            music: 8,
            sfx: 8,
            echo: true,
            difficulty: Difficulty::Normal,
            audio: String::new(),
        }
    }
}

impl Config {
    pub fn load() -> Config {
        Self::from_map(&read_map(&config_path()))
    }

    pub fn from_map(m: &HashMap<String, String>) -> Config {
        let mut c = Config::default();
        let b = |k: &str, d: bool| m.get(k).map(|v| v == "true").unwrap_or(d);
        let n = |k: &str, d: u8| {
            m.get(k)
                .and_then(|v| v.parse::<u8>().ok())
                .unwrap_or(d)
                .min(10)
        };
        c.fullscreen = b("fullscreen", c.fullscreen);
        c.crt = b("crt", c.crt);
        c.echo = b("echo", c.echo);
        c.music = n("music_volume", c.music);
        c.sfx = n("sfx_volume", c.sfx);
        if let Some(d) = m.get("difficulty").and_then(|d| Difficulty::parse(d)) {
            c.difficulty = d;
        }
        if let Some(a) = m.get("audio") {
            c.audio = a.clone();
        }
        c
    }

    pub fn to_toml(&self) -> String {
        format!(
            "# NIRUX settings (written by the game; edit freely while it is not running)\n\
             fullscreen = {}\ncrt = {}\nmusic_volume = {}\nsfx_volume = {}\necho = {}\n\
             difficulty = \"{}\"\n# A command that plays raw s16le stereo PCM from stdin; %r is the rate.\n\
             audio = \"{}\"\n",
            self.fullscreen,
            self.crt,
            self.music,
            self.sfx,
            self.echo,
            self.difficulty.name().to_ascii_lowercase(),
            self.audio.replace('"', "")
        )
    }

    pub fn save(&self) {
        let p = config_path();
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = write_atomic(&p, &self.to_toml());
    }
}

pub fn write_atomic(p: &Path, text: &str) -> std::io::Result<()> {
    let tmp = p.with_extension("tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, p)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Score {
    pub name: String,
    pub score: u32,
    pub sector: u8,
    pub difficulty: Difficulty,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Save {
    /// The highest sector reached (1 based) per difficulty.
    pub reached: [u8; 3],
    pub scores: Vec<Score>,
    pub cleared: bool,
}

pub const MAX_SCORES: usize = 10;

impl Default for Save {
    fn default() -> Self {
        let names = [
            "NRX", "SID", "VIC", "CIA", "ACE", "MAX", "ZAP", "JET", "ION", "RUN",
        ];
        let scores = names
            .iter()
            .enumerate()
            .map(|(i, n)| Score {
                name: n.to_string(),
                score: 50000 - i as u32 * 5000,
                sector: [3, 3, 2, 2, 2, 1, 1, 1, 1, 1][i],
                difficulty: Difficulty::Normal,
            })
            .collect();
        Save {
            reached: [1; 3],
            scores,
            cleared: false,
        }
    }
}

impl Save {
    pub fn path() -> PathBuf {
        data_dir().join("save.toml")
    }

    pub fn load() -> Save {
        Self::from_text(&std::fs::read_to_string(Self::path()).unwrap_or_default())
    }

    pub fn from_text(t: &str) -> Save {
        let mut s = Save::default();
        let m = parse_toml(t);
        for (i, k) in ["easy", "normal", "hard"].iter().enumerate() {
            if let Some(v) = m
                .get(&format!("reached.{k}"))
                .and_then(|v| v.parse::<u8>().ok())
            {
                s.reached[i] = v.clamp(1, crate::game::LEVELS as u8);
            }
        }
        s.cleared = m.get("cleared").is_some_and(|v| v == "true");
        let mut scores = Vec::new();
        for i in 0..MAX_SCORES {
            let Some(v) = m.get(&format!("scores.s{i}")) else {
                continue;
            };
            let parts: Vec<&str> = v.split_whitespace().collect();
            if parts.len() == 4 {
                if let (Ok(score), Ok(sector), Some(d)) = (
                    parts[1].parse(),
                    parts[2].parse(),
                    Difficulty::parse(parts[3]),
                ) {
                    scores.push(Score {
                        name: parts[0].chars().take(3).collect(),
                        score,
                        sector,
                        difficulty: d,
                    });
                }
            }
        }
        if !scores.is_empty() {
            scores.sort_by_key(|s| std::cmp::Reverse(s.score));
            s.scores = scores;
        }
        s
    }

    pub fn to_text(&self) -> String {
        let mut t = String::from("# NIRUX progress and high scores\n");
        t += &format!("cleared = {}\n[reached]\n", self.cleared);
        for (i, k) in ["easy", "normal", "hard"].iter().enumerate() {
            t += &format!("{k} = {}\n", self.reached[i]);
        }
        t += "[scores]\n";
        for (i, s) in self.scores.iter().enumerate() {
            let name: String = s
                .name
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '.' })
                .collect();
            t += &format!(
                "s{i} = \"{} {} {} {}\"\n",
                name,
                s.score,
                s.sector,
                s.difficulty.name().to_ascii_lowercase()
            );
        }
        t
    }

    pub fn save(&self) {
        let p = Self::path();
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let _ = write_atomic(&p, &self.to_text());
    }

    pub fn qualifies(&self, score: u32) -> bool {
        score > 0 && (self.scores.len() < MAX_SCORES || self.scores.iter().any(|s| score > s.score))
    }

    /// Insert a score; returns its place.
    pub fn insert(&mut self, s: Score) -> usize {
        let pos = self
            .scores
            .iter()
            .position(|x| s.score > x.score)
            .unwrap_or(self.scores.len());
        self.scores.insert(pos, s);
        self.scores.truncate(MAX_SCORES);
        pos
    }

    pub fn reach(&mut self, d: Difficulty, sector: u8) {
        let i = d.index();
        self.reached[i] = self.reached[i].max(sector.min(crate::game::LEVELS as u8));
    }

    /// The highest sector reached on any difficulty (level select).
    pub fn best_reached(&self) -> u8 {
        *self.reached.iter().max().unwrap_or(&1)
    }
}

/// Menu colours from the NIRUX theme: foreground, accent, muted.
pub fn theme_colors(root_theme: Option<&str>) -> [u32; 3] {
    let base = config_home();
    let wm = read_map(&base.join("niruwm/config.toml"));
    let name = root_theme
        .map(str::to_string)
        .or_else(|| wm.get("theme").cloned())
        .unwrap_or_else(|| "niru-noir".into());
    // Built-in NIRUX themes: foreground, focus, muted.
    let builtin: &[(&str, u32, u32, u32)] = &[
        ("niru-noir", 0xe5e5e5, 0x9a9aa2, 0xa8a8a8),
        ("niru-nord", 0xeceff4, 0x88c0d0, 0x9aa5b1),
        ("niru-satie", 0xdedad4, 0xb08d7d, 0x8f8b86),
        ("niru-carl-larsson", 0xf3ece0, 0xa63d2f, 0x7d6f5f),
        ("niru-forest-vigil", 0xd8ded2, 0x8aa87e, 0x96a292),
        ("niru-blood-of-urdr", 0xe8d6cf, 0xc96050, 0xb09a92),
        ("niru-root-and-stone", 0xeadfd2, 0xc19768, 0xb5a08c),
        ("niru-blue-horizon", 0xdce6eb, 0x86aec8, 0x93a6b2),
        ("niru-the-norns", 0xece9e4, 0xc9c2b9, 0xa6a19b),
        ("niru-winter-nights", 0xead9c3, 0xd8c4aa, 0xb9aa9a),
    ];
    let (_, mut fg, mut acc, mut muted) = builtin
        .iter()
        .copied()
        .find(|t| t.0 == name)
        .unwrap_or(builtin[0]);
    let hex = |s: &str| {
        let s = s.trim().trim_start_matches('#');
        (s.len() == 6)
            .then(|| u32::from_str_radix(s, 16).ok())
            .flatten()
    };
    let file = read_map(&base.join("niruwm/themes").join(format!("{name}.toml")));
    let get = |k: &str| {
        file.get(&format!("colors.{k}"))
            .or_else(|| file.get(k))
            .and_then(|v| hex(v))
    };
    if let Some(c) = get("foreground") {
        fg = c;
    }
    if let Some(c) = get("focus").or_else(|| get("accent")) {
        acc = c;
    }
    if let Some(c) = get("muted") {
        muted = c;
    }
    let app = read_map(&base.join("niruwm/appearance.toml"));
    if let Some(c) = app.get("colors.foreground").and_then(|v| hex(v)) {
        fg = c;
    }
    if let Some(c) = app
        .get("colors.focus")
        .or_else(|| app.get("colors.accent"))
        .and_then(|v| hex(v))
    {
        acc = c;
    }
    // Light themes have dark text; the game is drawn on black, so use
    // the accent's brightness for readability.
    let lum = |c: u32| {
        0.2126 * ((c >> 16) & 255) as f32
            + 0.7152 * ((c >> 8) & 255) as f32
            + 0.0722 * (c & 255) as f32
    };
    if lum(fg) < 110.0 {
        fg = 0xe8e8e8;
    }
    if lum(acc) < 60.0 {
        acc = 0x9ad284;
    }
    [fg, acc, muted]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toml_config_and_save_round_trip() {
        let m = parse_toml("a = 1\n# c\n[x]\nb = \"hi # there\"\nc = true # yes\n");
        assert_eq!(m["a"], "1");
        assert_eq!(m["x.b"], "hi # there");
        assert_eq!(m["x.c"], "true");
        let c = Config {
            crt: false,
            music: 3,
            difficulty: Difficulty::Hard,
            ..Config::default()
        };
        assert_eq!(Config::from_map(&parse_toml(&c.to_toml())), c);
        let mut s = Save::default();
        s.reach(Difficulty::Easy, 3);
        let pos = s.insert(Score {
            name: "ABC".into(),
            score: 99999,
            sector: 2,
            difficulty: Difficulty::Easy,
        });
        assert_eq!(pos, 0);
        let back = Save::from_text(&s.to_text());
        assert_eq!(back, s);
        assert_eq!(back.best_reached(), 3);
        assert!(s.qualifies(45001));
        assert!(!s.qualifies(10));
    }
}
