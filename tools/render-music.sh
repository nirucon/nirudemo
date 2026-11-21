#!/bin/sh
# Render the NIRUX music to Ogg Vorbis: tools/render-music.sh [OUTDIR]
# Needs a built nirudemo-song (cargo build --release) and ffmpeg. Every song
# is loudness-normalised the same way, so they sit at one level in a playlist.
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)
out=${1:-$here/target/music}
bin=$here/target/release/nirudemo-song
[ -x "$bin" ] || { echo 'render-music: build first (cargo build --release)' >&2; exit 1; }
command -v ffmpeg >/dev/null || { echo 'render-music: ffmpeg is missing' >&2; exit 1; }
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
mkdir -p "$out"
"$bin" "$tmp"
"$bin" --list | while IFS="$(printf '\t')" read -r number key title _seconds _about; do
  ffmpeg -nostdin -loglevel error -y -i "$tmp/nirux-$number-$key.wav" \
    -af loudnorm=I=-16:TP=-1.5:LRA=11 -ar 48000 -c:a libvorbis -q:a 5 \
    -map_metadata -1 -fflags +bitexact \
    -metadata title="$title" -metadata artist=NIRUX -metadata album='NIRUX music' \
    -metadata tracknumber="$number" "$out/nirux-$number-$key.ogg"
  echo "render-music: nirux-$number-$key.ogg"
done
