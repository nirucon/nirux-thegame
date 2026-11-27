//! Controls: the keyboard (arrows or WASD, Space, Ctrl or Z to fire, X or
//! Shift for the special weapon, Enter, Esc) and gamepads (see `pad.rs`),
//! merged into one state per frame with edges for the menus.

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
    pub pads: crate::pad::Pads,
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
            pads: crate::pad::Pads::new(),
            prev: [false; 8],
        }
    }

    /// Keyboard (and scripted) input only, for the self test.
    pub fn keyboard_only() -> Controls {
        let mut c = Controls::new();
        c.pads.enabled = false;
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
        self.pads.poll();
        let mut held = self.kb.held;
        let j = self.pads.actions();
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
