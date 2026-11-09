#!/usr/bin/env bash
set -euo pipefail

root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
source_file="$root/src/emoji.rs"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
destination="$data_home/cds/emoji"
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

# Parse only the actual Rust emoji catalog. Do not scan the rest of the file:
# comments and string literals contain braces and other non-emoji characters.
catalog_start = source.index("const EMOJIS: &[(&str, &str)] = &[")
catalog = source[catalog_start:].split("\n];", 1)[0]
emojis = dict.fromkeys(re.findall(r'\("([^"]+)"\s*,', catalog))

if not emojis:
    raise SystemExit("Could not parse the emoji catalog in src/emoji.rs")

downloaded = 0
missing = 0
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
    result = subprocess.run(
        ["curl", "--fail", "--silent", "--show-error", "--location", url, "--output", str(target)],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if result.returncode:
        target.unlink(missing_ok=True)
        print(f"Not available upstream; keeping Unicode fallback: {filename}")
        missing += 1
        continue
    downloaded += 1
    print(f"Downloaded {filename}")

license_target = destination / "LICENSE"
if not license_target.exists():
    subprocess.run(
        ["curl", "--fail", "--silent", "--show-error", "--location",
         f"{upstream}/LICENSE", "--output", str(license_target)],
        check=True,
    )

print(f"Installed {downloaded} Noto SVGs; {missing} unavailable sequences will use font fallback.")
print(f"Assets: {destination}")
PY
