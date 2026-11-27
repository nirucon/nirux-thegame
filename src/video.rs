//! The X11 window: one window with a hidden pointer, fullscreen through
//! `_NET_WM_STATE_FULLSCREEN` (or by covering the screen when no window
//! manager runs), the keyboard, and presenting the scaled picture with
//! MIT-SHM straight into shared memory (or `PutImage` in slices without it).
use crate::fb::Fb;
use crate::palette::Lut;
use crate::scale::{self, Layout};
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::shm::ConnectionExt as _;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

pub type Res<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ev {
    Key { sym: u32, down: bool },
    Close,
    Focus(bool),
    Visible(bool),
}

struct Shm {
    seg: u32,
    addr: *mut u8,
    size: usize,
}

pub struct Video {
    c: RustConnection,
    root: Window,
    win: Window,
    gc: Gcontext,
    depth: u8,
    screen_w: u16,
    screen_h: u16,
    wm_delete: Atom,
    net_wm_state: Atom,
    net_fullscreen: Atom,
    has_wm: bool,
    shm: Option<Shm>,
    shm_ok: bool,
    min_keycode: u8,
    per: u8,
    keysyms: Vec<u32>,
    pub w: i32,
    pub h: i32,
    pub fullscreen: bool,
    border: Option<u32>,
    border_layout: Option<Layout>,
    scratch: Vec<u32>,
}

const WIN_W: u16 = 960;
const WIN_H: u16 = 600;

impl Video {
    pub fn open(fullscreen: bool) -> Res<Video> {
        let (c, screen) = x11rb::connect(None)?;
        let s = c.setup().roots[screen].clone();
        let names = [
            "WM_PROTOCOLS",
            "WM_DELETE_WINDOW",
            "_NET_WM_NAME",
            "UTF8_STRING",
            "_NET_WM_STATE",
            "_NET_WM_STATE_FULLSCREEN",
            "_NET_WM_PID",
            "_NET_SUPPORTING_WM_CHECK",
            "_NET_WM_ICON",
        ];
        let cookies: Vec<_> = names
            .iter()
            .map(|n| c.intern_atom(false, n.as_bytes()))
            .collect::<Result<_, _>>()?;
        let mut a = Vec::new();
        for ck in cookies {
            a.push(ck.reply()?.atom);
        }
        let has_wm = c
            .get_property(false, s.root, a[7], AtomEnum::WINDOW, 0, 1)?
            .reply()
            .map(|r| r.value_len > 0)
            .unwrap_or(false);
        let win = c.generate_id()?;
        let (w, h) = if fullscreen && !has_wm {
            (s.width_in_pixels, s.height_in_pixels)
        } else {
            (WIN_W, WIN_H)
        };
        c.create_window(
            s.root_depth,
            win,
            s.root,
            0,
            0,
            w,
            h,
            0,
            WindowClass::INPUT_OUTPUT,
            s.root_visual,
            &CreateWindowAux::new()
                .background_pixel(0)
                .bit_gravity(Gravity::NORTH_WEST)
                .event_mask(
                    EventMask::EXPOSURE
                        | EventMask::KEY_PRESS
                        | EventMask::KEY_RELEASE
                        | EventMask::STRUCTURE_NOTIFY
                        | EventMask::VISIBILITY_CHANGE
                        | EventMask::FOCUS_CHANGE,
                ),
        )?;
        let class = b"nirux-thegame\0nirux-thegame\0";
        c.change_property8(
            PropMode::REPLACE,
            win,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            class,
        )?;
        c.change_property32(PropMode::REPLACE, win, a[0], AtomEnum::ATOM, &[a[1]])?;
        c.change_property32(
            PropMode::REPLACE,
            win,
            a[6],
            AtomEnum::CARDINAL,
            &[std::process::id()],
        )?;
        let title = b"NIRUX";
        c.change_property8(PropMode::REPLACE, win, a[2], a[3], title)?;
        c.change_property8(
            PropMode::REPLACE,
            win,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            title,
        )?;
        c.change_property32(PropMode::REPLACE, win, a[8], AtomEnum::CARDINAL, &icon())?;
        if fullscreen && has_wm {
            c.change_property32(PropMode::REPLACE, win, a[4], AtomEnum::ATOM, &[a[5]])?;
        }
        // An invisible pointer over the game.
        let pm = c.generate_id()?;
        c.create_pixmap(1, pm, win, 1, 1)?;
        let cur = c.generate_id()?;
        c.create_cursor(cur, pm, pm, 0, 0, 0, 0, 0, 0, 0, 0)?;
        c.change_window_attributes(win, &ChangeWindowAttributesAux::new().cursor(cur))?;
        c.free_pixmap(pm)?;
        let gc = c.generate_id()?;
        c.create_gc(gc, win, &CreateGCAux::new().graphics_exposures(0))?;
        let shm_ok = c
            .extension_information(x11rb::protocol::shm::X11_EXTENSION_NAME)?
            .is_some()
            && c.shm_query_version()
                .ok()
                .and_then(|r| r.reply().ok())
                .is_some()
            && std::env::var_os("NIRU_NO_SHM").is_none();
        let (min_keycode, max_keycode) = (c.setup().min_keycode, c.setup().max_keycode);
        let map = c
            .get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)?
            .reply()?;
        c.map_window(win)?;
        c.flush()?;
        Ok(Video {
            root: s.root,
            win,
            gc,
            depth: s.root_depth,
            screen_w: s.width_in_pixels,
            screen_h: s.height_in_pixels,
            wm_delete: a[1],
            net_wm_state: a[4],
            net_fullscreen: a[5],
            has_wm,
            shm: None,
            shm_ok,
            min_keycode,
            per: map.keysyms_per_keycode,
            keysyms: map.keysyms,
            w: w as i32,
            h: h as i32,
            fullscreen,
            border: None,
            border_layout: None,
            scratch: Vec::new(),
            c,
        })
    }

    pub fn set_fullscreen(&mut self, on: bool) -> Res<()> {
        self.fullscreen = on;
        if self.has_wm {
            let ev = ClientMessageEvent::new(
                32,
                self.win,
                self.net_wm_state,
                [u32::from(on), self.net_fullscreen, 0, 1, 0],
            );
            self.c.send_event(
                false,
                self.root,
                EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
                ev,
            )?;
        } else {
            let (w, h) = if on {
                (self.screen_w as u32, self.screen_h as u32)
            } else {
                (WIN_W as u32, WIN_H as u32)
            };
            self.c.configure_window(
                self.win,
                &ConfigureWindowAux::new().x(0).y(0).width(w).height(h),
            )?;
        }
        self.c.flush()?;
        Ok(())
    }

    fn keysym(&self, keycode: u8) -> u32 {
        let idx = (keycode.saturating_sub(self.min_keycode)) as usize * self.per as usize;
        self.keysyms.get(idx).copied().unwrap_or(0)
    }

    /// Everything that happened since the last call. Key repeats (a release
    /// followed by a press with the same time) are dropped.
    pub fn poll(&mut self) -> Res<Vec<Ev>> {
        let mut raw = Vec::new();
        while let Some(e) = self.c.poll_for_event()? {
            raw.push(e);
        }
        let mut out = Vec::new();
        let mut i = 0;
        while i < raw.len() {
            match &raw[i] {
                Event::KeyPress(k) => out.push(Ev::Key {
                    sym: self.keysym(k.detail),
                    down: true,
                }),
                Event::KeyRelease(k) => {
                    if let Some(Event::KeyPress(n)) = raw.get(i + 1) {
                        if n.detail == k.detail && n.time == k.time {
                            i += 2;
                            continue;
                        }
                    }
                    out.push(Ev::Key {
                        sym: self.keysym(k.detail),
                        down: false,
                    });
                }
                Event::ConfigureNotify(c) if c.window == self.win => {
                    if c.width as i32 != self.w || c.height as i32 != self.h {
                        self.w = c.width as i32;
                        self.h = c.height as i32;
                        self.border_layout = None;
                    }
                }
                Event::Expose(_) => self.border_layout = None,
                Event::ClientMessage(m) if m.data.as_data32()[0] == self.wm_delete => {
                    out.push(Ev::Close)
                }
                Event::FocusIn(_) => out.push(Ev::Focus(true)),
                Event::FocusOut(_) => out.push(Ev::Focus(false)),
                Event::VisibilityNotify(v) => {
                    out.push(Ev::Visible(v.state != Visibility::FULLY_OBSCURED))
                }
                Event::UnmapNotify(_) => out.push(Ev::Visible(false)),
                Event::MapNotify(_) => out.push(Ev::Visible(true)),
                _ => {}
            }
            i += 1;
        }
        Ok(out)
    }

    fn ensure_shm(&mut self, bytes: usize) -> bool {
        if !self.shm_ok {
            return false;
        }
        if self.shm.as_ref().is_some_and(|s| s.size >= bytes) {
            return true;
        }
        self.drop_shm();
        let size = bytes.next_multiple_of(1 << 20);
        unsafe {
            let id = libc::shmget(libc::IPC_PRIVATE, size, libc::IPC_CREAT | 0o600);
            if id < 0 {
                self.shm_ok = false;
                return false;
            }
            let addr = libc::shmat(id, std::ptr::null(), 0);
            if addr as isize == -1 {
                libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut());
                self.shm_ok = false;
                return false;
            }
            let Ok(seg) = self.c.generate_id() else {
                return false;
            };
            let attached = self
                .c
                .shm_attach(seg, id as u32, false)
                .ok()
                .and_then(|ck| ck.check().ok())
                .is_some();
            libc::shmctl(id, libc::IPC_RMID, std::ptr::null_mut());
            if !attached {
                libc::shmdt(addr);
                self.shm_ok = false;
                return false;
            }
            self.shm = Some(Shm {
                seg,
                addr: addr as *mut u8,
                size,
            });
        }
        true
    }

    fn drop_shm(&mut self) {
        if let Some(s) = self.shm.take() {
            let _ = self.c.shm_detach(s.seg);
            unsafe { libc::shmdt(s.addr as *const libc::c_void) };
        }
    }

    /// Paint the border: the window background in the border colour, and
    /// the four strips around the game cleared to it.
    fn paint_border(&mut self, lay: Layout, rgb: u32) -> Res<()> {
        self.c.change_window_attributes(
            self.win,
            &ChangeWindowAttributesAux::new().background_pixel(rgb),
        )?;
        let w = self.w as u16;
        let strips = [
            (0, 0, w, lay.y as u16),
            (
                0,
                (lay.y + lay.h) as i16,
                w,
                (self.h - lay.y - lay.h).max(0) as u16,
            ),
            (0, lay.y as i16, lay.x as u16, lay.h as u16),
            (
                (lay.x + lay.w) as i16,
                lay.y as i16,
                (self.w - lay.x - lay.w).max(0) as u16,
                lay.h as u16,
            ),
        ];
        for (x, y, sw, sh) in strips {
            if sw > 0 && sh > 0 {
                self.c.clear_area(false, self.win, x, y, sw, sh)?;
            }
        }
        Ok(())
    }

    /// Scale the picture into the window and wait until the server has it.
    pub fn present(&mut self, fb: &Fb, lut: &Lut, border: u8, crt: bool) -> Res<()> {
        let lay = scale::layout(self.w, self.h);
        let rgb = lut.rgb(border);
        if self.border != Some(rgb) || self.border_layout != Some(lay) {
            self.paint_border(lay, rgb)?;
            self.border = Some(rgb);
            self.border_layout = Some(lay);
        }
        let n = (lay.w * lay.h) as usize;
        if self.ensure_shm(n * 4) {
            let s = self.shm.as_ref().unwrap();
            // SAFETY: the segment is page aligned and at least n*4 bytes.
            let dst = unsafe { std::slice::from_raw_parts_mut(s.addr as *mut u32, n) };
            scale::upscale(&fb.px, lut, lay.scale, crt, dst);
            self.c.shm_put_image(
                self.win,
                self.gc,
                lay.w as u16,
                lay.h as u16,
                0,
                0,
                lay.w as u16,
                lay.h as u16,
                lay.x as i16,
                lay.y as i16,
                self.depth,
                ImageFormat::Z_PIXMAP.into(),
                false,
                s.seg,
                0,
            )?;
        } else {
            if self.scratch.len() < n {
                self.scratch.resize(n, 0);
            }
            scale::upscale(&fb.px, lut, lay.scale, crt, &mut self.scratch[..n]);
            // SAFETY: plain u32 pixels viewed as bytes.
            let bytes =
                unsafe { std::slice::from_raw_parts(self.scratch.as_ptr() as *const u8, n * 4) };
            let max = self.c.maximum_request_bytes().saturating_sub(64).max(4096);
            let row = lay.w as usize * 4;
            let rows = (max / row).max(1);
            let mut y = 0usize;
            while y < lay.h as usize {
                let k = rows.min(lay.h as usize - y);
                self.c.put_image(
                    ImageFormat::Z_PIXMAP,
                    self.win,
                    self.gc,
                    lay.w as u16,
                    k as u16,
                    lay.x as i16,
                    (lay.y as usize + y) as i16,
                    0,
                    self.depth,
                    &bytes[y * row..(y + k) * row],
                )?;
                y += k;
            }
        }
        self.c.get_input_focus()?.reply()?;
        Ok(())
    }
}

impl Drop for Video {
    fn drop(&mut self) {
        self.drop_shm();
        let _ = self.c.flush();
    }
}

/// A 32x32 window icon: a small ship over a raster bar, in C64 colours.
fn icon() -> Vec<u32> {
    let art = crate::art::icon_rows();
    let mut v = vec![32u32, 32];
    for y in 0..32 {
        for x in 0..32 {
            let band = [
                0x352879u32,
                0x6c5eb5,
                0x70a4b2,
                0xffffff,
                0x70a4b2,
                0x6c5eb5,
                0x352879,
            ];
            let bg = if (12..19).contains(&y) {
                band[(y - 12) as usize]
            } else {
                0x000000
            };
            let c = art[y as usize]
                .as_bytes()
                .get(x as usize)
                .copied()
                .unwrap_or(b'.');
            let px = match c {
                b'.' => bg,
                d => {
                    let i = (d as char).to_digit(16).unwrap_or(0) as usize;
                    crate::palette::C64[i]
                }
            };
            v.push(0xff00_0000 | px);
        }
    }
    v
}
