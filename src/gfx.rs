//! A small indexed-colour frame of 320 by 180 pixels, as on the home
//! computers: every pixel is a level 0 (black) to 255 (brightest) on a ramp
//! made from the theme colours, and the frame is scaled up with sharp pixels.

use crate::theme::Palette;

pub const W: usize = 320;
pub const H: usize = 180;

/// The colours of the 256 levels: black, the border colour, the accent
/// (focus), muted and the foreground, blended between.
pub struct Ramp {
    pub rgb: [u32; 256],
}

fn lerp_rgb(a: u32, b: u32, t: f32) -> u32 {
    let ch = |s: u32| {
        let x = ((a >> s) & 0xff) as f32;
        let y = ((b >> s) & 0xff) as f32;
        ((x + (y - x) * t).round().clamp(0.0, 255.0) as u32) << s
    };
    ch(16) | ch(8) | ch(0)
}

impl Ramp {
    pub fn new(p: &Palette) -> Self {
        let stops = [
            (0usize, 0x000000),
            (70, p.border),
            (150, p.focus),
            (205, p.muted),
            (255, p.foreground),
        ];
        let mut rgb = [0u32; 256];
        for w in stops.windows(2) {
            let (i0, c0) = w[0];
            let (i1, c1) = w[1];
            for (i, slot) in rgb.iter_mut().enumerate().take(i1 + 1).skip(i0) {
                *slot = lerp_rgb(c0, c1, (i - i0) as f32 / (i1 - i0) as f32);
            }
        }
        Ramp { rgb }
    }
}

pub struct Frame {
    pub px: Vec<u8>,
    pub z: Vec<f32>,
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            px: vec![0; W * H],
            z: vec![f32::INFINITY; W * H],
        }
    }

    pub fn clear(&mut self, level: u8) {
        self.px.fill(level);
        self.z.fill(f32::INFINITY);
    }

    /// Keep the brighter of the pixel and `level`.
    pub fn lift(&mut self, x: i32, y: i32, level: u8) {
        if x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H {
            let p = &mut self.px[y as usize * W + x as usize];
            *p = (*p).max(level);
        }
    }

    pub fn set(&mut self, x: i32, y: i32, level: u8) {
        if x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H {
            self.px[y as usize * W + x as usize] = level;
        }
    }

    /// Scale every level by `k` (0 to 1): fades.
    pub fn dim(&mut self, k: f32) {
        let k = k.clamp(0.0, 1.0);
        if k < 1.0 {
            for p in &mut self.px {
                *p = (*p as f32 * k) as u8;
            }
        }
    }

    /// 5 by 7 text, `scale` times larger; `None` returns only the width.
    pub fn text(&mut self, x: i32, y: i32, text: &str, scale: i32, level: u8) -> i32 {
        let mut cx = x;
        for ch in text.chars() {
            let rows = glyph(ch);
            for (r, bits) in rows.iter().enumerate() {
                for c in 0..5 {
                    if bits & (0x10 >> c) != 0 {
                        for dy in 0..scale {
                            for dx in 0..scale {
                                self.lift(cx + c * scale + dx, y + r as i32 * scale + dy, level);
                            }
                        }
                    }
                }
            }
            cx += 6 * scale;
        }
        cx - x
    }

    pub fn text_centered(&mut self, y: i32, text: &str, scale: i32, level: u8) {
        let w = text_width(text, scale);
        self.text((W as i32 - w) / 2, y, text, scale, level);
    }

    /// A filled triangle with depth test; points are screen x, y and depth.
    pub fn triangle(&mut self, a: [f32; 3], b: [f32; 3], c: [f32; 3], level: u8) {
        let area = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        if area.abs() < 1e-6 {
            return;
        }
        let min_x = a[0].min(b[0]).min(c[0]).floor().max(0.0) as i32;
        let max_x = a[0].max(b[0]).max(c[0]).ceil().min(W as f32 - 1.0) as i32;
        let min_y = a[1].min(b[1]).min(c[1]).floor().max(0.0) as i32;
        let max_y = a[1].max(b[1]).max(c[1]).ceil().min(H as f32 - 1.0) as i32;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let w0 = ((b[0] - px) * (c[1] - py) - (b[1] - py) * (c[0] - px)) / area;
                let w1 = ((c[0] - px) * (a[1] - py) - (c[1] - py) * (a[0] - px)) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }
                let z = w0 * a[2] + w1 * b[2] + w2 * c[2];
                let i = y as usize * W + x as usize;
                if z < self.z[i] {
                    self.z[i] = z;
                    self.px[i] = level;
                }
            }
        }
    }

    /// The frame as 32-bit BGRX pixels (X11 ZPixmap, little endian).
    pub fn to_bgrx(&self, ramp: &Ramp, out: &mut Vec<u8>) {
        out.clear();
        out.reserve(W * H * 4);
        for &p in &self.px {
            out.extend_from_slice(&ramp.rgb[p as usize].to_le_bytes());
        }
    }
}

pub fn text_width(text: &str, scale: i32) -> i32 {
    (text.chars().count() as i32 * 6 - 1).max(0) * scale
}

/// The 5 by 7 font: capitals, digits and a little punctuation; anything
/// else is a space.
pub fn glyph(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'B' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        'C' => [0x0e, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0e],
        'D' => [0x1e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1e],
        'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        'F' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x10],
        'G' => [0x0e, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0f],
        'H' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'I' => [0x0e, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0e],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0c],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        'M' => [0x11, 0x1b, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x11, 0x19, 0x15, 0x13, 0x11, 0x11],
        'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        'Q' => [0x0e, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0d],
        'R' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        'S' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0a],
        'X' => [0x11, 0x11, 0x0a, 0x04, 0x0a, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0a, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1f],
        '0' => [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
        '1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        '2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        '3' => [0x1f, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0e],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        '5' => [0x1f, 0x10, 0x1e, 0x01, 0x01, 0x11, 0x0e],
        '6' => [0x06, 0x08, 0x10, 0x1e, 0x11, 0x11, 0x0e],
        '7' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
        '9' => [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x02, 0x0c],
        '.' => [0, 0, 0, 0, 0, 0x0c, 0x0c],
        ',' => [0, 0, 0, 0, 0x0c, 0x04, 0x08],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0, 0x04],
        '?' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0, 0x04],
        '-' => [0, 0, 0, 0x1f, 0, 0, 0],
        '\'' => [0x04, 0x04, 0x08, 0, 0, 0, 0],
        ':' => [0, 0x0c, 0x0c, 0, 0x0c, 0x0c, 0],
        '/' => [0x01, 0x02, 0x02, 0x04, 0x08, 0x08, 0x10],
        _ => [0; 7],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ramp_runs_from_black_to_the_foreground() {
        let p = Palette::default();
        let r = Ramp::new(&p);
        assert_eq!(r.rgb[0], 0);
        assert_eq!(r.rgb[70], p.border);
        assert_eq!(r.rgb[150], p.focus);
        assert_eq!(r.rgb[255], p.foreground);
    }

    #[test]
    fn text_and_triangles_draw() {
        let mut f = Frame::new();
        assert_eq!(f.text(0, 0, "NI", 1, 200), 12);
        assert_eq!(f.px[0], 200, "the top left of N");
        assert_eq!(text_width("NIRUX", 2), 58);
        f.clear(0);
        f.triangle([10.0, 10.0, 1.0], [50.0, 10.0, 1.0], [10.0, 50.0, 1.0], 99);
        assert_eq!(f.px[20 * W + 20], 99);
        f.triangle([10.0, 10.0, 2.0], [50.0, 10.0, 2.0], [10.0, 50.0, 2.0], 7);
        assert_eq!(f.px[20 * W + 20], 99, "the nearer triangle stays");
        let mut out = Vec::new();
        f.to_bgrx(&Ramp::new(&Palette::default()), &mut out);
        assert_eq!(out.len(), W * H * 4);
    }
}
