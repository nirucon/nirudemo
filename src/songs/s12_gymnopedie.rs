//! "Gymnopedie": Erik Satie's Gymnopedie No. 1 (1888, public domain),
//! arranged for NIRUX in the sound of the C64 SID. D major, 3/4 at 70 BPM,
//! "Lent et douloureux", played twice as Satie wrote it (the repeat with its
//! first and second endings), and looped: the second ending's D minor turns
//! back into the first bar.
//!
//! Three voices like the SID: a soft triangle bass on the first beat, the
//! chord on the second and third beats (a pulse with slow pulse-width
//! movement the first time, a gentle arpeggio the second time) through a
//! resonant low-pass filter that sweeps over the whole piece, and the melody
//! with a delayed vibrato (triangle the first time, a filtered pulse the
//! second time). An echo of the melody and the dark ping-pong delay add a
//! little space. The notes are read from the score; nothing comes from a
//! recording, a MIDI file or another chiptune version.

use super::*;

const BPM: f32 = 70.0;
/// Bars in one time through: the 31 shared bars and an ending of 8.
pub(crate) const PASS_BARS: usize = 39;
pub(crate) const BARS: usize = 2 * PASS_BARS;

pub(crate) const BASS: usize = 0;
pub(crate) const CHORD: usize = 1;
pub(crate) const ARP: usize = 2;
pub(crate) const LEAD: usize = 3;
pub(crate) const PULSE_LEAD: usize = 4;
pub(crate) const ECHO: usize = 5;

/// The low note on the first beat of bars 1 to 31.
const SHARED_BASS: [&str; 31] = [
    "g2", "d2", "g2", "d2", "g2", "d2", "g2", "d2", "g2", "d2", "g2", "d2", "g2", "d2", "g2", "d2",
    "f#2", "b1", "e2", "e2", "d2", "a1", "d2", "d2", "d2", "d2", "d2", "d2", "d2", "d2", "d2",
];

/// The chord on the second and third beats of bars 1 to 31: G major 7
/// (over G) and D major 7 (over D) in turn, then the middle part.
const SHARED_CHORDS: [&str; 31] = [
    "b3 d4 f#4",
    "a3 c#4 f#4",
    "b3 d4 f#4",
    "a3 c#4 f#4",
    "b3 d4 f#4",
    "a3 c#4 f#4",
    "b3 d4 f#4",
    "a3 c#4 f#4",
    "b3 d4 f#4",
    "a3 c#4 f#4",
    "b3 d4 f#4",
    "a3 c#4 f#4",
    "b3 d4 f#4",
    "a3 c#4 f#4",
    "b3 d4 f#4",
    "a3 c#4 f#4",
    "a3 c#4 f#4",
    "b3 d4 f#4",
    "g3 b3",
    "b3 d4 g4",
    "f3 a3 d4",
    "a3 c4 e4",
    "g3 b3 e4",
    "d3 g3 b3 e4",
    "c3 e3 a3 d4",
    "c3 f#3 a3 d4",
    "a3 c4 f4",
    "a3 c4 e4",
    "d3 g3 b3 e4",
    "c3 e3 a3 d4",
    "c3 f#3 a3 d4",
];

/// The melody of bars 1 to 31 (a quarter note is 4 rows).
const SHARED_MELODY: &str = "r:48 |
    r:4 f#5:4 a5:4 | g5:4 f#5:4 c#5:4 | b4:4 c#5:4 d5:4 | a4:12 | f#4:48 |
    r:4 f#5:4 a5:4 | g5:4 f#5:4 c#5:4 | b4:4 c#5:4 d5:4 | a4:12 | c#5:12 | f#5:12 | e4:36 |
    a4:4 b4:4 c5:4 | e5:4 d5:4 b4:4 | d5:4 c5:4 b4:4 | d5:20 d5:4 |
    e5:4 f5:4 g5:4 | a5:4 c5:4 d5:4 | e5:4 d5:4 b4:4 | d5:20 d5:4";

/// One of the two endings, bars 32 to 39.
struct Ending {
    melody: &'static str,
    /// The low notes of bars 32 to 36.
    bass: [&'static str; 5],
    /// The chords of bars 32 to 36.
    chords: [&'static str; 5],
    /// The last chord, under the melody's D.
    last: &'static str,
}

/// The first ending turns to D major and back to the start.
const FIRST: Ending = Ending {
    melody: "g5:12 | f#5:12 | b4:4 a4:4 b4:4 | c#5:4 d5:4 e5:4 | c#5:4 d5:4 e5:4 | f#4:12 | c5:12 | d5:12",
    bass: ["e2", "f#2", "b1", "e2", "e2"],
    chords: [
        "b3 e4 g4",
        "a3 c#4 f#4",
        "b3 d4 f#4",
        "c#4 e4 a4",
        "a3 c#4 f#4 a4",
    ],
    last: "d4 f#4 a4",
};

/// The second ending, with F natural, closes in D minor.
const SECOND: Ending = Ending {
    melody:
        "g5:12 | f5:12 | b4:4 c5:4 f5:4 | e5:4 d5:4 c5:4 | e5:4 d5:4 c5:4 | f4:12 | c5:12 | d5:12",
    bass: ["e2", "e2", "e2", "e2", "e2"],
    chords: [
        "b3 e4 g4",
        "a3 d4 f4 a4",
        "a3 c4 f4",
        "c4 e4 a4",
        "a3 c4 f4 a4",
    ],
    last: "d4 f4 a4",
};

/// The chord voice: each chord as held pulse notes on `CHORD`, or as one
/// arpeggio on `ARP`.
fn chord(s: &mut Score, inst: usize, bar: f32, row: f32, rows: f32, notes: &str, vel: f32) {
    let p = pitches(notes);
    if inst == ARP {
        let n = s.note(inst, bar, row, p[0], rows, vel);
        n.arp = p.iter().map(|x| x - p[0]).collect();
    } else {
        // Each note gets the same share of the level, and a little spread
        // so the notes do not start as one click.
        let v = vel * 3.0 / p.len() as f32;
        for (k, &m) in p.iter().enumerate() {
            s.note(inst, bar, row + k as f32 * 0.1, m, rows - k as f32 * 0.1, v);
        }
    }
}

/// One time through the piece from `at`.
fn pass(s: &mut Score, at: f32, ending: &Ending, second: bool) {
    let (chord_inst, lead) = if second {
        (ARP, PULSE_LEAD)
    } else {
        (CHORD, LEAD)
    };
    let bass_bars = SHARED_BASS.iter().chain(ending.bass.iter());
    let chord_bars = SHARED_CHORDS.iter().chain(ending.chords.iter());
    for (i, (low, notes)) in bass_bars.zip(chord_bars).enumerate() {
        let bar = at + i as f32;
        // The repeated chords of bars 9 to 12 are marked forte.
        let vel = if (8..12).contains(&i) { 1.15 } else { 1.0 };
        s.note(BASS, bar, 0.0, pitch(low).unwrap(), 11.5, vel);
        chord(s, chord_inst, bar, 4.0, 7.5, notes, vel);
    }
    // Bar 37: the low E rings while the bass steps up, B and E, under a
    // short D and G chord.
    let b37 = at + 36.0;
    s.note(BASS, b37, 0.0, pitch("e2").unwrap(), 11.5, 1.0);
    s.note(BASS, b37, 4.0, pitch("b2").unwrap(), 3.8, 0.7);
    s.note(BASS, b37, 8.0, pitch("e3").unwrap(), 3.8, 0.7);
    chord(s, chord_inst, b37, 4.0, 3.6, "a3 d4", 0.9);
    chord(s, chord_inst, b37, 8.0, 3.6, "b3 d4 g4", 0.9);
    // Bars 38 and 39: an A minor 7 and the D chord, struck on the first beat.
    let b38 = at + 37.0;
    s.note(BASS, b38, 0.0, pitch("a2").unwrap(), 11.5, 1.0);
    s.note(BASS, b38, 0.0, pitch("g3").unwrap(), 11.5, 0.5);
    chord(s, chord_inst, b38, 0.0, 11.5, "c4 e4 a4", 0.9);
    let b39 = at + 38.0;
    s.note(BASS, b39, 0.0, pitch("d2").unwrap(), 11.0, 1.0);
    s.note(BASS, b39, 0.0, pitch("a2").unwrap(), 11.0, 0.5);
    chord(s, chord_inst, b39, 0.0, 11.0, ending.last, 0.85);

    let melody = format!("{SHARED_MELODY} | {}", ending.melody);
    let line = s.line(lead, at, &melody, 0.96);
    s.echo(&line, ECHO, 6.0, 0.8);
}

pub fn song() -> Song {
    let mut s = Score::new(BPM, 3);
    let bar = s.bar();
    let length = BARS as f32 * bar;
    // The filter opens through each time, a little more the second time,
    // and is back where it began when the loop starts again.
    let opening = sweep(
        bar,
        &[
            (0.0, 0.55),
            (8.0, 0.8),
            (12.0, 1.0),
            (21.0, 0.8),
            (29.0, 1.3),
            (39.0, 0.65),
            (47.0, 0.9),
            (51.0, 1.15),
            (60.0, 0.9),
            (68.0, 1.6),
            (74.0, 1.0),
            (78.0, 0.55),
        ],
    );
    let auto: Auto = Arc::new(move |t| opening(t.rem_euclid(length)));
    let lead_tri = Inst {
        wave: Wave::Tri,
        attack: 0.04,
        decay: 0.5,
        sustain: 0.75,
        release: 0.6,
        filter: Filter::Low,
        cutoff: 3200.0,
        vib_delay: 0.5,
        vib_depth: 0.16,
        vib_rate: 5.0,
        vol: 0.17,
        pan: 0.15,
        send: 0.6,
        ..Inst::default()
    };
    let insts = vec![
        // BASS
        Inst {
            wave: Wave::Tri,
            attack: 0.01,
            decay: 1.2,
            sustain: 0.5,
            release: 0.7,
            filter: Filter::Low,
            cutoff: 1100.0,
            vol: 0.3,
            send: 0.15,
            ..Inst::default()
        },
        // CHORD: held pulse notes with slow pulse-width movement.
        Inst {
            wave: Wave::Pulse,
            attack: 0.08,
            decay: 0.9,
            sustain: 0.55,
            release: 0.7,
            pw: 0.5,
            pwm_depth: 0.3,
            pwm_rate: 0.23,
            filter: Filter::Low,
            cutoff: 900.0,
            cutoff_env: 500.0,
            res: 0.5,
            auto: auto.clone(),
            vol: 0.045,
            pan: -0.2,
            send: 0.6,
            ..Inst::default()
        },
        // ARP: the same chords as one soft arpeggio.
        Inst {
            wave: Wave::Pulse,
            attack: 0.02,
            decay: 0.8,
            sustain: 0.45,
            release: 0.5,
            pw: 0.3,
            pwm_depth: 0.2,
            pwm_rate: 0.3,
            filter: Filter::Low,
            cutoff: 1000.0,
            cutoff_env: 600.0,
            res: 0.55,
            arp_rate: 25.0,
            auto,
            vol: 0.1,
            pan: -0.2,
            send: 0.6,
            ..Inst::default()
        },
        // LEAD, the first time: triangle.
        lead_tri.clone(),
        // PULSE_LEAD, the second time: a filtered pulse.
        Inst {
            wave: Wave::Pulse,
            decay: 0.6,
            sustain: 0.6,
            pw: 0.4,
            pwm_depth: 0.15,
            pwm_rate: 0.5,
            cutoff: 1400.0,
            cutoff_env: 1200.0,
            res: 0.3,
            vol: 0.11,
            send: 0.65,
            ..lead_tri.clone()
        },
        // ECHO of the melody: softer, on the other side.
        Inst {
            vol: 0.05,
            pan: -0.55,
            send: 0.9,
            ..lead_tri
        },
    ];
    pass(&mut s, 0.0, &FIRST, false);
    pass(&mut s, PASS_BARS as f32, &SECOND, true);

    Song {
        number: 12,
        key: "gymnopedie",
        title: "Gymnopédie",
        about: "D major, 3/4 at 70 BPM: Erik Satie's Gymnopédie No. 1 (1888), arranged for three SID voices",
        insts,
        notes: s.notes,
        length,
        delay: s.row * 6.0,
        credit: Some(Credit {
            title: "Gymnopedie No. 1",
            artist: "Erik Satie",
            comment: "NIRUX chiptune arrangement of Gymnopedie No. 1 (1888, public domain)",
        }),
        looped: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pitches on `inst` that start at `row` of `bar` (0-based).
    fn at(song: &Song, inst: usize, bar: usize, row: f32) -> Vec<f32> {
        let s = Score::new(BPM, 3);
        let t = s.at(bar as f32, row);
        let mut p: Vec<f32> = song
            .notes
            .iter()
            .filter(|n| n.inst == inst && (n.t - t).abs() < s.row * 0.5)
            .flat_map(|n| {
                if n.arp.is_empty() {
                    vec![n.midi]
                } else {
                    n.arp.iter().map(|a| n.midi + a).collect()
                }
            })
            .collect();
        p.sort_by(f32::total_cmp);
        p
    }

    fn names(text: &str) -> Vec<f32> {
        pitches(text)
    }

    #[test]
    fn length_and_bars() {
        let song = song();
        let bar = Score::new(BPM, 3).bar();
        assert_eq!(BARS, 78);
        assert!((song.length - 78.0 * bar).abs() < 1e-3);
        assert!(
            song.length > 150.0 && song.length < 210.0,
            "{}",
            song.length
        );
        assert!(song.looped);
        // Every bar has its bass note, and nothing starts after the end.
        for b in 0..BARS {
            assert!(!at(&song, BASS, b, 0.0).is_empty(), "bar {}", b + 1);
        }
        assert!(song.notes.iter().all(|n| n.t < song.length));
        assert_eq!(
            song.tags(),
            (
                "Gymnopedie No. 1",
                "Erik Satie",
                "NIRUX chiptune arrangement of Gymnopedie No. 1 (1888, public domain)"
            )
        );
        assert_eq!(song.file_name(), "nirux-12-gymnopedie.ogg");
    }

    #[test]
    fn the_alternating_chords() {
        let song = song();
        for pass in [0, PASS_BARS] {
            let chords = if pass == 0 { CHORD } else { ARP };
            for b in 0..16 {
                let (low, chord) = if b % 2 == 0 {
                    ("g2", "b3 d4 f#4")
                } else {
                    ("d2", "a3 c#4 f#4")
                };
                assert_eq!(at(&song, BASS, pass + b, 0.0), names(low), "bar {}", b + 1);
                assert_eq!(
                    at(&song, chords, pass + b, 4.0),
                    names(chord),
                    "bar {}",
                    b + 1
                );
            }
        }
    }

    #[test]
    fn the_melody() {
        let song = song();
        // (bar of the score, row, note) in the first time through.
        let first = [
            (5, 4.0, "f#5"),
            (5, 8.0, "a5"),
            (6, 0.0, "g5"),
            (6, 4.0, "f#5"),
            (6, 8.0, "c#5"),
            (7, 0.0, "b4"),
            (7, 4.0, "c#5"),
            (7, 8.0, "d5"),
            (8, 0.0, "a4"),
            (9, 0.0, "f#4"),
            (17, 0.0, "c#5"),
            (18, 0.0, "f#5"),
            (19, 0.0, "e4"),
            (22, 0.0, "a4"),
            (22, 4.0, "b4"),
            (22, 8.0, "c5"),
            (27, 4.0, "f5"),
            (28, 4.0, "c5"),
            (32, 0.0, "g5"),
            (33, 0.0, "f#5"),
            (39, 0.0, "d5"),
        ];
        for (bar, row, note) in first {
            assert_eq!(
                at(&song, LEAD, bar - 1, row),
                names(note),
                "bar {bar} row {row}"
            );
        }
        // The second ending has F natural where the first had F sharp.
        let second = [
            (33, 0.0, "f5"),
            (34, 4.0, "c5"),
            (34, 8.0, "f5"),
            (37, 0.0, "f4"),
        ];
        for (bar, row, note) in second {
            assert_eq!(
                at(&song, PULSE_LEAD, PASS_BARS + bar - 1, row),
                names(note),
                "second time, bar {bar} row {row}"
            );
        }
        // Bars 1 to 21 stay in D major: no C or F natural.
        let s = Score::new(BPM, 3);
        for n in song.notes.iter().filter(|n| n.inst == LEAD) {
            if n.t < s.at(21.0, 0.0) {
                let pc = n.midi as i32 % 12;
                assert!(pc != 0 && pc != 5, "{} at {}", n.midi, n.t);
            }
        }
        // 44 notes in the shared bars and 14 in each ending.
        let count = |inst| song.notes.iter().filter(|n| n.inst == inst).count();
        assert_eq!(count(LEAD), 58);
        assert_eq!(count(PULSE_LEAD), 58);
        assert_eq!(count(ECHO), 2 * count(LEAD));
    }

    #[test]
    fn the_endings() {
        let song = song();
        // Bar 39: D major the first time, D minor the second.
        assert_eq!(at(&song, CHORD, 38, 0.0), names("d4 f#4 a4"));
        assert_eq!(at(&song, ARP, BARS - 1, 0.0), names("d4 f4 a4"));
        assert_eq!(at(&song, BASS, BARS - 1, 0.0), names("d2 a2"));
        // The middle part: A minor over A, D7 over D.
        assert_eq!(at(&song, BASS, 21, 0.0), names("a1"));
        assert_eq!(at(&song, CHORD, 21, 4.0), names("a3 c4 e4"));
        assert_eq!(at(&song, CHORD, 25, 4.0), names("c3 f#3 a3 d4"));
    }
}
