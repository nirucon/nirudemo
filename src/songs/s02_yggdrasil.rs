//! "Yggdrasil": A minor, 84 BPM. The world tree rising: a heroic but dark
//! C64 piece with a bouncing bass, fast arpeggios and two lead themes.

use super::*;

pub fn song() -> Song {
    let mut s = Score::new(84.0, 4);
    let bar = s.bar();
    const DRONE: usize = 0;
    const PAD: usize = 1;
    const ARP: usize = 2;
    const BASS: usize = 3;
    const KIT: [usize; 3] = [4, 5, 6];
    const LEAD: usize = 7;
    const SHADOW: usize = 8;
    const WIND: usize = 9;
    let insts = vec![
        drone(0.085),
        pad(
            0.05,
            520.0,
            sweep(bar, &[(0.0, 0.4), (8.0, 1.5), (28.0, 1.6), (36.0, 0.4)]),
        ),
        arp(
            0.12,
            900.0,
            1800.0,
            sweep(bar, &[(4.0, 0.3), (12.0, 1.0), (28.0, 1.1), (32.0, 0.4)]),
            -0.35,
        ),
        bass(0.24, 330.0, 1000.0),
        kick(0.55),
        snare(0.5),
        hat(0.10),
        lead(0.16, 1800.0, 0.15),
        shadow(0.065, 1100.0, -0.55),
        wind(0.08, wobble(0.6, 0.13)),
    ];
    let loop_pads = [
        "a2 e3 c4",
        "f2 c3 a3",
        "c3 g3 e4",
        "g2 d3 b3",
        "a2 e3 c4",
        "f2 c3 a3",
        "e2 b2 g#3",
        "e2 b2 g#3",
    ];
    let loop_arps = [
        "a3 c4 e4",
        "f3 a3 c4",
        "g3 c4 e4",
        "g3 b3 d4",
        "a3 c4 e4",
        "f3 a3 c4",
        "g#3 b3 e4",
        "g#3 b3 e4",
    ];
    let roots = ["a2", "f2", "c3", "g2", "a2", "f2", "e2", "e2"];

    s.hold(DRONE, 0.0, 35.0, "a1", 1.0);
    s.hold(DRONE, 0.2, 34.0, "e2", 0.6);
    s.hold(WIND, 0.0, 8.0, "c9", 1.0);
    s.hold(WIND, 30.0, 6.0, "c9", 0.9);
    s.pads(PAD, 0.0, 4.0, &["a2 e3 c4"], 1.0);
    for k in 0..3 {
        s.pads(PAD, 4.0 + k as f32 * 8.0, 1.0, &loop_pads, 1.0);
        s.arps(ARP, 4.0 + k as f32 * 8.0, 1.0, &loop_arps, 4, true, 1.0);
    }
    for k in 0..3 {
        let pattern = if k == 0 {
            "0..o0.o.0..o5.o7"
        } else {
            "00o00o0o00o05.o7"
        };
        s.bass(BASS, 8.0 + k as f32 * 8.0, &roots, pattern, 0.85, 1.0);
    }
    s.drums(KIT, 12.0, 15, "k...s..hk.k.s..h", 1.0);
    s.drums(KIT, 27.0, 1, "k...s..hk.k.ssss", 1.0);

    let theme_a =
        "a4:4 c5:2 e5:2 a5:6 g5:2 | f5:6 e5:2 c5:8 | e5:4 g5:4 ~c6:6 b5:2 | d5:8 b4:4 g4:4 |
                   a4:2 b4:2 c5:4 e5:8 | f5:4 a5:4 ~g5:4 f5:4 | e5:6 d5:2 b4:8 | g#4:8 ~b4:8";
    let theme_b = "e5:2 a5:2 c6:4 b5:2 a5:2 e5:4 | f5:2 a5:2 c6:6 ~d6:2 c6:4 | e6:6 d6:2 c6:4 g5:4 | b5:6 a5:2 g5:4 d5:4 |
                   c6:4 b5:2 a5:2 e5:4 a5:4 | a5:4 c6:4 f6:6 e6:2 | ~e6:8 d6:4 b5:4 | g#5:6 a5:2 b5:8";
    let a = s.line(LEAD, 12.0, theme_a, 0.95);
    s.echo(&a, SHADOW, 3.0, 1.0);
    let b = s.line(LEAD, 20.0, theme_b, 0.95);
    s.echo(&b, SHADOW, 3.0, 1.0);

    // Breakdown and outro: the arpeggio alone, then the last chords.
    s.pads(
        PAD,
        28.0,
        1.0,
        &["a2 e3 c4", "f2 c3 a3", "e2 b2 g#3", "e2 b2 g#3"],
        1.0,
    );
    s.arps(
        ARP,
        28.0,
        1.0,
        &["a3 c4 e4", "f3 a3 c4", "g#3 b3 e4", "g#3 b3 e4"],
        4,
        true,
        0.9,
    );
    s.pads(PAD, 32.0, 1.0, &["a2 e3 c4", "f2 c3 a3", "e2 b2 g#3"], 1.0);
    s.pads(PAD, 35.0, 2.0, &["a2 e3 c4 a4"], 1.0);
    s.arps(
        ARP,
        32.0,
        1.0,
        &["a3 c4 e4", "f3 a3 c4", "g#3 b3 e4"],
        8,
        true,
        0.7,
    );
    let out = s.line(
        LEAD,
        28.0,
        "a5:16 | r:16 | e5:16 | r:16 | c5:16 | b4:16 | g#4:16 | ~a4:24",
        1.0,
    );
    s.echo(&out, SHADOW, 3.0, 1.0);

    let length = 37.0 * bar + 6.0;
    Song {
        number: 2,
        key: "yggdrasil",
        title: "Yggdrasil",
        about: "A minor, 84 BPM: the world tree rising, a bouncing bass and two lead themes",
        insts,
        notes: s.notes,
        length,
        delay: s.row * 3.0,
    }
}
