//! "Fimbulvetr": C minor, 56 BPM. The great winter: dark ambient with wind,
//! deep pads, a slow heartbeat and a far bell melody that rings out.

use super::*;

pub fn song() -> Song {
    let mut s = Score::new(56.0, 4);
    let bar = s.bar();
    const DRONE: usize = 0;
    const CHOIR: usize = 1;
    const BELL: usize = 2;
    const ECHO: usize = 3;
    const RIPPLE: usize = 4;
    const KIT: [usize; 3] = [5, 6, 7];
    const WIND: usize = 8;
    const PAD: usize = 9;
    let insts = vec![
        drone(0.09),
        choir(
            0.03,
            500.0,
            sweep(bar, &[(0.0, 0.5), (12.0, 1.4), (22.0, 1.2), (27.0, 0.4)]),
        ),
        bell(0.11, 0.2),
        Inst {
            vol: 0.045,
            pan: -0.6,
            ..bell(0.045, -0.6)
        },
        ripple(0.08, 1400.0, -0.3),
        Inst {
            vol: 0.35,
            send: 0.4,
            ..kick(0.35)
        },
        snare(0.0),
        hat(0.0),
        wind(0.11, wobble(0.7, 0.09)),
        pad(
            0.035,
            450.0,
            sweep(bar, &[(0.0, 0.5), (14.0, 1.3), (27.0, 0.4)]),
        ),
    ];
    let chords = [
        "c3 g3 eb4 d5",
        "ab2 eb3 c4 g4",
        "f2 c3 ab3 g4",
        "g2 d3 c4 g4",
        "c3 g3 eb4 d5",
        "ab2 eb3 c4 g4",
        "f2 c3 ab3 g4",
        "g2 d3 b3 f4",
        "c3 g3 eb4 d5",
        "ab2 eb3 c4 g4",
        "f2 c3 ab3 g4",
        "c3 g3 eb4 c5",
    ];
    let triads = [
        "c4 eb4 g4",
        "c4 eb4 ab4",
        "c4 f4 ab4",
        "c4 d4 g4",
        "c4 eb4 g4",
        "c4 eb4 ab4",
        "c4 f4 ab4",
        "b3 d4 g4",
    ];

    s.hold(DRONE, 0.0, 26.0, "c2", 1.0);
    s.hold(DRONE, 0.4, 25.0, "g2", 0.6);
    s.hold(WIND, 0.0, 26.0, "c9", 1.0);
    s.pads(CHOIR, 1.0, 2.0, &chords, 1.0);
    s.pads(PAD, 1.0, 2.0, &chords, 1.0);
    s.hold(CHOIR, 25.0, 2.0, "c3", 0.8);
    s.arps(RIPPLE, 9.0, 2.0, &triads, 8, false, 1.0);
    // A slow heartbeat.
    for b in (10..22).step_by(2) {
        s.drums(KIT, b as f32, 1, "k..k............", 1.0);
    }
    let bells = "g5:8 eb6:8 | d6:16 | r:16 | c6:8 bb5:8 | ab5:16 | r:16 | g5:8 f5:8 | ~g5:16 | r:16 |
                 eb6:8 f6:8 | g6:16 | r:16 | f6:8 eb6:8 | d6:16 | r:16 | b5:8 c6:8 | c6:16 | r:16 | r:16 | g5:16";
    let line = s.line(BELL, 4.0, bells, 1.0);
    s.echo(&line, ECHO, 6.0, 1.0);

    Song {
        number: 4,
        key: "fimbulvetr",
        title: "Fimbulvetr",
        about: "C minor, 56 BPM: the great winter, wind, deep pads, a slow heartbeat and far bells",
        insts,
        notes: s.notes,
        length: 27.0 * bar + 7.0,
        delay: s.row * 6.0,
        credit: None,
        looped: false,
    }
}
