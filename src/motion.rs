use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use amane::{Animation, Blend, Color, Easing, Service};

struct FramePulse;

impl Service for FramePulse {
    fn new() -> Self {
        Self
    }
}

static FRAME_TIMER: OnceLock<Option<mpsc::Sender<u64>>> = OnceLock::new();

/// Coalesce continuous animation redraws without running faster than requested.
#[track_caller]
pub fn request_frame_at_most(fps: u64) {
    // Register this window as dependent on timer pulses.
    drop(FramePulse::read());

    if std::env::var_os("CDS_TRACE_FRAMES").is_some() {
        static CALLERS: OnceLock<Mutex<HashSet<(&'static str, u32)>>> = OnceLock::new();
        let location = std::panic::Location::caller();
        let caller = (location.file(), location.line());
        let mut callers = CALLERS.get_or_init(|| Mutex::new(HashSet::new())).lock().unwrap();
        if callers.insert(caller) {
            eprintln!("CDS frame request: {}:{} ({} FPS)", caller.0, caller.1, fps);
        }
    }
    let interval_ms = (1000 / fps.clamp(1, 60)).max(1);

    let sender = FRAME_TIMER.get_or_init(|| {
        let (sender, receiver) = mpsc::channel::<u64>();

        match thread::Builder::new()
            .name(String::from("cds-frame-timer"))
            .spawn(move || {
                let mut last_frame = Instant::now();

                while let Ok(mut interval_ms) = receiver.recv() {
                    let mut interval = Duration::from_millis(interval_ms);
                    let mut deadline = last_frame + interval;

                    loop {
                        match receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                            Ok(next_interval) => {
                                interval_ms = interval_ms.min(next_interval);
                                interval = Duration::from_millis(interval_ms);
                                deadline = last_frame + interval;
                            }
                            Err(mpsc::RecvTimeoutError::Timeout) => break,
                            Err(mpsc::RecvTimeoutError::Disconnected) => return,
                        }
                    }

                    drop(FramePulse::write());
                    last_frame = Instant::now();
                }
            })
        {
            Ok(_) => Some(sender),
            Err(error) => {
                eprintln!("CDS: não foi possível iniciar o temporizador de frames: {error}");
                None
            }
        }
    });

    if let Some(sender) = sender {
        if sender.send(interval_ms).is_err() {
            drop(FramePulse::write());
        }
    } else {
        drop(FramePulse::write());
    }
}

/*
 * Material 3 Expressive motion: things enter quickly and settle softly,
 * using the decelerate curve by default. Durations follow Material 3
 * Expressive specifications for smooth, natural animations.
 */
const EXPAND: Duration = Duration::from_millis(400);
const COLLAPSE: Duration = Duration::from_millis(300);
const FADE: Duration = Duration::from_millis(200);
const STATE: Duration = Duration::from_millis(150);
const WORKSPACE: Duration = Duration::from_millis(320);
const SELECT: Duration = Duration::from_millis(200);
const OVERLAY_IN: Duration = Duration::from_millis(300);
const OVERLAY_OUT: Duration = Duration::from_millis(200);

/// Pick a duration from the active motion language. M3 is more expressive;
/// Fluent keeps desktop transitions brisk and direct.
pub fn style_duration(m3_ms: u64, fluent_ms: u64) -> Duration {
    if crate::settings::Settings::read().motion_style() == "fluent" {
        Duration::from_millis(fluent_ms)
    } else {
        Duration::from_millis(m3_ms)
    }
}

/*
 * one value moving toward a target. retargeting only happens when the
 * target itself changes, so an animation already on its way is never
 * restarted every frame (which would keep the shell drawing forever).
 */
struct Track<T: Blend> {
    target: T,
    animation: Animation<T>,
}

impl<T: Blend> Track<T> {
    fn new(value: T, duration: Duration) -> Self {
        Self {
            target: value,
            animation: Animation::new(value).duration(duration).easing(Easing::Out),
        }
    }

    fn to(&mut self, target: T, duration: Duration) {
        if self.target == target {
            return;
        }

        self.target = target;

        let current = self.animation.value();

        self.animation = Animation::new(current)
            .duration(duration)
            .easing(Easing::Out);

        self.animation.to(target);
    }

    fn snap(&mut self, value: T) {
        self.target = value;
        self.animation = Animation::new(value);
    }

    fn value(&self) -> T {
        self.animation.value()
    }
}

struct Morph {
    width: Track<f32>,
    height: Track<f32>,
    radius: Track<f32>,
}

impl Morph {
    fn new(width: f32, height: f32, radius: f32) -> Self {
        Self {
            width: Track::new(width, EXPAND),
            height: Track::new(height, EXPAND),
            radius: Track::new(radius, EXPAND),
        }
    }
}

thread_local! {
    static MORPHS: RefCell<HashMap<String, Morph>> = RefCell::new(HashMap::new());
    static FADES: RefCell<HashMap<String, (u64, Animation<f32>)>> = RefCell::new(HashMap::new());
    static HOVERS: RefCell<HashMap<String, (bool, Track<f32>)>> = RefCell::new(HashMap::new());
    static NUMBERS: RefCell<HashMap<String, Track<f32>>> = RefCell::new(HashMap::new());
    static COLORS: RefCell<HashMap<String, Track<Color>>> = RefCell::new(HashMap::new());
    static PRESENCES: RefCell<HashMap<String, Track<f32>>> = RefCell::new(HashMap::new());
}

// the island growing or shrinking between its compact, peek and dash sizes
pub fn follow(monitor: &str, width: f32, height: f32, radius: f32) -> (f32, f32, f32) {
    MORPHS.with(|morphs| {
        let mut morphs = morphs.borrow_mut();
        let morph = morphs
            .entry(String::from(monitor))
            .or_insert_with(|| Morph::new(width, height, radius));

        let area = width * height;
        let before = morph.width.value() * morph.height.value();
        let duration = if area >= before {
            style_duration(EXPAND.as_millis() as u64, 333)
        } else {
            style_duration(COLLAPSE.as_millis() as u64, 167)
        };

        morph.width.to(width, duration);
        morph.height.to(height, duration);
        morph.radius.to(radius, duration);

        (
            morph.width.value(),
            morph.height.value(),
            morph.radius.value(),
        )
    })
}

/*
 * a fade-in that restarts whenever `state` changes, keyed by the thing
 * fading (the island mode, the dash tab), so content arriving is soft
 */
pub fn fade(key: &str, state: u64) -> f32 {
    FADES.with(|fades| {
        let mut fades = fades.borrow_mut();

        let restart = !matches!(fades.get(key), Some((current, _)) if *current == state);

        if restart {
            let mut fresh = Animation::new(0.0).duration(FADE).easing(Easing::Out);
            fresh.to(1.0);

            fades.insert(String::from(key), (state, fresh));
        }

        let Some((_, animation)) = fades.get_mut(key) else {
            return 1.0;
        };

        animation.value()
    })
}

// how far an overlay has come in, from 0 when closed to 1 when fully shown
pub fn presence(key: &str, open: bool) -> f32 {
    PRESENCES.with(|presences| {
        let mut presences = presences.borrow_mut();
        let track = presences
            .entry(String::from(key))
            .or_insert_with(|| Track::new(if open { 1.0 } else { 0.0 }, OVERLAY_IN));

        track.to(
            if open { 1.0 } else { 0.0 },
            if open {
                style_duration(OVERLAY_IN.as_millis() as u64, 250)
            } else {
                style_duration(OVERLAY_OUT.as_millis() as u64, 167)
            },
        );

        track.value()
    })
}

/// Set an overlay's presence immediately, bypassing its exit animation.
pub fn presence_immediately(key: &str, open: bool) {
    let value = if open { 1.0 } else { 0.0 };

    PRESENCES.with(|presences| {
        let mut presences = presences.borrow_mut();
        presences
            .entry(String::from(key))
            .or_insert_with(|| Track::new(value, OVERLAY_IN))
            .snap(value);
    });
}

// pointer state layers: 0 when the pointer is away, 1 while it is over
pub fn set_hover(key: impl Into<String>, inside: bool) {
    HOVERS.with(|hovers| {
        let mut hovers = hovers.borrow_mut();
        let entry = hovers
            .entry(key.into())
            .or_insert_with(|| (false, Track::new(0.0, STATE)));

        entry.0 = inside;
    });

    crate::ui::Ui::refresh();
}

pub fn hover(key: &str) -> f32 {
    HOVERS.with(|hovers| {
        let mut hovers = hovers.borrow_mut();
        let Some((inside, track)) = hovers.get_mut(key) else {
            return 0.0;
        };

        track.to(
            if *inside { 1.0 } else { 0.0 },
            style_duration(STATE.as_millis() as u64, 120),
        );

        track.value()
    })
}

// a number that moves toward its target, like a workspace dot widening
pub fn number(key: &str, target: f32, duration: Duration) -> f32 {
    NUMBERS.with(|numbers| {
        let mut numbers = numbers.borrow_mut();
        let track = numbers
            .entry(String::from(key))
            .or_insert_with(|| Track::new(target, duration));

        track.to(target, duration);

        track.value()
    })
}

// a color that fades between two states, like a selected chip
pub fn color(key: &str, target: Color, duration: Duration) -> Color {
    COLORS.with(|colors| {
        let mut colors = colors.borrow_mut();
        let track = colors
            .entry(String::from(key))
            .or_insert_with(|| Track::new(target, duration));

        track.to(target, duration);

        track.value()
    })
}

pub const WORKSPACE_DURATION: Duration = WORKSPACE;
pub const SELECT_DURATION: Duration = SELECT;
