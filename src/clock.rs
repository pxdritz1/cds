use amane::Service;
use chrono::{Local, Timelike};

#[derive(Default)]
pub struct Clock {
    time: String,
    date: String,
    greeting: String,
}

impl Service for Clock {
    fn new() -> Self {
        Self::load()
    }

    fn listen() {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            Self::write().reload();
        }
    }
}

impl Clock {
    fn load() -> Self {
        let mut clock = Self::default();
        clock.reload();
        clock
    }

    fn reload(&mut self) {
        let now = Local::now();
        self.time = now.format("%H:%M").to_string();
        self.date = now.format("%A, %d de %B").to_string();

        let hour = now.hour();
        self.greeting = match hour {
            5..=11 => String::from("Bom dia"),
            12..=17 => String::from("Boa tarde"),
            _ => String::from("Boa noite"),
        };
    }

    pub fn time(&self) -> &str {
        &self.time
    }

    pub fn date(&self) -> &str {
        &self.date
    }

    pub fn greeting(&self) -> &str {
        &self.greeting
    }
}
