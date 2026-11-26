//! "Ginnungagap", the music of the first NIRUX demo. An original chiptune in
//! D minor at 76 BPM: dark ambient drones and pads, with the C64 tricks of
//! the 1980s on top (fast chord arpeggios, a bouncing pulse bass, vibrato and
//! slides in the lead, and an echo made from a delayed second voice).
//!
//! 32 bars of 16 rows (sixteenth notes):
//!   0-3   the void: drone, wind and a closed pad
//!   4-7   the arpeggio wakes, the filter opens
//!   8-15  bass and sparse drums; a far call in the lead's echo
//!   16-23 the lead melody
//!   24-27 breakdown: no drums, the filter closes
//!   28-31 outro: the last chords and a long D

use super::Song;
use crate::synth::{Filter, Inst, Note, Wave};
use std::sync::Arc;

pub const BPM: f32 = 76.0;
pub const ROW: f32 = 60.0 / BPM / 4.0;
pub const BAR: f32 = ROW * 16.0;
pub const BARS: usize = 32;
/// Song length in seconds, with room for the reverb tail.
pub const LENGTH: f32 = BAR * BARS as f32 + 6.0;
/// The echo delay: a dotted eighth.
pub const DELAY: f32 = ROW * 3.0;

const DRONE: usize = 0;
const PAD: usize = 1;
const ARP: usize = 2;
const BASS: usize = 3;
const KICK: usize = 4;
const SNARE: usize = 5;
const HAT: usize = 6;
const LEAD: usize = 7;
const ECHO: usize = 8;
const WIND: usize = 9;

fn bar_of(t: f32) -> f32 {
    t / BAR
}

fn pad_auto(t: f32) -> f32 {
    let b = bar_of(t);
    if b < 8.0 {
        0.4 + 1.1 * b / 8.0
    } else if b < 24.0 {
        1.5 + 0.3 * (t * 0.2).sin()
    } else {
        (1.5 - (b - 24.0) / 8.0 * 1.15).max(0.35)
    }
}

fn arp_auto(t: f32) -> f32 {
    let b = bar_of(t);
    if b < 8.0 {
        0.3 + 0.7 * ((b - 4.0) / 4.0).clamp(0.0, 1.0)
    } else if b < 24.0 {
        1.0 + 0.25 * (t * 0.3).sin()
    } else if b < 28.0 {
        1.0 - 0.65 * (b - 24.0) / 4.0
    } else {
        0.35
    }
}

fn wind_auto(t: f32) -> f32 {
    0.8 + 0.6 * (t * 0.13).sin() + 0.2 * (t * 0.37).sin()
}

pub fn instruments() -> Vec<Inst> {
    let d = Inst::default();
    vec![
        // DRONE: soft low triangles.
        Inst {
            wave: Wave::Tri,
            attack: 3.0,
            decay: 0.1,
            sustain: 1.0,
            release: 5.0,
            filter: Filter::Low,
            cutoff: 700.0,
            vol: 0.085,
            send: 0.5,
            ..d.clone()
        },
        // PAD: slow pulse-width movement behind a low-pass.
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
            cutoff: 520.0,
            res: 0.3,
            auto: Arc::new(pad_auto),
            vol: 0.05,
            pan: 0.0,
            send: 0.75,
            ..d.clone()
        },
        // ARP: fast chord arpeggio, re-struck every beat for a breathing pump.
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
            cutoff: 900.0,
            cutoff_env: 1700.0,
            res: 0.55,
            arp_rate: 50.0,
            auto: Arc::new(arp_auto),
            vol: 0.13,
            pan: -0.35,
            send: 0.5,
            ..d.clone()
        },
        // BASS: bouncing pulse bass.
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
            cutoff: 330.0,
            cutoff_env: 900.0,
            res: 0.45,
            vol: 0.24,
            send: 0.1,
            ..d.clone()
        },
        // KICK: a triangle that falls fast.
        Inst {
            wave: Wave::Tri,
            attack: 0.001,
            decay: 0.25,
            sustain: 0.0,
            release: 0.05,
            drop: 26.0,
            drop_time: 0.035,
            vol: 0.55,
            send: 0.1,
            ..d.clone()
        },
        // SNARE: band-passed noise.
        Inst {
            wave: Wave::Noise,
            attack: 0.001,
            decay: 0.22,
            sustain: 0.0,
            release: 0.1,
            filter: Filter::Band,
            cutoff: 1800.0,
            res: 0.2,
            vol: 0.55,
            send: 0.45,
            ..d.clone()
        },
        // HAT: thin high noise.
        Inst {
            wave: Wave::Noise,
            attack: 0.001,
            decay: 0.045,
            sustain: 0.0,
            release: 0.02,
            filter: Filter::High,
            cutoff: 6500.0,
            vol: 0.10,
            pan: 0.3,
            send: 0.2,
            ..d.clone()
        },
        // LEAD: pulse with delayed vibrato.
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
            cutoff: 1700.0,
            cutoff_env: 1200.0,
            res: 0.25,
            vib_delay: 0.25,
            vib_depth: 0.25,
            vib_rate: 5.5,
            vol: 0.16,
            pan: 0.15,
            send: 0.6,
            ..d.clone()
        },
        // ECHO: the lead again, later, darker and to the other side.
        Inst {
            wave: Wave::Pulse,
            attack: 0.02,
            decay: 0.4,
            sustain: 0.6,
            release: 0.6,
            pw: 0.45,
            pwm_depth: 0.25,
            pwm_rate: 0.7,
            filter: Filter::Low,
            cutoff: 1100.0,
            cutoff_env: 700.0,
            res: 0.25,
            vib_delay: 0.25,
            vib_depth: 0.25,
            vib_rate: 5.5,
            vol: 0.065,
            pan: -0.55,
            send: 0.8,
            ..d.clone()
        },
        // WIND: high noise through a moving band-pass.
        Inst {
            wave: Wave::Noise,
            attack: 4.0,
            decay: 0.1,
            sustain: 1.0,
            release: 5.0,
            filter: Filter::Band,
            cutoff: 700.0,
            res: 0.75,
            auto: Arc::new(wind_auto),
            vol: 0.08,
            pan: 0.2,
            send: 0.85,
            ..d.clone()
        },
    ]
}

struct Chord {
    bass: f32,
    triad: [f32; 3],
    pad: [f32; 3],
}

const DM: Chord = Chord {
    bass: 50.0,
    triad: [62.0, 65.0, 69.0],
    pad: [50.0, 57.0, 65.0],
};
const BB: Chord = Chord {
    bass: 46.0,
    triad: [58.0, 62.0, 65.0],
    pad: [46.0, 53.0, 62.0],
};
const GM: Chord = Chord {
    bass: 43.0,
    triad: [62.0, 67.0, 70.0],
    pad: [43.0, 50.0, 58.0],
};
const AM: Chord = Chord {
    bass: 45.0,
    triad: [61.0, 64.0, 69.0],
    pad: [45.0, 52.0, 61.0],
};
const EB: Chord = Chord {
    bass: 51.0,
    triad: [63.0, 67.0, 70.0],
    pad: [51.0, 58.0, 67.0],
};

/// The chord of each bar from bar 4 on.
fn chord(bar: usize) -> &'static Chord {
    const LOOP: [&Chord; 8] = [&DM, &BB, &GM, &AM, &DM, &EB, &BB, &AM];
    const END: [&Chord; 4] = [&DM, &EB, &AM, &DM];
    if bar >= 28 {
        END[bar - 28]
    } else {
        LOOP[(bar.max(4) - 4) % 8]
    }
}

fn at(bar: usize, row: f32) -> f32 {
    bar as f32 * BAR + row * ROW
}

/// (row from bar 16, MIDI note, rows long, slide from)
const MELODY: &[(f32, f32, f32, Option<f32>)] = &[
    (0.0, 74.0, 6.0, None),
    (6.0, 76.0, 2.0, None),
    (8.0, 77.0, 8.0, None),
    (16.0, 77.0, 6.0, None),
    (22.0, 74.0, 2.0, None),
    (24.0, 70.0, 8.0, None),
    (32.0, 67.0, 4.0, None),
    (36.0, 70.0, 4.0, None),
    (40.0, 74.0, 6.0, None),
    (46.0, 72.0, 2.0, None),
    (48.0, 73.0, 8.0, None),
    (56.0, 76.0, 4.0, None),
    (60.0, 69.0, 4.0, None),
    (64.0, 81.0, 8.0, Some(77.0)),
    (72.0, 79.0, 4.0, None),
    (76.0, 77.0, 4.0, None),
    (80.0, 79.0, 6.0, None),
    (86.0, 77.0, 2.0, None),
    (88.0, 75.0, 8.0, None),
    (96.0, 77.0, 6.0, None),
    (102.0, 74.0, 2.0, None),
    (104.0, 70.0, 8.0, None),
    (112.0, 69.0, 4.0, None),
    (116.0, 73.0, 4.0, None),
    (120.0, 76.0, 4.0, None),
    (124.0, 79.0, 4.0, None),
    // breakdown
    (128.0, 74.0, 16.0, Some(76.0)),
    (160.0, 77.0, 8.0, None),
    (168.0, 74.0, 8.0, None),
    (176.0, 73.0, 16.0, None),
    // outro
    (192.0, 69.0, 16.0, None),
    (208.0, 70.0, 16.0, None),
    (224.0, 73.0, 8.0, None),
    (232.0, 76.0, 8.0, None),
    (240.0, 74.0, 24.0, Some(73.0)),
];

pub fn notes() -> Vec<Note> {
    let mut n = Vec::new();
    let whole = BAR * BARS as f32;

    // The void: D and A held through the whole song.
    n.push(Note::new(0.0, whole - 2.0, 38.0, DRONE, 1.0));
    n.push(Note::new(0.5, whole - 3.0, 45.0, DRONE, 0.7));
    n.push(Note::new(0.0, 8.0 * BAR, 120.0, WIND, 1.0));
    n.push(Note::new(26.0 * BAR, 6.0 * BAR, 120.0, WIND, 0.9));

    // Pad: a closed D minor first, then the chords.
    for (i, &p) in DM.pad.iter().enumerate() {
        n.push(Note::new(
            at(0, 0.0) + i as f32 * 0.6,
            4.0 * BAR,
            p,
            PAD,
            1.0,
        ));
    }
    for bar in 4..BARS {
        let len = if bar == BARS - 1 {
            2.0 * BAR
        } else {
            BAR + ROW * 2.0
        };
        for &p in chord(bar).pad.iter() {
            n.push(Note::new(at(bar, 0.0), len, p, PAD, 1.0));
        }
    }

    // Arpeggio: every beat (every second beat in the outro).
    for bar in 4..BARS - 1 {
        let c = chord(bar);
        let step = if bar >= 28 { 8 } else { 4 };
        for beat in (0..16).step_by(step) {
            let mut note = Note::new(
                at(bar, beat as f32),
                ROW * step as f32 * 0.9,
                c.triad[0],
                ARP,
                1.0,
            );
            note.arp = vec![0.0, c.triad[1] - c.triad[0], c.triad[2] - c.triad[0], 12.0];
            n.push(note);
        }
    }

    // Bass: root and octave bouncing in sixteenths.
    const BOUNCE: [f32; 16] = [
        0.0, 0.0, 12.0, 0.0, 0.0, 12.0, 0.0, 12.0, 0.0, 0.0, 12.0, 0.0, 7.0, 0.0, 12.0, 10.0,
    ];
    const ACCENT: [f32; 16] = [
        1.0, 0.6, 0.8, 0.6, 0.9, 0.7, 0.6, 0.8, 1.0, 0.6, 0.8, 0.6, 0.9, 0.6, 0.8, 0.7,
    ];
    for bar in 8..28 {
        let root = chord(bar).bass;
        for row in 0..16 {
            // Rests let it breathe: a few rows are skipped in the quieter bars.
            if bar < 12 && row % 4 == 3 {
                continue;
            }
            n.push(Note::new(
                at(bar, row as f32),
                ROW * 0.85,
                root + BOUNCE[row],
                BASS,
                ACCENT[row],
            ));
        }
    }

    // Drums: kick and snare from bar 8, hats from 12, a roll into the lead.
    for bar in 8..24 {
        n.push(Note::new(at(bar, 0.0), 0.2, 52.0, KICK, 1.0));
        n.push(Note::new(at(bar, 10.0), 0.2, 52.0, KICK, 0.8));
        n.push(Note::new(at(bar, 8.0), 0.15, 96.0, SNARE, 1.0));
        if bar >= 12 {
            for row in [2, 6, 10, 14] {
                n.push(Note::new(
                    at(bar, row as f32),
                    0.04,
                    118.0,
                    HAT,
                    if row == 6 { 1.0 } else { 0.6 },
                ));
            }
        }
        if bar == 15 || bar == 23 {
            for row in [12, 13, 14, 15] {
                n.push(Note::new(
                    at(bar, row as f32),
                    0.08,
                    96.0,
                    SNARE,
                    0.35 + 0.15 * (row - 12) as f32,
                ));
            }
        }
    }

    // A far call before the melody, only in the echo voice.
    for &(row, midi, len) in &[
        (0.0, 81.0, 12.0),
        (32.0, 79.0, 6.0),
        (38.0, 77.0, 10.0),
        (64.0, 81.0, 12.0),
        (96.0, 76.0, 6.0),
        (102.0, 74.0, 10.0),
    ] {
        n.push(Note::new(at(8, row), len * ROW, midi, ECHO, 1.2));
    }

    // The lead, and its echo three rows later.
    for &(row, midi, len, slide) in MELODY {
        let t = at(16, row);
        let mut lead = Note::new(t, len * ROW * 0.95, midi, LEAD, 1.0);
        lead.slide_from = slide;
        lead.slide_time = ROW * 1.5;
        let mut echo = lead.clone();
        echo.t = t + DELAY;
        echo.inst = ECHO;
        n.push(lead);
        n.push(echo);
    }
    n
}

pub fn song() -> Song {
    Song {
        number: 1,
        key: "ginnungagap",
        title: "Ginnungagap",
        about:
            "D minor, 76 BPM: the void, a waking arpeggio, a bouncing bass and a lead with its echo",
        insts: instruments(),
        notes: notes(),
        length: LENGTH,
        delay: DELAY,
        credit: None,
        looped: false,
    }
}
