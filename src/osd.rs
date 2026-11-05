/*
 * on screen displays: a small pill that pops up above the bottom edge when
 * the volume, brightness, microphone or a battery state changes, then fades.
 * the icon swaps with the level (low, medium, high, muted, almost empty).
 */
use std::cell::RefCell;
use std::io::{BufRead, BufReader};
use std::collections::{HashMap, VecDeque};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use amane::{
    Audio, Battery, Brightness, Center, Horizontal, Layer, LayerWindow, Margin, Monitor, Padding,
    Parent, Rectangle, Row, Service, Shadow, Vertical, Widget, Zone,
};

use crate::glyphs;
use crate::m3::{Type, boxed, icon, scale, wavy};
use crate::motion;
use crate::overlay::Overlay;
use crate::theme::{Theme, fade};

const WINDOW_WIDTH: f32 = 340.0;
const WINDOW_HEIGHT: f32 = 96.0;
const PILL_WIDTH: f32 = 288.0;
const PILL_HEIGHT: f32 = 56.0;
const SHOWN: Duration = Duration::from_secs(5);
const ALERT: Duration = Duration::from_secs(5);
const MAX_OUTPUT_VOLUME: f32 = 150.0;

// Amane's Audio API intentionally clamps output volume to 100%. Read the
// PulseAudio-compatible server directly so OSDs can represent Sound Boost.
struct SoundLevel(u8);

impl Service for SoundLevel {
    fn new() -> Self {
        Self(read_output_volume().unwrap_or_else(|| Audio::read().volume()))
    }

    fn update(&mut self) -> bool {
        let Some(volume) = read_output_volume() else {
            return false;
        };
        if self.0 == volume {
            false
        } else {
            self.0 = volume;
            true
        }
    }

    fn listen() {
        loop {
            let child = Command::new("pactl")
                .arg("subscribe")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn();

            let Ok(mut child) = child else {
                thread::sleep(Duration::from_secs(5));
                continue;
            };

            if let Some(stdout) = child.stdout.take() {
                for line in BufReader::new(stdout).lines() {
                    let Ok(line) = line else { break };
                    if line.contains("sink") || line.contains("server") {
                        let mut sound = Self::write();
                        if !sound.update() {
                            sound.quiet();
                        }
                    }
                }
            }

            let _ = child.kill();
            let _ = child.wait();
            thread::sleep(Duration::from_secs(2));
        }
    }
}

fn read_output_volume() -> Option<u8> {
    let output = Command::new("pactl")
        .args(["get-sink-volume", "@DEFAULT_SINK@"])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let volume = String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .filter_map(|part| part.trim_end_matches(|c: char| !c.is_ascii_digit() && c != '%')
            .strip_suffix('%')
            .and_then(|value| value.parse::<u16>().ok()))
        .max()?;

    Some(volume.min(150) as u8)
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Volume,
    Brightness,
    Microphone,
    Charger,
    LowBattery,
}

#[derive(Clone, Copy, PartialEq)]
struct Snapshot {
    volume: u8,
    muted: bool,
    microphone_muted: bool,
    brightness: u8,
    battery: u8,
    charging: bool,
    battery_present: bool,
}

struct Memory {
    last: Snapshot,
    kind: Kind,
    since: Option<Instant>,
    pending: VecDeque<Kind>,
}

thread_local! {
    static MEMORY: RefCell<HashMap<String, Memory>> = RefCell::new(HashMap::new());
}

fn snapshot() -> Snapshot {
    let audio = Audio::read();
    let volume = SoundLevel::read().0;
    let brightness = Brightness::read();
    let battery = Battery::read();

    Snapshot {
        volume,
        muted: audio.muted(),
        microphone_muted: audio.microphone_muted(),
        brightness: if brightness.present() {
            brightness.percent()
        } else {
            0
        },
        battery: battery.percent(),
        charging: battery.charging(),
        battery_present: battery.present(),
    }
}

// compares with what this monitor saw last and returns what to show, if anything
fn observe(monitor: &str, now: Snapshot) -> Option<(Kind, Duration, f32)> {
    MEMORY.with(|memory| {
        let mut memory = memory.borrow_mut();

        let Some(entry) = memory.get_mut(monitor) else {
            memory.insert(
                String::from(monitor),
                Memory {
                    last: now,
                    kind: Kind::Volume,
                    since: None,
                    pending: VecDeque::new(),
                },
            );

            return None;
        };

        let before = entry.last;
        let mut changed = Vec::with_capacity(5);

        if (now.volume, now.muted) != (before.volume, before.muted) {
            changed.push(Kind::Volume);
        }
        if now.microphone_muted != before.microphone_muted {
            changed.push(Kind::Microphone);
        }
        if now.brightness != before.brightness {
            changed.push(Kind::Brightness);
        }
        if now.battery_present && now.charging != before.charging {
            changed.push(Kind::Charger);
        }
        if now.battery_present
            && !now.charging
            && now.battery <= 15
            && (before.battery > 15 || (now.battery <= 5 && before.battery > 5))
        {
            changed.push(Kind::LowBattery);
        }

        entry.last = now;

        let now_instant = Instant::now();
        for kind in changed {
            if entry.since.is_some() && kind == entry.kind {
                // Repeated input extends the active OSD's five-second window.
                entry.since = Some(now_instant);
            } else if !entry.pending.contains(&kind) {
                // Keep simultaneous transitions instead of overwriting them
                // when the snapshot advances to the newest system state.
                entry.pending.push_back(kind);
            }
        }

        let expired = entry.since.is_some_and(|since| {
            let lifetime = if matches!(entry.kind, Kind::LowBattery) { ALERT } else { SHOWN };
            now_instant.duration_since(since) >= lifetime
        });

        if expired {
            entry.since = None;
        }

        if entry.since.is_none() {
            if let Some(next) = entry.pending.pop_front() {
                entry.kind = next;
                entry.since = Some(now_instant);
            } else {
                return None;
            }
        }

        let since = entry.since?;
        let lifetime = if matches!(entry.kind, Kind::LowBattery) {
            ALERT
        } else {
            SHOWN
        };
        let age = now_instant.duration_since(since);

        if age >= lifetime {
            return None;
        }

        Some((entry.kind, lifetime, age.as_secs_f32()))
    })
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{Kind, MEMORY, SHOWN, Snapshot, observe};

    #[test]
    fn queues_simultaneous_volume_and_microphone_changes() {
        MEMORY.with(|memory| {
            memory.borrow_mut().remove("test-simultaneous-osd");
        });

        let before = Snapshot {
            volume: 35,
            muted: false,
            microphone_muted: false,
            brightness: 60,
            battery: 80,
            charging: false,
            battery_present: true,
        };
        let after = Snapshot {
            volume: 48,
            microphone_muted: true,
            ..before
        };

        assert!(observe("test-simultaneous-osd", before).is_none());
        assert!(matches!(
            observe("test-simultaneous-osd", after),
            Some((Kind::Volume, _, _))
        ));
        MEMORY.with(|memory| {
            let mut memory = memory.borrow_mut();
            memory.get_mut("test-simultaneous-osd").unwrap().since =
                Some(Instant::now() - SHOWN - Duration::from_millis(1));
        });

        assert!(matches!(
            observe("test-simultaneous-osd", after),
            Some((Kind::Microphone, _, _))
        ));
    }
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    let theme = Theme::read();
    let scheme = theme.scheme();

    let now = snapshot();
    let active = observe(&monitor.name, now);

    let focused = Overlay::on_focused_monitor(&monitor.name);

    let key = format!("osd:{}", monitor.name);
    let presence = motion::presence(&key, active.is_some() && focused);

    if active.is_some() || presence > 0.001 {
        motion::request_frame_at_most(30);
    }

    let kind = active.map(|(kind, _, _)| kind).unwrap_or_else(|| {
        MEMORY.with(|memory| {
            memory
                .borrow()
                .get(&monitor.name)
                .map_or(Kind::Volume, |entry| entry.kind)
        })
    });

    let alert = matches!(kind, Kind::LowBattery)
        || (matches!(kind, Kind::Volume) && now.muted)
        || (matches!(kind, Kind::Microphone) && now.microphone_muted);

    let (container, on_container) = if matches!(kind, Kind::LowBattery) {
        (scheme.error_container, scheme.on_error_container)
    } else if alert {
        (scheme.secondary_container, scheme.on_secondary_container)
    } else {
        (scheme.primary_container, scheme.on_primary_container)
    };

    let (glyph, fraction, label): (&str, Option<f32>, String) = match kind {
        Kind::Volume => (
            glyphs::volume(now.volume, now.muted),
            Some(if now.muted {
                0.0
            } else {
                (f32::from(now.volume) / MAX_OUTPUT_VOLUME).clamp(0.0, 1.0)
            }),
            if now.muted {
                String::from(crate::lang::text("Mudo", "Muted"))
            } else {
                format!("{}%", now.volume)
            },
        ),
        Kind::Brightness => (
            glyphs::brightness(now.brightness),
            Some(f32::from(now.brightness) / 100.0),
            format!("{}%", now.brightness),
        ),
        Kind::Microphone => (
            glyphs::microphone(now.microphone_muted),
            None,
            String::from(if now.microphone_muted {
                crate::lang::text("Microfone desligado", "Microphone muted")
            } else {
                crate::lang::text("Microfone ligado", "Microphone on")
            }),
        ),
        Kind::Charger => (
            glyphs::battery(now.battery, now.charging),
            None,
            if now.charging {
                format!("{} · {}%", crate::lang::text("Carregando", "Charging"), now.battery)
            } else {
                format!("{} · {}%", crate::lang::text("Na bateria", "On battery"), now.battery)
            },
        ),
        Kind::LowBattery => (
            glyphs::battery(now.battery, false),
            None,
            format!("{} · {}%", crate::lang::text("Bateria fraca", "Low battery"), now.battery),
        ),
    };

    let badge = Rectangle::new()
        .width(40.0)
        .height(40.0)
        .fill(container)
        .radius(20.0)
        .align_child(Center, Center)
        .child(icon(glyph, 22.0, on_container));

    let mut children: Vec<Box<dyn Widget>> = vec![boxed(badge)];

    if let Some(fraction) = fraction {
        let value = motion::number(
            &format!("{key}:fraction"),
            fraction,
            Duration::from_millis(120),
        );

        children.push(boxed(wavy(
            value,
            140.0,
            false,
            scheme.primary,
            scheme.surface_container_highest,
        )));

        children.push(boxed(
            Rectangle::new()
                .width(52.0)
                .height(24.0)
                .align_child(Center, Center)
                .child(scale(Type::LabelLarge, label, scheme.on_surface)),
        ));
    } else {
        children.push(boxed(scale(Type::TitleSmall, label, scheme.on_surface)));
    }

    let pill = Rectangle::new()
        .width(PILL_WIDTH)
        .height(PILL_HEIGHT)
        .fill(scheme.surface_container_high)
        .radius(PILL_HEIGHT / 2.0)
        .border(1.0, fade(scheme.outline_variant, 80))
        .shadow(
            Shadow::drop(scheme.shadow)
                .opacity(0.38)
                .blur(24.0)
                .offset(0.0, 8.0),
        )
        .padding(Padding {
            top: 0.0,
            right: 16.0,
            bottom: 0.0,
            left: 8.0,
        })
        .opacity(presence)
        .scale(0.92 + 0.08 * presence)
        .translate(0.0, 12.0 * (1.0 - presence))
        .align_child(Center, Center)
        .child(Row::new(children).gap(12.0).align(Center).width(Parent));

    LayerWindow::new()
        .width(WINDOW_WIDTH)
        .height(WINDOW_HEIGHT)
        .anchor_vertical(Vertical::Bottom)
        .anchor_horizontal(Horizontal::Middle)
        .layer(Layer::Overlay)
        .space(Zone::Ignore)
        .margin(Margin {
            bottom: 56,
            ..Margin::default()
        })
        .visible(focused && (active.is_some() || presence > 0.001))
        .namespace("cds-osd")
        .click_through()
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .align_child(Center, Center)
                .child(pill),
        )
}
