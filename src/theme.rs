//! The NIRUX theme colours, read the way NIRUSCREENSAVER reads them: the
//! theme named in `~/.config/niruwm/config.toml`, a user theme file in
//! `~/.config/niruwm/themes/NAME.toml`, or `appearance.toml`, falling back
//! to the built-in suite themes and Niru Noir. The demo always draws on
//! black; these colours become its ramp from dark to bright.

use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub foreground: u32,
    pub muted: u32,
    pub border: u32,
    pub focus: u32,
}

impl Default for Palette {
    fn default() -> Self {
        builtin("niru-noir").unwrap()
    }
}

pub fn builtin(name: &str) -> Option<Palette> {
    let (foreground, muted, border, focus) = match name {
        "niru-noir" => (0xe5e5e5, 0xa8a8a8, 0x2a2a2d, 0x5a5a60),
        "niru-nord" => (0xeceff4, 0x9aa5b1, 0x3b4252, 0x88c0d0),
        "niru-satie" => (0xdedad4, 0x8f8b86, 0x2a2724, 0xb08d7d),
        "niru-carl-larsson" => (0x3b3228, 0x7d6f5f, 0xd8cdbb, 0xa63d2f),
        "niru-forest-vigil" => (0xd8ded2, 0x96a292, 0x2b362c, 0x8aa87e),
        "niru-blood-of-urdr" => (0xe8d6cf, 0xb09a92, 0x3a1d1b, 0xc96050),
        "niru-root-and-stone" => (0xeadfd2, 0xb5a08c, 0x3a2a1e, 0xc19768),
        "niru-blue-horizon" => (0xdce6eb, 0x93a6b2, 0x243846, 0x86aec8),
        "niru-the-norns" => (0xece9e4, 0xa6a19b, 0x3b3936, 0xc9c2b9),
        "niru-winter-nights" => (0xead9c3, 0xb9aa9a, 0x4e5157, 0xd8c4aa),
        _ => return None,
    };
    Some(Palette {
        foreground,
        muted,
        border,
        focus,
    })
}

pub fn parse_hex(s: &str) -> Option<u32> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    u32::from_str_radix(s, 16).ok()
}

fn config_base() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config")
        })
}

fn read_toml(path: &Path) -> Option<toml::Value> {
    std::fs::read_to_string(path).ok()?.parse().ok()
}

fn apply(result: &mut Palette, get: impl Fn(&str) -> Option<u32>) {
    for (field, key) in [
        (&mut result.foreground, "foreground"),
        (&mut result.muted, "muted"),
        (&mut result.border, "border"),
        (&mut result.focus, "focus"),
    ] {
        if let Some(c) = get(key) {
            *field = c;
        }
    }
}

/// The palette under `base` (normally `~/.config`).
pub fn load_from(base: &Path) -> Palette {
    let name = read_toml(&base.join("niruwm/config.toml"))
        .and_then(|v| v.get("theme").and_then(|t| t.as_str()).map(str::to_string))
        .unwrap_or_else(|| "niru-noir".into());
    let mut result = builtin(&name).unwrap_or_default();
    let file = base.join("niruwm/themes").join(format!("{name}.toml"));
    if let Some(theme) = read_toml(&file) {
        apply(&mut result, |k| {
            theme.get(k).and_then(|v| v.as_str()).and_then(parse_hex)
        });
    } else if let Some(app) = read_toml(&base.join("niruwm/appearance.toml")) {
        if app.get("schema").and_then(|v| v.as_integer()) == Some(1) {
            apply(&mut result, |k| {
                app.get("colors")?.get(k)?.as_str().and_then(parse_hex)
            });
        }
    }
    result
}

pub fn load() -> Palette {
    load_from(&config_base())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn themes_are_read_like_the_screensaver() {
        let dir = std::env::temp_dir().join(format!("nirudemo-theme-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("niruwm/themes")).unwrap();
        assert_eq!(load_from(&dir), Palette::default());
        std::fs::write(dir.join("niruwm/config.toml"), "theme = \"niru-nord\"\n").unwrap();
        assert_eq!(load_from(&dir).focus, 0x88c0d0);
        std::fs::write(
            dir.join("niruwm/themes/niru-nord.toml"),
            "focus = \"#123456\"\n",
        )
        .unwrap();
        assert_eq!(load_from(&dir).focus, 0x123456);
        assert_eq!(load_from(&dir).foreground, 0xeceff4);
        std::fs::remove_dir_all(dir).unwrap();
        assert_eq!(parse_hex("zzzzzz"), None);
    }
}
