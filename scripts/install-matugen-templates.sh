#!/usr/bin/env bash
set -Eeuo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
matugen_home="$config_home/matugen"
template_home="$matugen_home/templates/chloe-dynamic-shell"
generated_home="$matugen_home/generated"
prism_home="$data_home/PrismLauncher"
flatpak_prism_home="$HOME/.var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher"

mkdir -p "$template_home" "$generated_home" "$config_home/niri" "$config_home/zed/themes" "$config_home/gtk-3.0" "$config_home/gtk-4.0" "$config_home/qt5ct/colors" "$config_home/qt6ct/colors" "$config_home/alacritty" "$config_home/kitty" "$config_home/micro/colorschemes" "$config_home/nvim/colors" "$config_home/helix/themes" "$config_home/spicetify/Themes/Comfy"

install -m644 "$repo_root/matugen/zed.json" "$template_home/zed.json"
install -m644 "$repo_root/matugen/prism.json" "$template_home/prism.json"
install -m644 "$repo_root/matugen/prism.qss" "$template_home/prism.qss"
install -m644 "$repo_root/matugen/gtk.css" "$template_home/gtk.css"
install -m644 "$repo_root/matugen/qtct.conf" "$template_home/qtct.conf"
install -m644 "$repo_root/matugen/zen.css" "$template_home/zen.css"
install -m644 "$repo_root/matugen/niri.kdl" "$template_home/niri.kdl"
install -m644 "$repo_root/matugen/alacritty.toml" "$template_home/alacritty.toml"
install -m644 "$repo_root/matugen/kitty.conf" "$template_home/kitty.conf"
install -m644 "$repo_root/matugen/micro.micro" "$template_home/micro.micro"
install -m644 "$repo_root/matugen/neovim.lua" "$template_home/neovim.lua"
install -m644 "$repo_root/matugen/helix.toml" "$template_home/helix.toml"
install -m644 "$repo_root/matugen/spicetify.ini" "$template_home/spicetify.ini"

if [[ -d "$flatpak_prism_home" ]]; then
    prism_home="$flatpak_prism_home"
fi

mkdir -p "$prism_home/themes/Chloe Dynamic Shell"

python - "$matugen_home/config.toml" "$template_home" "$config_home" "$prism_home" "$generated_home" <<'PY'
from pathlib import Path
import sys

config = Path(sys.argv[1])
templates = Path(sys.argv[2])
config_home = Path(sys.argv[3])
prism_home = Path(sys.argv[4])
generated = Path(sys.argv[5])

entries = {
    "cds_zed": (templates / "zed.json", config_home / "zed/themes/Chloe Dynamic Shell.json"),
    "cds_prism": (templates / "prism.json", prism_home / "themes/Chloe Dynamic Shell/theme.json"),
    "cds_prism_stylesheet": (templates / "prism.qss", prism_home / "themes/Chloe Dynamic Shell/themeStyle.css"),
    "cds_gtk3": (templates / "gtk.css", config_home / "gtk-3.0/colors.css"),
    "cds_gtk4": (templates / "gtk.css", config_home / "gtk-4.0/colors.css"),
    "cds_qt5ct": (templates / "qtct.conf", config_home / "qt5ct/colors/chloe-dynamic-shell.conf"),
    "cds_qt6ct": (templates / "qtct.conf", config_home / "qt6ct/colors/chloe-dynamic-shell.conf"),
    "cds_zen": (templates / "zen.css", generated / "zen-userChrome.css"),
    "cds_niri": (templates / "niri.kdl", config_home / "niri/matugen.kdl"),
    "cds_alacritty": (templates / "alacritty.toml", config_home / "alacritty/cds-colors.toml"),
    "cds_kitty": (templates / "kitty.conf", config_home / "kitty/chloe-dynamic-shell.conf"),
    "cds_micro": (templates / "micro.micro", config_home / "micro/colorschemes/chloe-dynamic-shell.micro"),
    "cds_neovim": (templates / "neovim.lua", config_home / "nvim/colors/chloe-dynamic-shell.lua"),
    "cds_helix": (templates / "helix.toml", config_home / "helix/themes/chloe-dynamic-shell.toml"),
    "cds_spicetify": (templates / "spicetify.ini", config_home / "spicetify/Themes/Comfy/color.ini"),
}

text = config.read_text() if config.exists() else ""
for name, (source, target) in entries.items():
    header = f"[templates.{name}]"
    if header in text:
        continue
    text += f"\n{header}\ninput_path = \"{source}\"\noutput_path = \"{target}\"\n"
config.parent.mkdir(parents=True, exist_ok=True)
config.write_text(text.lstrip("\n"))

def set_ini_value(path, section, key, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    content = path.read_text() if path.exists() else ""
    section_header = f"[{section}]"
    lines = content.splitlines()
    start = next((i for i, line in enumerate(lines) if line.strip() == section_header), None)
    if start is None:
        if content and not content.endswith("\n"):
            content += "\n"
        content += f"\n{section_header}\n{key}={value}\n"
    else:
        end = next((i for i in range(start + 1, len(lines)) if lines[i].lstrip().startswith("[") and lines[i].rstrip().endswith("]")), len(lines))
        match = next((i for i in range(start + 1, end) if lines[i].split("=", 1)[0].strip() == key), None)
        if match is None:
            lines.insert(end, f"{key}={value}")
        else:
            lines[match] = f"{key}={value}"
        content = "\n".join(lines) + "\n"
    path.write_text(content)

for qt in ("qt5ct", "qt6ct"):
    set_ini_value(config_home / qt / f"{qt}.conf", "Appearance", "custom_palette", "true")
    set_ini_value(config_home / qt / f"{qt}.conf", "Appearance", "color_scheme_path", str(config_home / qt / "colors/chloe-dynamic-shell.conf"))

for gtk in ("gtk-3.0", "gtk-4.0"):
    css = config_home / gtk / "gtk.css"
    css.parent.mkdir(parents=True, exist_ok=True)

    # A dangling gtk.css symlink makes Path.exists() return False, but
    # write_text() still follows the link and fails with ENOENT.
    if css.is_symlink() and not css.exists():
        backup = css.with_name(css.name + ".cds-broken-symlink-backup")
        suffix = 1
        while backup.exists() or backup.is_symlink():
            backup = css.with_name(css.name + f".cds-broken-symlink-backup-{suffix}")
            suffix += 1
        css.rename(backup)
        print(f"Moved dangling GTK stylesheet symlink to {backup}")

    import_line = '@import "colors.css";'
    old = css.read_text() if css.exists() else ""
    if import_line not in old:
        if css.exists():
            backup = css.with_name(css.name + ".cds-backup")
            if not backup.exists():
                backup.write_text(old)

        # Recreate the parent immediately before writing. This also handles
        # external config managers that may recreate/remove GTK config dirs.
        css.parent.mkdir(parents=True, exist_ok=True)
        css.write_text(import_line + "\n" + old)

zen_profiles = []
for profile_root in (
    Path.home() / ".zen",
    Path.home() / ".var/app/app.zen_browser.zen/.zen",
):
    profiles_ini = profile_root / "profiles.ini"
    if not profiles_ini.exists():
        continue
    import configparser
    parser = configparser.RawConfigParser()
    parser.read(profiles_ini)
    for section in parser.sections():
        if not section.startswith("Profile"):
            continue
        relative = parser.get(section, "IsRelative", fallback="1") == "1"
        raw = parser.get(section, "Path", fallback="")
        if raw:
            profile = (profile_root / raw) if relative else Path(raw)
            if profile.is_dir():
                zen_profiles.append(profile)

generated_zen = (generated / "zen-userChrome.css").resolve().as_uri()
for profile in zen_profiles:
    chrome = profile / "chrome"
    chrome.mkdir(parents=True, exist_ok=True)
    user_chrome = chrome / "userChrome.css"
    import_line = f'@import url("{generated_zen}");'
    existing = user_chrome.read_text() if user_chrome.exists() else ""
    if import_line not in existing:
        user_chrome.write_text(import_line + "\n" + existing)
    user_js = profile / "user.js"
    pref = 'user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);'
    prefs = user_js.read_text() if user_js.exists() else ""
    if pref not in prefs:
        user_js.write_text(prefs.rstrip() + ("\n" if prefs else "") + pref + "\n")

print(f"Installed Matugen templates in {templates}")
print(f"Registered {len(entries)} application templates in {config}")
print(f"Configured Zen Browser profiles: {len(zen_profiles)}")
PY

printf '[CDS] Matugen templates installed for Zed, Prism Launcher, Zen Browser, GTK 3/4, Qt 5/6, Niri, Alacritty, Kitty, Micro, Neovim, Helix and Spicetify (Comfy).\n'
printf '[CDS] Select "Chloe Dynamic Shell" in Zed and Prism Launcher after generating the first palette.\n'
printf '[CDS] Zen Browser profiles were configured where existing profiles were detected.\n'
printf '[CDS] Qt palette files are configured for qt5ct and qt6ct; QT_QPA_PLATFORMTHEME remains under your control.\n'
