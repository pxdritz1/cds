use std::path::PathBuf;

use anyhow::{Context, Result};
use tokio::process::Command;

use crate::paths::Paths;

pub const HISTORY_LIMIT: usize = 80;
pub const MAX_BYTES: usize = 200_000;

#[derive(Debug, Clone, Default)]
pub struct ClipboardHistory {
    entries: Vec<String>,
}

impl ClipboardHistory {
    pub fn load() -> Self {
        let mut history = Self::default();
        let Ok(path) = history_path() else {
            return history;
        };
        let Ok(contents) = std::fs::read_to_string(path) else {
            return history;
        };

        for line in contents.lines().filter(|line| !line.is_empty()) {
            history.entries.push(unescape(line));
        }
        history.entries.truncate(HISTORY_LIMIT);
        history
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<String> {
        let query = query.trim().to_lowercase();
        self.entries
            .iter()
            .filter(|entry| query.is_empty() || entry.to_lowercase().contains(&query))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Add one text selection. Sensitive selections, empty text and oversized
    /// entries are ignored. Duplicate entries move back to the top.
    pub fn push(&mut self, text: &str) -> Result<bool> {
        let text = text.trim_end_matches('\n');
        if text.trim().is_empty() || text.len() > MAX_BYTES {
            return Ok(false);
        }

        self.insert(text.to_owned());
        self.save()?;
        Ok(true)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.save()
    }

    fn insert(&mut self, text: String) {
        self.entries.retain(|entry| entry != &text);
        self.entries.insert(0, text);
        self.entries.truncate(HISTORY_LIMIT);
    }

    fn save(&self) -> Result<()> {
        let path = history_path()?;
        let parent = path.parent().context("clipboard history path has no parent")?;
        std::fs::create_dir_all(parent)?;

        let mut contents = String::new();
        for entry in &self.entries {
            contents.push_str(&escape(entry));
            contents.push('\n');
        }

        let temporary = path.with_extension("conf.tmp");
        std::fs::write(&temporary, contents)?;
        std::fs::rename(&temporary, &path)
            .with_context(|| format!("could not save clipboard history to {}", path.display()))?;
        Ok(())
    }
}

pub fn history_path() -> Result<PathBuf> {
    Ok(Paths::discover()?.cache.join("clipboard.conf"))
}

pub async fn copy_text(text: &str) -> Result<()> {
    let status = Command::new("wl-copy")
        .arg("--")
        .arg(text)
        .status()
        .await
        .context("could not start wl-copy")?;

    if !status.success() {
        anyhow::bail!("wl-copy exited with status {status}");
    }
    Ok(())
}

fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn unescape(line: &str) -> String {
    let mut text = String::new();
    let mut characters = line.chars();

    while let Some(character) = characters.next() {
        if character != '\\' {
            text.push(character);
            continue;
        }

        match characters.next() {
            Some('n') => text.push('\n'),
            Some('r') => text.push('\r'),
            Some('\\') => text.push('\\'),
            Some(other) => {
                text.push('\\');
                text.push(other);
            }
            None => text.push('\\'),
        }
    }

    text
}

#[cfg(test)]
mod tests {
    use super::{ClipboardHistory, HISTORY_LIMIT};

    #[test]
    fn deduplicates_and_keeps_newest_first() {
        let mut history = ClipboardHistory::default();
        history.insert("first".into());
        history.insert("second".into());
        history.insert("first".into());

        assert_eq!(history.entries(), &["first", "second"]);
    }

    #[test]
    fn enforces_history_limit() {
        let mut history = ClipboardHistory::default();
        for index in 0..HISTORY_LIMIT + 5 {
            history.insert(index.to_string());
        }

        assert_eq!(history.entries().len(), HISTORY_LIMIT);
        assert_eq!(history.entries()[0], (HISTORY_LIMIT + 4).to_string());
    }

    #[test]
    fn escapes_multiline_text_without_losing_backslashes() {
        let original = "line one\nline two\\path";
        assert_eq!(super::unescape(&super::escape(original)), original);
    }
}
