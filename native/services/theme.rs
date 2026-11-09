use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::DynamicImage;
use material_color_utils::{
    MaterializedScheme,
    theme_from_color,
    theme_from_image,
    utils::color_utils::Argb,
};
use serde::{Deserialize, Serialize};

use crate::paths::Paths;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemeTokens {
    pub source_color: String,
    pub light: SchemeTokens,
    pub dark: SchemeTokens,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemeTokens {
    pub primary: String,
    pub on_primary: String,
    pub primary_container: String,
    pub on_primary_container: String,
    pub secondary: String,
    pub on_secondary: String,
    pub tertiary: String,
    pub on_tertiary: String,
    pub background: String,
    pub on_background: String,
    pub surface: String,
    pub on_surface: String,
    pub surface_container_low: String,
    pub surface_container: String,
    pub surface_container_high: String,
    pub surface_variant: String,
    pub on_surface_variant: String,
    pub outline: String,
    pub error: String,
    pub on_error: String,
}

impl ThemeTokens {
    pub fn from_hex(hex: &str) -> Result<Self> {
        let source = Argb::from_hex(hex)
            .with_context(|| format!("invalid source color: {hex}"))?;
        let theme = theme_from_color(source).call();

        Ok(Self {
            source_color: hex_string(source),
            light: SchemeTokens::from_scheme(&theme.schemes.light),
            dark: SchemeTokens::from_scheme(&theme.schemes.dark),
        })
    }

    pub fn from_image(image: &DynamicImage) -> Result<Self> {
        let theme = theme_from_image(image)
            .call()
            .context("could not extract a Material 3 source color from the image")?;

        Ok(Self {
            source_color: hex_string(theme.schemes.light.source_color),
            light: SchemeTokens::from_scheme(&theme.schemes.light),
            dark: SchemeTokens::from_scheme(&theme.schemes.dark),
        })
    }

    pub fn load() -> Result<Option<Self>> {
        let path = theme_path()?;
        if !path.is_file() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("could not read theme file {}", path.display()))?;
        let theme = serde_json::from_str(&contents)
            .with_context(|| format!("invalid theme file {}", path.display()))?;

        Ok(Some(theme))
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = theme_path()?;
        let parent = path.parent().context("theme path has no parent directory")?;
        std::fs::create_dir_all(parent)?;

        let temporary = path.with_extension("json.tmp");
        let contents = serde_json::to_vec_pretty(self)?;
        std::fs::write(&temporary, contents)?;
        std::fs::rename(&temporary, &path)
            .with_context(|| format!("could not atomically write {}", path.display()))?;

        Ok(path)
    }
}

impl SchemeTokens {
    fn from_scheme(scheme: &MaterializedScheme) -> Self {
        Self {
            primary: hex_string(scheme.primary),
            on_primary: hex_string(scheme.on_primary),
            primary_container: hex_string(scheme.primary_container),
            on_primary_container: hex_string(scheme.on_primary_container),
            secondary: hex_string(scheme.secondary),
            on_secondary: hex_string(scheme.on_secondary),
            tertiary: hex_string(scheme.tertiary),
            on_tertiary: hex_string(scheme.on_tertiary),
            background: hex_string(scheme.background),
            on_background: hex_string(scheme.on_background),
            surface: hex_string(scheme.surface),
            on_surface: hex_string(scheme.on_surface),
            surface_container_low: hex_string(scheme.surface_container_low),
            surface_container: hex_string(scheme.surface_container),
            surface_container_high: hex_string(scheme.surface_container_high),
            surface_variant: hex_string(scheme.surface_variant),
            on_surface_variant: hex_string(scheme.on_surface_variant),
            outline: hex_string(scheme.outline),
            error: hex_string(scheme.error),
            on_error: hex_string(scheme.on_error),
        }
    }
}

pub fn generate_from_wallpaper(path: &Path) -> Result<ThemeTokens> {
    let image = image::open(path)
        .with_context(|| format!("could not decode wallpaper {}", path.display()))?;
    ThemeTokens::from_image(&image)
}

pub fn generate_from_hex(hex: &str) -> Result<ThemeTokens> {
    ThemeTokens::from_hex(hex)
}

pub fn theme_path() -> Result<PathBuf> {
    Ok(Paths::discover()?.config.join("theme.json"))
}

fn hex_string(color: Argb) -> String {
    format!("#{:06X}", color.0 & 0x00FF_FFFF)
}

#[cfg(test)]
mod tests {
    use super::ThemeTokens;

    #[test]
    fn generates_light_and_dark_material_schemes() {
        let theme = ThemeTokens::from_hex("#6750A4").unwrap();
        assert_eq!(theme.source_color, "#6750A4");
        assert_ne!(theme.light.primary, theme.dark.primary);
        assert!(!theme.light.surface.is_empty());
        assert!(!theme.dark.surface.is_empty());
    }

    #[test]
    fn rejects_invalid_hex_colors() {
        assert!(ThemeTokens::from_hex("not-a-color").is_err());
    }
}
