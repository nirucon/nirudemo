//! "Gimle": A minor, 64 BPM. The hall that stands after Ragnarok: a calm,
//! hopeful close with a slow arpeggio, soft keys and a flute, ending on a
//! major chord.

use super::*;

pub fn song() -> Song {
    let mut s = Score::new(64.0, 4);
    let bar = s.bar();
    const DRONE: usize = 0;
    const PAD: usize = 1;
    const RIPPLE: usize = 2;
    const KEYS: usize = 3;
    const FLUTE: usize = 4;
    const ECHO: usize = 5;
    const BELL: usize = 6;
    let insts = vec![
        drone(0.06),
        pad(
            0.04,
            500.0,
            sweep(bar, &[(0.0, 0.5), (8.0, 1.3), (24.0, 1.4), (28.0, 0.7)]),
        ),
        ripple(0.09, 1600.0, -0.3),
        keys(0.10, 900.0, 0.2),
        flute(0.12, 0.1),
        Inst {
            vol: 0.05,
            pan: -0.5,
            send: 0.9,
            ..flute(0.05, -0.5)
        },
        bell(0.06, 0.4),
    ];
    let intro = ["a2 e3 c4", "a2 e3 c4", "f2 c3 e4", "f2 c3 e4"];
    let main = [
        "a2 e3 c4",
        "f2 c3 e4",
        "c3 g3 e4",
        "g2 d3 b3",
        "a2 e3 c4",
        "f2 c3 e4",
        "d3 a3 c4 f4",
        "e2 b2 g#3",
    ];
    let main_arps = [
        "a3 c4 e4 b4",
        "f3 a3 c4 e4",
        "c4 e4 g4 d5",
        "g3 b3 d4 a4",
        "a3 c4 e4 b4",
        "f3 a3 c4 e4",
        "d4 f4 a4 c5",
        "e4 g#4 b4 e5",
    ];
    let close = [
        "f2 c3 a3", "g2 d3 b3", "e2 b2 g3", "a2 e3 c4", "f2 c3 a3", "g2 d3 b3",
    ];
    let close_arps = [
        "f3 a3 c4 e4",
        "g3 b3 d4 a4",
        "e3 g3 b3 d4",
        "a3 c4 e4 b4",
        "f3 a3 c4 e4",
        "g3 b3 d4 a4",
    ];

    s.hold(DRONE, 0.0, 27.0, "a1", 1.0);
    s.pads(PAD, 0.0, 1.0, &intro, 1.0);
    s.arps(
        RIPPLE,
        0.0,
        1.0,
        &["a3 c4 e4 b4", "a3 c4 e4 b4", "f3 a3 c4 e4", "f3 a3 c4 e4"],
        8,
        true,
        0.9,
    );
    s.pads(PAD, 4.0, 2.0, &main, 1.0);
    s.arps(RIPPLE, 4.0, 2.0, &main_arps, 8, true, 1.0);
    for (i, chord) in main.iter().enumerate() {
        let b = 4.0 + i as f32 * 2.0;
        for (k, p) in pitches(chord).into_iter().enumerate() {
            s.note(KEYS, b, k as f32 * 0.5, p + 12.0, 6.0, 0.5);
            s.note(KEYS, b + 1.0, 8.0 + k as f32 * 0.5, p + 12.0, 6.0, 0.4);
        }
    }
    s.pads(PAD, 20.0, 1.0, &close, 1.0);
    s.arps(RIPPLE, 20.0, 1.0, &close_arps, 4, true, 1.0);
    // The end: A major, the third raised.
    s.pads(PAD, 26.0, 2.0, &["a2 e3 c#4 a4"], 1.0);
    s.arps(RIPPLE, 26.0, 1.0, &["a3 c#4 e4 b4"], 8, true, 0.8);

    let melody = "e5:8 a5:8 | c6:12 b5:4 | a5:8 e5:8 | c5:16 | e5:8 g5:8 | c6:12 d6:4 | b5:8 g5:8 | d5:16 |
                  e5:8 a5:8 | c6:8 e6:8 | f6:12 e6:4 | c6:16 | d6:8 c6:4 a5:4 | f5:16 | e5:8 g#5:8 | b5:16";
    let ids = s.line(FLUTE, 4.0, melody, 0.96);
    s.echo(&ids, ECHO, 6.0, 1.0);
    let close_line = s.line(
        FLUTE,
        20.0,
        "a5:16 | b5:16 | g5:16 | e5:16 | c6:16 | d6:16 | ~c#6:32",
        1.0,
    );
    s.echo(&close_line, ECHO, 6.0, 1.0);
    s.line(BELL, 26.0, "a6:16 | e6:16", 1.0);

    Song {
        number: 11,
        key: "gimle",
        title: "Gimle",
        about: "A minor, 64 BPM: a calm, hopeful close that ends on a major chord",
        insts,
        notes: s.notes,
        length: 28.0 * bar + 7.0,
        delay: s.row * 6.0,
        credit: None,
        looped: false,
    }
}
