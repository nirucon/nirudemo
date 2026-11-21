//! "Ragnarok": C sharp minor, 96 BPM. The end of the world: a heartbeat
//! out of the dark, the full C64 band, a high climax with a rising growl,
//! a sudden collapse into wind, and one bell in the ashes.

use super::*;

pub fn song() -> Song {
    let mut s = Score::new(96.0, 4);
    let bar = s.bar();
    const DRONE: usize = 0;
    const PAD: usize = 1;
    const ARP: usize = 2;
    const BASS: usize = 3;
    const KIT: [usize; 3] = [4, 5, 6];
    const LEAD: usize = 7;
    const SHADOW: usize = 8;
    const GROWL: usize = 9;
    const WIND: usize = 10;
    const BELL: usize = 11;
    let insts = vec![
        drone(0.08),
        pad(
            0.045,
            560.0,
            sweep(bar, &[(0.0, 0.4), (16.0, 1.5), (38.0, 1.8), (39.0, 0.3)]),
        ),
        arp(
            0.12,
            950.0,
            1900.0,
            sweep(bar, &[(8.0, 0.3), (16.0, 1.0), (32.0, 1.1), (38.0, 1.6)]),
            -0.35,
        ),
        bass(0.25, 340.0, 1100.0),
        kick(0.6),
        snare(0.55),
        hat(0.10),
        lead(0.16, 2000.0, 0.15),
        shadow(0.065, 1200.0, -0.55),
        growl(0.11, 200.0, sweep(bar, &[(32.0, 0.6), (38.0, 5.0)])),
        wind(
            0.13,
            sweep(bar, &[(0.0, 0.5), (8.0, 1.0), (38.0, 2.5), (44.0, 0.6)]),
        ),
        bell(0.11, 0.2),
    ];
    let arps1 = ["c#4 e4 g#4", "c#4 e4 a4", "d#4 f#4 b4", "c4 d#4 g#4"];
    let arps2 = ["c#4 e4 g#4", "c#4 f#4 a4", "c#4 e4 a4", "c4 d#4 g#4"];
    let pads1 = ["c#3 g#3 e4", "a2 e3 c#4", "b2 f#3 d#4", "g#2 d#3 c4"];
    let pads2 = ["c#3 g#3 e4", "f#2 c#3 a3", "a2 e3 c#4", "g#2 d#3 c4"];
    let roots1 = ["c#2", "a1", "b1", "g#1"];
    let roots2 = ["c#2", "f#2", "a1", "g#1"];

    s.hold(DRONE, 0.0, 44.0, "c#2", 1.0);
    s.hold(WIND, 0.0, 40.0, "c9", 1.0);
    // A heartbeat in the dark, then a ticking clock.
    s.drums(KIT, 0.0, 8, "k..k............", 0.7);
    s.drums(KIT, 4.0, 4, "h...h...h...h...", 0.6);
    for k in 0..8 {
        let b = 8.0 + k as f32 * 4.0;
        let (a, p, r) = if k % 2 == 0 {
            (&arps1, &pads1, &roots1)
        } else {
            (&arps2, &pads2, &roots2)
        };
        s.arps(ARP, b, 1.0, a, if k < 6 { 4 } else { 2 }, true, 1.0);
        s.pads(PAD, b, 1.0, p, 1.0);
        s.bass(
            BASS,
            b,
            r,
            if k < 2 {
                "0...o...0...o.5."
            } else {
                "0o0o5o0o0o0o7o5o"
            },
            0.8,
            1.0,
        );
        if k >= 6 {
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
    s.pads(PAD, 4.0, 4.0, &["c#3 g#3 e4"], 1.0);
    s.drums(KIT, 16.0, 15, "k.h.s.hkk.hhs.hH", 1.0);
    s.drums(KIT, 31.0, 1, "k.h.s.hkk.s.ssss", 1.0);
    s.drums(KIT, 32.0, 6, "k.hks.hkk.hks.hH", 1.1);
    s.drums(KIT, 38.0, 1, "k.k.k.k.ssssssss", 1.2);

    let a = "g#4:4 c#5:4 e5:4 g#5:4 | a5:8 e5:8 | f#5:4 d#5:4 b4:4 f#5:4 | ~g#5:12 c5:4 |
             c#5:2 e5:2 g#5:4 c#6:8 | b5:4 a5:4 f#5:8 | e5:4 a5:4 c#6:4 e6:4 | ~d#6:16";
    let b = "c#6:2 b5:2 c#6:2 e6:2 g#6:8 | a6:6 g#6:2 e6:8 | f#6:4 d#6:4 b5:4 d#6:4 | c6:4 d#6:4 g#6:8 |
             e6:2 d#6:2 c#6:4 g#5:4 c#6:4 | a5:4 c#6:4 f#6:8 | e6:4 c#6:4 a5:4 e6:4 | ~g#6:16";
    for (at, text) in [(16.0, a), (24.0, a), (32.0, b)] {
        let ids = s.line(LEAD, at, text, 0.92);
        s.echo(&ids, SHADOW, 3.0, 1.0);
    }
    // The collapse: one last hit, then only wind, the drone and a bell.
    s.note(GROWL, 39.0, 0.0, pitch("c#1").unwrap(), 8.0, 1.0);
    s.pads(PAD, 40.0, 4.0, &["c#3 g#3 e4"], 0.7);
    let ash = s.line(BELL, 40.0, "g#5:16 | e5:16 | c#5:16 | g#4:16", 1.0);
    s.echo(&ash, SHADOW, 6.0, 0.6);

    Song {
        number: 10,
        key: "ragnarok",
        title: "Ragnarok",
        about: "C sharp minor, 96 BPM: a heartbeat, the full band, a rising climax and a collapse into wind",
        insts,
        notes: s.notes,
        length: 44.0 * bar + 7.0,
        delay: s.row * 3.0,
    }
}
