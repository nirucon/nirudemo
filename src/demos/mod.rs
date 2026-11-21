//! The demos. Each has a number, a key, a title (the same as its song's), a
//! length and a soundtrack from [`crate::songs`]. A new demo is a new module
//! here and one more entry in [`all`]; everything else (the command line,
//! the launcher's Demos view, the screensaver scene) picks it up.

use crate::gfx::Frame;

mod ginnungagap;

/// What a demo knows about the machine it runs on.
#[derive(Clone, Debug, Default)]
pub struct Context {
    /// The host name, for the greetings.
    pub host: String,
}

pub trait Demo {
    /// Draw the frame at `t` seconds from the start.
    fn draw(&mut self, frame: &mut Frame, t: f32);
}

pub struct Info {
    pub number: u32,
    pub key: &'static str,
    pub title: &'static str,
    /// The number of its song in [`crate::songs`].
    pub song: u32,
    pub seconds: f32,
    pub about: &'static str,
    pub make: fn(&Context) -> Box<dyn Demo>,
}

pub fn all() -> Vec<Info> {
    vec![ginnungagap::info()]
}

/// A demo by number or key; `None` means the first.
pub fn find(which: Option<&str>) -> Option<Info> {
    let list = all();
    match which {
        None => list.into_iter().next(),
        Some(w) => {
            let w = w.trim().to_ascii_lowercase();
            list.into_iter()
                .find(|d| d.number.to_string() == w || d.key == w)
        }
    }
}

/// `--list`: NUMBER, KEY, TITLE, SECONDS and the description, tab separated.
pub fn list_lines() -> Vec<String> {
    all()
        .iter()
        .map(|d| {
            format!(
                "{}\t{}\t{}\t{:.0}\t{}",
                d.number, d.key, d.title, d.seconds, d.about
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_demo_has_a_song_and_draws() {
        let songs = crate::songs::all();
        for d in all() {
            let song = songs
                .iter()
                .find(|s| s.number == d.song)
                .expect("its song exists");
            assert!(
                d.seconds <= song.length + 0.5,
                "{} is longer than its music",
                d.key
            );
            assert_eq!(d.title, song.title, "a demo is named after its song");
            let mut demo = (d.make)(&Context {
                host: "othala".into(),
            });
            let mut f = Frame::new();
            let mut lit = 0;
            for k in 0..200 {
                demo.draw(&mut f, k as f32 * d.seconds / 200.0);
                lit += f.px.iter().filter(|&&p| p > 0).count();
            }
            assert!(lit > 0, "{} draws something", d.key);
        }
        assert_eq!(find(None).unwrap().number, 1);
        assert_eq!(find(Some("Ginnungagap")).unwrap().number, 1);
        assert!(find(Some("99")).is_none());
        assert!(list_lines()[0].starts_with("1\tginnungagap\tGinnungagap\t"));
    }
}
