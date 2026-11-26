//! Render the NIRUX music to WAV files: `nirudemo-song OUTDIR [SONG...]`
//! writes `nirux-N-key.wav` for every song (or the ones named by number or
//! key). `--list` prints, tab separated, the number, key, title tag,
//! seconds, artist tag, 1 for a looped song (else 0) and the comment tag
//! (empty for NIRUX's own songs), for `tools/render-music.sh`.
use nirudemo::{songs, synth};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--list") {
        for s in songs::all() {
            let (title, artist, comment) = s.tags();
            println!(
                "{}\t{}\t{}\t{:.0}\t{}\t{}\t{}",
                s.number,
                s.key,
                title,
                s.length,
                artist,
                u8::from(s.looped),
                comment
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
