//! Integer upscaling of the 320x200 picture with optional CRT scanlines.
//! The game area is scaled by the largest whole factor that fits the
//! window; what is left over is the C64 border, which the X server paints
//! as the window background, so it costs nothing per frame.
use crate::fb::{H, W};
use crate::palette::Lut;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub scale: i32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Where the game goes in a window of `ww` x `wh` pixels.
pub fn layout(ww: i32, wh: i32) -> Layout {
    let scale = (ww / W).min(wh / H).max(1);
    let (w, h) = (W * scale, H * scale);
    Layout {
        scale,
        x: ((ww - w) / 2).max(0),
        y: ((wh - h) / 2).max(0),
        w,
        h,
    }
}

/// How many of the `s` output rows of each game row are dimmed.
pub fn dark_rows(s: i32) -> i32 {
    match s {
        0 | 1 => 0,
        2..=3 => 1,
        _ => (s / 3).max(1),
    }
}

/// Scale `src` (palette indices) into `dst` (0x00RRGGBB, `w*s` per row).
#[cfg(test)]
pub fn upscale(src: &[u8], lut: &Lut, s: i32, crt: bool, dst: &mut [u32]) {
    upscale_rows(src, lut, s, crt, dst, 0, H as usize);
}

/// The same for game rows `r0..r1` only (the rest of `dst` is untouched).
pub fn upscale_rows(
    src: &[u8],
    lut: &Lut,
    s: i32,
    crt: bool,
    dst: &mut [u32],
    r0: usize,
    r1: usize,
) {
    let s = s.max(1) as usize;
    let ow = W as usize * s;
    let dark = if crt { dark_rows(s as i32) as usize } else { 0 };
    for y in r0..r1.min(H as usize) {
        let srow = &src[y * W as usize..(y + 1) * W as usize];
        let base = y * s * ow;
        {
            let out = &mut dst[base..base + ow];
            if s == 1 {
                for (o, &p) in out.iter_mut().zip(srow) {
                    *o = lut.color[p as usize];
                }
            } else {
                for (chunk, &p) in out.chunks_exact_mut(s).zip(srow) {
                    chunk.fill(lut.color[p as usize]);
                }
            }
        }
        let bright = s - dark;
        for r in 1..bright {
            let (a, b) = dst.split_at_mut(base + r * ow);
            b[..ow].copy_from_slice(&a[base..base + ow]);
        }
        if dark > 0 {
            let first = base + bright * ow;
            {
                let out = &mut dst[first..first + ow];
                for (chunk, &p) in out.chunks_exact_mut(s).zip(srow) {
                    chunk.fill(lut.dark[p as usize]);
                }
            }
            for r in 1..dark {
                let (a, b) = dst.split_at_mut(first + r * ow);
                b[..ow].copy_from_slice(&a[first..first + ow]);
            }
        }
    }
}

/// The rows that differ between two pictures: (first, one past the last).
pub fn changed_rows(a: &[u8], b: &[u8]) -> (usize, usize) {
    let w = W as usize;
    let h = H as usize;
    let same = |y: usize| a[y * w..(y + 1) * w] == b[y * w..(y + 1) * w];
    let Some(first) = (0..h).find(|&y| !same(y)) else {
        return (0, 0);
    };
    let last = (first..h).rev().find(|&y| !same(y)).unwrap_or(first);
    (first, last + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layouts_fit_common_screens() {
        assert_eq!(layout(1920, 1080).scale, 5);
        assert_eq!(layout(1366, 768).scale, 3);
        assert_eq!(layout(2560, 1600).scale, 8);
        assert_eq!(layout(3840, 2160).scale, 10);
        assert_eq!(layout(200, 100).scale, 1);
        let l = layout(1920, 1080);
        assert_eq!((l.x, l.y, l.w, l.h), (160, 40, 1600, 1000));
    }

    #[test]
    fn changed_rows_finds_the_band() {
        let a = vec![0u8; (W * H) as usize];
        let mut b = a.clone();
        assert_eq!(changed_rows(&a, &b), (0, 0));
        b[(W * 10 + 5) as usize] = 1;
        b[(W * 40) as usize] = 1;
        assert_eq!(changed_rows(&a, &b), (10, 41));
    }

    #[test]
    fn upscale_copies_and_dims_rows() {
        let lut = Lut::new([0, 0, 0]);
        let mut src = vec![0u8; (W * H) as usize];
        src[0] = 1;
        let s = 3;
        let mut dst = vec![0u32; (W * H * s * s) as usize];
        upscale(&src, &lut, s, true, &mut dst);
        let ow = (W * s) as usize;
        assert_eq!(dst[0], 0xffffff);
        assert_eq!(dst[2], 0xffffff);
        assert_eq!(dst[3], 0);
        assert_eq!(dst[ow], 0xffffff);
        assert_eq!(dst[2 * ow], lut.dark[1]);
        upscale(&src, &lut, s, false, &mut dst);
        assert_eq!(dst[2 * ow], 0xffffff);
    }
}

#[cfg(test)]
mod bench {
    use super::*;
    #[test]
    #[ignore]
    fn upscale_speed() {
        let lut = Lut::new([0, 0, 0]);
        let src: Vec<u8> = (0..(W * H) as usize).map(|i| (i % 16) as u8).collect();
        for s in [3, 5, 8] {
            let mut dst = vec![0u32; (W * H * s * s) as usize];
            let t = std::time::Instant::now();
            for _ in 0..300 {
                upscale(&src, &lut, s, true, &mut dst);
            }
            println!(
                "scale {s}: {:.3} ms per frame",
                t.elapsed().as_secs_f64() * 1000.0 / 300.0
            );
        }
    }
}
