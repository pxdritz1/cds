use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use amane::Service;

#[derive(Default)]
pub struct Clock {
    minute_stamp: u64,
    pub time: String,
    pub date: String,
    pub month_label: String,
    pub month: u32,
    pub year: i32,
    pub day: u32,
    pub weekday: u32,
}

impl Service for Clock {
    fn new() -> Self {
        let mut clock = Self::default();
        clock.update();
        clock
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    fn update(&mut self) -> bool {
        let minute_stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 60;

        if minute_stamp == self.minute_stamp && !self.time.is_empty() {
            return false;
        }

        let output = Command::new("date")
            .arg("+%H:%M|%A, %d %B|%B %Y|%m|%Y|%d|%u")
            .output();

        let Ok(output) = output else {
            return false;
        };

        if !output.status.success() {
            return false;
        }

        let value = String::from_utf8_lossy(&output.stdout);
        let fields: Vec<&str> = value.trim().split('|').collect();

        if fields.len() != 7 {
            return false;
        }

        let month = fields[3].parse().unwrap_or(1);
        let year = fields[4].parse().unwrap_or(1970);
        let day = fields[5].parse().unwrap_or(1);
        let weekday = fields[6].parse().unwrap_or(1);

        let changed = self.time != fields[0]
            || self.date != fields[1]
            || self.month_label != fields[2]
            || self.month != month
            || self.year != year
            || self.day != day
            || self.weekday != weekday;

        self.minute_stamp = minute_stamp;
        self.time = fields[0].to_string();
        self.date = fields[1].to_string();
        self.month_label = fields[2].to_string();
        self.month = month;
        self.year = year;
        self.day = day;
        self.weekday = weekday;

        changed
    }
}
