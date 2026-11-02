use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

use amane::{Animation, Blend, Color, Easing};

/*
 * material 3 motion: things enter quickly and settle softly, so the
 * decelerate curve is the default. durations stay short enough that
 * the shell still feels immediate.
 */
const EXPAND: Duration = Duration::from_millis(420);
const COLLAPSE: Duration = Duration::from_millis(280);
const FADE: Duration = Duration::from_millis(220);
const STATE: Duration = Duration::from_millis(150);
const WORKSPACE: Duration = Duration::from_millis(260);
const SELECT: Duration = Duration::from_millis(240);
const OVERLAY_IN: Duration = Duration::from_millis(220);
const OVERLAY_OUT: Duration = Duration::from_millis(150);

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
        let duration = if area >= before { EXPAND } else { COLLAPSE };

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
            if open { OVERLAY_IN } else { OVERLAY_OUT },
        );

        track.value()
    })
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

        track.to(if *inside { 1.0 } else { 0.0 }, STATE);

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
