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
 * the hourly forecast rides the same answer for the weather tab's strip.
 */
#[derive(Default)]
pub struct Weather {
    condition: String,
    temperature: String,
    humidity: String,
    wind: String,
    code: u8,
    hourly: Vec<Hour>,
}

// one hour of the weather tab's strip
pub struct Hour {
    pub label: String,
    pub temperature: String,
    pub glyph: &'static str,
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

    // the next hours of the forecast, for the weather tab's strip
    pub fn hourly(&self) -> &[Hour] {
        &self.hourly
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
            hourly: parse_hourly(answer),
        })
    }
}

/*
 * the hourly arrays live in "hourly": times as "T16:00" strings, codes and
 * temperatures as comma separated lists of numbers kept in line
 */
fn parse_hourly(answer: &str) -> Vec<Hour> {
    const HOURS: usize = 12;

    let start = match answer.find("\"hourly\":{") {
        Some(start) => start,
        None => return Vec::new(),
    };

    let hourly = &answer[start..];

    let times = json_array(hourly, "time");

    let codes = json_array(hourly, "weather_code");

    let temps = json_array(hourly, "temperature_2m");

    let unit = json_field(&answer[..start], "temperature_2m")
        .unwrap_or("")
        .replace('"', "");

    let now = current_hour(&times);

    let mut hours = Vec::new();

    for step in 0..HOURS {
        let index = now + step;

        let Some(label) = times.get(index).map(|time| hour_of(time)) else {
            break;
        };

        let (Some(code), Some(temperature)) = (
            codes.get(index).and_then(|code| code.parse::<u8>().ok()),
            temps.get(index),
        ) else {
            break;
        };

        hours.push(Hour {
            label,
            temperature: format!("{temperature}{unit}"),
            glyph: glyph_for(code),
        });
    }

    hours
}

// the index of the first hourly entry from this hour onward
fn current_hour(times: &[&str]) -> usize {
    let stamp = date_minute();

    let mut found = 0;

    for (index, time) in times.iter().enumerate() {
        if let Some(later) = hour_minute(time) {
            if later >= stamp {
                found = index;

                return found;
            }
        }
    }

    found
}

// this hour and minute as minutes of the day, from the date command
fn date_minute() -> u32 {
    let output = Command::new("date").args(["+%H:%M"]).output().ok();

    let stamp = output
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_default();

    hour_minute(stamp.trim()).unwrap_or(0)
}

// "16:45" as minutes of the day
fn hour_minute(stamp: &str) -> Option<u32> {
    let (hour, minute) = stamp.split_once(':')?;

    Some(hour.parse::<u32>().ok()? * 60 + minute.parse::<u32>().ok()?)
}

// "2026-11-02T16:00" becomes "16h"
fn hour_of(time: &str) -> String {
    let hour = time
        .rsplit('T')
        .next()
        .unwrap_or(time)
        .split(':')
        .next()
        .unwrap_or("?");

    format!("{hour}h")
}

// numbers or strings of one json array, read in line as plain values
fn json_array<'a>(json: &'a str, key: &str) -> Vec<&'a str> {
    let needle = format!("\"{key}\":[");

    let Some(start) = json.find(&needle) else {
        return Vec::new();
    };

    let rest = &json[start + needle.len()..];

    let Some(end) = rest.find(']') else {
        return Vec::new();
    };

    let body = rest[..end].trim();

    let mut values = Vec::new();

    // comma separators outside of quotes slice the body in place
    let mut piece_start = 0;

    let mut in_string = false;

    for (index, letter) in body.char_indices() {
        match letter {
            '"' => in_string = !in_string,
            ',' if !in_string => {
                let piece = body[piece_start..index].trim();

                if !piece.is_empty() {
                    values.push(piece);
                }

                piece_start = index + 1;
            }
            _ => {}
        }
    }

    let last = body[piece_start..].trim();

    if !last.is_empty() {
        values.push(last);
    }

    values
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

// the forecast url for one pair of coordinates
fn coordinates_url(latitude: f32, longitude: f32) -> String {
    format!(
        "https://api.open-meteo.com/v1/forecast?latitude={latitude:.4}&longitude={longitude:.4}&current=temperature_2m,relative_humidity_2m,wind_speed_10m,weather_code&hourly=temperature_2m,weather_code&forecast_hours=13&timezone=auto"
    )
}

// the spot for the forecast: the setting first, then the ip, then a fallback
fn location_url() -> String {
    if let Some((latitude, longitude)) = crate::settings::Settings::read().coordinates() {
        return coordinates_url(latitude, longitude);
    }

    /*
     * the ip tells where the machine is; open-meteo wants coordinates
     * and the ip-api keeps them as plain numbers without a key needed
     */
    let ip = Command::new("curl")
        .args(["-m", "6", "-s", "http://ip-api.com/json/?fields=lat,lon"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned());

    if let Some(text) = ip {
        if let (Some(latitude), Some(longitude)) =
            (json_field(&text, "lat"), json_field(&text, "lon"))
        {
            if let (Ok(latitude), Ok(longitude)) = (
                latitude.trim().parse::<f32>(),
                longitude.trim().parse::<f32>(),
            ) {
                return coordinates_url(latitude, longitude);
            }
        }
    }

    // nothing answered: the dash still shows the region default
    coordinates_url(-20.44, -54.65)
}

fn fetch() -> Option<String> {
    let location = location_url();

    let output = Command::new("curl")
        .args(["-m", "8", "-s", &location])
        .output()
        .ok()?;

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
