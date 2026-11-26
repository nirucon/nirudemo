# Changelog

## 0.2.0

- Song 12, "Gymnopédie": Gymnopédie No. 1 by Erik Satie (1888, public
  domain), arranged for NIRUX in the SID sound (bass, chords with pulse-width
  movement or a soft arpeggio, a lead with delayed vibrato, a resonant filter
  sweep and a quiet echo). It is `nirux-12-gymnopedie.ogg` in the music
  archive, tagged title `Gymnopedie No. 1`, artist `Erik Satie`, track 12,
  and it loops without a seam. It has no demo of its own.
- Songs can carry a credit (title, artist and comment tags) and can be
  rendered as seamless loops; `tools/render-music.sh` gives a looped song one
  measured gain instead of the dynamic loudness filter. Songs 1 to 11 render
  exactly as before.

## 0.1.0

- First release: demo 1, "Ginnungagap", on every screen in the theme colours,
  with its music; any key or click ends it, and a NIRULOCK on the same display
  ends it at once.
- `--into WINDOW [--loop] [--silent]` for the NIRUSCREENSAVER `demo` scene.
- `--list` and `--songs` for the launcher and NIRUX.
- The NIRUX music: eleven original chiptunes (`nirux-1-ginnungagap` to
  `nirux-11-gimle`), rendered from source by a small SID-like synthesizer and
  shipped as `nirudemo-music.tar`.
