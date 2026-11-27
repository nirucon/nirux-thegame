//! Gamepads through Linux evdev (`/dev/input/event*`), with the classic
//! joystick interface (`/dev/input/js*`) as a fallback.
//!
//! Devices are found in sysfs first (`/sys/class/input/eventN/device`), so
//! keyboards, mice, touchpads and motion sensors are never opened: only a
//! node that has gamepad or joystick buttons and axes is. The standard
//! codes cover PS3 pads (hid-sony, the d-pad as `BTN_DPAD_*` buttons),
//! PS4 and PS5 pads (the d-pad as `ABS_HAT0X/Y`), Xbox pads (xpad),
//! 8BitDo and generic USB pads (`BTN_TRIGGER` and friends). Gamepad nodes
//! are normally readable by the logged in user; when one is found but not
//! readable, that is reported once. Unplugging and plugging in again is
//! noticed by scanning every few seconds while no pad is open.
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const EV_KEY: u16 = 0x01;
pub const EV_ABS: u16 = 0x03;

pub const ABS_X: u16 = 0x00;
pub const ABS_Y: u16 = 0x01;
pub const ABS_Z: u16 = 0x02;
pub const ABS_RZ: u16 = 0x05;
pub const ABS_HAT0X: u16 = 0x10;
pub const ABS_HAT0Y: u16 = 0x11;

pub const BTN_JOYSTICK: u16 = 0x120; // BTN_TRIGGER
pub const BTN_THUMB: u16 = 0x121;
pub const BTN_THUMB2: u16 = 0x122;
pub const BTN_TOP: u16 = 0x123;
pub const BTN_BASE3: u16 = 0x128;
pub const BTN_BASE4: u16 = 0x129;
pub const BTN_SOUTH: u16 = 0x130; // BTN_GAMEPAD, BTN_A, PS cross
pub const BTN_EAST: u16 = 0x131; // BTN_B, PS circle
pub const BTN_NORTH: u16 = 0x133; // BTN_X on some pads, PS triangle
pub const BTN_WEST: u16 = 0x134; // PS square
pub const BTN_TL: u16 = 0x136;
pub const BTN_TR: u16 = 0x137;
pub const BTN_TL2: u16 = 0x138;
pub const BTN_TR2: u16 = 0x139;
pub const BTN_SELECT: u16 = 0x13a;
pub const BTN_START: u16 = 0x13b;
pub const BTN_MODE: u16 = 0x13c;
pub const BTN_DPAD_UP: u16 = 0x220;
pub const BTN_DPAD_DOWN: u16 = 0x221;
pub const BTN_DPAD_LEFT: u16 = 0x222;
pub const BTN_DPAD_RIGHT: u16 = 0x223;

const INPUT_PROP_ACCELEROMETER: usize = 0x06;

/// The range of an axis, from EVIOCGABS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AbsRange {
    pub min: i32,
    pub max: i32,
    pub flat: i32,
}

impl Default for AbsRange {
    fn default() -> Self {
        AbsRange {
            min: -32768,
            max: 32767,
            flat: 0,
        }
    }
}

/// What a pad is doing, decoded from its events.
#[derive(Clone, Debug, Default)]
pub struct PadState {
    pub ranges: [AbsRange; 0x12],
    pub x: f32,
    pub y: f32,
    pub hat_x: i32,
    pub hat_y: i32,
    pub lt: f32,
    pub rt: f32,
    buttons: Vec<u16>,
}

impl PadState {
    pub fn down(&self, code: u16) -> bool {
        self.buttons.contains(&code)
    }

    /// Feed one event.
    pub fn apply(&mut self, typ: u16, code: u16, value: i32) {
        match typ {
            EV_KEY => {
                self.buttons.retain(|&b| b != code);
                if value != 0 {
                    self.buttons.push(code);
                }
            }
            EV_ABS => {
                let r = self.ranges.get(code as usize).copied().unwrap_or_default();
                let norm = || {
                    let span = (r.max - r.min).max(1) as f32;
                    let mid = (r.max + r.min) as f32 / 2.0;
                    let v = (value as f32 - mid) / (span / 2.0);
                    // The driver's own dead zone, at least 25 percent.
                    let dead = (r.flat as f32 / (span / 2.0)).max(0.25);
                    if v.abs() < dead {
                        0.0
                    } else {
                        v.clamp(-1.0, 1.0)
                    }
                };
                match code {
                    ABS_X => self.x = norm(),
                    ABS_Y => self.y = norm(),
                    ABS_HAT0X => self.hat_x = value.signum(),
                    ABS_HAT0Y => self.hat_y = value.signum(),
                    ABS_Z => {
                        let span = (r.max - r.min).max(1) as f32;
                        self.lt = (value - r.min) as f32 / span;
                    }
                    ABS_RZ => {
                        let span = (r.max - r.min).max(1) as f32;
                        self.rt = (value - r.min) as f32 / span;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Left, right, up, down, fire, special, start, back.
    pub fn actions(&self) -> [bool; 8] {
        let d = |c| self.down(c);
        let any = |cs: &[u16]| cs.iter().any(|&c| d(c));
        [
            self.x < -0.5 || self.hat_x < 0 || d(BTN_DPAD_LEFT),
            self.x > 0.5 || self.hat_x > 0 || d(BTN_DPAD_RIGHT),
            self.y < -0.5 || self.hat_y < 0 || d(BTN_DPAD_UP),
            self.y > 0.5 || self.hat_y > 0 || d(BTN_DPAD_DOWN),
            any(&[BTN_SOUTH, BTN_WEST, BTN_TR, BTN_TR2, BTN_JOYSTICK, BTN_TOP]) || self.rt > 0.5,
            any(&[BTN_EAST, BTN_NORTH, BTN_TL, BTN_TL2, BTN_THUMB, BTN_THUMB2]) || self.lt > 0.5,
            any(&[BTN_START, BTN_BASE4]),
            any(&[BTN_SELECT, BTN_MODE, BTN_BASE3]),
        ]
    }
}

/// A bitmap from sysfs: space separated hex words, most significant first.
pub fn parse_bits(s: &str) -> Vec<u64> {
    let mut words: Vec<u64> = s
        .split_whitespace()
        .filter_map(|w| u64::from_str_radix(w, 16).ok())
        .collect();
    words.reverse();
    words
}

pub fn bit(bits: &[u64], n: usize) -> bool {
    bits.get(n / 64).is_some_and(|w| w & (1 << (n % 64)) != 0)
}

/// Is this a gamepad or joystick (from its sysfs capabilities)?
pub fn is_pad(ev: &[u64], key: &[u64], abs: &[u64], props: &[u64]) -> bool {
    let has_key = bit(ev, EV_KEY as usize);
    let has_abs = bit(ev, EV_ABS as usize);
    let buttons = bit(key, BTN_SOUTH as usize) || bit(key, BTN_JOYSTICK as usize);
    let stick = bit(abs, ABS_X as usize) || bit(abs, ABS_HAT0X as usize);
    has_key && has_abs && buttons && stick && !bit(props, INPUT_PROP_ACCELEROMETER)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    None,
    Connected(String),
    NotReadable(String),
}

struct Open {
    file: std::fs::File,
    state: PadState,
    evdev: bool,
}

pub struct Pads {
    open: Option<Open>,
    next_scan: Instant,
    pub status: Status,
    warned: bool,
    pub enabled: bool,
    sys: PathBuf,
}

const fn ioc_read(nr: u64, size: u64) -> u64 {
    (2 << 30) | (size << 16) | ((b'E' as u64) << 8) | nr
}

fn read_trim(p: &Path) -> String {
    std::fs::read_to_string(p)
        .unwrap_or_default()
        .trim()
        .to_string()
}

impl Default for Pads {
    fn default() -> Self {
        Self::new()
    }
}

impl Pads {
    pub fn new() -> Pads {
        Pads {
            open: None,
            next_scan: Instant::now(),
            status: Status::None,
            warned: false,
            enabled: std::env::var_os("NIRUX_NO_JOYSTICK").is_none(),
            sys: PathBuf::from("/sys/class/input"),
        }
    }

    fn scan(&mut self) {
        let mut found_unreadable: Option<String> = None;
        let mut nodes: Vec<(u32, PathBuf)> = std::fs::read_dir(&self.sys)
            .map(|d| {
                d.flatten()
                    .filter_map(|e| {
                        let n = e.file_name().to_string_lossy().to_string();
                        n.strip_prefix("event")
                            .and_then(|k| k.parse().ok())
                            .map(|k| (k, e.path()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        nodes.sort();
        for (k, path) in nodes {
            let dev = path.join("device");
            let caps = |n: &str| parse_bits(&read_trim(&dev.join("capabilities").join(n)));
            if !is_pad(
                &caps("ev"),
                &caps("key"),
                &caps("abs"),
                &parse_bits(&read_trim(&dev.join("properties"))),
            ) {
                continue;
            }
            let name = read_trim(&dev.join("name"));
            let node = format!("/dev/input/event{k}");
            match std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(&node)
            {
                Ok(file) => {
                    let mut state = PadState::default();
                    for axis in [ABS_X, ABS_Y, ABS_Z, ABS_RZ, ABS_HAT0X, ABS_HAT0Y] {
                        let mut info = libc::input_absinfo {
                            value: 0,
                            minimum: 0,
                            maximum: 0,
                            fuzz: 0,
                            flat: 0,
                            resolution: 0,
                        };
                        let req = ioc_read(
                            0x40 + axis as u64,
                            std::mem::size_of::<libc::input_absinfo>() as u64,
                        );
                        // SAFETY: EVIOCGABS fills an input_absinfo.
                        let rc = unsafe { libc::ioctl(file.as_raw_fd(), req as _, &mut info) };
                        if rc >= 0 && info.maximum > info.minimum {
                            state.ranges[axis as usize] = AbsRange {
                                min: info.minimum,
                                max: info.maximum,
                                flat: info.flat,
                            };
                            state.apply(EV_ABS, axis, info.value);
                        }
                    }
                    self.status = Status::Connected(name);
                    self.open = Some(Open {
                        file,
                        state,
                        evdev: true,
                    });
                    return;
                }
                Err(_) => {
                    found_unreadable.get_or_insert(name);
                }
            }
        }
        // The old joystick interface.
        for i in 0..8 {
            let p = format!("/dev/input/js{i}");
            if let Ok(file) = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(&p)
            {
                let name = read_trim(Path::new(&format!("/sys/class/input/js{i}/device/name")));
                let name = if name.is_empty() { p } else { name };
                self.status = Status::Connected(name);
                self.open = Some(Open {
                    file,
                    state: PadState::default(),
                    evdev: false,
                });
                return;
            }
        }
        if let Some(n) = found_unreadable {
            if !self.warned {
                eprintln!(
                    "nirux-thegame: found the gamepad \"{n}\" but cannot read it; \
                     check the permissions of /dev/input/event*"
                );
                self.warned = true;
            }
            self.status = Status::NotReadable(n);
        } else {
            self.status = Status::None;
        }
    }

    pub fn poll(&mut self) {
        if !self.enabled {
            return;
        }
        if self.open.is_none() {
            if Instant::now() >= self.next_scan {
                self.next_scan = Instant::now() + Duration::from_secs(3);
                self.scan();
            }
            return;
        }
        let Some(o) = self.open.as_mut() else {
            return;
        };
        let mut gone = false;
        let mut buf = [0u8; 24 * 64];
        loop {
            match o.file.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if o.evdev {
                        let sz = std::mem::size_of::<libc::input_event>();
                        for e in buf[..n].chunks_exact(sz) {
                            let typ = u16::from_ne_bytes([e[sz - 8], e[sz - 7]]);
                            let code = u16::from_ne_bytes([e[sz - 6], e[sz - 5]]);
                            let value =
                                i32::from_ne_bytes([e[sz - 4], e[sz - 3], e[sz - 2], e[sz - 1]]);
                            o.state.apply(typ, code, value);
                        }
                    } else {
                        for e in buf[..n].chunks_exact(8) {
                            js_event(
                                &mut o.state,
                                e[6] & 0x7f,
                                e[7],
                                i16::from_ne_bytes([e[4], e[5]]),
                            );
                        }
                    }
                    if n < buf.len() {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => {
                    gone = true;
                    break;
                }
            }
        }
        if gone {
            self.open = None;
            self.status = Status::None;
            self.next_scan = Instant::now() + Duration::from_millis(500);
        }
    }

    pub fn actions(&self) -> [bool; 8] {
        self.open
            .as_ref()
            .map(|o| o.state.actions())
            .unwrap_or([false; 8])
    }
}

/// The joystick interface: buttons 0..n, axes 0..n (Xbox style layout).
fn js_event(s: &mut PadState, kind: u8, num: u8, value: i16) {
    match kind {
        1 => {
            let code = match num {
                0 => BTN_SOUTH,
                1 => BTN_EAST,
                2 => BTN_WEST,
                3 => BTN_NORTH,
                4 => BTN_TL,
                5 => BTN_TR,
                6 => BTN_SELECT,
                7 => BTN_START,
                8 => BTN_MODE,
                _ => return,
            };
            s.apply(EV_KEY, code, value as i32);
        }
        2 => {
            let code = match num {
                0 => ABS_X,
                1 => ABS_Y,
                6 => ABS_HAT0X,
                7 => ABS_HAT0Y,
                _ => return,
            };
            s.apply(EV_ABS, code, value as i32);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(s: &mut PadState, evs: &[(u16, u16, i32)]) {
        for &(t, c, v) in evs {
            s.apply(t, c, v);
        }
    }

    #[test]
    fn ps3_dpad_buttons_and_stick() {
        // hid-sony: sticks 0..255, the d-pad as buttons, cross is SOUTH.
        let mut s = PadState::default();
        for a in [ABS_X, ABS_Y] {
            s.ranges[a as usize] = AbsRange {
                min: 0,
                max: 255,
                flat: 15,
            };
        }
        feed(
            &mut s,
            &[
                (EV_ABS, ABS_X, 128),
                (EV_ABS, ABS_Y, 127),
                (EV_KEY, BTN_DPAD_UP, 1),
                (EV_KEY, BTN_SOUTH, 1),
            ],
        );
        let a = s.actions();
        assert!(a[2] && a[4] && !a[0] && !a[1] && !a[3]);
        feed(
            &mut s,
            &[
                (EV_KEY, BTN_DPAD_UP, 0),
                (EV_KEY, BTN_SOUTH, 0),
                (EV_ABS, ABS_X, 0),
                (EV_KEY, BTN_EAST, 1),
            ],
        );
        let a = s.actions();
        assert!(a[0] && a[5] && !a[2] && !a[4]);
        // Start and select.
        feed(&mut s, &[(EV_KEY, BTN_START, 1), (EV_KEY, BTN_SELECT, 1)]);
        assert!(s.actions()[6] && s.actions()[7]);
    }

    #[test]
    fn ps4_hat_and_xbox_stick_with_dead_zone() {
        let mut s = PadState::default();
        s.ranges[ABS_HAT0X as usize] = AbsRange {
            min: -1,
            max: 1,
            flat: 0,
        };
        feed(&mut s, &[(EV_ABS, ABS_HAT0X, 1), (EV_ABS, ABS_HAT0Y, -1)]);
        let a = s.actions();
        assert!(a[1] && a[2]);
        // Xbox: -32768..32767, a small push stays inside the dead zone.
        let mut x = PadState::default();
        x.ranges[ABS_X as usize] = AbsRange {
            min: -32768,
            max: 32767,
            flat: 128,
        };
        feed(&mut x, &[(EV_ABS, ABS_X, 4000)]);
        assert!(!x.actions()[1]);
        feed(&mut x, &[(EV_ABS, ABS_X, 30000)]);
        assert!(x.actions()[1]);
        // The right trigger as an axis fires.
        x.ranges[ABS_RZ as usize] = AbsRange {
            min: 0,
            max: 255,
            flat: 0,
        };
        feed(&mut x, &[(EV_ABS, ABS_RZ, 250)]);
        assert!(x.actions()[4]);
    }

    #[test]
    fn generic_usb_pad_and_js_fallback() {
        let mut s = PadState::default();
        feed(&mut s, &[(EV_KEY, BTN_JOYSTICK, 1), (EV_KEY, BTN_BASE4, 1)]);
        assert!(s.actions()[4] && s.actions()[6]);
        let mut j = PadState::default();
        js_event(&mut j, 1, 1, 1);
        js_event(&mut j, 2, 7, 32767);
        assert!(j.actions()[5] && j.actions()[3]);
    }

    #[test]
    fn classification_from_sysfs_bitmaps() {
        // An Xbox pad: EV_SYN, EV_KEY, EV_ABS, EV_FF.
        let ev = parse_bits("20000b");
        let key = parse_bits("7cdb000000000000 0 0 0 0");
        let abs = parse_bits("3003f");
        assert!(is_pad(&ev, &key, &abs, &[]));
        // A keyboard: keys but no gamepad buttons and no axes.
        let kb_key = parse_bits("fffffffffffffffe ffffffffffffffff");
        assert!(!is_pad(&parse_bits("120013"), &kb_key, &[], &[]));
        // The PS3 motion sensors: axes and an accelerometer property.
        assert!(!is_pad(&ev, &key, &abs, &parse_bits("40")));
        assert!(bit(&parse_bits("1 0"), 64));
    }
}
