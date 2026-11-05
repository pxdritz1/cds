#!/usr/bin/env bash
set -euo pipefail

root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source_file="$root/src/emoji.rs"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
destination="$data_home/chloe-dynamic-shell/emoji"
upstream="https://raw.githubusercontent.com/googlefonts/noto-emoji/main/2D/svg"

if ! command -v curl >/dev/null 2>&1; then
    printf 'CDS: curl is required to download the Noto Emoji artwork.\n' >&2
    exit 1
fi

mkdir -p "$destination"

python - "$source_file" "$destination" "$upstream" <<'PY'
import pathlib
import re
import subprocess
import sys

source = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
destination = pathlib.Path(sys.argv[2])
upstream = sys.argv[3]
catalog = source.split("#[derive(Default)]", 1)[0]
emojis = dict.fromkeys(re.findall(r'\("([^"]+)"\s*,', catalog))

for emoji in emojis:
    codepoints = [
        f"{ord(character):x}"
        for character in emoji
        if ord(character) != 0xFE0F
    ]
    filename = "emoji_u" + "_".join(codepoints) + ".svg"
    target = destination / filename

    if target.is_file() and target.stat().st_size:
        continue

    url = f"{upstream}/{filename}"
    print(f"Downloading {filename}")
    subprocess.run(
        ["curl", "--fail", "--silent", "--show-error", "--location", url, "--output", str(target)],
        check=True,
    )

license_target = destination / "LICENSE"
if not license_target.exists():
    subprocess.run(
        ["curl", "--fail", "--silent", "--show-error", "--location",
         f"{upstream}/LICENSE", "--output", str(license_target)],
        check=True,
    )

print(f"Installed {len(emojis)} Noto Emoji SVGs in {destination}")
PY
