//! "Hel": E minor, 3/4 at 62 BPM. A calm, Satie-like walk through the quiet
//! realm: a low note on the first beat, a soft chord on the second, a slow
//! triangle melody above, swaying between two chords.

use super::*;

/// Bass on the first beat, the chord on the second, for each bar.
pub(super) fn sway(s: &mut Score, keys: usize, bar: f32, chords: &[(&str, &str)], vel: f32) {
    for (i, (low, chord)) in chords.iter().enumerate() {
        let b = bar + i as f32;
        let rpb = s.rows_per_bar as f32;
        s.note(keys, b, 0.0, pitch(low).unwrap(), rpb - 1.0, vel);
        for (k, p) in pitches(chord).into_iter().enumerate() {
            s.note(keys, b, 4.0 + k as f32 * 0.12, p, rpb - 5.0, vel * 0.55);
        }
    }
}

pub fn song() -> Song {
    let mut s = Score::new(62.0, 3);
    let bar = s.bar();
    const DRONE: usize = 0;
    const CHOIR: usize = 1;
    const KEYS: usize = 2;
    const FLUTE: usize = 3;
    const ECHO: usize = 4;
    let insts = vec![
        drone(0.06),
        choir(
            0.022,
            600.0,
            sweep(bar, &[(0.0, 0.6), (20.0, 1.3), (28.0, 1.0), (40.0, 0.5)]),
        ),
        keys(0.13, 900.0, -0.1),
        flute(0.12, 0.15),
        Inst {
            vol: 0.05,
            pan: -0.5,
            send: 0.9,
            ..flute(0.05, -0.5)
        },
    ];
    let em = ("e2", "g3 b3 d4 f#4");
    let cmaj7 = ("c2", "g3 b3 e4");
    let am7 = ("a1", "g3 c4 e4");
    let bm7 = ("b1", "a3 d4 f#4");
    let c7 = ("c2", "g3 b3 e4");
    let b7 = ("b1", "a3 d#4 f#4");
    let sway_ab: Vec<_> = (0..16)
        .map(|i| if i % 2 == 0 { em } else { cmaj7 })
        .collect();

    s.hold(DRONE, 0.0, 39.0, "e2", 1.0);
    sway(&mut s, KEYS, 0.0, &[em, cmaj7, em, cmaj7], 1.0);
    sway(&mut s, KEYS, 4.0, &sway_ab, 1.0);
    sway(
        &mut s,
        KEYS,
        20.0,
        &[am7, am7, bm7, bm7, c7, c7, b7, b7],
        1.0,
    );
    sway(&mut s, KEYS, 28.0, &sway_ab[..8], 1.0);
    sway(&mut s, KEYS, 36.0, &[em, cmaj7, em], 0.8);
    s.pads(
        CHOIR,
        4.0,
        2.0,
        &[
            "e3 b3 g4", "e3 b3 g4", "e3 b3 g4", "e3 b3 g4", "e3 b3 g4", "e3 b3 g4", "e3 b3 g4",
            "e3 b3 g4",
        ],
        1.0,
    );
    s.pads(
        CHOIR,
        20.0,
        2.0,
        &["a2 e3 c4", "b2 f#3 d4", "c3 g3 e4", "b2 f#3 d#4"],
        1.0,
    );
    s.pads(CHOIR, 36.0, 3.0, &["e3 b3 g4"], 1.0);

    let a = "r:12 | r:4 b4:4 d5:4 | f#5:4 e5:4 b4:4 | a4:12 | r:12 | r:4 g4:4 a4:4 | b4:8 d5:4 | e5:12 |
             r:12 | r:4 e5:4 g5:4 | f#5:4 d5:4 b4:4 | c5:12 | r:4 b4:4 a4:4 | g4:4 e4:4 g4:4 | f#4:12 | e4:12";
    let b = "e5:8 d5:4 | c5:12 | f#5:8 e5:4 | d5:12 | g5:8 f#5:4 | e5:8 b4:4 | d#5:12 | ~f#5:12";
    let first = s.line(FLUTE, 4.0, a, 0.97);
    s.echo(&first, ECHO, 6.0, 1.0);
    let middle = s.line(FLUTE, 20.0, b, 0.97);
    s.echo(&middle, ECHO, 6.0, 1.0);
    let again = s.line(
        FLUTE,
        28.0,
        "r:12 | r:4 b4:4 d5:4 | f#5:4 e5:4 b4:4 | a4:12 | r:12 | r:4 g4:4 a4:4 | b4:8 g4:4 | e4:36",
        0.97,
    );
    s.echo(&again, ECHO, 6.0, 1.0);

    Song {
        number: 3,
        key: "hel",
        title: "Hel",
        about: "E minor, 3/4 at 62 BPM: a calm, Satie-like sway between two chords",
        insts,
        notes: s.notes,
        length: 39.0 * bar + 6.0,
        delay: s.row * 6.0,
        credit: None,
        looped: false,
    }
}
