//! What an iPhone or an iPad needs beyond the desktop's start: where the game's files are,
//! where the log goes, and the controllers.
//!
//! The window, the fingers and the keyboard come through winit as on the desktop (lib.rs);
//! the on-screen controls are touch.rs, as on Android.
//!
//! Files. The working folder is the app's Documents folder, which the Files app shows
//! ("On My iPhone", under the app's name) and which a computer reaches through the Finder:
//!   disc/      the extracted disc, if the app was built without one inside it
//!   saves/     the memory card's files
//!   config/    the settings the game saves
//!   textures/  texture packs
//!   log.txt    everything the desktop build prints to the console
//!   env.txt    debug environment variables, one NAME=value per line (see main.rs)
//! The disc is looked for there first and then in the app itself (a `disc` folder beside
//! the program), which is how tools/ios.sh packages it.
//!
//! Controllers are read as on the desktop: the bottom face button is A, the right one B,
//! the left one X, the top one Y, the triggers are L and R, both shoulder buttons and the
//! options button are Z, Menu is Start, the left stick moves and the right stick turns the
//! camera. The first controller connected is the one read.

use std::ffi::c_void;
use std::path::PathBuf;
use std::time::Duration;

use objc2::rc::autoreleasepool;
use objc2::runtime::{AnyObject, Bool};
use objc2::{class, msg_send};

use crate::input;

#[link(name = "GameController", kind = "framework")]
extern "C" {}

#[link(name = "AVFAudio", kind = "framework")]
extern "C" {
    static AVAudioSessionCategoryPlayback: *mut AnyObject;
}

extern "C" {
    fn dup2(from: i32, to: i32) -> i32;
}

/// Has the game heard with the ring switch on silent, as a console's would be: left alone,
/// iOS takes an app's sound for interface noises and mutes it with the switch.
fn sound_through_silent_switch() {
    // SAFETY: AVAudioSession's own messages, with the types its headers give them.
    unsafe {
        let session: *mut AnyObject = msg_send![class!(AVAudioSession), sharedInstance];
        let ok: Bool = msg_send![session, setCategory: AVAudioSessionCategoryPlayback, error: std::ptr::null_mut::<*mut AnyObject>()];
        if !ok.as_bool() {
            eprintln!("[sound] the audio session keeps its default category");
        }
    }
}

/// Enters the Documents folder, sends the log to a file there, reads env.txt, and returns
/// where the disc is.
pub fn prepare() -> PathBuf {
    use std::os::fd::IntoRawFd;

    let home = PathBuf::from(std::env::var_os("HOME").expect("the app has no home folder"));
    let folder = home.join("Documents");
    let _ = std::fs::create_dir_all(&folder);
    std::env::set_current_dir(&folder).expect("cannot enter the app's Documents folder");
    // Started from Xcode or the simulator's console the log is wanted there instead.
    if std::env::var_os("TSSM_CONSOLE").is_none() {
        if let Ok(log) = std::fs::File::create("log.txt") {
            let fd = log.into_raw_fd();
            // SAFETY: both descriptors are open; nothing else is running yet.
            unsafe {
                dup2(fd, 1);
                dup2(fd, 2);
            }
        }
    }
    eprintln!("[ios] documents folder: {}", folder.display());
    // A stop at an SDK call nobody has implemented looks like a crash on a phone; carry on
    // instead, as TSSM_SURVEY does. A name with nothing after the `=` in env.txt unsets it.
    std::env::set_var("TSSM_SURVEY", "1");
    for line in std::fs::read_to_string("env.txt").unwrap_or_default().lines() {
        match line.split('#').next().unwrap().split_once('=').map(|(name, value)| (name.trim(), value.trim())) {
            Some((name, "")) => std::env::remove_var(name),
            Some((name, value)) => std::env::set_var(name, value),
            None => {}
        }
    }
    sound_through_silent_switch();
    let _ = std::fs::create_dir_all("saves");
    let _ = std::fs::create_dir_all("config");

    let has_game = |disc: &PathBuf| disc.join("sys/main.dol").exists();
    let own = PathBuf::from("disc");
    if has_game(&own) {
        return own;
    }
    let bundled = std::env::current_exe().ok().and_then(|exe| exe.parent().map(|app| app.join("disc")));
    match bundled {
        Some(disc) if has_game(&disc) => disc,
        _ => {
            eprintln!("[ios] no game data: copy the extracted disc to {}/disc", folder.display());
            own
        }
    }
}

/// A controller's state in the terms `input::set_gamepad` takes.
#[derive(Clone, Copy, PartialEq, Default)]
struct Pad {
    buttons: u32,
    stick: [f32; 2],
    camera: [f32; 2],
    triggers: [f32; 2],
}

/// Reads the first controller that has the full set of controls, if there is one.
///
/// SAFETY: the messages are GameController's own, with the types its headers give them.
unsafe fn read_pad() -> Option<Pad> {
    let controllers: *mut AnyObject = msg_send![class!(GCController), controllers];
    if controllers.is_null() {
        return None;
    }
    let count: usize = msg_send![controllers, count];
    for index in 0..count {
        let controller: *mut AnyObject = msg_send![controllers, objectAtIndex: index];
        let pad: *mut AnyObject = msg_send![controller, extendedGamepad];
        if pad.is_null() {
            continue;
        }
        let pressed = |button: *mut AnyObject| -> bool {
            if button.is_null() {
                return false;
            }
            let held: Bool = msg_send![button, isPressed];
            held.as_bool()
        };
        let value = |input: *mut AnyObject| -> f32 {
            if input.is_null() {
                return 0.0;
            }
            msg_send![input, value]
        };
        let stick = |stick: *mut AnyObject| -> [f32; 2] {
            let x: *mut AnyObject = msg_send![stick, xAxis];
            let y: *mut AnyObject = msg_send![stick, yAxis];
            [value(x), value(y)]
        };
        let mut buttons = 0;
        let mut set = |held: bool, bit: u32| {
            if held {
                buttons |= bit;
            }
        };
        set(pressed(msg_send![pad, buttonA]), input::A);
        set(pressed(msg_send![pad, buttonB]), input::B);
        set(pressed(msg_send![pad, buttonX]), input::X);
        set(pressed(msg_send![pad, buttonY]), input::Y);
        set(pressed(msg_send![pad, leftShoulder]), input::Z);
        set(pressed(msg_send![pad, rightShoulder]), input::Z);
        set(pressed(msg_send![pad, buttonOptions]), input::Z);
        set(pressed(msg_send![pad, buttonMenu]), input::START);
        let dpad: *mut AnyObject = msg_send![pad, dpad];
        set(pressed(msg_send![dpad, up]), input::UP);
        set(pressed(msg_send![dpad, down]), input::DOWN);
        set(pressed(msg_send![dpad, left]), input::LEFT);
        set(pressed(msg_send![dpad, right]), input::RIGHT);
        return Some(Pad {
            buttons,
            stick: stick(msg_send![pad, leftThumbstick]),
            camera: stick(msg_send![pad, rightThumbstick]),
            triggers: [value(msg_send![pad, leftTrigger]), value(msg_send![pad, rightTrigger])],
        });
    }
    None
}

/// Starts reading controllers on a thread of its own, a few times a frame. They can be
/// connected at any time. Only a change is passed on, so that a controller lying untouched
/// does not take over from the touch screen.
pub fn start_gamepads() {
    let watch = || {
        let mut last: Option<Pad> = None;
        loop {
            std::thread::sleep(Duration::from_millis(4));
            // SAFETY: see `read_pad`; the pool frees what the framework hands out.
            let now = autoreleasepool(|_| unsafe { read_pad() });
            if now == last {
                continue;
            }
            match (last.is_some(), now) {
                (false, Some(_)) => eprintln!("[input] controller connected"),
                (true, None) => eprintln!("[input] controller disconnected"),
                _ => {}
            }
            // A controller that has gone leaves nothing held.
            let pad = now.unwrap_or_default();
            if last.is_some() || pad != Pad::default() {
                input::set_gamepad(pad.buttons, pad.stick, pad.camera, pad.triggers);
            }
            last = now;
        }
    };
    std::thread::Builder::new().name("gamepads".into()).spawn(watch).expect("cannot start the gamepad thread");
}

/// Keeps the linker from dropping the framework when nothing else names it.
#[allow(dead_code)]
fn _anchor() -> *const c_void {
    std::ptr::null()
}
