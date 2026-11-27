//! NIRUX, the hero: the guardian the Core made and woke when NULL took it.
//! No face and no voice; a pale helmet with a green chevron visor, a blue
//! suit and a red scarf. These colours and shapes are the canon (see
//! docs/LORE.md), so every NIRUX game draws the hero the same way.
//!
//! The portrait is drawn by code at any size: shaded with an ordered
//! dither the way C64 artists shaded by hand, then given a black outline so
//! it reads on raster bars and stars alike.
use crate::fb::{Fb, Sprite};
use crate::palette::*;

/// The canon colours.
pub const ARMOUR: [u8; 4] = [DGREY, GREY, LGREY, WHITE];
pub const SUIT: [u8; 4] = [BLACK, BLUE, LBLUE, CYAN];
pub const SCARF: [u8; 3] = [BROWN, RED, LRED];
pub const VISOR: u8 = LGREEN;
pub const VISOR_HALO: u8 = GREEN;

const BAYER: [[f32; 4]; 4] = [
    [0.03, 0.53, 0.16, 0.66],
    [0.78, 0.28, 0.91, 0.41],
    [0.22, 0.72, 0.09, 0.59],
    [0.97, 0.47, 0.84, 0.34],
];

/// A shade from a ramp, dithered: `v` from 0 (dark) to 1 (light).
fn shade(ramp: &[u8], v: f32, x: i32, y: i32) -> u8 {
    let n = ramp.len() as f32 - 1.0;
    let f = v.clamp(0.0, 1.0) * n;
    let i = f.floor();
    let th = BAYER[(y & 3) as usize][(x & 3) as usize];
    let k = if f - i > th { i + 1.0 } else { i };
    ramp[(k as usize).min(ramp.len() - 1)]
}

/// Light from the upper left, a little in front.
fn lit(nx: f32, ny: f32, nz: f32) -> f32 {
    let (lx, ly, lz) = (-0.48f32, -0.58f32, 0.66f32);
    (nx * lx + ny * ly + nz * lz).max(0.0)
}

/// The visor line: a wide, shallow chevron pointing down.
fn visor_v(u: f32) -> f32 {
    0.10 - 0.30 * u.abs()
}

/// The portrait, `r` pixels to the helmet's radius (16 is a small badge,
/// 30 fills a story screen). `t` animates the scarf and the visor glint;
/// `glow` from 0 to 1 lights the visor (0 is asleep, as in the prologue).
pub fn portrait(r: f32, t: u32, glow: f32) -> Sprite {
    let w = (r * 4.4) as i32;
    let h = (r * 3.0) as i32;
    // The helmet's centre in the sprite.
    let (cx, cy) = (r * 1.65, r * 1.05);
    let mut px = vec![CLEAR; (w * h) as usize];
    let tf = t as f32;
    let glint = ((t % 240) as f32 / 60.0) - 1.6;
    for y in 0..h {
        for x in 0..w {
            let u = (x as f32 + 0.5 - cx) / r;
            let v = (y as f32 + 0.5 - cy) / r;
            let mut c = CLEAR;
            // The scarf's tail: out from the neck and up into the wind,
            // forked at the end.
            {
                let s = (u - 0.3) / 2.4;
                if (0.0..=1.0).contains(&s) {
                    let ph = tf * 0.08 - s * 5.5;
                    let mid = 0.58 - s * 0.55 + ph.sin() * 0.2 * s;
                    let half = 0.2 * (1.0 - s * 0.5);
                    let d = v - mid;
                    let fork = s > 0.72 && d.abs() < half * 0.28;
                    if d.abs() < half && !fork && !(s > 0.95 && (x + y) % 2 == 0) {
                        let fold = ph.cos() * 0.5 + 0.5;
                        let l = 0.45 + fold * 0.55 - (d / half) * 0.3;
                        c = shade(&SCARF, l, x, y);
                        if (d.abs() - half).abs() < 0.04 && d < 0.0 {
                            c = LRED;
                        }
                    }
                }
            }
            // The chest: a plate with a collar and the Core's mark.
            if v > 0.5 {
                let half = 0.62 + (v - 0.5) * 0.75;
                if u.abs() < half {
                    let mut l = 0.5 - u * 0.4 - (v - 0.5) * 0.35;
                    // The collar's V and the centre seam.
                    let collar = (v - 0.62 - u.abs() * 0.7).abs() < 0.05;
                    if collar {
                        l += 0.45;
                    }
                    if u.abs() < 0.03 && v > 0.9 {
                        l -= 0.4;
                    }
                    c = shade(&SUIT, l, x, y);
                    let (du, dv) = (u.abs(), (v - 1.08).abs());
                    if du + dv < 0.12 {
                        c = match (glow > 0.4, du + dv < 0.05) {
                            (true, true) => WHITE,
                            (true, false) => VISOR,
                            (false, _) => DGREY,
                        };
                    } else if du + dv < 0.18 {
                        c = BLACK;
                    }
                }
            }
            // The shoulder guards: two plates each, a lit rim on top.
            for side in [-1.0f32, 1.0] {
                for (ou, ov, ru, rv, dim) in
                    [(1.02, 1.18, 0.46, 0.3, 0.75), (0.9, 0.88, 0.52, 0.36, 1.0)]
                {
                    let (pu, pv) = ((u - side * ou) / ru, (v - ov) / rv);
                    let q = pu * pu + pv * pv;
                    if q < 1.0 {
                        let nz = (1.0 - q).sqrt();
                        let mut l = (lit(pu, pv, nz) * 1.1 + 0.1) * dim;
                        if q > 0.62 && q < 0.8 && pv < 0.0 {
                            l = 1.0;
                        }
                        c = if q > 0.88 {
                            BLACK
                        } else {
                            shade(&SUIT, l, x, y)
                        };
                    }
                }
            }
            // The neck.
            if u.abs() < 0.3 && (0.40..0.75).contains(&v) && c == CLEAR {
                c = shade(&ARMOUR, 0.15 - u * 0.2, x, y);
            }
            // The scarf wrapped round the neck.
            {
                let mid = 0.60 + 0.14 * u * u;
                let d = v - mid;
                if u.abs() < 0.58 && d.abs() < 0.13 {
                    let l = 0.75 - u * 0.4 - d * 2.0 + ((u * 9.0).sin() * 0.12);
                    c = shade(&SCARF, l, x, y);
                }
            }
            // The helmet: a dome and a jaw guard.
            let (hu, hv) = (u / 0.66, (v + 0.04) / 0.74);
            let q = hu * hu + hv * hv;
            let jaw = (0.15..0.56).contains(&v) && u.abs() < 0.54 - (v - 0.15) * 0.75;
            if q < 1.0 || jaw {
                let (nx, ny, nz) = if q < 1.0 {
                    (hu, hv, (1.0 - q).sqrt())
                } else {
                    (u * 1.2, 0.5, 0.6)
                };
                let mut l = lit(nx, ny, nz) * 1.15;
                // A ridge over the crown.
                if u.abs() < 0.05 && v < -0.2 {
                    l += 0.3;
                } else if (0.05..0.1).contains(&u.abs()) && v < -0.2 {
                    l -= 0.3;
                }
                c = shade(&ARMOUR, l, x, y);
                // Rim light from the right.
                if q < 1.0 && q > 0.78 && nx > 0.55 {
                    c = LBLUE;
                }
                if q < 1.0 && q > 0.9 && nx < -0.2 {
                    c = BLACK;
                }
                // The jaw seam.
                if jaw && q >= 1.0 && (v - 0.33).abs() < 0.03 {
                    c = DGREY;
                }
                // The visor plate and its slit.
                let vv = visor_v(u);
                let d = v - vv;
                if u.abs() < 0.54 && d.abs() < 0.14 {
                    c = if d > 0.1 { DGREY } else { BLACK };
                    // A reflection running down the plate.
                    if ((u + d * 1.6) - 0.28).abs() < 0.03 {
                        c = BLUE;
                    }
                    if glow > 0.0 {
                        if d.abs() < 0.075 && glow > 0.4 {
                            c = VISOR_HALO;
                        }
                        if d.abs() < 0.045 {
                            c = if glow > 0.7 { VISOR } else { VISOR_HALO };
                            if glow > 0.95 && (u - glint).abs() < 0.07 {
                                c = WHITE;
                            }
                        }
                    }
                }
            }
            // The bust fades out at the bottom, dithered.
            if v > 1.45 && c != CLEAR {
                let k = (v - 1.45) / 0.45;
                if BAYER[(y & 3) as usize][(x & 3) as usize] < k {
                    c = CLEAR;
                }
            }
            px[(y * w + x) as usize] = c;
        }
    }
    outline(Sprite { w, h, px })
}

/// A black line round the sprite's shape.
fn outline(s: Sprite) -> Sprite {
    let mut o = s.clone();
    for y in 0..s.h {
        for x in 0..s.w {
            if s.px[(y * s.w + x) as usize] != CLEAR {
                continue;
            }
            let near = [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                nx >= 0
                    && ny >= 0
                    && nx < s.w
                    && ny < s.h
                    && s.px[(ny * s.w + nx) as usize] != CLEAR
            });
            if near {
                o.px[(y * s.w + x) as usize] = BLACK;
            }
        }
    }
    o
}

/// Draws the portrait with its helmet centre near (x, y).
pub fn draw(fb: &mut Fb, x: i32, y: i32, r: f32, t: u32, glow: f32) {
    let s = portrait(r, t, glow);
    fb.blit(&s, x - (r * 1.65) as i32, y - (r * 1.05) as i32);
}

/// The hero on foot, seen from above, over a body already drawn at (x, y)
/// facing (fx, fy): the pale helmet, the visor's green glint towards where
/// NIRUX looks, and the scarf's end fluttering behind.
pub fn topdown(fb: &mut Fb, x: i32, y: i32, fx: f32, fy: f32, t: u32) {
    let (px, py) = (-fy, fx);
    let flut = ((t / 5) % 3) as f32 - 1.0;
    for (k, c) in [(3.5f32, SCARF[1]), (5.0, SCARF[2]), (6.5, SCARF[1])] {
        let sx = x as f32 - fx * k + px * flut * (k - 3.0) * 0.5;
        let sy = y as f32 - 1.0 - fy * k + py * flut * (k - 3.0) * 0.5;
        fb.set(sx.round() as i32, sy.round() as i32, c);
    }
    let hx = x - fx.round() as i32 / 2;
    fb.disc(hx, y - 2, 2, ARMOUR[2]);
    fb.set(hx - 1, y - 3, WHITE);
    fb.set(
        (hx as f32 + fx * 1.6).round() as i32,
        (y as f32 - 2.0 + fy * 1.6).round() as i32,
        VISOR,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_portrait_has_its_canon_parts_at_every_size() {
        for r in [12.0, 18.0, 30.0] {
            let s = portrait(r, 0, 1.0);
            let has = |c: u8| s.px.contains(&c);
            assert!(has(VISOR), "visor at {r}");
            assert!(
                has(SCARF[1]) && has(SUIT[2]) && has(ARMOUR[2]),
                "colours at {r}"
            );
            // The outline closes the shape: no edge pixel is coloured.
            for x in 0..s.w {
                assert_eq!(s.px[x as usize], CLEAR);
            }
        }
        // Asleep, the visor is dark.
        assert!(!portrait(18.0, 0, 0.0).px.contains(&VISOR));
    }
}
