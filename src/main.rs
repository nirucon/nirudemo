//! NIRUDEMO: plays a demo on every screen, with its music.
//!
//! - `nirudemo [N|KEY]` plays a demo (default: the first) over all screens;
//!   any key or click ends it.
//! - `nirudemo --into WINDOW [--loop] [--silent] [N|KEY]` draws into an
//!   existing window (the NIRUSCREENSAVER `demo` scene) and takes no input.
//! - `nirudemo --list` and `--songs` list the demos and the music.
//!
//! The music is `~/.local/share/nirux/music/nirux-N-key.ogg`, played with
//! pw-play (or ffplay or mpv). It stays off with `--silent` or while Do Not
//! Disturb is on. The demo ends at once when a NIRULOCK starts, so it never
//! holds the keyboard a lock needs.

use nirudemo::{demos, gfx, songs, theme};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::randr::ConnectionExt as _;
use x11rb::protocol::render::{self, ConnectionExt as _, PictOp};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const FPS: f32 = 50.0;
/// Keys pressed this soon after the start are the ones that started it.
const GRACE: Duration = Duration::from_millis(400);

static STOP: AtomicBool = AtomicBool::new(false);

extern "C" fn on_signal(_: libc::c_int) {
    STOP.store(true, Ordering::SeqCst);
}

type Res<T> = Result<T, Box<dyn std::error::Error>>;

struct Args {
    into: Option<u32>,
    looped: bool,
    silent: bool,
    which: Option<String>,
}

fn usage() -> &'static str {
    "usage: nirudemo [N|KEY]                       play a demo (any key ends it)\n\
     \x20      nirudemo --into WINDOW [--loop] [--silent] [N|KEY]\n\
     \x20      nirudemo --list | --songs | --version | --help"
}

fn main() {
    let mut args = Args {
        into: None,
        looped: false,
        silent: false,
        which: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--version" | "-V" => {
                println!("NIRUDEMO {VERSION}");
                return;
            }
            "--help" | "-h" => {
                println!("{}", usage());
                return;
            }
            "--list" => {
                for line in demos::list_lines() {
                    println!("{line}");
                }
                return;
            }
            "--songs" => {
                for s in songs::all() {
                    println!(
                        "{}\t{}\t{}\t{:.0}\t{}\t{}",
                        s.number,
                        s.key,
                        s.title,
                        s.length,
                        s.file_name(),
                        s.about
                    );
                }
                return;
            }
            "--still" => {
                // For development: the frame at SECONDS as a PPM image,
                // drawn without a display.
                let t: f32 = it.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let out = it.next().unwrap_or_else(|| "still.ppm".into());
                let info =
                    demos::find(it.next().as_deref()).unwrap_or_else(|| fail("no such demo"));
                let mut demo = (info.make)(&demos::Context { host: host_name() });
                let mut frame = gfx::Frame::new();
                let mut k = 0.0;
                while k <= t {
                    demo.draw(&mut frame, k);
                    k += 1.0 / FPS;
                }
                demo.draw(&mut frame, t);
                let ramp = gfx::Ramp::new(&theme::load());
                let mut ppm = format!("P6 {} {} 255\n", gfx::W, gfx::H).into_bytes();
                for &p in &frame.px {
                    let c = ramp.rgb[p as usize];
                    ppm.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8]);
                }
                std::fs::write(&out, ppm).unwrap_or_else(|e| fail(&e.to_string()));
                return;
            }
            "--loop" => args.looped = true,
            "--silent" => args.silent = true,
            "--into" => {
                let w = it.next().unwrap_or_default();
                let parsed = w
                    .strip_prefix("0x")
                    .map(|h| u32::from_str_radix(h, 16))
                    .unwrap_or_else(|| w.parse());
                match parsed {
                    Ok(id) if id != 0 => args.into = Some(id),
                    _ => fail(&format!("--into needs a window id, not {w:?}")),
                }
            }
            other if other.starts_with('-') => {
                fail(&format!("unknown option {other}\n{}", usage()))
            }
            other => args.which = Some(other.to_string()),
        }
    }
    let Some(info) = demos::find(args.which.as_deref()) else {
        fail(&format!(
            "no demo {:?}; nirudemo --list shows them",
            args.which.unwrap_or_default()
        ));
    };
    // SAFETY: plain signal handlers that only store to an atomic.
    unsafe {
        libc::signal(libc::SIGTERM, on_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGINT, on_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGHUP, on_signal as *const () as libc::sighandler_t);
    }
    if let Err(e) = run(&args, &info) {
        fail(&e.to_string());
    }
}

fn fail(message: &str) -> ! {
    eprintln!("nirudemo: {message}");
    std::process::exit(2);
}

fn host_name() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .or_else(|_| std::fs::read_to_string("/etc/hostname"))
        .unwrap_or_default()
        .trim()
        .to_string()
}

// ---- music ------------------------------------------------------------------------

fn music_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")
        })
        .join("nirux/music")
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join(name))
        .find(|p| {
            use std::os::unix::fs::PermissionsExt;
            std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
}

/// True when `nirunotify --dnd status` says Do Not Disturb is on. A missing
/// or slow notifier counts as off.
fn dnd_on() -> bool {
    let Some(program) = which("nirunotify") else {
        return false;
    };
    let Ok(mut child) = Command::new(program)
        .args(["--dnd", "status"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return false;
    };
    let until = Instant::now() + Duration::from_millis(800);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < until => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
        }
    }
    let mut out = String::new();
    if let Some(mut s) = child.stdout.take() {
        let _ = std::io::Read::read_to_string(&mut s, &mut out);
    }
    out.split_whitespace().any(|w| w == "on")
}

struct Player {
    child: Child,
}

impl Player {
    fn start(file: &PathBuf) -> Option<Self> {
        let candidates: [(&str, &[&str]); 3] = [
            ("pw-play", &[]),
            ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
            ("mpv", &["--no-video", "--really-quiet", "--no-terminal"]),
        ];
        let (program, extra) = candidates
            .iter()
            .find_map(|(name, extra)| which(name).map(|p| (p, *extra)))?;
        // SAFETY: getpid has no preconditions.
        let parent = unsafe { libc::getpid() };
        let mut command = Command::new(program);
        command
            .args(extra)
            .arg(file)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        // SAFETY: only async-signal-safe calls between fork and exec. The
        // player gets its own process group and dies with the demo.
        unsafe {
            command.pre_exec(move || {
                if libc::setsid() < 0
                    || libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) != 0
                    || libc::getppid() != parent
                {
                    libc::_exit(127);
                }
                Ok(())
            });
        }
        command.spawn().ok().map(|child| Player { child })
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        let pgid = self.child.id() as i32;
        // SAFETY: kill(2) on the process group the player leads.
        unsafe { libc::kill(-pgid, libc::SIGTERM) };
        let until = Instant::now() + Duration::from_millis(400);
        while matches!(self.child.try_wait(), Ok(None)) && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(10));
        }
        // SAFETY: as above.
        unsafe { libc::kill(-pgid, libc::SIGKILL) };
        let _ = self.child.wait();
    }
}

/// A NIRULOCK of this user on this display means the demo must give way.
fn lock_running() -> bool {
    // SAFETY: getuid has no preconditions.
    let me = unsafe { libc::getuid() };
    let display = std::env::var("DISPLAY").unwrap_or_default();
    let same_display = |p: &std::path::Path| {
        std::fs::read(p.join("environ")).is_ok_and(|env| {
            env.split(|&b| b == 0).any(|kv| {
                kv.strip_prefix(b"DISPLAY=")
                    .is_some_and(|d| d == display.as_bytes())
            })
        })
    };
    let Ok(dir) = std::fs::read_dir("/proc") else {
        return false;
    };
    dir.flatten().any(|e| {
        let p = e.path();
        std::fs::read_to_string(p.join("comm")).is_ok_and(|c| c.trim() == "nirulock")
            && std::fs::read_to_string(p.join("status")).is_ok_and(|s| {
                s.lines()
                    .find(|l| l.starts_with("Uid:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|u| u.parse::<u32>().ok())
                    == Some(me)
            })
            && same_display(&p)
    })
}

// ---- X11 ----------------------------------------------------------------------------

fn fixed(v: f32) -> i32 {
    (v * 65536.0).round() as i32
}

fn format_for_visual(
    formats: &render::QueryPictFormatsReply,
    visual: Visualid,
) -> Option<render::Pictformat> {
    formats
        .screens
        .iter()
        .flat_map(|s| s.depths.iter())
        .flat_map(|d| d.visuals.iter())
        .find(|v| v.visual == visual)
        .map(|v| v.format)
}

struct Screen {
    conn: RustConnection,
    win: Window,
    own_window: bool,
    pixmap: Pixmap,
    gc: Gcontext,
    src: render::Picture,
    dst: render::Picture,
    depth: u8,
    areas: Vec<(i16, i16, u16, u16)>,
}

impl Screen {
    fn open(into: Option<u32>) -> Res<Self> {
        let (conn, screen_num) = x11rb::connect(None)?;
        if conn
            .extension_information(render::X11_EXTENSION_NAME)?
            .is_none()
        {
            return Err("the X server has no RENDER extension".into());
        }
        let screen = conn.setup().roots[screen_num].clone();
        let formats = conn.render_query_pict_formats()?.reply()?;
        let (win, visual, depth, own_window) = match into {
            Some(w) => {
                let attrs = conn
                    .get_window_attributes(w)?
                    .reply()
                    .map_err(|_| format!("no window 0x{w:x}"))?;
                let geo = conn.get_geometry(w)?.reply()?;
                conn.change_window_attributes(
                    w,
                    &ChangeWindowAttributesAux::new()
                        .event_mask(EventMask::STRUCTURE_NOTIFY | EventMask::EXPOSURE),
                )?;
                (w, attrs.visual, geo.depth, false)
            }
            None => {
                let win = conn.generate_id()?;
                conn.create_window(
                    screen.root_depth,
                    win,
                    screen.root,
                    0,
                    0,
                    screen.width_in_pixels,
                    screen.height_in_pixels,
                    0,
                    WindowClass::INPUT_OUTPUT,
                    screen.root_visual,
                    &CreateWindowAux::new()
                        .background_pixel(screen.black_pixel)
                        .override_redirect(1)
                        .event_mask(
                            EventMask::KEY_PRESS
                                | EventMask::BUTTON_PRESS
                                | EventMask::EXPOSURE
                                | EventMask::STRUCTURE_NOTIFY,
                        ),
                )?;
                conn.change_property8(
                    PropMode::REPLACE,
                    win,
                    AtomEnum::WM_CLASS,
                    AtomEnum::STRING,
                    b"nirudemo\0nirudemo\0",
                )?;
                conn.change_property8(
                    PropMode::REPLACE,
                    win,
                    AtomEnum::WM_NAME,
                    AtomEnum::STRING,
                    b"NIRUDEMO",
                )?;
                // A blank cursor.
                let blank = conn.generate_id()?;
                conn.create_pixmap(1, blank, win, 1, 1)?;
                let cursor = conn.generate_id()?;
                conn.create_cursor(cursor, blank, blank, 0, 0, 0, 0, 0, 0, 0, 0)?;
                conn.change_window_attributes(
                    win,
                    &ChangeWindowAttributesAux::new().cursor(cursor),
                )?;
                conn.map_window(win)?;
                conn.configure_window(
                    win,
                    &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                )?;
                conn.flush()?;
                (win, screen.root_visual, screen.root_depth, true)
            }
        };
        if depth != 24 && depth != 32 {
            return Err(format!("unsupported window depth {depth}").into());
        }
        let dst_format =
            format_for_visual(&formats, visual).ok_or("no picture format for the window")?;
        let src_format = formats
            .formats
            .iter()
            .find(|f| f.depth == depth && f.type_ == render::PictType::DIRECT)
            .map(|f| f.id)
            .ok_or("no picture format for the frame")?;
        let pixmap = conn.generate_id()?;
        conn.create_pixmap(depth, pixmap, win, gfx::W as u16, gfx::H as u16)?;
        let gc = conn.generate_id()?;
        conn.create_gc(gc, pixmap, &CreateGCAux::new())?;
        let src = conn.generate_id()?;
        conn.render_create_picture(src, pixmap, src_format, &render::CreatePictureAux::new())?;
        conn.render_set_picture_filter(src, b"nearest", &[])?;
        let dst = conn.generate_id()?;
        // Draw over child windows too: a host window (such as a terminal
        // used for testing) may have children covering it.
        conn.render_create_picture(
            dst,
            win,
            dst_format,
            &render::CreatePictureAux::new().subwindowmode(SubwindowMode::INCLUDE_INFERIORS),
        )?;
        let mut s = Screen {
            conn,
            win,
            own_window,
            pixmap,
            gc,
            src,
            dst,
            depth,
            areas: Vec::new(),
        };
        s.find_areas(screen.root)?;
        Ok(s)
    }

    /// Where to draw: every monitor for our own window, the whole window
    /// otherwise.
    fn find_areas(&mut self, root: Window) -> Res<()> {
        let geo = self.conn.get_geometry(self.win)?.reply()?;
        self.areas = vec![(0, 0, geo.width, geo.height)];
        if self.own_window {
            if let Ok(Ok(reply)) = self.conn.randr_get_monitors(root, true).map(|c| c.reply()) {
                let list: Vec<_> = reply
                    .monitors
                    .iter()
                    .filter(|m| m.width > 0 && m.height > 0)
                    .map(|m| (m.x, m.y, m.width, m.height))
                    .collect();
                if !list.is_empty() {
                    self.areas = list;
                }
            }
        }
        self.clear_all()
    }

    fn clear_all(&self) -> Res<()> {
        let geo = self.conn.get_geometry(self.win)?.reply()?;
        let black = render::Color {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0xffff,
        };
        self.conn.render_fill_rectangles(
            PictOp::SRC,
            self.dst,
            black,
            &[Rectangle {
                x: 0,
                y: 0,
                width: geo.width,
                height: geo.height,
            }],
        )?;
        Ok(())
    }

    fn grab(&self) -> Res<()> {
        let until = Instant::now() + Duration::from_secs(1);
        loop {
            let k = self
                .conn
                .grab_keyboard(
                    true,
                    self.win,
                    x11rb::CURRENT_TIME,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )?
                .reply()?;
            if k.status == GrabStatus::SUCCESS {
                break;
            }
            if Instant::now() > until {
                return Err("cannot take the keyboard (another program holds it)".into());
            }
            std::thread::sleep(Duration::from_millis(30));
        }
        let _ = self
            .conn
            .grab_pointer(
                true,
                self.win,
                EventMask::BUTTON_PRESS,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
                self.win,
                x11rb::NONE,
                x11rb::CURRENT_TIME,
            )?
            .reply();
        Ok(())
    }

    fn show(&self, bgrx: &[u8]) -> Res<()> {
        self.conn.put_image(
            ImageFormat::Z_PIXMAP,
            self.pixmap,
            self.gc,
            gfx::W as u16,
            gfx::H as u16,
            0,
            0,
            0,
            self.depth,
            bgrx,
        )?;
        for &(x, y, w, h) in &self.areas {
            let scale = (w as f32 / gfx::W as f32).min(h as f32 / gfx::H as f32);
            let dw = (gfx::W as f32 * scale) as u16;
            let dh = (gfx::H as f32 * scale) as u16;
            let inv = fixed(1.0 / scale);
            let t = render::Transform {
                matrix11: inv,
                matrix12: 0,
                matrix13: 0,
                matrix21: 0,
                matrix22: inv,
                matrix23: 0,
                matrix31: 0,
                matrix32: 0,
                matrix33: fixed(1.0),
            };
            self.conn.render_set_picture_transform(self.src, t)?;
            let ox = x + ((w - dw) / 2) as i16;
            let oy = y + ((h - dh) / 2) as i16;
            self.conn.render_composite(
                PictOp::SRC,
                self.src,
                x11rb::NONE,
                self.dst,
                0,
                0,
                0,
                0,
                ox,
                oy,
                dw,
                dh,
            )?;
        }
        self.conn.flush()?;
        Ok(())
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let _ = self.conn.render_free_picture(self.src);
        let _ = self.conn.render_free_picture(self.dst);
        let _ = self.conn.free_pixmap(self.pixmap);
        if self.own_window {
            let _ = self.conn.ungrab_keyboard(x11rb::CURRENT_TIME);
            let _ = self.conn.ungrab_pointer(x11rb::CURRENT_TIME);
            let _ = self.conn.destroy_window(self.win);
        }
        let _ = self.conn.flush();
    }
}

fn run(args: &Args, info: &demos::Info) -> Res<()> {
    let screen = Screen::open(args.into)?;
    let root = screen.conn.setup().roots[0].root;
    let mut screen = screen;
    if screen.own_window {
        screen.grab()?;
    }
    let ramp = gfx::Ramp::new(&theme::load());
    let ctx = demos::Context { host: host_name() };
    let mut demo = (info.make)(&ctx);
    let song = songs::all().into_iter().find(|s| s.number == info.song);
    let music = song
        .map(|s| music_dir().join(s.file_name()))
        .filter(|p| p.is_file());
    let silent = args.silent || dnd_on();
    let mut frame = gfx::Frame::new();
    let mut bytes = Vec::new();
    let period = Duration::from_secs_f32(1.0 / FPS);
    let mut lock_check = Instant::now();
    'outer: loop {
        let start = Instant::now();
        let _player = if silent {
            None
        } else {
            music.as_ref().and_then(Player::start)
        };
        let mut next = start;
        loop {
            if STOP.load(Ordering::SeqCst) {
                break 'outer;
            }
            while let Some(event) = screen.conn.poll_for_event()? {
                match event {
                    Event::KeyPress(_) | Event::ButtonPress(_)
                        if screen.own_window && start.elapsed() > GRACE =>
                    {
                        break 'outer
                    }
                    Event::DestroyNotify(e) if e.window == screen.win => break 'outer,
                    Event::ConfigureNotify(e) if e.window == screen.win => {
                        screen.find_areas(root)?
                    }
                    Event::Expose(_) => screen.clear_all()?,
                    Event::Error(e) => eprintln!("nirudemo: X error {e:?}"),
                    _ => {}
                }
            }
            if screen.own_window && lock_check.elapsed() > Duration::from_millis(400) {
                lock_check = Instant::now();
                if lock_running() {
                    eprintln!("nirudemo: a lock started; the demo ends");
                    break 'outer;
                }
            }
            let t = start.elapsed().as_secs_f32();
            if t >= info.seconds {
                if args.looped {
                    continue 'outer;
                }
                break 'outer;
            }
            demo.draw(&mut frame, t);
            frame.to_bgrx(&ramp, &mut bytes);
            screen.show(&bytes)?;
            next += period;
            let now = Instant::now();
            if next > now {
                std::thread::sleep(next - now);
            } else {
                next = now;
            }
        }
    }
    Ok(())
}
