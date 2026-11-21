//! Demo 1, "Ginnungagap", on its song (32 bars of 3.16 seconds):
//!   0-8   stars out of the void, "NIRUX PRESENTS", the title, raster bars
//!   8-16  a low-poly N turning over a scrolling grid in fog, pulsing on the kick
//!   16-24 plasma and a sine scroller while the lead plays
//!   24-28 a tunnel that darkens as the filter closes
//!   28-32 stars again, the greetings and the end

use super::{Context, Demo, Info};
use crate::gfx::{text_width, Frame, H, W};
use crate::songs::s01_ginnungagap::{BAR, ROW};

pub fn info() -> Info {
    Info {
        number: 1,
        key: "ginnungagap",
        title: "Ginnungagap",
        song: 1,
        seconds: 32.0 * BAR + 4.0,
        about: "Stars, a low-poly NIRUX N in fog, plasma, a sine scroller and a tunnel",
        make: |ctx| Box::new(Ginnungagap::new(ctx)),
    }
}

struct Star {
    x: f32,
    y: f32,
    z: f32,
}

struct Ginnungagap {
    stars: Vec<Star>,
    seed: u32,
    last: f32,
    scroller: String,
    host: String,
    tunnel: Vec<(f32, f32, f32)>,
}

impl Ginnungagap {
    fn new(ctx: &Context) -> Self {
        let host = ctx.host.trim().to_ascii_uppercase();
        let from = if host.is_empty() {
            String::new()
        } else {
            format!("GREETINGS FROM {host} ")
        };
        let scroller = format!(
            "          NIRUX PRESENTS GINNUNGAGAP   ...   CODE, MUSIC AND PIXELS MADE FOR NIRUX   ...   \
             {from}TO EVERY NIRUX MACHINE OUT THERE   ...   SINE WAVES, A FEW TRIANGLES AND A LOT OF DARKNESS   ...   \
             ANY KEY ENDS THE DEMO          "
        );
        let mut g = Ginnungagap {
            stars: Vec::new(),
            seed: 0x1234_5678,
            last: 0.0,
            scroller,
            host,
            tunnel: Vec::with_capacity(W * H),
        };
        for _ in 0..260 {
            let s = g.new_star(true);
            g.stars.push(s);
        }
        for y in 0..H {
            for x in 0..W {
                let dx = x as f32 - W as f32 / 2.0;
                let dy = (y as f32 - H as f32 / 2.0) * 1.2;
                let r = (dx * dx + dy * dy).sqrt().max(1.0);
                g.tunnel.push((
                    900.0 / r,
                    dy.atan2(dx) / std::f32::consts::PI * 64.0,
                    (r / 110.0).min(1.0),
                ));
            }
        }
        g
    }

    fn rand(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        self.seed as f32 / u32::MAX as f32
    }

    fn new_star(&mut self, anywhere: bool) -> Star {
        Star {
            x: self.rand() * 2.0 - 1.0,
            y: self.rand() * 2.0 - 1.0,
            z: if anywhere {
                self.rand() * 0.95 + 0.05
            } else {
                1.0
            },
        }
    }

    fn stars(&mut self, f: &mut Frame, dt: f32, speed: f32, bright: f32) {
        for i in 0..self.stars.len() {
            self.stars[i].z -= dt * speed;
            if self.stars[i].z <= 0.02 {
                self.stars[i] = self.new_star(false);
            }
            let s = &self.stars[i];
            let sx = W as f32 / 2.0 + s.x / s.z * 140.0;
            let sy = H as f32 / 2.0 + s.y / s.z * 100.0;
            let level = ((1.0 - s.z) * 255.0 * bright) as u8;
            f.lift(sx as i32, sy as i32, level);
            if s.z < 0.3 {
                f.lift(sx as i32 + 1, sy as i32, level / 2);
            }
        }
    }
}

/// How long ago the last kick was (the song's kick on rows 0 and 10 of
/// bars 8 to 23), as a flash from 1 down to 0.
fn kick(t: f32) -> f32 {
    let bar = (t / BAR).floor();
    if !(8.0..24.0).contains(&bar) {
        return 0.0;
    }
    let row = (t - bar * BAR) / ROW;
    let since = if row >= 10.0 { row - 10.0 } else { row };
    (-since * ROW * 7.0).exp()
}

fn smooth(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Fade in from `a`, hold, fade out towards `b` (in bars), 0 to 1.
fn window(b: f32, a: f32, z: f32, fade: f32) -> f32 {
    smooth(a, a + fade, b) * (1.0 - smooth(z - fade, z, b))
}

fn raster_bars(f: &mut Frame, t: f32, strength: f32) {
    for k in 0..3 {
        let centre = H as f32 / 2.0 + (t * 1.3 + k as f32 * 2.1).sin() * 62.0;
        for dy in -9..=9 {
            let y = centre as i32 + dy;
            let level =
                ((1.0 - (dy as f32).abs() / 9.5) * (120.0 + 50.0 * k as f32) * strength) as u8;
            for x in 0..W as i32 {
                f.lift(x, y, level);
            }
        }
    }
}

/// The three blocks of an N: two upright bars and the diagonal, as front
/// faces (x, y) that are extruded in depth.
const N_SHAPE: [[(f32, f32); 4]; 3] = [
    [(-0.8, -1.0), (-0.4, -1.0), (-0.4, 1.0), (-0.8, 1.0)],
    [(0.4, -1.0), (0.8, -1.0), (0.8, 1.0), (0.4, 1.0)],
    [(-0.8, 1.0), (-0.4, 1.0), (0.8, -1.0), (0.4, -1.0)],
];

fn rotate(p: [f32; 3], ay: f32, ax: f32) -> [f32; 3] {
    let (sy, cy) = ay.sin_cos();
    let (sx, cx) = ax.sin_cos();
    let x = p[0] * cy + p[2] * sy;
    let z = -p[0] * sy + p[2] * cy;
    let y = p[1] * cx - z * sx;
    let z = p[1] * sx + z * cx;
    [x, y, z]
}

/// The N turned by `ay`, `ax`, at distance `dist`, `size` on screen, its
/// faces lit from the front left and fogged with depth.
fn draw_n(f: &mut Frame, ay: f32, ax: f32, dist: f32, size: f32, cy: f32, glow: f32) {
    let depth = 0.22;
    let project = |p: [f32; 3]| -> [f32; 3] {
        let z = p[2] + dist;
        [W as f32 / 2.0 + p[0] * size / z, cy - p[1] * size / z, z]
    };
    for quad in N_SHAPE.iter() {
        let front: Vec<[f32; 3]> = quad
            .iter()
            .map(|&(x, y)| rotate([x, y, -depth], ay, ax))
            .collect();
        let back: Vec<[f32; 3]> = quad
            .iter()
            .map(|&(x, y)| rotate([x, y, depth], ay, ax))
            .collect();
        let mut faces: Vec<[[f32; 3]; 4]> = vec![
            [front[0], front[1], front[2], front[3]],
            [back[3], back[2], back[1], back[0]],
        ];
        for i in 0..4 {
            let j = (i + 1) % 4;
            faces.push([front[i], back[i], back[j], front[j]]);
        }
        for q in faces {
            let u = [q[1][0] - q[0][0], q[1][1] - q[0][1], q[1][2] - q[0][2]];
            let v = [q[3][0] - q[0][0], q[3][1] - q[0][1], q[3][2] - q[0][2]];
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-6);
            let light = [-0.4f32, 0.5, -0.77];
            let diffuse = ((n[0] * light[0] + n[1] * light[1] + n[2] * light[2]) / len).abs();
            let mid_z = (q[0][2] + q[2][2]) / 2.0 + dist;
            let fog = (1.0 - (mid_z - dist + 1.0) * 0.25).clamp(0.35, 1.0);
            let level = ((70.0 + 185.0 * diffuse) * fog * (0.8 + 0.2 * glow)).min(255.0) as u8;
            let p: Vec<[f32; 3]> = q.iter().map(|&x| project(x)).collect();
            f.triangle(p[0], p[1], p[2], level);
            f.triangle(p[0], p[2], p[3], level);
        }
    }
}

fn grid(f: &mut Frame, t: f32, glow: f32, strength: f32) {
    let horizon = H as f32 * 0.55;
    for y in (horizon as usize + 2)..H {
        let z = 22.0 / (y as f32 - horizon);
        let fog = (1.0 - z / 7.0).clamp(0.0, 1.0);
        let v = z + t * 1.6;
        let along = (v.fract() < 0.06 * (1.0 + z)) as u8 as f32;
        for x in 0..W {
            let wx = (x as f32 - W as f32 / 2.0) * z / 60.0;
            let across = (wx.rem_euclid(1.0) < 0.04 * (1.0 + z)) as u8 as f32;
            let line = along.max(across);
            if line > 0.0 {
                let level = (line * fog * (110.0 + 90.0 * glow) * strength) as u8;
                f.lift(x as i32, y as i32, level);
            }
        }
    }
}

fn plasma(f: &mut Frame, t: f32, strength: f32) {
    for y in 0..H {
        for x in 0..W {
            let (xf, yf) = (x as f32, y as f32);
            let dx = xf - 160.0 - (t * 0.7).sin() * 60.0;
            let dy = yf - 90.0 - (t * 0.9).cos() * 30.0;
            let v = (xf * 0.045 + t).sin()
                + (yf * 0.06 - t * 1.3).sin()
                + ((xf + yf) * 0.03 + t * 0.7).sin()
                + ((dx * dx + dy * dy).sqrt() * 0.07 - t * 2.0).sin();
            let level = ((v + 4.0) / 8.0 * 125.0 * strength) as u8;
            f.set(x as i32, y as i32, level);
        }
    }
}

impl Ginnungagap {
    fn scroller(&self, f: &mut Frame, t: f32, strength: f32) {
        let scale = 2;
        let chars: Vec<char> = self.scroller.chars().collect();
        let cell = 6 * scale;
        let offset = t * 55.0;
        let first = (offset / cell as f32) as usize;
        for i in 0..(W as i32 / cell + 2) as usize {
            let idx = (first + i) % chars.len();
            let x = (i as f32 * cell as f32 - offset % cell as f32) as i32;
            let y = (H as f32 / 2.0 - 7.0 + ((x as f32) * 0.035 + t * 2.6).sin() * 34.0) as i32;
            let s = chars[idx].to_string();
            f.text(x + 2, y + 2, &s, scale, (40.0 * strength) as u8);
            f.text(x, y, &s, scale, (255.0 * strength) as u8);
        }
    }

    fn tunnel(&self, f: &mut Frame, t: f32, strength: f32) {
        let du = t * 9.0;
        let dv = t * 6.0;
        for (i, &(dist, angle, shade)) in self.tunnel.iter().enumerate() {
            let u = (dist + du) as i32;
            let v = (angle + dv) as i32;
            let tex = (((u ^ v) & 31) as f32 / 31.0) * 0.7 + 0.3 * (((u >> 3) & 1) as f32);
            f.px[i] = (tex * shade * 200.0 * strength) as u8;
        }
    }
}

impl Demo for Ginnungagap {
    fn draw(&mut self, f: &mut Frame, t: f32) {
        let dt = (t - self.last).clamp(0.0, 0.1);
        self.last = t;
        let b = t / BAR;
        f.clear(0);
        let glow = kick(t);
        if b < 8.0 {
            self.stars(f, dt, 0.18 + b * 0.03, smooth(0.0, 1.5, b));
            raster_bars(f, t, smooth(6.0, 7.5, b) * (1.0 - smooth(7.6, 8.0, b)));
            let a = window(b, 0.5, 3.8, 0.6);
            if a > 0.0 {
                f.text_centered(62, "NIRUX", 4, (255.0 * a) as u8);
                f.text_centered(102, "PRESENTS", 1, (190.0 * a) as u8);
            }
            let a = window(b, 4.0, 8.0, 0.4);
            if a > 0.0 {
                let title = "GINNUNGAGAP";
                let shown = (((b - 4.0) * 16.0 / 4.0) as usize + 1).min(title.len());
                let x0 = (W as i32 - text_width(title, 2)) / 2;
                for (i, ch) in title.chars().take(shown).enumerate() {
                    let y = 82 + ((t * 3.0 + i as f32 * 0.6).sin() * 6.0) as i32;
                    f.text(x0 + i as i32 * 12, y, &ch.to_string(), 2, (255.0 * a) as u8);
                }
            }
        } else if b < 16.0 {
            let a = window(b, 8.0, 16.0, 0.5);
            self.stars(f, dt, 0.05, 0.35 * a);
            grid(f, t, glow, a);
            // The N turns to and fro, never so far that it shows its back
            // (mirrored) or its edge (a pillar); it comes closer over the part.
            let turn = 0.75 * ((b - 8.0) * 0.8).sin();
            let size = 95.0 + 12.0 * glow;
            let dist = 5.0 - 1.2 * smooth(8.0, 15.0, b);
            draw_n(f, turn, 0.18 * (t * 0.8).sin(), dist, size, 62.0, glow);
            f.dim(a);
        } else if b < 24.0 {
            let a = window(b, 16.0, 24.0, 0.4);
            plasma(f, t, a * (0.85 + 0.25 * glow));
            self.scroller(f, t - 16.0 * BAR, a);
        } else if b < 28.0 {
            let a = window(b, 24.0, 28.0, 0.3);
            let closing = 1.0 - 0.55 * smooth(24.0, 28.0, b);
            self.tunnel(f, t, a * closing);
        } else {
            let a = window(b, 28.0, 33.2, 0.8);
            self.stars(f, dt, 0.12 * (1.0 - smooth(28.0, 33.0, b)) + 0.01, 0.9 * a);
            draw_n(
                f,
                0.6 * ((b - 28.0) * 1.1).sin(),
                0.15,
                4.5,
                110.0,
                70.0,
                0.0,
            );
            let g = window(b, 28.5, 30.8, 0.4);
            if g > 0.0 && !self.host.is_empty() {
                f.text_centered(
                    140,
                    &format!("GREETINGS FROM {}", self.host),
                    1,
                    (220.0 * g) as u8,
                );
            } else if g > 0.0 {
                f.text_centered(
                    140,
                    "GREETINGS TO EVERY NIRUX MACHINE",
                    1,
                    (220.0 * g) as u8,
                );
            }
            let e = window(b, 31.0, 33.2, 0.5);
            if e > 0.0 {
                f.text_centered(140, "THANK YOU FOR WATCHING", 1, (220.0 * e) as u8);
            }
            f.dim(a);
        }
    }
}
