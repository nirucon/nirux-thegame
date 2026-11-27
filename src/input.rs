//! Controls: the keyboard (arrows or WASD, Space, Ctrl or Z to fire, X or
//! Shift for the special weapon, Enter, Esc) and a joystick or gamepad
//! through the Linux joystick interface (`/dev/input/js*`), which needs no
//! extra libraries and is readable by the logged in user on most systems.
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub fire: bool,
    pub special: bool,
    // Pressed this frame.
    pub p_left: bool,
    pub p_right: bool,
    pub p_up: bool,
    pub p_down: bool,
    pub p_fire: bool,
    pub p_special: bool,
    pub p_start: bool,
    pub p_back: bool,
}

impl Input {
    pub fn any_pressed(&self) -> bool {
        self.p_fire || self.p_start || self.p_special || self.p_back
    }
    pub fn dx(&self) -> f32 {
        (self.right as i32 - self.left as i32) as f32
    }
    pub fn dy(&self) -> f32 {
        (self.down as i32 - self.up as i32) as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Fire,
    Special,
    Start,
    Back,
}

pub const K_F11: u32 = 0xffc8;
pub const K_M: u32 = b'm' as u32;

pub fn map_key(sym: u32) -> Option<Key> {
    Some(match sym {
        0xff51 | 0x61 | 0xff96 => Key::Left,
        0xff53 | 0x64 | 0xff98 => Key::Right,
        0xff52 | 0x77 | 0xff97 => Key::Up,
        0xff54 | 0x73 | 0xff99 => Key::Down,
        0x20 | 0xffe3 | 0xffe4 | 0x7a | 0x6a => Key::Fire,
        0x78 | 0xffe1 | 0xffe2 | 0xffe9 | 0x6b => Key::Special,
        0xff0d | 0xff8d => Key::Start,
        0xff1b | 0x70 | 0xff08 => Key::Back,
        _ => return None,
    })
}

#[derive(Default)]
struct State {
    held: [bool; 8],
}

pub struct Controls {
    kb: State,
    pressed: [bool; 8],
    joy: Joy,
    prev: [bool; 8],
}

impl Default for Controls {
    fn default() -> Self {
        Self::new()
    }
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            kb: State::default(),
            pressed: [false; 8],
            joy: Joy::new(),
            prev: [false; 8],
        }
    }

    /// Keyboard (and scripted) input only, for the self test.
    pub fn keyboard_only() -> Controls {
        let mut c = Controls::new();
        c.joy.enabled = false;
        c
    }

    pub fn key(&mut self, k: Key, down: bool) {
        let i = k as usize;
        if down && !self.kb.held[i] {
            self.pressed[i] = true;
        }
        self.kb.held[i] = down;
    }

    pub fn release_all(&mut self) {
        self.kb.held = [false; 8];
    }

    /// The state for this frame (call once per frame).
    pub fn frame(&mut self) -> Input {
        self.joy.poll();
        let mut held = self.kb.held;
        let j = self.joy.held();
        for i in 0..8 {
            held[i] |= j[i];
        }
        let mut p = self.pressed;
        for i in 0..8 {
            // A joystick press counts on its rising edge.
            if j[i] && !self.prev[i] {
                p[i] = true;
            }
        }
        self.prev = j;
        self.pressed = [false; 8];
        Input {
            left: held[0],
            right: held[1],
            up: held[2],
            down: held[3],
            fire: held[4],
            special: held[5],
            p_left: p[0],
            p_right: p[1],
            p_up: p[2],
            p_down: p[3],
            p_fire: p[4],
            p_special: p[5],
            p_start: p[6],
            p_back: p[7],
        }
    }
}

struct Joy {
    file: Option<std::fs::File>,
    name: Option<String>,
    axes: [i16; 8],
    buttons: [bool; 16],
    next_scan: Instant,
    /// False for scripted runs (the self test): a joystick device on the
    /// machine, real or virtual, must not steer them.
    enabled: bool,
}

impl Joy {
    fn new() -> Joy {
        Joy {
            enabled: true,
            file: None,
            name: None,
            axes: [0; 8],
            buttons: [false; 16],
            next_scan: Instant::now(),
        }
    }

    fn scan(&mut self) {
        if std::env::var_os("NIRUX_NO_JOYSTICK").is_some() {
            return;
        }
        for i in 0..8 {
            let path = format!("/dev/input/js{i}");
            if let Ok(f) = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(&path)
            {
                self.file = Some(f);
                self.name = Some(path);
                self.axes = [0; 8];
                self.buttons = [false; 16];
                return;
            }
        }
    }

    fn poll(&mut self) {
        if !self.enabled {
            return;
        }
        if self.file.is_none() {
            if Instant::now() >= self.next_scan {
                self.next_scan = Instant::now() + Duration::from_secs(3);
                self.scan();
            }
            return;
        }
        let mut buf = [0u8; 8 * 32];
        loop {
            let Some(f) = self.file.as_mut() else {
                return;
            };
            match f.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    for e in buf[..n].chunks_exact(8) {
                        let value = i16::from_le_bytes([e[4], e[5]]);
                        let kind = e[6] & 0x7f;
                        let num = e[7] as usize;
                        match kind {
                            1 if num < 16 => self.buttons[num] = value != 0,
                            2 if num < 8 => self.axes[num] = value,
                            _ => {}
                        }
                    }
                    if n < buf.len() {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => {
                    // Unplugged.
                    self.file = None;
                    self.name = None;
                    self.axes = [0; 8];
                    self.buttons = [false; 16];
                    break;
                }
            }
        }
    }

    /// Left stick or d-pad (axes 6 and 7 on most pads); A or the trigger
    /// fires, B is the special, Start pauses, Select goes back.
    fn held(&self) -> [bool; 8] {
        let t = 12000;
        let a = &self.axes;
        let b = &self.buttons;
        [
            a[0] < -t || a[6] < -t,
            a[0] > t || a[6] > t,
            a[1] < -t || a[7] < -t,
            a[1] > t || a[7] > t,
            b[0] || b[2] || b[5],
            b[1] || b[3] || b[4],
            b[7] || b[9],
            b[6] || b[8],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keys_map_and_edges_fire_once() {
        assert_eq!(map_key(0xff51), Some(Key::Left));
        assert_eq!(map_key(0x20), Some(Key::Fire));
        assert_eq!(map_key(0x41), None);
        let mut c = Controls::new();
        c.key(Key::Fire, true);
        let a = c.frame();
        assert!(a.fire && a.p_fire);
        let b = c.frame();
        assert!(b.fire && !b.p_fire);
        c.key(Key::Fire, false);
        assert!(!c.frame().fire);
    }
}
