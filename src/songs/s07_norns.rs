//! "Norns": G minor, 72 BPM. Urd, Verdandi and Skuld weave fate: three
//! short figures of different lengths drift against each other, and a
//! flute sings above the threads.

use super::*;

/// A figure of `notes`, `rows` each, repeated from `from` to `to` (bars).
fn thread(s: &mut Score, inst: usize, from: f32, to: f32, notes: &str, rows: f32, vel: f32) {
    let p = pitches(notes);
    let mut row = 0.0;
    let end = (to - from) * s.rows_per_bar as f32;
    let mut i = 0;
    while row < end {
        s.note(inst, from, row, p[i % p.len()], rows * 0.8, vel);
        row += rows;
        i += 1;
    }
}

pub fn song() -> Song {
    let mut s = Score::new(72.0, 4);
    let bar = s.bar();
    const DRONE: usize = 0;
    const PAD: usize = 1;
    const URD: usize = 2;
    const VERDANDI: usize = 3;
    const SKULD: usize = 4;
    const BASS: usize = 5;
    const FLUTE: usize = 6;
    const ECHO: usize = 7;
    const KIT: [usize; 3] = [8, 9, 10];
    let insts = vec![
        drone(0.07),
        pad(
            0.04,
            480.0,
            sweep(bar, &[(0.0, 0.5), (14.0, 1.4), (30.0, 1.4), (34.0, 0.4)]),
        ),
        Inst {
            decay: 0.5,
            ..keys(0.09, 1100.0, -0.45)
        },
        Inst {
            decay: 0.7,
            ..keys(0.07, 1500.0, 0.45)
        },
        bell(0.06, 0.0),
        Inst {
            decay: 0.6,
            sustain: 0.5,
            release: 0.3,
            ..bass(0.2, 300.0, 600.0)
        },
        flute(0.12, 0.1),
        Inst {
            vol: 0.05,
            pan: -0.5,
            send: 0.9,
            ..flute(0.05, -0.5)
        },
        kick(0.0),
        snare(0.0),
        hat(0.07),
    ];
    let chords = [
        "g2 d3 bb3",
        "eb2 bb2 g3",
        "c3 g3 eb4",
        "d3 a3 f#4",
        "g2 d3 bb3",
        "bb2 f3 d4",
        "f2 c3 a3",
        "d3 a3 f#4",
    ];
    let roots = ["g1", "eb2", "c2", "d2", "g1", "bb1", "f2", "d2"];

    s.hold(DRONE, 0.0, 33.0, "g1", 1.0);
    for k in 0..2 {
        let b = 1.0 + k as f32 * 16.0;
        s.pads(PAD, b, 2.0, &chords, 1.0);
        for (i, r) in roots.iter().enumerate() {
            let rb = b + i as f32 * 2.0;
            for half in [0.0, 8.0, 16.0, 24.0] {
                s.note(
                    BASS,
                    rb,
                    half,
                    pitch(r).unwrap(),
                    6.0,
                    if half == 0.0 { 1.0 } else { 0.7 },
                );
            }
        }
    }
    s.pads(PAD, 33.0, 2.0, &["g2 d3 bb3 g4"], 1.0);
    thread(&mut s, URD, 1.0, 31.0, "g4 d5 bb4 d5 c5 d5", 2.0, 1.0);
    thread(&mut s, VERDANDI, 5.0, 29.0, "bb5 g5 d5 g5 c6", 3.0, 1.0);
    thread(&mut s, SKULD, 9.0, 27.0, "d6 bb5 g6", 5.0, 1.0);
    s.drums(KIT, 9.0, 18, "..h...h...h...hh", 1.0);

    let melody = "d5:8 g5:8 | bb5:12 a5:4 | g5:8 eb5:8 | g5:16 | c5:8 eb5:4 g5:4 | f5:12 eb5:4 | d5:8 f#5:8 | a5:16 |
                  bb5:8 a5:4 g5:4 | d5:16 | f5:8 bb5:8 | d6:16 | c6:8 a5:8 | f5:16 | f#5:8 a5:8 | d5:16";
    let ids = s.line(FLUTE, 17.0, melody, 0.96);
    s.echo(&ids, ECHO, 6.0, 1.0);
    let end = s.line(FLUTE, 33.0, "~g5:32", 1.0);
    s.echo(&end, ECHO, 6.0, 1.0);

    Song {
        number: 7,
        key: "norns",
        title: "Norns",
        about: "G minor, 72 BPM: three short figures of different lengths drift against each other",
        insts,
        notes: s.notes,
        length: 35.0 * bar + 6.0,
        delay: s.row * 3.0,
        credit: None,
        looped: false,
    }
}
