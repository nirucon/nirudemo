# NIRUDEMO

Demoscene demos for NIRUX, in your theme colours, and the NIRUX music.

**Status: beta** · Part of the [NIRUX](https://github.com/nirucon/nirux) suite · [MIT License](LICENSE)

## About

NIRUDEMO is the NIRUX easter egg: a small realtime demo in the spirit of the
home computer and console intros of the 1980s and 1990s, with stars, a
low-poly 3D logo in fog, plasma, a sine scroller and a tunnel. It draws at
320 by 180 pixels and scales that up with sharp pixels on every screen, and
all its colours come from the active NIRUX theme.

It also makes the NIRUX music: eleven original chiptunes in minor keys,
between dark ambient, the C64 sound (fast arpeggios, pulse bass, vibrato and
echo) and calm pieces in the spirit of Erik Satie. They are written as notes
in Rust and rendered by a small SID-like synthesizer, so every release builds
the same music from source. NIRUX installs them in
`~/.local/share/nirux/music/` as `nirux-1-ginnungagap.ogg` to
`nirux-11-gimle.ogg`, and they are the default pause music of NIRULOCK and
the NIRUSCREENSAVER music scenes.

## Use

| Command | Does |
| --- | --- |
| `nirudemo` | Plays demo 1 on every screen with its music. Any key or click ends it. |
| `nirudemo N` or `nirudemo KEY` | Plays demo N (or by name, such as `ginnungagap`). |
| `nirudemo --list` | Lists the demos: number, key, title, seconds and a description, tab separated. |
| `nirudemo --songs` | Lists the music: number, key, title, seconds, file name and a description. |
| `nirudemo --into WINDOW [--loop] [--silent]` | Draws into an existing window and takes no input (the NIRUSCREENSAVER `demo` scene). |

In NIRUX the demo is also on `Super+Alt+D`, on `demo` in the launcher
(`Super+P`), and in the launcher's Demos view (`nirulauncher --demos`).

The music plays with `pw-play`, `ffplay` or `mpv`, whichever is installed
first. It stays off with `--silent` and while Do Not Disturb is on. When a
NIRULOCK starts on the same display, the demo ends at once, so it never holds
the keyboard the lock needs.

## The demos

| # | Title | Music | Parts |
| --- | --- | --- | --- |
| 1 | Ginnungagap | nirux-1-ginnungagap | Stars out of the void, the title, raster bars, a turning N over a grid in fog, plasma with a sine scroller, a tunnel, greetings |

## The music

| # | Title | Character |
| --- | --- | --- |
| 1 | Ginnungagap | D minor, 76 BPM: the void, a waking arpeggio, a bouncing bass and a lead with its echo |
| 2 | Yggdrasil | A minor, 84 BPM: the world tree rising, a bouncing bass and two lead themes |
| 3 | Hel | E minor, 3/4: a calm, Satie-like sway between two chords |
| 4 | Fimbulvetr | C minor, 56 BPM: wind, deep pads, a slow heartbeat and far bells |
| 5 | Mimir | F sharp minor, 60 BPM: slow and free like a gnossienne |
| 6 | Surtr | E minor, 104 BPM: the most driving piece, with a growling breakdown |
| 7 | Norns | G minor, 72 BPM: three figures of different lengths drift against each other |
| 8 | Nidhogg | B Phrygian, 68 BPM: a heavy growl and circling bells |
| 9 | Vanaheim | F minor, 3/4: a pastoral, Satie-like waltz |
| 10 | Ragnarok | C sharp minor, 96 BPM: a heartbeat, a climax and a collapse into wind |
| 11 | Gimle | A minor, 64 BPM: a calm, hopeful close that ends on a major chord |

## Adding a demo or a song

- A song is a module in `src/songs/` written with the `Score` notation (notes
  such as `d5:4 ~f5!:8`, chords, arpeggios, bass and drum patterns) and one
  line in `songs::all()`.
- A demo is a module in `src/demos/` that implements `Demo::draw(frame, t)`
  and one line in `demos::all()`, naming its song. The command line, the
  launcher's Demos view and the screensaver scene pick it up by themselves.
- `cargo build --release && tools/render-music.sh` renders the music to
  `target/music/`; `nirudemo --still SECONDS FILE.ppm` draws one frame
  without a display.

## Build

```sh
cargo build --release
sh tests/smoke.sh target/release/nirudemo
```

Rust 1.75 or later. The release builds a static binary and the music archive
`nirudemo-music.tar`; NIRUX installs both.
