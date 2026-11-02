use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;
use amane::{Animation, Easing};

struct Motion {
    width: Animation,
    height: Animation,
    radius: Animation,
}

thread_local! {
    static MOTIONS: RefCell<HashMap<String, Motion>> = RefCell::new(HashMap::new());
}

impl Motion {
    fn new(width: f32, height: f32, radius: f32) -> Self {
        let make = |value| {
            Animation::new(value)
                .duration(Duration::from_millis(400))
                .easing(Easing::Out)
        };

        Self {
            width: make(width),
            height: make(height),
            radius: make(radius),
        }
    }
}

pub fn follow(monitor: &str, width: f32, height: f32, radius: f32) -> (f32, f32, f32) {
    MOTIONS.with(|motions| {
        let mut motions = motions.borrow_mut();
        let motion = motions
            .entry(String::from(monitor))
            .or_insert_with(|| Motion::new(width, height, radius));

        motion.width.to(width);
        motion.height.to(height);
        motion.radius.to(radius);

        (
            motion.width.value(),
            motion.height.value(),
            motion.radius.value(),
        )
    })
}
