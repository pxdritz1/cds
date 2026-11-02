use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

use amane::Service;

use crate::theme::cache_home;

/*
 * the weather card for the dash: open-meteo is asked on this thread every
 * 15 minutes, and the window only reads the last answer, so the dash never
 * waits on the network.
 * the location comes from the machine's public ip; wttr.in answers no
 * more, so the numbers come as plain json fields from open-meteo.
 */
#[derive(Default)]
pub struct Weather {
    condition: String,
    temperature: String,
    humidity: String,
    wind: String,
    code: u8,
}

impl Service for Weather {
    fn new() -> Self {
        // the last answer from a previous run puts numbers on the first card
        match cache_path().and_then(|path| fs::read_to_string(path).ok()) {
            Some(cached) => Self::parse(&cached).unwrap_or_default(),
            None => Self::default(),
        }
    }

    fn listen() {
        loop {
            if let Some(answer) = fetch() {
                if let Some(weather) = Self::parse(&answer) {
                    let mut current = Self::write();

                    *current = weather;
                }
            }

            thread::sleep(Duration::from_secs(15 * 60));
        }
    }
}

impl Weather {
    pub fn condition(&self) -> &str {
        self.condition.as_str()
    }

    pub fn temperature(&self) -> &str {
        self.temperature.as_str()
    }

    pub fn humidity(&self) -> &str {
        self.humidity.as_str()
    }

    pub fn wind(&self) -> &str {
        self.wind.as_str()
    }

    // the icon glyph the wmo code maps to, in material symbols
    pub fn glyph(&self) -> &'static str {
        glyph_for(self.code)
    }

    // parses one open-meteo answer into a weather
    fn parse(answer: &str) -> Option<Self> {
        /*
         * the data lives in "current", and the units ride a few bytes
         * earlier inside "current_units"; both use the same field names,
         * so the numbers are read after current, the units before it
         */
        let start = answer.find("\"current\":{")?;

        let numbers = &answer[start..];

        let code: u8 = json_field(numbers, "weather_code")?.parse().ok()?;

        let unit = |key: &str| {
            json_field(&answer[..start], key)
                .unwrap_or("")
                .replace('"', "")
        };

        Some(Self {
            condition: String::from(condition_for(code)),
            temperature: format!(
                "{}{}",
                json_field(numbers, "temperature_2m")?,
                unit("temperature_2m"),
            ),
            humidity: format!(
                "{}{}",
                json_field(numbers, "relative_humidity_2m")?,
                unit("relative_humidity_2m"),
            ),
            wind: format!(
                "{} {}",
                json_field(numbers, "wind_speed_10m")?,
                unit("wind_speed_10m"),
            ),
            code,
        })
    }
}

// the wmo weather codes of open-meteo, translated
fn condition_for(code: u8) -> &'static str {
    match code {
        0 => "Céu limpo",
        1..=2 => "Parcialmente nublado",
        3 => "Nublado",
        45 | 48 => "Névoa",
        51..=57 => "Garoa",
        61..=67 => "Chuva",
        71..=77 => "Neve",
        80..=82 => "Chuva forte",
        85 | 86 => "Neve forte",
        95..=99 => "Tempestade",
        _ => "Sem dados",
    }
}

// the icon glyph the wmo code maps to, in material symbols
pub fn glyph_for(code: u8) -> &'static str {
    use crate::island::widgets::{CLOUDY, FOGGY, RAINY, SNOWING, STORM, SUNNY};

    match code {
        0 => SUNNY,
        45 | 48 => FOGGY,
        71..=77 | 85 | 86 => SNOWING,
        95..=99 => STORM,
        51..=67 | 80..=82 => RAINY,
        _ => CLOUDY,
    }
}

/*
 * reads one plain json value, like "temperature_2m": 27.9,
 */
fn json_field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\":");

    let start = json.find(&needle)? + needle.len();

    let rest = &json[start..];

    let end = rest
        .find(|character| character == ',' || character == '}')
        .unwrap_or(rest.len());

    Some(rest[..end].trim())
}

fn location_url() -> Option<String> {
    /*
     * the ip tells where the machine is; open-meteo wants coordinates
     * and the ip-api keeps them as plain numbers without a key needed
     */
    let output = Command::new("curl")
        .args(["-m", "6", "-s", "http://ip-api.com/json/?fields=lat,lon"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();

    let latitude = json_field(&text, "lat")?;
    let longitude = json_field(&text, "lon")?;

    Some(format!(
        "https://api.open-meteo.com/v1/forecast?latitude={latitude}&longitude={longitude}&current=temperature_2m,relative_humidity_2m,wind_speed_10m,weather_code"
    ))
}

fn fetch() -> Option<String> {
    let location = location_url().or_else(|| {
        Some(String::from(
            "https://api.open-meteo.com/v1/forecast?latitude=-20.44&longitude=-54.65&current=temperature_2m,relative_humidity_2m,wind_speed_10m,weather_code",
        ))
    })?;

    let output = Command::new("curl").args(["-m", "8", "-s", &location]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();

    if text.is_empty() || !text.contains("weather_code") {
        return None;
    }

    cache(&text);

    Some(text)
}

fn cache(text: &str) {
    let folder = cache_home().join("chloe-dynamic-shell");

    if fs::create_dir_all(&folder).is_err() {
        return;
    }

    let _ = fs::write(folder.join("weather.json"), text);
}

fn cache_path() -> Option<std::path::PathBuf> {
    Some(cache_home().join("chloe-dynamic-shell/weather.json"))
}

pub fn command(arguments: &[String]) -> String {
    if !arguments.is_empty() {
        return String::from("usage: weather");
    }

    let weather = Weather::read();

    format!(
        "{} | {} | humid. {} | vento {}",
        weather.condition(),
        weather.temperature(),
        weather.humidity(),
        weather.wind()
    )
}
