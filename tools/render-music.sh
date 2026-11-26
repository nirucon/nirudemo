#!/bin/sh
# Render the NIRUX music to Ogg Vorbis: tools/render-music.sh [OUTDIR]
# Needs a built nirudemo-song (cargo build --release) and ffmpeg. Every song
# is loudness-normalised to about the same level, so they sit at one level in a
# playlist; a looped song gets one fixed gain so its loop stays seamless.
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)
out=${1:-$here/target/music}
bin=$here/target/release/nirudemo-song
[ -x "$bin" ] || { echo 'render-music: build first (cargo build --release)' >&2; exit 1; }
command -v ffmpeg >/dev/null || { echo 'render-music: ffmpeg is missing' >&2; exit 1; }
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
mkdir -p "$out"
gain=
"$bin" "$tmp"
"$bin" --list | while IFS="$(printf '\t')" read -r number key title _seconds artist looped comment; do
  wav=$tmp/nirux-$number-$key.wav
  if [ "$looped" = 1 ]; then
    # A looped song gets one fixed gain, measured first, so its end still
    # meets its start without a step: to -15 LUFS (where the dynamic
    # loudnorm leaves the others) and at most -1.5 dBTP.
    gain=$(ffmpeg -nostdin -hide_banner -i "$wav" -af loudnorm=I=-16:TP=-1.5:LRA=11:print_format=json \
      -f null - 2>&1 | awk -F '"' '
        $2 == "input_i" { i = $4 } $2 == "input_tp" { tp = $4 }
        END { g = -15 - i; if (tp + g > -1.5) g = -1.5 - tp; printf "%.2f", g }')
    filter="volume=${gain}dB"
  else
    filter=loudnorm=I=-16:TP=-1.5:LRA=11
  fi
  set -- -metadata title="$title" -metadata artist="$artist" -metadata album='NIRUX music' \
    -metadata tracknumber="$number"
  [ -z "$comment" ] || set -- "$@" -metadata comment="$comment"
  ffmpeg -nostdin -loglevel error -y -i "$wav" \
    -af "$filter" -ar 48000 -c:a libvorbis -q:a 5 \
    -map_metadata -1 -fflags +bitexact "$@" "$out/nirux-$number-$key.ogg"
  echo "render-music: nirux-$number-$key.ogg${gain:+ (gain $gain dB)}"
  gain=
done
