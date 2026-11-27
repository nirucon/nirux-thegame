//! The 320x200 picture in palette indices and everything that draws on it:
//! pixels, boxes, lines, circles, filled polygons, sprites and text.
use crate::font;
use crate::palette::CLEAR;

pub const W: i32 = 320;
pub const H: i32 = 200;

pub struct Fb {
    pub px: Vec<u8>,
}

/// A sprite: palette indices, `CLEAR` where it is transparent.
#[derive(Clone, Debug)]
pub struct Sprite {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

fn hex(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => CLEAR,
    }
}

impl Sprite {
    /// Rows of hex digits (the C64 colour of each pixel) and dots.
    pub fn parse(rows: &[&str]) -> Sprite {
        let w = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32;
        let h = rows.len() as i32;
        let mut px = vec![CLEAR; (w * h) as usize];
        for (y, r) in rows.iter().enumerate() {
            for (x, c) in r.bytes().enumerate() {
                px[y * w as usize + x] = hex(c);
            }
        }
        Sprite { w, h, px }
    }

    /// Nearest-neighbour doubled in both directions.
    pub fn doubled(&self) -> Sprite {
        let (w, h) = (self.w * 2, self.h * 2);
        let mut px = vec![CLEAR; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                px[(y * w + x) as usize] = self.px[((y / 2) * self.w + x / 2) as usize];
            }
        }
        Sprite { w, h, px }
    }
}

impl Default for Fb {
    fn default() -> Self {
        Self::new()
    }
}

impl Fb {
    pub fn new() -> Fb {
        Fb {
            px: vec![0; (W * H) as usize],
        }
    }

    pub fn clear(&mut self, c: u8) {
        self.px.fill(c);
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: u8) {
        if (x as u32) < W as u32 && (y as u32) < H as u32 {
            self.px[(y * W + x) as usize] = c;
        }
    }

    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, c: u8) {
        if !(0..H).contains(&y) {
            return;
        }
        let a = x0.max(0);
        let b = x1.min(W - 1);
        if a > b {
            return;
        }
        let row = (y * W) as usize;
        self.px[row + a as usize..=row + b as usize].fill(c);
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u8) {
        for yy in y.max(0)..(y + h).min(H) {
            self.hline(x, x + w - 1, yy, c);
        }
    }

    pub fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, c: u8) {
        self.hline(x, x + w - 1, y, c);
        self.hline(x, x + w - 1, y + h - 1, c);
        for yy in y..y + h {
            self.set(x, yy, c);
            self.set(x + w - 1, yy, c);
        }
    }

    /// Darken whatever is in the box by mapping each colour through `map`.
    pub fn remap(&mut self, x: i32, y: i32, w: i32, h: i32, map: &[u8; 16]) {
        for yy in y.max(0)..(y + h).min(H) {
            for xx in x.max(0)..(x + w).min(W) {
                let p = &mut self.px[(yy * W + xx) as usize];
                if *p < 16 {
                    *p = map[*p as usize];
                }
            }
        }
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: u8) {
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut n = 0;
        loop {
            self.set(x, y, c);
            if (x == x1 && y == y1) || n > 2000 {
                break;
            }
            n += 1;
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn disc(&mut self, cx: i32, cy: i32, r: i32, c: u8) {
        if r < 0 {
            return;
        }
        for dy in -r..=r {
            let w = (((r * r - dy * dy) as f32).sqrt() + 0.35) as i32;
            self.hline(cx - w, cx + w, cy + dy, c);
        }
    }

    pub fn circle(&mut self, cx: i32, cy: i32, r: i32, c: u8) {
        if r <= 0 {
            self.set(cx, cy, c);
            return;
        }
        let (mut x, mut y, mut d) = (r, 0, 1 - r);
        while x >= y {
            for (a, b) in [
                (x, y),
                (y, x),
                (-y, x),
                (-x, y),
                (-x, -y),
                (-y, -x),
                (y, -x),
                (x, -y),
            ] {
                self.set(cx + a, cy + b, c);
            }
            y += 1;
            if d < 0 {
                d += 2 * y + 1;
            } else {
                x -= 1;
                d += 2 * (y - x) + 1;
            }
        }
    }

    /// A filled convex (or simple) polygon, scanline by scanline.
    pub fn poly(&mut self, pts: &[(f32, f32)], c: u8) {
        if pts.len() < 3 {
            return;
        }
        let ymin = pts.iter().map(|p| p.1).fold(f32::MAX, f32::min).floor() as i32;
        let ymax = pts.iter().map(|p| p.1).fold(f32::MIN, f32::max).ceil() as i32;
        let mut xs = [0f32; 16];
        for y in ymin.max(0)..=ymax.min(H - 1) {
            let fy = y as f32 + 0.5;
            let mut n = 0;
            for i in 0..pts.len() {
                let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
                if (a.1 <= fy && b.1 > fy) || (b.1 <= fy && a.1 > fy) {
                    let t = (fy - a.1) / (b.1 - a.1);
                    if n < xs.len() {
                        xs[n] = a.0 + t * (b.0 - a.0);
                        n += 1;
                    }
                }
            }
            let xs = &mut xs[..n];
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            for pair in xs.chunks(2) {
                if pair.len() == 2 {
                    self.hline(pair[0].round() as i32, pair[1].round() as i32 - 1, y, c);
                }
            }
        }
    }

    pub fn blit(&mut self, s: &Sprite, x: i32, y: i32) {
        for sy in 0..s.h {
            let yy = y + sy;
            if !(0..H).contains(&yy) {
                continue;
            }
            let row = (sy * s.w) as usize;
            for sx in 0..s.w {
                let p = s.px[row + sx as usize];
                if p != CLEAR {
                    self.set(x + sx, yy, p);
                }
            }
        }
    }

    /// The sprite's shape in one colour (hit flashes, shadows).
    pub fn blit_solid(&mut self, s: &Sprite, x: i32, y: i32, c: u8) {
        for sy in 0..s.h {
            let row = (sy * s.w) as usize;
            for sx in 0..s.w {
                if s.px[row + sx as usize] != CLEAR {
                    self.set(x + sx, y + sy, c);
                }
            }
        }
    }

    /// Centred on (x, y).
    pub fn blit_c(&mut self, s: &Sprite, x: i32, y: i32) {
        self.blit(s, x - s.w / 2, y - s.h / 2);
    }

    pub fn char(&mut self, x: i32, y: i32, ch: char, c: u8) {
        let g = font::glyph(ch);
        for (r, bits) in g.iter().enumerate() {
            if *bits == 0 {
                continue;
            }
            for b in 0..8 {
                if bits & (0x80 >> b) != 0 {
                    self.set(x + b, y + r as i32, c);
                }
            }
        }
    }

    /// Text in the 8x8 font; returns the width drawn.
    pub fn text(&mut self, x: i32, y: i32, s: &str, c: u8) -> i32 {
        let mut cx = x;
        for ch in s.chars() {
            self.char(cx, y, ch, c);
            cx += 8;
        }
        cx - x
    }

    /// Text with a one pixel drop shadow.
    pub fn text_sh(&mut self, x: i32, y: i32, s: &str, c: u8, shadow: u8) -> i32 {
        self.text(x + 1, y + 1, s, shadow);
        self.text(x, y, s, c)
    }

    pub fn text_center(&mut self, y: i32, s: &str, c: u8) {
        let w = s.chars().count() as i32 * 8;
        self.text((W - w) / 2, y, s, c);
    }

    pub fn text_center_sh(&mut self, y: i32, s: &str, c: u8, shadow: u8) {
        let w = s.chars().count() as i32 * 8;
        self.text_sh((W - w) / 2, y, s, c, shadow);
    }

    /// Text scaled up by `k` in both directions.
    pub fn text_big(&mut self, x: i32, y: i32, s: &str, k: i32, c: u8) {
        let mut cx = x;
        for ch in s.chars() {
            let g = font::glyph(ch);
            for (r, bits) in g.iter().enumerate() {
                for b in 0..8 {
                    if bits & (0x80 >> b) != 0 {
                        self.rect(cx + b * k, y + r as i32 * k, k, k, c);
                    }
                }
            }
            cx += 8 * k;
        }
    }

    pub fn text_big_center(&mut self, y: i32, s: &str, k: i32, c: u8, shadow: Option<u8>) {
        let w = s.chars().count() as i32 * 8 * k;
        let x = (W - w) / 2;
        if let Some(sh) = shadow {
            self.text_big(x + k.max(1), y + k.max(1), s, k, sh);
        }
        self.text_big(x, y, s, k, c);
    }
}

pub fn text_w(s: &str) -> i32 {
    s.chars().count() as i32 * 8
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drawing_is_clipped_and_sprites_parse() {
        let mut fb = Fb::new();
        fb.rect(-10, -10, 400, 300, 3);
        assert!(fb.px.iter().all(|&p| p == 3));
        fb.line(-50, -50, 500, 500, 1);
        fb.disc(0, 0, 40, 2);
        fb.poly(&[(-10.0, 5.0), (400.0, 5.0), (100.0, 300.0)], 4);
        let s = Sprite::parse(&["1.2", "..f"]);
        assert_eq!((s.w, s.h), (3, 2));
        assert_eq!(s.px[1], CLEAR);
        fb.blit(&s, 318, 199);
        assert_eq!(fb.text(0, 0, "AB", 1), 16);
    }
}
