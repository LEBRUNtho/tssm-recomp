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
//!   textures/  texture packs (one packaged with the app, in its own `textures`, counts too)
//!   log.txt    everything the desktop build prints to the console
//!   env.txt    debug environment variables, one NAME=value per line (see main.rs)
//! The disc is looked for there first and then in the app itself (a `disc` folder beside
//! the program), which is how tools/ios.sh packages it.
//!
//! Controllers are read as on the desktop: the bottom face button is A, the right one B,
//! the left one X, the top one Y, the triggers are L and R, both shoulder buttons and the
//! options button are Z, Menu is Start, the left stick moves and the right stick turns the
//! camera. The first controller connected is the one read.
//!
//! The game's rumble goes to the controller when that is what was used last, and otherwise,
//! under the touch controls, to the phone itself (Core Haptics).

use std::ffi::c_void;
use std::path::PathBuf;
use std::time::Duration;

use objc2::rc::{autoreleasepool, Allocated, Retained};
use objc2::runtime::{AnyObject, Bool};
use objc2::{class, msg_send};

use crate::{input, touch};

#[link(name = "GameController", kind = "framework")]
extern "C" {
    static GCHapticsLocalityDefault: *const AnyObject;
}

#[link(name = "CoreHaptics", kind = "framework")]
extern "C" {
    static CHHapticEventTypeHapticContinuous: *const AnyObject;
    static CHHapticEventParameterIDHapticIntensity: *const AnyObject;
    static CHHapticEventParameterIDHapticSharpness: *const AnyObject;
}

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

    // A texture pack packaged with the app counts beside the ones in Documents/textures.
    if let Some(app) = std::env::current_exe().ok().and_then(|exe| exe.parent().map(|app| app.to_path_buf())) {
        crate::pack::also_look_in(app.join("textures"));
    }
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

/// The first of the languages the phone is set to, as "fr-FR".
pub fn language() -> Option<String> {
    // SAFETY: Foundation's own messages; the strings are copied before the pool is drained.
    autoreleasepool(|_| unsafe {
        let languages: *mut AnyObject = msg_send![class!(NSLocale), preferredLanguages];
        if languages.is_null() {
            return None;
        }
        let count: usize = msg_send![languages, count];
        if count == 0 {
            return None;
        }
        let first: *mut AnyObject = msg_send![languages, objectAtIndex: 0usize];
        let text: *const std::ffi::c_char = msg_send![first, UTF8String];
        (!text.is_null()).then(|| std::ffi::CStr::from_ptr(text).to_string_lossy().into_owned())
    })
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
        let mut rumble = Rumble::default();
        loop {
            std::thread::sleep(Duration::from_millis(4));
            // SAFETY: see `Rumble::follow`.
            autoreleasepool(|_| unsafe { rumble.follow() });
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
            if last.is_some() != now.is_some() {
                // Its rumble is made anew for the controller there is now.
                rumble.forget_controller();
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

/// What rumbles: the phone itself under the touch controls, or the controller.
#[derive(Clone, Copy, PartialEq)]
enum Motor {
    Phone,
    Controller,
}

/// The longest a single rumble is asked to last; a longer one is started again. Core Haptics
/// plays a continuous event for 30 seconds at most.
const RUMBLE_SECONDS: f64 = 20.0;

/// Follows the game's rumble motor with Core Haptics engines: the phone's, and the one the
/// controller offers if it can rumble.
#[derive(Default)]
struct Rumble {
    phone: Option<Retained<AnyObject>>,
    controller: Option<Retained<AnyObject>>,
    /// Engines that could not be had are not asked for again (a phone that cannot rumble; a
    /// controller until another is connected).
    no_phone: bool,
    no_controller: bool,
    /// What is rumbling, the player that makes it, and since when.
    running: Option<(Motor, Retained<AnyObject>, std::time::Instant)>,
}

impl Rumble {
    fn forget_controller(&mut self) {
        if self.running.as_ref().is_some_and(|running| running.0 == Motor::Controller) {
            self.running = None;
        }
        self.controller = None;
        self.no_controller = false;
    }

    /// Starts or stops the rumble as the game's motor now is.
    ///
    /// SAFETY: the messages are Core Haptics' and GameController's own, with the types their
    /// headers give them; called inside an autorelease pool.
    unsafe fn follow(&mut self) {
        // TSSM_RUMBLE_TEST=1 rumbles the phone for the first seconds, whatever the game does
        // and whatever the device says it can do: for trying the messages below out.
        static TEST: std::sync::OnceLock<Option<std::time::Instant>> = std::sync::OnceLock::new();
        let test = TEST.get_or_init(|| std::env::var_os("TSSM_RUMBLE_TEST").map(|_| std::time::Instant::now()));
        let testing = test.is_some_and(|since| (2.0..4.0).contains(&since.elapsed().as_secs_f64()));
        let wanted = if testing {
            self.no_phone = false;
            Some(Motor::Phone)
        } else if !input::motor() {
            None
        } else if input::gamepad_last() {
            Some(Motor::Controller)
        } else {
            touch::enabled().then_some(Motor::Phone)
        };
        let fresh = self.running.as_ref().is_some_and(|running| running.2.elapsed().as_secs_f64() < RUMBLE_SECONDS - 1.0);
        if wanted == self.running.as_ref().map(|running| running.0) && (fresh || wanted.is_none()) {
            return;
        }
        if let Some((_, player, _)) = self.running.take() {
            let _: bool = msg_send![&*player, stopAtTime: 0.0f64, error: std::ptr::null_mut::<*mut AnyObject>()];
        }
        let Some(motor) = wanted else { return };
        let Some(engine) = self.engine(motor) else { return };
        match start_rumble(&engine) {
            Some(player) => self.running = Some((motor, player, std::time::Instant::now())),
            // An engine stops for good when the app leaves the screen or the controller
            // goes: the next rumble has a new one made.
            None => match motor {
                Motor::Phone => self.phone = None,
                Motor::Controller => self.controller = None,
            },
        }
    }

    /// The engine for `motor`, made the first time it is wanted.
    unsafe fn engine(&mut self, motor: Motor) -> Option<Retained<AnyObject>> {
        match motor {
            Motor::Phone => {
                if self.phone.is_none() && !self.no_phone {
                    let capabilities: Option<Retained<AnyObject>> = msg_send![class!(CHHapticEngine), capabilitiesForHardware];
                    let able = match &capabilities {
                        Some(capabilities) => msg_send![&**capabilities, supportsHaptics],
                        None => false,
                    };
                    if able || std::env::var_os("TSSM_RUMBLE_TEST").is_some() {
                        let engine: Allocated<AnyObject> = msg_send![class!(CHHapticEngine), alloc];
                        self.phone = msg_send![engine, initAndReturnError: std::ptr::null_mut::<*mut AnyObject>()];
                    }
                    if self.phone.is_none() {
                        self.no_phone = true;
                        eprintln!("[input] this device cannot rumble");
                    }
                }
                self.phone.clone()
            }
            Motor::Controller => {
                if self.controller.is_none() && !self.no_controller {
                    let haptics = first_controller().and_then(|controller| {
                        let haptics: Option<Retained<AnyObject>> = msg_send![&*controller, haptics];
                        haptics
                    });
                    if let Some(haptics) = haptics {
                        self.controller = msg_send![&*haptics, createEngineWithLocality: GCHapticsLocalityDefault];
                    }
                    if self.controller.is_none() {
                        self.no_controller = true;
                        eprintln!("[input] this controller cannot rumble");
                    }
                }
                self.controller.clone()
            }
        }
    }
}

/// The first controller that has the full set of controls: the one `read_pad` reads.
unsafe fn first_controller() -> Option<Retained<AnyObject>> {
    let controllers: Option<Retained<AnyObject>> = msg_send![class!(GCController), controllers];
    let controllers = controllers?;
    let count: usize = msg_send![&*controllers, count];
    (0..count).find_map(|index| {
        let controller: Option<Retained<AnyObject>> = msg_send![&*controllers, objectAtIndex: index];
        let controller = controller?;
        let pad: *mut AnyObject = msg_send![&*controller, extendedGamepad];
        (!pad.is_null()).then_some(controller)
    })
}

/// Starts a steady rumble on `engine` that lasts `RUMBLE_SECONDS` unless stopped; the player
/// that stops it.
unsafe fn start_rumble(engine: &AnyObject) -> Option<Retained<AnyObject>> {
    let no_error = std::ptr::null_mut::<*mut AnyObject>;
    let started: bool = msg_send![engine, startAndReturnError: no_error()];
    if !started {
        return None;
    }
    if std::env::var_os("TSSM_RUMBLE_TEST").is_some() {
        eprintln!("[input] rumble engine started");
    }
    let parameter = |id: *const AnyObject, value: f32| -> Option<Retained<AnyObject>> {
        let parameter: Allocated<AnyObject> = msg_send![class!(CHHapticEventParameter), alloc];
        msg_send![parameter, initWithParameterID: id, value: value]
    };
    let intensity = parameter(CHHapticEventParameterIDHapticIntensity, 1.0)?;
    let sharpness = parameter(CHHapticEventParameterIDHapticSharpness, 0.4)?;
    let parameters = [&*intensity as *const AnyObject, &*sharpness as *const AnyObject];
    let parameters: Option<Retained<AnyObject>> = msg_send![class!(NSArray), arrayWithObjects: parameters.as_ptr(), count: 2usize];
    let event: Allocated<AnyObject> = msg_send![class!(CHHapticEvent), alloc];
    let event: Option<Retained<AnyObject>> = msg_send![
        event,
        initWithEventType: CHHapticEventTypeHapticContinuous,
        parameters: &*parameters?,
        relativeTime: 0.0f64,
        duration: RUMBLE_SECONDS
    ];
    let events: Option<Retained<AnyObject>> = msg_send![class!(NSArray), arrayWithObject: &*event?];
    let none: Option<Retained<AnyObject>> = msg_send![class!(NSArray), array];
    let pattern: Allocated<AnyObject> = msg_send![class!(CHHapticPattern), alloc];
    let pattern: Option<Retained<AnyObject>> = msg_send![pattern, initWithEvents: &*events?, parameters: &*none?, error: no_error()];
    let player: Option<Retained<AnyObject>> = msg_send![engine, createPlayerWithPattern: &*pattern?, error: no_error()];
    let player = player?;
    let playing: bool = msg_send![&*player, startAtTime: 0.0f64, error: no_error()];
    if std::env::var_os("TSSM_RUMBLE_TEST").is_some() {
        eprintln!("[input] rumble player started: {playing}");
    }
    playing.then_some(player)
}

/// Keeps the linker from dropping the framework when nothing else names it.
#[allow(dead_code)]
fn _anchor() -> *const c_void {
    std::ptr::null()
}
