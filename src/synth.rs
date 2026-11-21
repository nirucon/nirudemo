//! A small synthesizer in the spirit of the C64 SID: pulse with moving pulse
//! width, triangle, saw and pitched noise, an ADSR envelope, a resonant
//! filter, arpeggios, vibrato, slides and pitch drops. A song is a list of
//! notes; every note is rendered on its own and mixed, then a dark delay and
//! a reverb are added. The output is deterministic: the same song always
//! renders to the same samples.

use std::f32::consts::PI;
use std::sync::Arc;

/// A cutoff multiplier over the song (absolute seconds).
pub type Auto = Arc<dyn Fn(f32) -> f32 + Send + Sync>;

pub fn unity() -> Auto {
    Arc::new(|_| 1.0)
}

pub const SR: u32 = 48_000;
const SRF: f32 = SR as f32;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Wave {
    Pulse,
    Tri,
    Saw,
    Noise,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Filter {
    Off,
    Low,
    Band,
    High,
}

/// How an instrument sounds. Times are seconds, `cutoff` is Hz, `pan` is
/// -1 (left) to 1 (right), `send` is the amount sent to delay and reverb.
#[derive(Clone)]
pub struct Inst {
    pub wave: Wave,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub pw: f32,
    pub pwm_depth: f32,
    pub pwm_rate: f32,
    pub filter: Filter,
    pub cutoff: f32,
    pub cutoff_env: f32,
    pub res: f32,
    pub vib_delay: f32,
    pub vib_depth: f32,
    pub vib_rate: f32,
    pub arp_rate: f32,
    pub drop: f32,
    pub drop_time: f32,
    pub vol: f32,
    pub pan: f32,
    pub send: f32,
    /// Cutoff multiplier over the song (absolute seconds), for long sweeps.
    pub auto: Auto,
}

impl Default for Inst {
    fn default() -> Self {
        Inst {
            wave: Wave::Pulse,
            attack: 0.005,
            decay: 0.2,
            sustain: 0.6,
            release: 0.2,
            pw: 0.5,
            pwm_depth: 0.0,
            pwm_rate: 0.0,
            filter: Filter::Off,
            cutoff: 20_000.0,
            cutoff_env: 0.0,
            res: 0.0,
            vib_delay: 0.0,
            vib_depth: 0.0,
            vib_rate: 0.0,
            arp_rate: 50.0,
            drop: 0.0,
            drop_time: 0.05,
            vol: 0.2,
            pan: 0.0,
            send: 0.2,
            auto: unity(),
        }
    }
}

/// One note: start and gate length in seconds, MIDI pitch (fractions allowed).
#[derive(Clone, Debug)]
pub struct Note {
    pub t: f32,
    pub dur: f32,
    pub midi: f32,
    pub inst: usize,
    pub vel: f32,
    /// Semitone offsets stepped through at the instrument's `arp_rate`.
    pub arp: Vec<f32>,
    pub slide_from: Option<f32>,
    pub slide_time: f32,
}

impl Note {
    pub fn new(t: f32, dur: f32, midi: f32, inst: usize, vel: f32) -> Self {
        Note {
            t,
            dur,
            midi,
            inst,
            vel,
            arp: Vec::new(),
            slide_from: None,
            slide_time: 0.0,
        }
    }
}

fn polyblep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let x = t / dt;
        x + x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;
        x * x + x + x + 1.0
    } else {
        0.0
    }
}

fn env_gate(inst: &Inst, tt: f32) -> f32 {
    let a = inst.attack.max(0.001);
    if tt < a {
        tt / a
    } else if tt < a + inst.decay {
        1.0 - (1.0 - inst.sustain) * (tt - a) / inst.decay.max(0.001)
    } else {
        inst.sustain
    }
}

fn render_note(inst: &Inst, note: &Note, dry: &mut [[f32; 2]], wet: &mut [f32]) {
    let start = (note.t * SRF) as usize;
    let total = ((note.dur + inst.release) * SRF) as usize;
    let level_at_release = env_gate(inst, note.dur);
    let angle = (inst.pan.clamp(-1.0, 1.0) + 1.0) * PI / 4.0;
    let (gl, gr) = (angle.cos(), angle.sin());
    let mut phase = 0.0f32;
    let mut noise = 0.0f32;
    let mut rng: u32 = 0x9E37_79B9 ^ (start as u32).wrapping_mul(2_654_435_761) | 1;
    let (mut ic1, mut ic2) = (0.0f32, 0.0f32);
    let k = 2.0 - 2.0 * inst.res.clamp(0.0, 0.97);
    for i in 0..total {
        let idx = start + i;
        if idx >= dry.len() {
            break;
        }
        let tt = i as f32 / SRF;
        let tabs = note.t + tt;
        let e = if tt < note.dur {
            env_gate(inst, tt)
        } else {
            level_at_release * (1.0 - (tt - note.dur) / inst.release.max(0.001)).max(0.0)
        };
        let mut m = note.midi;
        if let Some(from) = note.slide_from {
            if tt < note.slide_time {
                m = from + (note.midi - from) * (tt / note.slide_time);
            }
        }
        if !note.arp.is_empty() {
            m += note.arp[(tt * inst.arp_rate) as usize % note.arp.len()];
        }
        if inst.vib_depth > 0.0 && tt > inst.vib_delay {
            let ramp = ((tt - inst.vib_delay) / 0.4).min(1.0);
            m += inst.vib_depth * ramp * (2.0 * PI * inst.vib_rate * tt).sin();
        }
        if inst.drop > 0.0 {
            m -= inst.drop * (1.0 - (-tt / inst.drop_time).exp());
        }
        let f = 440.0 * 2f32.powf((m - 69.0) / 12.0);
        let dt = (f / SRF).min(0.49);
        phase += dt;
        let wrapped = phase >= 1.0;
        if wrapped {
            phase -= 1.0;
        }
        let s = match inst.wave {
            Wave::Pulse => {
                let pw = (inst.pw + inst.pwm_depth * (2.0 * PI * inst.pwm_rate * tabs).sin())
                    .clamp(0.05, 0.95);
                let mut v = if phase < pw { 1.0 } else { -1.0 };
                v += polyblep(phase, dt);
                v -= polyblep((phase - pw).rem_euclid(1.0), dt);
                v
            }
            Wave::Saw => 2.0 * phase - 1.0 - polyblep(phase, dt),
            Wave::Tri => 4.0 * (phase - 0.5).abs() - 1.0,
            Wave::Noise => {
                if wrapped {
                    rng ^= rng << 13;
                    rng ^= rng >> 17;
                    rng ^= rng << 5;
                    noise = rng as f32 / u32::MAX as f32 * 2.0 - 1.0;
                }
                noise
            }
        };
        let s = if inst.filter == Filter::Off {
            s
        } else {
            let fc =
                (inst.cutoff * (inst.auto)(tabs) + inst.cutoff_env * e).clamp(20.0, 0.45 * SRF);
            let g = (PI * fc / SRF).tan();
            let a1 = 1.0 / (1.0 + g * (g + k));
            let a2 = g * a1;
            let a3 = g * a2;
            let v3 = s - ic2;
            let v1 = a1 * ic1 + a2 * v3;
            let v2 = ic2 + a2 * ic1 + a3 * v3;
            ic1 = 2.0 * v1 - ic1;
            ic2 = 2.0 * v2 - ic2;
            match inst.filter {
                Filter::Low => v2,
                Filter::Band => v1,
                _ => s - k * v1 - v2,
            }
        };
        let out = s * e * inst.vol * note.vel;
        dry[idx][0] += out * gl;
        dry[idx][1] += out * gr;
        wet[idx] += out * inst.send;
    }
}

struct Comb {
    buf: Vec<f32>,
    pos: usize,
    store: f32,
}

impl Comb {
    fn new(len: usize) -> Self {
        Comb {
            buf: vec![0.0; len],
            pos: 0,
            store: 0.0,
        }
    }
    fn tick(&mut self, x: f32, fb: f32, damp: f32) -> f32 {
        let y = self.buf[self.pos];
        self.store = y * (1.0 - damp) + self.store * damp;
        self.buf[self.pos] = x + self.store * fb;
        self.pos = (self.pos + 1) % self.buf.len();
        y
    }
}

struct Allpass {
    buf: Vec<f32>,
    pos: usize,
}

impl Allpass {
    fn new(len: usize) -> Self {
        Allpass {
            buf: vec![0.0; len],
            pos: 0,
        }
    }
    fn tick(&mut self, x: f32) -> f32 {
        let b = self.buf[self.pos];
        let y = b - x;
        self.buf[self.pos] = x + b * 0.5;
        self.pos = (self.pos + 1) % self.buf.len();
        y
    }
}

/// Ping-pong delay with a dark (low-passed) feedback, and a small reverb.
fn effects(wet: &[f32], delay_s: f32) -> Vec<[f32; 2]> {
    let n = wet.len();
    let mut out = vec![[0.0f32; 2]; n];
    let dlen = (delay_s * SRF) as usize;
    let (mut dl, mut dr) = (vec![0.0f32; dlen], vec![0.0f32; dlen]);
    let (mut lpl, mut lpr) = (0.0f32, 0.0f32);
    let scale = SRF / 44_100.0;
    let lens = [1116, 1188, 1277, 1356, 1422, 1491];
    let mut combs: [Vec<Comb>; 2] = [
        lens.iter()
            .map(|&l| Comb::new((l as f32 * scale) as usize))
            .collect(),
        lens.iter()
            .map(|&l| Comb::new(((l + 23) as f32 * scale) as usize))
            .collect(),
    ];
    let mut aps: [Vec<Allpass>; 2] = [
        [556, 441, 341]
            .iter()
            .map(|&l| Allpass::new((l as f32 * scale) as usize))
            .collect(),
        [579, 464, 364]
            .iter()
            .map(|&l| Allpass::new((l as f32 * scale) as usize))
            .collect(),
    ];
    // Lows never go to the effects: a sustained bass note would make the
    // combs ring and drown everything else.
    let (mut hp_x, mut hp_y) = (0.0f32, 0.0f32);
    let hp_a = 1.0 / (1.0 + 2.0 * PI * 220.0 / SRF);
    for i in 0..n {
        hp_y = hp_a * (hp_y + wet[i] - hp_x);
        hp_x = wet[i];
        let x = hp_y;
        let p = i % dlen;
        let (yl, yr) = (dl[p], dr[p]);
        lpl += (yr - lpl) * 0.3;
        lpr += (yl - lpr) * 0.3;
        dl[p] = x + lpl * 0.42;
        dr[p] = lpr * 0.42;
        for ch in 0..2 {
            let mut r = 0.0;
            for c in combs[ch].iter_mut() {
                r += c.tick(x * 0.12, 0.85, 0.45);
            }
            for a in aps[ch].iter_mut() {
                r = a.tick(r);
            }
            out[i][ch] = r * 0.22 + if ch == 0 { yl } else { yr } * 0.4;
        }
    }
    out
}

/// Render the notes to stereo samples of `length` seconds.
pub fn render(insts: &[Inst], notes: &[Note], length: f32, delay_s: f32) -> Vec<[f32; 2]> {
    let n = (length * SRF) as usize;
    let mut dry = vec![[0.0f32; 2]; n];
    let mut wet = vec![0.0f32; n];
    for note in notes {
        render_note(&insts[note.inst], note, &mut dry, &mut wet);
    }
    let fx = effects(&wet, delay_s);
    for (d, f) in dry.iter_mut().zip(fx.iter()) {
        for ch in 0..2 {
            d[ch] = (d[ch] + f[ch]).tanh();
        }
    }
    dry
}

/// 16-bit stereo PCM WAV.
pub fn wav_bytes(samples: &[[f32; 2]]) -> Vec<u8> {
    let data_len = samples.len() * 4;
    let mut b = Vec::with_capacity(44 + data_len);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len as u32).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&SR.to_le_bytes());
    b.extend_from_slice(&(SR * 4).to_le_bytes());
    b.extend_from_slice(&4u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(data_len as u32).to_le_bytes());
    for s in samples {
        for ch in s {
            b.extend_from_slice(&((ch.clamp(-1.0, 1.0) * 32_767.0) as i16).to_le_bytes());
        }
    }
    b
}
