//! "Surtr": E minor, 104 BPM. The fire giant: the most driving piece, full
//! C64 style with octave arpeggios, a running bass, drums, a growling
//! breakdown and a high climax.

use super::*;

pub fn song() -> Song {
    let mut s = Score::new(104.0, 4);
    let bar = s.bar();
    const DRONE: usize = 0;
    const PAD: usize = 1;
    const ARP: usize = 2;
    const BASS: usize = 3;
    const KIT: [usize; 3] = [4, 5, 6];
    const LEAD: usize = 7;
    const SHADOW: usize = 8;
    const GROWL: usize = 9;
    let insts = vec![
        drone(0.07),
        pad(
            0.045,
            600.0,
            sweep(bar, &[(0.0, 0.6), (8.0, 1.5), (40.0, 1.5), (44.0, 0.4)]),
        ),
        arp(
            0.12,
            1000.0,
            2000.0,
            sweep(
                bar,
                &[
                    (0.0, 0.25),
                    (4.0, 1.0),
                    (24.0, 1.2),
                    (28.0, 0.5),
                    (32.0, 1.3),
                    (44.0, 0.4),
                ],
            ),
            -0.35,
        ),
        bass(0.25, 360.0, 1100.0),
        kick(0.6),
        snare(0.55),
        hat(0.11),
        lead(0.16, 2000.0, 0.15),
        shadow(0.065, 1200.0, -0.55),
        growl(
            0.11,
            220.0,
            sweep(bar, &[(24.0, 0.6), (31.0, 4.0), (32.0, 0.6)]),
        ),
    ];
    let arps1 = ["e4 g4 b4", "c4 e4 g4", "d4 f#4 a4", "d#4 f#4 b4"];
    let arps2 = ["e4 g4 b4", "d4 g4 b4", "c4 e4 a4", "d#4 f#4 b4"];
    let pads1 = ["e2 b2 g3", "c2 g2 e3", "d2 a2 f#3", "b1 f#2 d#3"];
    let pads2 = ["e2 b2 g3", "g2 d3 b3", "a2 e3 c4", "b1 f#2 d#3"];
    let roots1 = ["e2", "c2", "d2", "b1"];
    let roots2 = ["e2", "g2", "a2", "b1"];

    s.hold(DRONE, 0.0, 43.0, "e2", 1.0);
    for k in 0..11 {
        let b = k as f32 * 4.0;
        let (a, p, r) = if k % 2 == 0 {
            (&arps1, &pads1, &roots1)
        } else {
            (&arps2, &pads2, &roots2)
        };
        s.arps(ARP, b, 1.0, a, 2, true, 1.0);
        s.pads(PAD, b, 1.0, p, 1.0);
        if (1..10).contains(&k) {
            s.bass(BASS, b, r, "0o0o5o0o0o0o7o5o", 0.8, 1.0);
        }
        if (6..8).contains(&k) {
            for (i, root) in r.iter().enumerate() {
                s.note(
                    GROWL,
                    b + i as f32,
                    0.0,
                    pitch(root).unwrap() - 12.0,
                    15.0,
                    1.0,
                );
            }
        }
    }
    s.drums(KIT, 8.0, 15, "k.h.s.hkk.hhs.hH", 1.0);
    s.drums(KIT, 23.0, 1, "k.h.s.hkk.s.ssss", 1.0);
    s.drums(KIT, 24.0, 8, "k.......k.......", 0.8);
    s.drums(KIT, 32.0, 7, "k.h.s.hkk.hhs.hH", 1.0);
    s.drums(KIT, 39.0, 1, "k.k.s.k.s.s.ssss", 1.0);
    s.drums(KIT, 40.0, 1, "k...............", 1.2);

    let a = "b4:2 e5:2 g5:2 b5:6 a5:2 g5:2 | ~e5:6 g5:2 c6:8 | a5:4 f#5:4 d5:4 a5:4 | b5:12 a5:2 f#5:2 |
             g5:4 e5:2 g5:2 b5:4 e6:4 | d6:6 b5:2 g5:8 | c6:4 b5:2 a5:2 e5:4 c6:4 | ~b5:8 d#6:4 f#6:4";
    let b = "e6:8 d6:4 b5:4 | c6:6 b5:2 g5:8 | a5:4 d6:4 f#6:8 | ~d#6:12 b5:4 |
             e6:2 f#6:2 g6:4 f#6:4 e6:4 | d6:4 b5:4 g5:4 b5:4 | c6:4 e6:4 a6:8 | ~b5:16";
    for (at, text) in [(8.0, a), (16.0, a), (32.0, b)] {
        let ids = s.line(LEAD, at, text, 0.92);
        s.echo(&ids, SHADOW, 3.0, 1.0);
    }
    let end = s.line(LEAD, 40.0, "e6:16 | b5:16 | g5:16 | e5:16", 1.0);
    s.echo(&end, SHADOW, 3.0, 1.0);

    Song {
        number: 6,
        key: "surtr",
        title: "Surtr",
        about: "E minor, 104 BPM: the fire giant, a running bass, drums, a growling breakdown and a high climax",
        insts,
        notes: s.notes,
        length: 44.0 * bar + 6.0,
        delay: s.row * 3.0,
        credit: None,
        looped: false,
    }
}
