use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use amane::Service;

/// The focused Niri window, refreshed off the rendering thread.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct FocusedWindow {
    title: String,
    app_id: String,
}

impl Service for FocusedWindow {
    fn new() -> Self {
        read().unwrap_or_default()
    }

    fn listen() {
        loop {
            // Niri's event stream sends an initial state and then only changes.
            // Keep one process alive instead of spawning a query every second.
            let mut child = match Command::new("niri")
                .args(["msg", "--json", "event-stream"])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => child,
                Err(_) => {
                    thread::sleep(Duration::from_secs(3));
                    continue;
                }
            };

            let Some(stdout) = child.stdout.take() else {
                let _ = child.kill();
                thread::sleep(Duration::from_secs(1));
                continue;
            };

            publish(read());
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                let Ok(line) = line else {
                    break;
                };

                if line.contains("WindowFocusChanged")
                    || line.contains("WindowOpenedOrChanged")
                    || line.contains("WindowsChanged")
                {
                    publish(read());
                }
            }

            let _ = child.kill();
            thread::sleep(Duration::from_secs(1));
        }
    }
}

impl FocusedWindow {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }
}

fn publish(fresh: Option<FocusedWindow>) {
    let Some(fresh) = fresh else {
        return;
    };

    if *FocusedWindow::read() != fresh {
        *FocusedWindow::write() = fresh;
    }
}

fn read() -> Option<FocusedWindow> {
    let output = Command::new("niri")
        .args(["msg", "--json", "focused-window"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json = String::from_utf8(output.stdout).ok()?;

    Some(FocusedWindow {
        title: json_string(&json, "title").unwrap_or_default(),
        app_id: json_string(&json, "app_id").unwrap_or_default(),
    })
}

/// Extract a JSON string without introducing a parser dependency just for
/// two fields. Handles escaped quotes, slashes, control characters and
/// UTF-16 code points emitted by Niri.
fn json_string(input: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\"");
    let start = input.find(&marker)? + marker.len();
    let tail = &input[start..];
    let colon = tail.find(':')?;
    let mut chars = tail[colon + 1..].chars().peekable();

    while matches!(chars.peek(), Some(character) if character.is_whitespace()) {
        chars.next();
    }

    if chars.next()? != '"' {
        return None;
    }

    let mut value = String::new();

    while let Some(character) = chars.next() {
        match character {
            '"' => return Some(value),
            '\\' => {
                let escaped = chars.next()?;
                match escaped {
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    '/' => value.push('/'),
                    'b' => value.push('\u{0008}'),
                    'f' => value.push('\u{000c}'),
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    'u' => {
                        let digits: String = chars.by_ref().take(4).collect();
                        let code = u32::from_str_radix(&digits, 16).ok()?;
                        value.push(char::from_u32(code)?);
                    }
                    _ => return None,
                }
            }
            character if character.is_control() => return None,
            other => value.push(other),
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::json_string;

    #[test]
    fn parses_escaped_window_titles() {
        assert_eq!(
            json_string(r#"{"title":"A \"quoted\" title","app_id":"editor"}"#, "title"),
            Some(String::from("A \"quoted\" title")),
        );
    }

    #[test]
    fn parses_unicode_escapes() {
        assert_eq!(
            json_string(r#"{"title":"Chloe \u2728"}"#, "title"),
            Some(String::from("Chloe ✨")),
        );
    }
}
