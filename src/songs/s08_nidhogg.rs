//! "Nidhogg": B Phrygian, 68 BPM. The dragon gnawing the roots: a heavy
//! growling bass, the dark half step from B to C, slow booms and bells
//! that circle the same few notes.

use super::*;

pub fn song() -> Song {
    let mut s = Score::new(68.0, 4);
    let bar = s.bar();
    const DRONE: usize = 0;
    const GROWL: usize = 1;
    const ARP: usize = 2;
    const KIT: [usize; 3] = [3, 4, 5];
    const BELL: usize = 6;
    const ECHO: usize = 7;
    const WIND: usize = 8;
    const PAD: usize = 9;
    let insts = vec![
        drone(0.09),
        growl(
            0.13,
            160.0,
            sweep(
                bar,
                &[
                    (0.0, 0.8),
                    (8.0, 1.6),
                    (16.0, 2.2),
                    (24.0, 3.5),
                    (27.0, 0.6),
                    (30.0, 0.4),
                ],
            ),
        ),
        arp(
            0.09,
            700.0,
            1200.0,
            sweep(bar, &[(4.0, 0.3), (12.0, 1.0), (24.0, 1.1), (28.0, 0.3)]),
            -0.4,
        ),
        Inst {
            decay: 0.6,
            vol: 0.65,
            send: 0.35,
            ..kick(0.65)
        },
        Inst {
            decay: 0.5,
            cutoff: 900.0,
            send: 0.7,
            ..snare(0.45)
        },
        hat(0.06),
        bell(0.10, 0.25),
        Inst {
            vol: 0.04,
            pan: -0.6,
            ..bell(0.04, -0.6)
        },
        wind(
            0.10,
            sweep(bar, &[(0.0, 0.6), (12.0, 1.4), (24.0, 0.8), (30.0, 1.6)]),
        ),
        pad(0.035, 420.0, wobble(0.4, 0.15)),
    ];
    let loop_chords = [
        "b3 d4 f#4",
        "b3 d4 f#4",
        "c4 e4 g4",
        "b3 d4 f#4",
        "b3 d4 f#4",
        "b3 d4 g4",
        "c4 e4 g4",
        "a#3 c#4 f#4",
    ];
    let pads = [
        "b2 f#3 d4",
        "b2 f#3 d4",
        "c3 g3 e4",
        "b2 f#3 d4",
        "b2 f#3 d4",
        "g2 d3 b3",
        "c3 g3 e4",
        "f#2 c#3 a#3",
    ];
    let roots = ["b1", "b1", "c2", "b1", "b1", "g1", "c2", "f#1"];

    s.hold(DRONE, 0.0, 29.0, "b1", 1.0);
    s.hold(WIND, 0.0, 29.0, "c9", 1.0);
    for k in 0..3 {
        let b = k as f32 * 8.0;
        for (i, r) in roots.iter().enumerate() {
            s.note(GROWL, b + i as f32, 0.0, pitch(r).unwrap(), 10.0, 1.0);
            s.note(GROWL, b + i as f32, 12.0, pitch(r).unwrap() + 1.0, 3.5, 0.7);
        }
        s.pads(PAD, b, 1.0, &pads, 1.0);
        // The arpeggio wakes halfway through the first round.
        let first = if k == 0 { 4 } else { 0 };
        s.arps(
            ARP,
            b + first as f32,
            1.0,
            &loop_chords[first..],
            4,
            false,
            1.0,
        );
    }
    s.drums(KIT, 8.0, 15, "k.......k...s...", 1.0);
    s.drums(KIT, 12.0, 11, "..h...h...h...h.", 1.0);
    s.drums(KIT, 23.0, 1, "k.......k.s.s.ss", 1.0);
    s.pads(PAD, 24.0, 2.0, &["b2 f#3 d4", "c3 g3 e4", "b2 f#3 b3"], 1.0);
    s.note(GROWL, 24.0, 0.0, pitch("b1").unwrap(), 48.0, 1.0);
    s.note(GROWL, 27.0, 0.0, pitch("b0").unwrap(), 32.0, 0.9);

    let bells = "b5:8 c6:8 | b5:16 | r:16 | f#5:8 g5:8 | f#5:16 | r:16 | d6:8 c6:8 | b5:8 a5:8 | g5:16 | f#5:8 g5:4 f#5:4 | ~b4:16 | r:16";
    let ids = s.line(BELL, 12.0, bells, 1.0);
    s.echo(&ids, ECHO, 6.0, 1.0);
    let end = s.line(BELL, 25.0, "c6:16 | b5:16 | r:16 | b4:16", 1.0);
    s.echo(&end, ECHO, 6.0, 1.0);

    Song {
        number: 8,
        key: "nidhogg",
        title: "Nidhogg",
        about: "B Phrygian, 68 BPM: a heavy growl, the dark half step from B to C, slow booms and circling bells",
        insts,
        notes: s.notes,
        length: 29.0 * bar + 7.0,
        delay: s.row * 6.0,
        credit: None,
        looped: false,
    }
}
