//! "Mimir": F sharp minor, 60 BPM. The well of wisdom: slow and free like a
//! Satie gnossienne, a low note and three soft chords per bar, a melody
//! with small turns and the augmented second of the harmonic minor.

use super::*;

/// The low note on the first beat, the chord on beats two to four.
fn steps(s: &mut Score, keys: usize, bar: f32, chords: &[&str], vel: f32) {
    for (i, name) in chords.iter().enumerate() {
        let (low, chord) = match *name {
            "f#m" => ("f#2", "a3 c#4 f#4"),
            "bm" => ("b1", "b3 d4 f#4"),
            "c#" => ("c#2", "g#3 c#4 e#4"),
            "d" => ("d2", "a3 d4 f#4"),
            other => panic!("chord {other}"),
        };
        let b = bar + i as f32;
        s.note(keys, b, 0.0, pitch(low).unwrap(), 15.0, vel);
        for beat in [4.0, 8.0, 12.0] {
            for (k, p) in pitches(chord).into_iter().enumerate() {
                s.note(keys, b, beat + k as f32 * 0.1, p, 3.0, vel * 0.42);
            }
        }
    }
}

pub fn song() -> Song {
    let mut s = Score::new(60.0, 4);
    let bar = s.bar();
    const DRONE: usize = 0;
    const CHOIR: usize = 1;
    const KEYS: usize = 2;
    const MELODY: usize = 3;
    const ECHO: usize = 4;
    let insts = vec![
        drone(0.06),
        choir(0.018, 520.0, wobble(0.3, 0.1)),
        keys(0.12, 850.0, -0.15),
        Inst {
            decay: 2.2,
            ..keys(0.14, 1300.0, 0.2)
        },
        Inst {
            decay: 2.2,
            ..keys(0.05, 900.0, -0.55)
        },
    ];
    s.hold(DRONE, 0.0, 23.5, "f#2", 1.0);
    steps(&mut s, KEYS, 0.0, &["f#m", "f#m"], 1.0);
    let a_chords = ["f#m", "f#m", "bm", "f#m", "f#m", "c#", "f#m", "f#m"];
    let b_chords = ["d", "d", "bm", "c#", "f#m", "bm", "c#", "c#"];
    steps(&mut s, KEYS, 2.0, &a_chords, 1.0);
    steps(&mut s, KEYS, 10.0, &b_chords, 1.0);
    steps(
        &mut s,
        KEYS,
        18.0,
        &["f#m", "bm", "c#", "f#m", "f#m", "f#m"],
        0.9,
    );
    s.pads(CHOIR, 2.0, 8.0, &["f#3 c#4 a4"], 1.0);
    s.pads(
        CHOIR,
        10.0,
        2.0,
        &["d3 a3 f#4", "b2 f#3 d4", "f#3 c#4 a4", "c#3 g#3 e#4"],
        1.0,
    );
    s.pads(CHOIR, 18.0, 6.0, &["f#3 c#4 a4"], 1.0);

    let a = "c#5:4 ~d5:2 c#5:2 b4:4 c#5:4 | a4:12 r:4 | d5:4 ~e5:2 d5:2 c#5:4 b4:4 | c#5:16 |
             f#5:4 e#5:2 f#5:2 g#5:4 a5:4 | g#5:6 f#5:2 e#5:8 | f#5:4 c#5:4 a4:4 ~b4:2 a4:2 | f#4:16";
    let b = "a5:8 f#5:4 a5:4 | ~b5:4 a5:4 f#5:8 | d5:4 f#5:4 b5:8 | e#5:8 g#5:4 ~a5:2 g#5:2 |
             a5:4 g#5:2 f#5:2 c#5:8 | d5:4 c#5:2 b4:2 f#4:8 | g#4:4 ~a4:2 g#4:2 e#4:8 | c#5:16";
    let end =
        "c#5:4 ~d5:2 c#5:2 b4:4 c#5:4 | d5:4 ~e5:2 d5:2 c#5:4 b4:4 | g#4:6 a4:2 e#4:8 | f#4:32";
    for (bar_at, text) in [(2.0, a), (10.0, b), (18.0, end)] {
        let ids = s.line(MELODY, bar_at, text, 0.95);
        s.echo(&ids, ECHO, 3.0, 1.0);
    }

    Song {
        number: 5,
        key: "mimir",
        title: "Mimir",
        about:
            "F sharp minor, 60 BPM: slow and free like a gnossienne, with small turns in the melody",
        insts,
        notes: s.notes,
        length: 24.0 * bar + 6.0,
        delay: s.row * 3.0,
    }
}
