//! The NIRUX music: eleven original chiptunes in minor keys, between dark
//! ambient, the C64 sound of the 1980s and calm, Satie-like pieces. Each is
//! installed as `nirux-N-key.ogg` and plays in the lock and the screensaver;
//! a demo may use one as its soundtrack.
//!
//! A song is written with [`Score`]: notes in a small text notation (see
//! [`Score::line`]), chords, arpeggios, bass and drum patterns, and an echo
//! voice, on a grid of sixteenth-note rows.

use crate::synth::{self, Auto, Filter, Inst, Note, Wave};
use std::sync::Arc;

pub mod s01_ginnungagap;
mod s02_yggdrasil;
mod s03_hel;
mod s04_fimbulvetr;
mod s05_mimir;
mod s06_surtr;
mod s07_norns;
mod s08_nidhogg;
mod s09_vanaheim;
mod s10_ragnarok;
mod s11_gimle;

pub struct Song {
    pub number: u32,
    pub key: &'static str,
    pub title: &'static str,
    /// One line about the piece.
    pub about: &'static str,
    pub insts: Vec<Inst>,
    pub notes: Vec<Note>,
    /// Seconds, including the reverb tail.
    pub length: f32,
    /// Echo delay of the effects, seconds.
    pub delay: f32,
}

impl Song {
    pub fn file_name(&self) -> String {
        format!("nirux-{}-{}.ogg", self.number, self.key)
    }

    pub fn render(&self) -> Vec<[f32; 2]> {
        synth::render(&self.insts, &self.notes, self.length, self.delay)
    }
}

pub fn all() -> Vec<Song> {
    vec![
        s01_ginnungagap::song(),
        s02_yggdrasil::song(),
        s03_hel::song(),
        s04_fimbulvetr::song(),
        s05_mimir::song(),
        s06_surtr::song(),
        s07_norns::song(),
        s08_nidhogg::song(),
        s09_vanaheim::song(),
        s10_ragnarok::song(),
        s11_gimle::song(),
    ]
}

/// A song by number or key.
pub fn find(which: &str) -> Option<Song> {
    let which = which.trim().to_ascii_lowercase();
    all()
        .into_iter()
        .find(|s| s.number.to_string() == which || s.key == which)
}

// ---- the notation --------------------------------------------------------------

/// MIDI number of a note name such as `d5`, `f#3`, `bb2` (octave 4 holds
/// middle C, 60).
pub fn pitch(name: &str) -> Option<f32> {
    let b = name.as_bytes();
    let base = match b.first()?.to_ascii_lowercase() {
        b'c' => 0,
        b'd' => 2,
        b'e' => 4,
        b'f' => 5,
        b'g' => 7,
        b'a' => 9,
        b'b' => 11,
        _ => return None,
    };
    let mut i = 1;
    let mut acc = 0i32;
    while i < b.len() && (b[i] == b'#' || b[i] == b'b') {
        acc += if b[i] == b'#' { 1 } else { -1 };
        i += 1;
    }
    let octave: i32 = name.get(i..)?.parse().ok()?;
    Some(((octave + 1) * 12 + base + acc) as f32)
}

/// Several note names separated by spaces.
pub fn pitches(names: &str) -> Vec<f32> {
    names
        .split_whitespace()
        .map(|n| pitch(n).unwrap_or_else(|| panic!("bad note {n:?}")))
        .collect()
}

/// The timing grid of a song and the notes written so far.
pub struct Score {
    pub row: f32,
    pub rows_per_bar: usize,
    pub notes: Vec<Note>,
}

impl Score {
    /// `bpm` quarter notes a minute, `beats` quarter notes a bar.
    pub fn new(bpm: f32, beats: usize) -> Self {
        Score {
            row: 60.0 / bpm / 4.0,
            rows_per_bar: beats * 4,
            notes: Vec::new(),
        }
    }

    pub fn bar(&self) -> f32 {
        self.row * self.rows_per_bar as f32
    }

    pub fn at(&self, bar: f32, row: f32) -> f32 {
        bar * self.bar() + row * self.row
    }

    pub fn note(
        &mut self,
        inst: usize,
        bar: f32,
        row: f32,
        midi: f32,
        rows: f32,
        vel: f32,
    ) -> &mut Note {
        let t = self.at(bar, row);
        self.notes
            .push(Note::new(t, rows * self.row, midi, inst, vel));
        self.notes.last_mut().unwrap()
    }

    /// A melodic line from `bar`: tokens `NOTE:ROWS`, `r:ROWS` for a rest,
    /// `~NOTE:ROWS` slides from the previous note, `NOTE!:ROWS` is accented,
    /// `NOTE?:ROWS` is soft, `|` is ignored (for bar lines). `gate` is the
    /// part of each note that sounds. Returns the notes' indices.
    pub fn line(&mut self, inst: usize, bar: f32, text: &str, gate: f32) -> Vec<usize> {
        let mut row = 0.0;
        let mut previous: Option<f32> = None;
        let mut out = Vec::new();
        for token in text.split_whitespace() {
            if token == "|" {
                continue;
            }
            let (name, rows) = token
                .split_once(':')
                .unwrap_or_else(|| panic!("bad token {token:?}"));
            let rows: f32 = rows
                .parse()
                .unwrap_or_else(|_| panic!("bad length in {token:?}"));
            if name == "r" {
                row += rows;
                continue;
            }
            let slide = name.starts_with('~');
            let name = name.trim_start_matches('~');
            let (name, vel) = if let Some(n) = name.strip_suffix('!') {
                (n, 1.25)
            } else if let Some(n) = name.strip_suffix('?') {
                (n, 0.6)
            } else {
                (name, 1.0)
            };
            let midi = pitch(name).unwrap_or_else(|| panic!("bad note in {token:?}"));
            let row_s = self.row;
            let n = self.note(inst, bar, row, midi, rows * gate, vel);
            if slide {
                n.slide_from = previous;
                n.slide_time = row_s * 1.5;
            }
            out.push(self.notes.len() - 1);
            previous = Some(midi);
            row += rows;
        }
        out
    }

    /// Copies of the notes `which` on instrument `inst`, `rows` later.
    pub fn echo(&mut self, which: &[usize], inst: usize, rows: f32, vel: f32) {
        let shift = rows * self.row;
        let copies: Vec<Note> = which
            .iter()
            .map(|&i| {
                let mut n = self.notes[i].clone();
                n.t += shift;
                n.inst = inst;
                n.vel *= vel;
                n
            })
            .collect();
        self.notes.extend(copies);
    }

    /// Hold each chord for its bar(s): `chords[i]` from `bar + i * bars`.
    pub fn pads(&mut self, inst: usize, bar: f32, bars: f32, chords: &[&str], vel: f32) {
        for (i, chord) in chords.iter().enumerate() {
            for (k, p) in pitches(chord).into_iter().enumerate() {
                let b = bar + i as f32 * bars;
                // A little spread so the notes do not start as one click.
                let rows = bars * self.rows_per_bar as f32 + 1.0;
                self.note(inst, b, k as f32 * 0.15, p, rows, vel);
            }
        }
    }

    /// An arpeggio through each chord, struck every `step` rows for `bars`
    /// bars per chord; `octave` adds the octave above as a last step.
    #[allow(clippy::too_many_arguments)]
    pub fn arps(
        &mut self,
        inst: usize,
        bar: f32,
        bars: f32,
        chords: &[&str],
        step: usize,
        octave: bool,
        vel: f32,
    ) {
        let per_chord = (bars * self.rows_per_bar as f32) as usize;
        for (i, chord) in chords.iter().enumerate() {
            let p = pitches(chord);
            let mut arp: Vec<f32> = p.iter().map(|x| x - p[0]).collect();
            if octave {
                arp.push(12.0);
            }
            for r in (0..per_chord).step_by(step) {
                let n = self.note(
                    inst,
                    bar + i as f32 * bars,
                    r as f32,
                    p[0],
                    step as f32 * 0.9,
                    vel,
                );
                n.arp = arp.clone();
            }
        }
    }

    /// A bass pattern per bar: `pattern` has one entry per row, `.` for a
    /// rest, `0` for the root, `o` for the octave, `5` for the fifth, `7`
    /// for the minor seventh, `3` for the minor third, `-` for the note a
    /// whole tone below. `roots[i]` plays from `bar + i`.
    pub fn bass(
        &mut self,
        inst: usize,
        bar: f32,
        roots: &[&str],
        pattern: &str,
        gate: f32,
        vel: f32,
    ) {
        let steps: Vec<char> = pattern.chars().filter(|c| !c.is_whitespace()).collect();
        for (i, root) in roots.iter().enumerate() {
            let root = pitch(root).unwrap_or_else(|| panic!("bad root {root:?}"));
            for (r, c) in steps.iter().enumerate() {
                let offset = match c {
                    '0' => 0.0,
                    'o' => 12.0,
                    '5' => 7.0,
                    '7' => 10.0,
                    '3' => 3.0,
                    '-' => -2.0,
                    _ => continue,
                };
                let accent = if r % 4 == 0 { 1.0 } else { 0.75 };
                self.note(
                    inst,
                    bar + i as f32,
                    r as f32,
                    root + offset,
                    gate,
                    vel * accent,
                );
            }
        }
    }

    /// Drums for `bars` bars from `bar`: one character per row; `k` kick,
    /// `s` snare, `h` hat, `H` loud hat, `.` nothing.
    pub fn drums(&mut self, kit: [usize; 3], bar: f32, bars: usize, pattern: &str, vel: f32) {
        let steps: Vec<char> = pattern.chars().filter(|c| !c.is_whitespace()).collect();
        for b in 0..bars {
            for (r, c) in steps.iter().enumerate() {
                let (inst, midi, len, v) = match c {
                    'k' => (kit[0], 52.0, 0.2, 1.0),
                    's' => (kit[1], 96.0, 0.15, 1.0),
                    'h' => (kit[2], 118.0, 0.04, 0.6),
                    'H' => (kit[2], 118.0, 0.06, 1.0),
                    _ => continue,
                };
                let t = self.at(bar + b as f32, r as f32);
                self.notes.push(Note::new(t, len, midi, inst, v * vel));
            }
        }
    }

    /// A long note that holds from `bar` for `bars` bars.
    pub fn hold(&mut self, inst: usize, bar: f32, bars: f32, note: &str, vel: f32) {
        let rows = bars * self.rows_per_bar as f32;
        self.note(inst, bar, 0.0, pitch(note).unwrap(), rows, vel);
    }
}

// ---- the instruments -------------------------------------------------------------

/// A cutoff sweep through points (bar, multiplier), straight lines between.
pub fn sweep(bar_seconds: f32, points: &[(f32, f32)]) -> Auto {
    let pts = points.to_vec();
    Arc::new(move |t| {
        let b = t / bar_seconds;
        if b <= pts[0].0 {
            return pts[0].1;
        }
        for w in pts.windows(2) {
            if b <= w[1].0 {
                let x = (b - w[0].0) / (w[1].0 - w[0].0).max(1e-6);
                return w[0].1 + (w[1].1 - w[0].1) * x;
            }
        }
        pts[pts.len() - 1].1
    })
}

/// A slow wobble around 1.
pub fn wobble(depth: f32, rate: f32) -> Auto {
    Arc::new(move |t| 1.0 + depth * (t * rate).sin())
}

fn d() -> Inst {
    Inst::default()
}

pub fn drone(vol: f32) -> Inst {
    Inst {
        wave: Wave::Tri,
        attack: 3.0,
        decay: 0.1,
        sustain: 1.0,
        release: 5.0,
        filter: Filter::Low,
        cutoff: 700.0,
        vol,
        send: 0.5,
        ..d()
    }
}

pub fn pad(vol: f32, cutoff: f32, auto: Auto) -> Inst {
    Inst {
        wave: Wave::Pulse,
        attack: 0.8,
        decay: 0.5,
        sustain: 0.85,
        release: 1.6,
        pw: 0.5,
        pwm_depth: 0.35,
        pwm_rate: 0.17,
        filter: Filter::Low,
        cutoff,
        res: 0.3,
        auto,
        vol,
        send: 0.75,
        ..d()
    }
}

/// A darker, rounder pad: triangle through the filter.
pub fn choir(vol: f32, cutoff: f32, auto: Auto) -> Inst {
    Inst {
        wave: Wave::Saw,
        attack: 1.6,
        decay: 0.8,
        sustain: 0.8,
        release: 2.5,
        filter: Filter::Low,
        cutoff,
        res: 0.2,
        auto,
        vib_delay: 0.6,
        vib_depth: 0.06,
        vib_rate: 4.0,
        vol,
        send: 0.85,
        ..d()
    }
}

pub fn arp(vol: f32, cutoff: f32, env: f32, auto: Auto, pan: f32) -> Inst {
    Inst {
        wave: Wave::Pulse,
        attack: 0.01,
        decay: 0.4,
        sustain: 0.25,
        release: 0.35,
        pw: 0.25,
        pwm_depth: 0.15,
        pwm_rate: 0.4,
        filter: Filter::Low,
        cutoff,
        cutoff_env: env,
        res: 0.55,
        arp_rate: 50.0,
        auto,
        vol,
        pan,
        send: 0.5,
        ..d()
    }
}

/// A slower arpeggio of single notes for calm pieces (25 steps a second).
pub fn ripple(vol: f32, cutoff: f32, pan: f32) -> Inst {
    Inst {
        wave: Wave::Tri,
        attack: 0.01,
        decay: 0.6,
        sustain: 0.3,
        release: 0.6,
        filter: Filter::Low,
        cutoff,
        arp_rate: 12.5,
        vol,
        pan,
        send: 0.6,
        ..d()
    }
}

pub fn bass(vol: f32, cutoff: f32, env: f32) -> Inst {
    Inst {
        wave: Wave::Pulse,
        attack: 0.003,
        decay: 0.18,
        sustain: 0.35,
        release: 0.08,
        pw: 0.4,
        pwm_depth: 0.2,
        pwm_rate: 0.3,
        filter: Filter::Low,
        cutoff,
        cutoff_env: env,
        res: 0.45,
        vol,
        send: 0.1,
        ..d()
    }
}

/// A growling saw bass with a resonant sweep.
pub fn growl(vol: f32, cutoff: f32, auto: Auto) -> Inst {
    Inst {
        wave: Wave::Saw,
        attack: 0.01,
        decay: 0.5,
        sustain: 0.7,
        release: 0.3,
        filter: Filter::Low,
        cutoff,
        cutoff_env: 300.0,
        res: 0.7,
        auto,
        vol,
        send: 0.15,
        ..d()
    }
}

pub fn kick(vol: f32) -> Inst {
    Inst {
        wave: Wave::Tri,
        attack: 0.001,
        decay: 0.25,
        sustain: 0.0,
        release: 0.05,
        drop: 26.0,
        drop_time: 0.035,
        vol,
        send: 0.1,
        ..d()
    }
}

pub fn snare(vol: f32) -> Inst {
    Inst {
        wave: Wave::Noise,
        attack: 0.001,
        decay: 0.22,
        sustain: 0.0,
        release: 0.1,
        filter: Filter::Band,
        cutoff: 1800.0,
        res: 0.2,
        vol,
        send: 0.45,
        ..d()
    }
}

pub fn hat(vol: f32) -> Inst {
    Inst {
        wave: Wave::Noise,
        attack: 0.001,
        decay: 0.045,
        sustain: 0.0,
        release: 0.02,
        filter: Filter::High,
        cutoff: 6500.0,
        vol,
        pan: 0.3,
        send: 0.2,
        ..d()
    }
}

pub fn lead(vol: f32, cutoff: f32, pan: f32) -> Inst {
    Inst {
        wave: Wave::Pulse,
        attack: 0.02,
        decay: 0.4,
        sustain: 0.6,
        release: 0.5,
        pw: 0.45,
        pwm_depth: 0.25,
        pwm_rate: 0.7,
        filter: Filter::Low,
        cutoff,
        cutoff_env: 1200.0,
        res: 0.25,
        vib_delay: 0.25,
        vib_depth: 0.25,
        vib_rate: 5.5,
        vol,
        pan,
        send: 0.6,
        ..d()
    }
}

/// The lead's echo: darker and wetter.
pub fn shadow(vol: f32, cutoff: f32, pan: f32) -> Inst {
    Inst {
        release: 0.6,
        cutoff_env: 700.0,
        send: 0.8,
        ..lead(vol, cutoff, pan)
    }
}

/// A soft plucked "piano" for the calm pieces: pulse and decay, no sustain.
pub fn keys(vol: f32, cutoff: f32, pan: f32) -> Inst {
    Inst {
        wave: Wave::Pulse,
        attack: 0.004,
        decay: 1.6,
        sustain: 0.0,
        release: 0.8,
        pw: 0.3,
        pwm_depth: 0.08,
        pwm_rate: 0.25,
        filter: Filter::Low,
        cutoff,
        cutoff_env: 1400.0,
        res: 0.1,
        vol,
        pan,
        send: 0.55,
        ..d()
    }
}

/// A bell: triangle with a long ring.
pub fn bell(vol: f32, pan: f32) -> Inst {
    Inst {
        wave: Wave::Tri,
        attack: 0.002,
        decay: 2.5,
        sustain: 0.0,
        release: 1.5,
        vib_delay: 0.3,
        vib_depth: 0.05,
        vib_rate: 3.0,
        vol,
        pan,
        send: 0.9,
        ..d()
    }
}

/// A flute-like triangle lead with gentle vibrato.
pub fn flute(vol: f32, pan: f32) -> Inst {
    Inst {
        wave: Wave::Tri,
        attack: 0.08,
        decay: 0.3,
        sustain: 0.8,
        release: 0.5,
        filter: Filter::Low,
        cutoff: 3000.0,
        vib_delay: 0.35,
        vib_depth: 0.18,
        vib_rate: 5.0,
        vol,
        pan,
        send: 0.65,
        ..d()
    }
}

pub fn wind(vol: f32, auto: Auto) -> Inst {
    Inst {
        wave: Wave::Noise,
        attack: 4.0,
        decay: 0.1,
        sustain: 1.0,
        release: 5.0,
        filter: Filter::Band,
        cutoff: 700.0,
        res: 0.75,
        auto,
        vol,
        pan: 0.2,
        send: 0.85,
        ..d()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_names() {
        assert_eq!(pitch("c4"), Some(60.0));
        assert_eq!(pitch("a4"), Some(69.0));
        assert_eq!(pitch("bb3"), Some(58.0));
        assert_eq!(pitch("f#5"), Some(78.0));
        assert_eq!(pitch("x4"), None);
    }

    #[test]
    fn the_line_notation() {
        let mut s = Score::new(120.0, 4);
        let ids = s.line(0, 1.0, "d5:4 r:4 ~f5!:8 | g4?:2", 1.0);
        assert_eq!(ids.len(), 3);
        let n = &s.notes;
        assert!((n[0].t - s.bar()).abs() < 1e-4);
        assert_eq!(n[1].slide_from, Some(74.0));
        assert!((n[1].t - s.at(1.0, 8.0)).abs() < 1e-4);
        assert!(n[1].vel > 1.0 && n[2].vel < 1.0);
    }

    #[test]
    fn every_song_is_unique_and_sane() {
        let songs = all();
        assert_eq!(songs.len(), 11);
        for (i, s) in songs.iter().enumerate() {
            assert_eq!(s.number as usize, i + 1);
            assert!(s.key.bytes().all(|b| b.is_ascii_lowercase()), "{}", s.key);
            assert!(
                s.length > 60.0 && s.length < 240.0,
                "{} is {} s",
                s.key,
                s.length
            );
            assert!(s.notes.iter().all(|n| n.inst < s.insts.len()), "{}", s.key);
            assert!(
                s.notes.iter().all(|n| n.t >= 0.0 && n.t < s.length),
                "{}",
                s.key
            );
            assert!(songs.iter().filter(|o| o.key == s.key).count() == 1);
        }
        assert_eq!(find("1").unwrap().key, "ginnungagap");
        assert_eq!(find("Gimle").unwrap().number, 11);
    }
}
