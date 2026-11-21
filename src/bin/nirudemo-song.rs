//! Render the demo music to a WAV file: nirudemo-song OUT.wav [INSTRUMENT...]
//! Instrument numbers after the file name render only those (for mixing).
use nirudemo::{song, synth};

fn main() {
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or_else(|| "ginnungagap.wav".into());
    let solo: Vec<usize> = args.filter_map(|a| a.parse().ok()).collect();
    let notes: Vec<_> = song::notes().into_iter().filter(|n| solo.is_empty() || solo.contains(&n.inst)).collect();
    let samples = synth::render(&song::instruments(), &notes, song::LENGTH, song::DELAY);
    std::fs::write(&out, synth::wav_bytes(&samples)).expect("write wav");
    eprintln!("nirudemo-song: {out} ({:.1} s)", song::LENGTH);
}
