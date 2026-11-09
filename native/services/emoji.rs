use std::path::PathBuf;

use anyhow::{Context, Result};
use emojis::Emoji;
use tokio::process::Command;

use crate::paths::Paths;
use crate::services::fuzzy;

#[derive(Debug, Clone, Copy)]
pub struct EmojiEntry {
    pub emoji: &'static Emoji,
    pub group: &'static str,
}

impl EmojiEntry {
    pub fn glyph(self) -> &'static str {
        self.emoji.as_str()
    }

    pub fn name(self) -> &'static str {
        self.emoji.name()
    }

    pub fn shortcode(self) -> Option<&'static str> {
        self.emoji.shortcode()
    }
}

/// Search the full Unicode emoji catalog, including the localized aliases
/// retained from the previous CDS picker. The catalog itself tracks the
/// current Unicode release and includes skin-tone variants through a separate
/// helper, so the base grid stays compact.
pub fn search(query: &str, limit: usize) -> Vec<EmojiEntry> {
    let query = query.trim().to_lowercase();

    if query.is_empty() {
        return emojis::iter()
            .take(limit)
            .map(|emoji| EmojiEntry {
                emoji,
                group: group_name(emoji.group()),
            })
            .collect();
    }

    let mut matches = emojis::iter()
        .filter_map(|emoji| {
            let glyph = emoji.as_str();
            let aliases = portuguese_aliases(glyph).join(" ");
            let shortcode = emoji.shortcode().unwrap_or_default().replace('_', " ");
            let haystack = format!("{} {} {}", emoji.name(), shortcode, aliases);
            let score = fuzzy::score(&query, &haystack)?;

            Some((
                score as u32,
                EmojiEntry {
                    emoji,
                    group: group_name(emoji.group()),
                },
            ))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .cmp(left_score)
            .then_with(|| left.name().cmp(right.name()))
    });

    matches
        .into_iter()
        .take(limit)
        .map(|(_, entry)| entry)
        .collect()
}

/// Return all supported skin-tone variants for an emoji, if the Unicode
/// sequence supports them. The selected base emoji is always included.
pub fn skin_tones(emoji: &'static Emoji) -> Vec<&'static Emoji> {
    emoji
        .skin_tones()
        .map(|tones| tones.collect())
        .unwrap_or_else(|| vec![emoji])
}

/// Resolve the official Noto SVG asset installed by the CDS asset installer.
/// If an asset is absent, callers should render the Unicode glyph with the
/// system emoji font as a graceful fallback.
pub fn asset_path(emoji: &Emoji) -> Option<PathBuf> {
    let paths = Paths::discover().ok()?;
    let codepoints = emoji
        .as_str()
        .chars()
        .filter(|character| *character != '\u{fe0f}')
        .map(|character| format!("{:x}", character as u32))
        .collect::<Vec<_>>()
        .join("_");

    let path = paths
        .data
        .join("emoji")
        .join(format!("emoji_u{codepoints}.svg"));

    path.is_file().then_some(path)
}

pub async fn copy_to_clipboard(emoji: &Emoji) -> Result<()> {
    copy_text(emoji.as_str()).await
}

pub async fn copy_text(value: &str) -> Result<()> {
    let status = Command::new("wl-copy")
        .arg("--")
        .arg(value)
        .status()
        .await
        .context("could not start wl-copy")?;

    if !status.success() {
        anyhow::bail!("wl-copy exited with status {status}");
    }

    Ok(())
}

fn group_name(group: emojis::Group) -> &'static str {
    match format!("{group:?}").as_str() {
        "SmileysAndEmotion" => "Smileys & Emotion",
        "PeopleAndBody" => "People & Body",
        "AnimalsAndNature" => "Animals & Nature",
        "FoodAndDrink" => "Food & Drink",
        "TravelAndPlaces" => "Travel & Places",
        "Activities" => "Activities",
        "Objects" => "Objects",
        "Symbols" => "Symbols",
        "Flags" => "Flags",
        _ => "Other",
    }
}

fn portuguese_aliases(glyph: &str) -> &'static [&'static str] {
    match glyph {
        "😀" => &["sorriso", "feliz", "alegre"],
        "😂" | "🤣" => &["rir", "rindo", "chorando de rir", "risada"],
        "😭" | "😢" => &["chorar", "choro", "triste", "lágrimas"],
        "🥹" => &["emocionado", "segurando lágrimas", "fofo"],
        "❤️" | "💗" | "💖" | "💕" => &["coração", "amor", "paixão"],
        "🩷" => &["coração rosa", "rosa", "amor"],
        "🔥" => &["fogo", "quente", "brabo"],
        "✨" => &["brilho", "magia", "estrela", "brilhando"],
        "🎉" | "🎊" => &["festa", "comemoração", "parabéns"],
        "🌈" => &["arco-íris", "orgulho"],
        "☕" => &["café"],
        "🍕" => &["pizza"],
        "🍔" => &["hambúrguer", "lanche"],
        "🐈" | "🐈‍⬛" => &["gato", "gatinho"],
        "🐕" => &["cachorro", "cão", "dog"],
        "🚀" => &["foguete", "lançamento"],
        "💻" => &["computador", "pc", "notebook"],
        "🎮" => &["jogo", "videogame", "controle"],
        "🏳️‍⚧️" => &["bandeira trans", "trans", "orgulho trans"],
        "🏳️‍🌈" => &["bandeira lgbt", "orgulho", "arco-íris"],
        "👍" => &["positivo", "joinha", "curtir"],
        "🙏" => &["obrigado", "por favor", "oração"],
        "💀" => &["caveira", "morto", "morrendo"],
        "🫠" => &["derretendo", "vergonha", "calor"],
        "💅" => &["unha", "slay", "diva"],
        "🫶" => &["mãos coração", "amor"],
        "☀️" => &["sol", "ensolarado"],
        "🌙" => &["lua", "noite"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::{search, skin_tones};

    #[test]
    fn search_includes_current_unicode_catalog() {
        let results = search("rocket", 10);
        assert!(results.iter().any(|entry| entry.glyph() == "🚀"));
    }

    #[test]
    fn search_includes_portuguese_aliases() {
        let results = search("coração", 20);
        assert!(results.iter().any(|entry| entry.glyph() == "❤️"));
    }

    #[test]
    fn skin_tone_fallback_keeps_the_original_emoji() {
        let heart = emojis::get("❤️").unwrap();
        assert_eq!(skin_tones(heart)[0].as_str(), heart.as_str());
    }
}
