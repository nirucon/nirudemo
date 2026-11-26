#!/bin/sh
# Smoke test of a built nirudemo without a display: tests/smoke.sh BINARY
set -eu
bin=${1:-target/release/nirudemo}
fail() { echo "smoke: $*" >&2; exit 1; }
"$bin" --version | grep -Eq '^NIRUDEMO [0-9]+\.[0-9]+\.[0-9]+$' || fail 'version line'
"$bin" --help | grep -q -- '--into' || fail 'help'
list=$("$bin" --list)
printf '%s\n' "$list" | head -1 | grep -q "^1	ginnungagap	Ginnungagap	[0-9]*	" || fail "list: $list"
printf '%s\n' "$list" | awk -F '\t' 'NF < 5 || $1 !~ /^[0-9]+$/ || $4 !~ /^[0-9]+$/ {bad = 1} END {exit bad}' || fail "list fields: $list"
[ "$("$bin" --songs | wc -l)" -eq 12 ] || fail 'twelve songs'
"$bin" --songs | awk -F '\t' '$5 != "nirux-" $1 "-" $2 ".ogg" {bad = 1} END {exit bad}' || fail 'song file names'
if "$bin" 99 2>/dev/null; then fail 'an unknown demo must fail'; fi
if "$bin" --into nothing 2>/dev/null; then fail 'a bad window id must fail'; fi
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
XDG_CONFIG_HOME=$tmp "$bin" --still 40 "$tmp/still.ppm"
head -c 2 "$tmp/still.ppm" | grep -q P6 || fail 'still image'
echo 'smoke: ok'
