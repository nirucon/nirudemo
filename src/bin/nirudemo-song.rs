//! Render the NIRUX music to WAV files: `nirudemo-song OUTDIR [SONG...]`
//! writes `nirux-N-key.wav` for every song (or the ones named by number or
//! key). `--list` prints number, key, title and seconds.
use nirudemo::{songs, synth};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--list") {
        for s in songs::all() {
            println!(
                "{}\t{}\t{}\t{:.0}\t{}",
                s.number, s.key, s.title, s.length, s.about
            );
        }
        return;
    }
    let Some(out) = args.first() else {
        eprintln!("usage: nirudemo-song OUTDIR [SONG...] | --list");
        std::process::exit(2);
    };
    std::fs::create_dir_all(out).expect("output folder");
    let wanted = &args[1..];
    for s in songs::all() {
        if !wanted.is_empty()
            && !wanted
                .iter()
                .any(|w| songs::find(w).is_some_and(|f| f.number == s.number))
        {
            continue;
        }
        let path = format!("{out}/{}", s.file_name().replace(".ogg", ".wav"));
        std::fs::write(&path, synth::wav_bytes(&s.render())).expect("write wav");
        eprintln!("nirudemo-song: {path} ({:.1} s)", s.length);
    }
}
