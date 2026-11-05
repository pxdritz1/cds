#!/usr/bin/env bash
set -euo pipefail

root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source_file="$root/docs/niri-config.kdl"
config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
target="$config_home/niri/config.kdl"

if [[ ! -f "$source_file" ]]; then
    printf 'CDS: não encontrei o modelo em %s\n' "$source_file" >&2
    exit 1
fi

mkdir -p "$(dirname -- "$target")"

if [[ -f "$target" ]]; then
    backup="$target.bak.$(date +%Y%m%d-%H%M%S)"
    cp -- "$target" "$backup"
    printf 'Backup salvo em: %s\n' "$backup"
fi

install -Dm644 "$source_file" "$target"
printf 'Configuração do Niri instalada em: %s\n' "$target"
printf 'Revise o arquivo antes de reiniciar a sessão.\n'
