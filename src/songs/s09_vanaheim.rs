//! "Vanaheim": F minor, 3/4 at 66 BPM. The home of the Vanir: a pastoral,
//! Satie-like waltz, a soft low note and chord in each bar, a flute melody
//! and a slow ripple above.

use super::s03_hel::sway;
use super::*;

pub fn song() -> Song {
    let mut s = Score::new(66.0, 3);
    let bar = s.bar();
    const DRONE: usize = 0;
    const CHOIR: usize = 1;
    const KEYS: usize = 2;
    const FLUTE: usize = 3;
    const ECHO: usize = 4;
    const RIPPLE: usize = 5;
    let insts = vec![
        drone(0.055),
        choir(
            0.02,
            560.0,
            sweep(bar, &[(0.0, 0.6), (20.0, 1.3), (28.0, 1.0), (40.0, 0.5)]),
        ),
        keys(0.13, 950.0, -0.15),
        flute(0.12, 0.15),
        Inst {
            vol: 0.05,
            pan: -0.5,
            send: 0.9,
            ..flute(0.05, -0.5)
        },
        ripple(0.05, 1800.0, 0.4),
    ];
    let fm = ("f2", "ab3 c4 eb4");
    let db = ("db2", "ab3 c4 f4");
    let bbm = ("bb1", "ab3 db4 f4");
    let c = ("c2", "bb3 e4 g4");
    let eb = ("eb2", "g3 bb3 db4");
    let cycle: Vec<_> = (0..16).map(|i| [fm, db, bbm, c][i % 4]).collect();

    s.hold(DRONE, 0.0, 39.0, "f2", 1.0);
    sway(&mut s, KEYS, 0.0, &cycle[..4], 1.0);
    sway(&mut s, KEYS, 4.0, &cycle, 1.0);
    sway(&mut s, KEYS, 20.0, &[db, db, bbm, bbm, eb, eb, c, c], 1.0);
    sway(&mut s, KEYS, 28.0, &cycle[..8], 1.0);
    sway(&mut s, KEYS, 36.0, &[fm, db, fm], 0.8);
    s.pads(
        CHOIR,
        4.0,
        4.0,
        &["f3 c4 ab4", "f3 c4 ab4", "f3 c4 ab4", "f3 c4 ab4"],
        1.0,
    );
    s.pads(
        CHOIR,
        20.0,
        2.0,
        &["db3 ab3 f4", "bb2 f3 db4", "eb3 bb3 g4", "c3 g3 e4"],
        1.0,
    );
    s.pads(CHOIR, 36.0, 3.0, &["f3 c4 ab4"], 1.0);
    s.arps(
        RIPPLE,
        20.0,
        2.0,
        &["ab4 db5 f5", "bb4 db5 f5", "bb4 eb5 g5", "c5 e5 g5"],
        12,
        true,
        1.0,
    );

    let a = "c5:12 | ab4:8 f4:4 | bb4:8 db5:4 | ~e4:12 | f4:4 ab4:4 c5:4 | db5:8 c5:4 | bb4:4 ab4:4 f4:4 | g4:12 |
             c5:8 eb5:4 | f5:8 db5:4 | bb4:4 c5:4 db5:4 | e5:8 g5:4 | f5:12 | f5:8 eb5:2 db5:2 | db5:8 bb4:4 | c5:12";
    let b =
        "f5:8 ab5:4 | ~bb5:8 ab5:4 | f5:12 | db5:12 | g5:8 bb5:4 | eb6:8 db6:4 | c6:12 | ~e5:12";
    let last = "c5:12 | ab4:8 f4:4 | bb4:8 db5:4 | ~e4:12 | f4:4 ab4:4 c5:4 | db5:8 c5:4 | bb4:4 g4:4 e4:4 | f4:36";
    for (at, text) in [(4.0, a), (20.0, b), (28.0, last)] {
        let ids = s.line(FLUTE, at, text, 0.97);
        s.echo(&ids, ECHO, 6.0, 1.0);
    }

    Song {
        number: 9,
        key: "vanaheim",
        title: "Vanaheim",
        about:
            "F minor, 3/4 at 66 BPM: a pastoral, Satie-like waltz with a flute and a slow ripple",
        insts,
        notes: s.notes,
        length: 39.0 * bar + 6.0,
        delay: s.row * 6.0,
        credit: None,
        looped: false,
    }
}
