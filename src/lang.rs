use crate::settings::Settings;

/// Returns the string matching the language selected in Settings.
pub fn text<'a>(pt_br: &'a str, en: &'a str) -> &'a str {
    if Settings::read().language() == "en" { en } else { pt_br }
}
