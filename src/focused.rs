use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use amane::{Apps, Service};

/// The focused Niri window, refreshed off the rendering thread.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct FocusedWindow {
    title: String,
    app_id: String,
    icon_path: Option<String>,
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

    pub fn icon_path(&self) -> Option<&str> {
        self.icon_path.as_deref()
    }
}

fn publish(fresh: Option<FocusedWindow>) {
    let Some(fresh) = fresh else {
        return;
    };

    let changed = {
        let current = FocusedWindow::read();
        *current != fresh
    };

    if changed {
        *FocusedWindow::write() = fresh;
    }
}

fn read() -> Option<FocusedWindow> {
    let socket = std::env::var_os("NIRI_SOCKET")?;
    let mut stream = UnixStream::connect(socket).ok()?;
    stream.write_all(b"\"FocusedWindow\"\n").ok()?;

    let mut json = String::new();
    BufReader::new(stream).read_line(&mut json).ok()?;

    let title = json_string(&json, "title").unwrap_or_default();
    let app_id = json_string(&json, "app_id").unwrap_or_default();
    let app_id_lower = app_id.to_lowercase();

    // Resolve the icon only when focus changes, not on every bar redraw.
    let apps = Apps::read();
    let icon_path = apps
        .list()
        .iter()
        .find(|app| {
            app.name().eq_ignore_ascii_case(&app_id)
                || (!app_id_lower.is_empty()
                    && app.exec().to_lowercase().contains(&app_id_lower))
        })
        .and_then(|app| app.icon_path())
        .map(|path| path.to_string_lossy().into_owned());
    drop(apps);

    Some(FocusedWindow {
        title,
        app_id,
        icon_path,
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
