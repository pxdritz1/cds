use amane::Service;

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

            let (time, date, hour) = local_now();

            // only the minute is shown, so the rest of the seconds stay quiet
            let clock = Self::read();

            if clock.time == time && clock.date == date {
                continue;
            }

            drop(clock);

            let mut clock = Self::write();

            clock.time = time;
            clock.date = date;
            clock.greeting = greeting(hour);
        }
    }
}

impl Clock {
    fn load() -> Self {
        let (time, date, hour) = local_now();

        Self {
            time,
            date,
            greeting: greeting(hour),
        }
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

fn greeting(hour: u32) -> String {
    match hour {
        5..=11 => String::from("Bom dia"),
        12..=17 => String::from("Boa tarde"),
        _ => String::from("Boa noite"),
    }
}

// the local time comes from date(1), so the shell needs no date crate
fn local_now() -> (String, String, u32) {
    let Some(output) = run_date() else {
        return (String::from("--:--"), String::new(), 0);
    };

    let mut lines = output.lines();

    let time = lines.next().unwrap_or_default().trim().to_string();
    let date = lines.next().unwrap_or_default().trim().to_string();
    let hour = lines
        .next()
        .and_then(|hour| hour.trim().parse().ok())
        .unwrap_or(0);

    (time, date, hour)
}

fn run_date() -> Option<String> {
    for program in ["date", "/usr/bin/date"] {
        let output = std::process::Command::new(program)
            .arg("+%H:%M%n%A, %d de %B%n%H")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                return String::from_utf8(output.stdout).ok();
            }
        }
    }

    None
}
