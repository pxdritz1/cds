use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use freedesktop_desktop_entry::{
    DesktopEntry, Iter, current_desktop, default_paths, get_languages_from_env,
};
use freedesktop_icons::{default_theme_gtk, lookup};

use crate::services::fuzzy;

#[derive(Debug, Clone)]
pub struct DesktopApp {
    pub id: String,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub icon: Option<PathBuf>,
    pub terminal: bool,
    pub working_directory: Option<PathBuf>,
    exec: Vec<String>,
}

impl DesktopApp {
    pub fn launch(&self) -> Result<()> {
        let (program, args) = self.exec.split_first().context("desktop entry has no Exec command")?;

        let (program, args) = if self.terminal {
            let terminal = env::var("TERMINAL").unwrap_or_else(|_| "alacritty".into());
            let mut terminal = shell_words::split(&terminal)
                .unwrap_or_else(|_| vec!["alacritty".into()]);
            if terminal.is_empty() {
                terminal.push("alacritty".into());
            }
            let executable = terminal.remove(0);
            let mut terminal_args = terminal;
            terminal_args.push("-e".into());
            terminal_args.push(program.clone());
            terminal_args.extend(args.iter().cloned());
            (executable, terminal_args)
        } else {
            (program.clone(), args.to_vec())
        };

        let mut command = Command::new(&program);
        command.args(args);

        if let Some(directory) = &self.working_directory {
            if directory.is_dir() {
                command.current_dir(directory);
            }
        }

        command.spawn().with_context(|| {
            format!("could not launch {} ({})", self.name, program)
        })?;

        Ok(())
    }
}

pub fn scan() -> Vec<DesktopApp> {
    let locales = get_languages_from_env();
    let desktops = current_desktop().unwrap_or_default();
    let mut seen = HashSet::new();

    let mut apps = Iter::new(default_paths())
        .entries(Some(&locales))
        .filter_map(|entry| {
            if entry.type_() != Some("Application")
                || entry.hidden()
                || entry.no_display()
                || !visible_in_current_desktop(&entry, &desktops)
            {
                return None;
            }

            if entry
                .try_exec()
                .is_some_and(|executable| !executable_exists(executable))
            {
                return None;
            }

            if !seen.insert(entry.appid.clone()) {
                return None;
            }

            let exec = entry.parse_exec().ok()?;
            let (name, description, keywords) = localized_metadata(&entry, &locales);
            let icon = entry.icon().and_then(resolve_icon);

            Some(DesktopApp {
                id: entry.appid.clone(),
                name,
                description,
                keywords,
                icon,
                terminal: entry.terminal(),
                working_directory: entry.path().map(PathBuf::from),
                exec,
            })
        })
        .collect::<Vec<_>>();

    apps.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });

    apps
}

pub fn search<'a>(apps: &'a [DesktopApp], query: &str, limit: usize) -> Vec<&'a DesktopApp> {
    let query = query.trim();
    if query.is_empty() {
        return apps.iter().take(limit).collect();
    }

    let mut matches = apps
        .iter()
        .filter_map(|app| {
            let haystack = format!(
                "{} {} {} {}",
                app.name,
                app.id,
                app.description,
                app.keywords.join(" ")
            );

            let score = fuzzy::score(query, &haystack)?;
            Some((score, app))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .cmp(left_score)
            .then_with(|| left.name.cmp(&right.name))
    });

    matches
        .into_iter()
        .take(limit)
        .map(|(_, app)| app)
        .collect()
}

fn localized_metadata(entry: &DesktopEntry, locales: &[String]) -> (String, String, Vec<String>) {
    let name = entry
        .full_name(locales)
        .map(|name| name.into_owned())
        .unwrap_or_else(|| entry.appid.clone());
    let description = entry
        .comment(locales)
        .map(|comment| comment.into_owned())
        .unwrap_or_default();
    let keywords = entry
        .keywords(locales)
        .unwrap_or_default()
        .into_iter()
        .map(|keyword| keyword.into_owned())
        .collect();

    (name, description, keywords)
}

fn visible_in_current_desktop(entry: &DesktopEntry, desktops: &[String]) -> bool {
    if let Some(only_show_in) = entry.only_show_in() {
        if !only_show_in
            .iter()
            .any(|desktop| desktops.iter().any(|current| current.eq_ignore_ascii_case(desktop)))
        {
            return false;
        }
    }

    if let Some(not_show_in) = entry.not_show_in() {
        if not_show_in
            .iter()
            .any(|desktop| desktops.iter().any(|current| current.eq_ignore_ascii_case(desktop)))
        {
            return false;
        }
    }

    true
}

fn executable_exists(executable: &str) -> bool {
    let path = Path::new(executable);
    if path.components().count() > 1 {
        return path.is_file();
    }

    env::var_os("PATH").is_some_and(|paths| {
        env::split_paths(&paths).any(|directory| directory.join(executable).is_file())
    })
}

fn resolve_icon(icon: &str) -> Option<PathBuf> {
    let path = Path::new(icon);
    if path.is_absolute() && path.is_file() {
        return Some(path.to_path_buf());
    }

    if let Some(theme) = default_theme_gtk() {
        lookup(icon)
            .with_size(96)
            .with_cache()
            .with_theme(&theme)
            .find()
    } else {
        lookup(icon).with_size(96).with_cache().find()
    }
}

pub fn launch_by_id(apps: &[DesktopApp], id: &str) -> Result<()> {
    let app = apps
        .iter()
        .find(|app| app.id == id)
        .context("application was not found in the desktop catalog")?;

    app.launch()
}

#[cfg(test)]
mod tests {
    use super::{DesktopApp, search};

    fn app(name: &str, id: &str) -> DesktopApp {
        DesktopApp {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            keywords: vec![],
            icon: None,
            terminal: false,
            working_directory: None,
            exec: vec!["true".into()],
        }
    }

    #[test]
    fn fuzzy_search_ranks_matching_apps() {
        let apps = vec![
            app("System Settings", "org.kde.systemsettings"),
            app("Mozilla Firefox", "firefox.desktop"),
            app("Files", "org.gnome.Nautilus"),
        ];

        let results = search(&apps, "firefox", 5);
        assert_eq!(results.first().map(|app| app.name.as_str()), Some("Mozilla Firefox"));
    }
}
