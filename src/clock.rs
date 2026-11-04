use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::Service;

#[derive(Default, PartialEq)]
pub struct Clock {
    time24: String,
    time12: String,
    date: String,
    short_date: String,
    greeting: String,
    year: i32,
    month: u32,
    day: u32,
    weekday: u32,
}

impl Service for Clock {
    fn new() -> Self {
        read().unwrap_or_default()
    }

    fn listen() {
        loop {
            // The UI displays minutes, so polling every five seconds is sufficient.
            thread::sleep(Duration::from_secs(5));

            let Some(fresh) = read() else {
                continue;
            };

            if *Self::read() == fresh {
                continue;
            }

            *Self::write() = fresh;
        }
    }
}

impl Clock {
    pub fn time(&self, clock24: bool) -> &str {
        if self.time24.is_empty() {
            return "--:--";
        }

        if clock24 { &self.time24 } else { &self.time12 }
    }

    pub fn date(&self) -> &str {
        &self.date
    }

    pub fn short_date(&self) -> &str {
        &self.short_date
    }

    pub fn greeting(&self) -> &str {
        &self.greeting
    }

    pub fn year(&self) -> i32 {
        self.year
    }

    pub fn month(&self) -> u32 {
        self.month
    }

    pub fn day(&self) -> u32 {
        self.day
    }

    pub fn weekday(&self) -> u32 {
        self.weekday
    }
}

fn greeting(hour: u32) -> &'static str {
    match hour {
        5..=11 => crate::lang::text("Bom dia", "Good morning"),
        12..=17 => crate::lang::text("Boa tarde", "Good afternoon"),
        _ => crate::lang::text("Boa noite", "Good evening"),
    }
}

fn read() -> Option<Clock> {
    let output = ["date", "/usr/bin/date"].iter().find_map(|program| {
        let mut command = Command::new(program);
        if crate::settings::Settings::read().language() == "en" {
            command.env("LC_TIME", "C");
        }
        command
            .arg(if crate::settings::Settings::read().language() == "en" { "+%H:%M%n%I:%M %p%n%A, %B %d%n%a, %b %d%n%H%n%Y %m %d %w" } else { "+%H:%M%n%I:%M %p%n%A, %d de %B%n%a, %d %b%n%H%n%Y %m %d %w" })
            .output()
            .ok()
            .filter(|output| output.status.success())
    })?;

    let text = String::from_utf8(output.stdout).ok()?;
    let mut lines = text.lines().map(str::trim);

    let time24 = lines.next()?.to_string();
    let time12 = lines.next()?.to_string();
    let date = lines.next()?.to_string();
    let short_date = lines.next()?.to_string();
    let hour: u32 = lines.next()?.parse().ok()?;

    let mut numbers = lines.next()?.split_whitespace();

    let year = numbers.next()?.parse().ok()?;
    let month = numbers.next()?.parse().ok()?;
    let day = numbers.next()?.parse().ok()?;
    let weekday = numbers.next()?.parse().ok()?;

    Some(Clock {
        time24,
        time12,
        date,
        short_date,
        greeting: greeting(hour).to_string(),
        year,
        month,
        day,
        weekday,
    })
}
