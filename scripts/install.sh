#!/usr/bin/env bash
set -Eeuo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
cache_home="${XDG_CACHE_HOME:-$HOME/.cache}"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
install_niri_config=0

log() {
    printf '[CDS] %s\n' "$1"
}

fail() {
    printf '[CDS] ERROR: %s\n' "$1" >&2
    exit 1
}

for argument in "$@"; do
    case "$argument" in
        --install-niri-config) install_niri_config=1 ;;
        *) fail "Unknown option: $argument" ;;
    esac
done

[[ "$EUID" -ne 0 ]] || fail "Run this installer as a regular user, not with sudo."
command -v pacman >/dev/null 2>&1 || fail "This installer supports Arch Linux and CachyOS."
command -v sudo >/dev/null 2>&1 || fail "sudo is required."
[[ -f "$repo_root/Cargo.toml" && -f "$repo_root/native/main.rs" ]] || fail "Run this script from the native CDS source checkout."

log "Installing native build and runtime dependencies."
sudo pacman -S --needed --noconfirm \
    base-devel git rustup pkgconf gcc clang llvm \
    wayland libxkbcommon fontconfig freetype2 expat vulkan-icd-loader \
    libpulse pam curl python \
    wl-clipboard hyprpicker swayidle brightnessctl grim slurp \
    pipewire wireplumber playerctl qt5ct qt6ct

export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v cargo >/dev/null 2>&1; then
    log "Initializing the Rust stable toolchain."
    rustup default stable
fi
command -v cargo >/dev/null 2>&1 || fail "Cargo is unavailable after Rust initialization."

log "Installing the native CDS executable."
cargo install --path "$repo_root" --bin cds --force

mkdir -p \
    "$config_home/cds" \
    "$cache_home/cds" \
    "$data_home/cds"

if command -v matugen >/dev/null 2>&1; then
    log "Installing Matugen templates."
    bash "$repo_root/scripts/install-matugen-templates.sh"
else
    log "Matugen is not installed; skipping optional template registration."
fi

log "Installing emoji artwork assets."
bash "$repo_root/scripts/install-emoji-assets.sh"

if [[ "$install_niri_config" -eq 1 ]]; then
    niri_config="$config_home/niri/config.kdl"
    stamp="$(date +%Y%m%d-%H%M%S)"
    if [[ -f "$niri_config" ]]; then
        cp -a -- "$niri_config" "$niri_config.backup-$stamp"
        log "Existing Niri configuration backed up."
    fi
    install -Dm644 "$repo_root/docs/niri-config.kdl" "$niri_config"
    log "Installed the example Niri configuration."
else
    log "Leaving the existing Niri configuration untouched."
    log "Use --install-niri-config only if you explicitly want the repository example."
fi

log "Installation completed."
printf '\n'
printf 'Executable: %s\n' "$HOME/.cargo/bin/cds"
printf 'Config:     %s/cds\n' "$config_home"
printf 'Cache:      %s/cds\n' "$cache_home"
printf '\n'
printf 'Start the shell with: cds run\n'
printf 'Development:          cds dev\n'
printf 'Optimized build:      cds compile\n'
