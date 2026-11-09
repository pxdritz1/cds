#!/usr/bin/env bash
set -Eeuo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
cache_home="${XDG_CACHE_HOME:-$HOME/.cache}"
amane_config="$config_home/amane"
niri_config="$config_home/niri/config.kdl"
matugen_config="$config_home/matugen/config.toml"
temporary_dir="$(mktemp -d)"
backup_stamp="$(date +%Y%m%d-%H%M%S)"
matugen_legacy_disabled=0

cleanup() {
    rm -rf -- "$temporary_dir"
}
trap cleanup EXIT

log() {
    printf '[CDS] %s\n' "$1"
}

fail() {
    printf '[CDS] ERROR: %s\n' "$1" >&2
    exit 1
}

[[ "${EUID}" -ne 0 ]] || fail "Run this installer as a regular user, not with sudo."

command -v pacman >/dev/null 2>&1 || fail "This installer currently supports Arch Linux and CachyOS only."
command -v sudo >/dev/null 2>&1 || fail "sudo is required."
[[ -f "$repo_root/Cargo.toml" && -f "$repo_root/src/main.rs" && -f "$repo_root/docs/niri-config.kdl" ]] || fail "Run this script from a complete Chloe Dynamic Shell repository checkout."

log "Installing system dependencies."
sudo pacman -S --needed --noconfirm \
    base-devel git rustup pkgconf gcc \
    wayland libxkbcommon fontconfig freetype2 expat vulkan-icd-loader \
    libpulse pam curl python \
    wl-clipboard hyprpicker swayidle brightnessctl grim slurp \
    pipewire wireplumber playerctl qt5ct qt6ct

if command -v yay >/dev/null 2>&1; then
    aur_helper="yay"
elif command -v paru >/dev/null 2>&1; then
    aur_helper="paru"
else
    fail "Install an AUR helper (yay or paru), then run this installer again. niri-git and matugen-bin are AUR packages."
fi

log "Installing niri-git and matugen-bin."
"$aur_helper" -S --needed --noconfirm niri-git matugen-bin

if ! command -v cargo >/dev/null 2>&1; then
    log "Initializing the Rust toolchain."
    rustup default stable
fi

export PATH="$HOME/.cargo/bin:$PATH"
command -v cargo >/dev/null 2>&1 || fail "Cargo is unavailable. Install Rust with rustup and retry."

log "Installing Amane and its CLI."
git clone --depth 1 https://github.com/MystiaFin/amane.git "$temporary_dir/amane"
(
    cd "$temporary_dir/amane"
    cargo install --path cli --locked
)

export PATH="$HOME/.cargo/bin:$PATH"
command -v amane >/dev/null 2>&1 || fail "Amane installation did not produce an executable."

if [[ -e "$amane_config" ]]; then
    backup="$amane_config.backup-$backup_stamp"
    cp -a -- "$amane_config" "$backup"
    rm -rf -- "$amane_config"
    log "Existing Amane configuration backed up to $backup."
fi

log "Creating the Amane configuration directory."
amane new "$amane_config"

log "Installing Chloe Dynamic Shell sources."
mkdir -p "$amane_config/src"
cp -a "$repo_root/src/." "$amane_config/src/"

log "Compiling Chloe Dynamic Shell."
amane compile

if [[ -f "$niri_config" ]]; then
    backup="$niri_config.backup-$backup_stamp"
    cp -a -- "$niri_config" "$backup"
    log "Niri configuration backed up to $backup."
fi

install -Dm644 "$repo_root/docs/niri-config.kdl" "$niri_config"
log "Installed the Chloe Dynamic Shell Niri configuration."

if [[ -f "$matugen_config" ]]; then
    backup="$matugen_config.backup-$backup_stamp"
    cp -a -- "$matugen_config" "$backup"
    python - "$matugen_config" <<'PY'
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
text = path.read_text()
lines = text.splitlines()
output = []
section = []

def flush():
    global section
    if not section:
        return
    block = "\n".join(section)
    legacy = any(name in block for name in ("niri-colors.kdl", "colors.scss", "alacritty.toml"))
    if legacy and not all(line.lstrip().startswith("#") for line in section):
        output.append("# Disabled by Chloe Dynamic Shell installer; remove the leading # to restore.")
        output.extend("# " + line if line else "#" for line in section)
    else:
        output.extend(section)
    section = []

for line in lines:
    if re.match(r"^\s*\[", line):
        flush()
    section.append(line)
flush()
path.write_text("\n".join(output) + "\n")
PY
    log "Backed up Matugen configuration to $backup."
    if grep -q "niri-colors.kdl" "$matugen_config" || grep -q "colors.scss" "$matugen_config" || grep -q "alacritty.toml" "$matugen_config"; then
        matugen_legacy_disabled=1
    fi
    log "Processed existing Matugen template configuration."
else
    log "No existing Matugen configuration was found; CDS will create its runtime template configuration."
fi

log "Installing Matugen application templates."
bash "$repo_root/scripts/install-matugen-templates.sh"

log "Installing the emoji artwork assets."
bash "$repo_root/scripts/install-emoji-assets.sh"

log "Installation completed."
printf '\n'
printf 'Amane configuration: %s\n' "$amane_config"
printf 'Compiled shell: %s\n' "$cache_home/amane/amane-shell"
printf 'Niri configuration: %s\n' "$niri_config"
printf 'Matugen: matugen-bin\n'
printf 'Niri: niri-git\n'
printf '\n'
if [[ "$matugen_legacy_disabled" -eq 1 ]]; then
    printf 'Legacy Matugen sections were commented out, not deleted. Remove the leading # from a section in %s to restore it.\n' "$matugen_config"
fi
printf 'Select Chloe Dynamic Shell in Zed and Prism Launcher after the first Matugen palette generation.\n'
printf 'For Qt applications, set QT_QPA_PLATFORMTHEME to qt5ct or qt6ct according to the application Qt version.\n'
printf 'Review the installed Niri configuration before restarting your session.\n'
