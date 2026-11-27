//! The 16 colours of the Commodore 64 (the "Pepto" PAL measurements), plus a
//! few extra entries the menus fill with the NIRUX theme colours. The game
//! draws palette indices into an 8-bit framebuffer; the lookup tables turn
//! them into pixels only when the picture is scaled to the window.

pub const BLACK: u8 = 0;
pub const WHITE: u8 = 1;
pub const RED: u8 = 2;
pub const CYAN: u8 = 3;
pub const PURPLE: u8 = 4;
pub const GREEN: u8 = 5;
pub const BLUE: u8 = 6;
pub const YELLOW: u8 = 7;
pub const ORANGE: u8 = 8;
pub const BROWN: u8 = 9;
pub const LRED: u8 = 10;
pub const DGREY: u8 = 11;
pub const GREY: u8 = 12;
pub const LGREEN: u8 = 13;
pub const LBLUE: u8 = 14;
pub const LGREY: u8 = 15;

/// Theme colours for menu text (filled from the NIRUX theme).
pub const THEME_FG: u8 = 16;
pub const THEME_ACCENT: u8 = 17;
pub const THEME_MUTED: u8 = 18;

/// A value that is never drawn (transparent sprite pixels).
pub const CLEAR: u8 = 255;

pub const C64: [u32; 16] = [
    0x000000, 0xffffff, 0x68372b, 0x70a4b2, 0x6f3d86, 0x588d43, 0x352879, 0xb8c76f, 0x6f4f25,
    0x433900, 0x9a6759, 0x444444, 0x6c6c6c, 0x9ad284, 0x6c5eb5, 0x959595,
];

/// Brightness ramps used for raster bars, fire and fades (dark to light).
pub const RAMP_BLUE: [u8; 7] = [BLUE, LBLUE, CYAN, WHITE, CYAN, LBLUE, BLUE];
pub const RAMP_FIRE: [u8; 7] = [BROWN, RED, ORANGE, LRED, YELLOW, WHITE, YELLOW];
pub const RAMP_GREY: [u8; 7] = [DGREY, GREY, LGREY, WHITE, LGREY, GREY, DGREY];
pub const RAMP_GREEN: [u8; 7] = [BROWN, GREEN, LGREEN, WHITE, LGREEN, GREEN, BROWN];
pub const RAMP_PURPLE: [u8; 7] = [BLUE, PURPLE, LRED, LGREY, LRED, PURPLE, BLUE];

/// Explosion colours from hot to cold.
pub const HEAT: [u8; 8] = [WHITE, YELLOW, LRED, ORANGE, RED, BROWN, DGREY, BLACK];

/// The full 256-entry lookup table (0x00RRGGBB) and a darker copy for the
/// CRT scanlines.
pub struct Lut {
    pub color: [u32; 256],
    pub dark: [u32; 256],
}

fn scale_rgb(c: u32, f: f32) -> u32 {
    let ch = |s: u32| (((c >> s) & 0xff) as f32 * f).round().min(255.0) as u32;
    (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

impl Lut {
    pub fn new(theme: [u32; 3]) -> Lut {
        let mut color = [0u32; 256];
        color[..16].copy_from_slice(&C64);
        color[16] = theme[0];
        color[17] = theme[1];
        color[18] = theme[2];
        let mut dark = [0u32; 256];
        for i in 0..256 {
            dark[i] = scale_rgb(color[i], 0.52);
        }
        Lut { color, dark }
    }

    pub fn rgb(&self, idx: u8) -> u32 {
        self.color[idx as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lut_and_nearest() {
        let l = Lut::new([0x123456, 0xffffff, 0]);
        assert_eq!(l.rgb(WHITE), 0xffffff);
        assert_eq!(l.rgb(THEME_FG), 0x123456);
        assert!(l.dark[WHITE as usize] < 0x909090);
    }
}
