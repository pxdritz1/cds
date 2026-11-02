#!/usr/bin/env python3
"""Generate static Google Sans Flex instances for the Amane renderer.

Amane's ttf-parser build (0.25.1, without the `gvar-alloc` feature) only
keeps 32 variation tuples on the stack. Google Sans Flex has glyphs with
more than 200, so every outline silently fails and text disappears.

Instancing the variable font removes `gvar`/`fvar` and leaves the four
weights the shell uses, which the renderer can outline normally.

    python -m pip install fonttools
    python scripts/google-sans-static.py GoogleSansFlex.ttf out/
"""

import argparse
from pathlib import Path

from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

WEIGHTS = [
    (400, "Regular"),
    (500, "Medium"),
    (600, "SemiBold"),
    (700, "Bold"),
]

# the default position of every axis except weight
AXES = {"opsz": 18, "wdth": 100, "GRAD": 0, "ROND": 0, "slnt": 0}

FAMILY = "Google Sans Flex"


def instance(source: Path, weight: int, style: str, target: Path) -> None:
    font = TTFont(source)

    instantiateVariableFont(
        font,
        {**AXES, "wght": weight},
        inplace=True,
        updateFontNames=False,
    )

    name = font["name"]
    # drop the variable font's axis and instance names
    name.names = [
        record
        for record in name.names
        if record.nameID in (0, 1, 2, 3, 4, 5, 6, 13, 14)
    ]

    full = f"{FAMILY} {style}"
    postscript = f"GoogleSansFlex-{style}"

    for name_id, value in (
        (1, FAMILY),
        (2, style),
        (4, full),
        (6, postscript),
        (16, FAMILY),
        (17, style),
    ):
        name.setName(value, name_id, 3, 1, 0x409)
        name.setName(value, name_id, 1, 0, 0)

    os2 = font["OS/2"]
    os2.usWeightClass = weight

    if style == "Bold":
        os2.fsSelection = (os2.fsSelection & ~0x40) | 0x20
        font["head"].macStyle |= 0x01
    else:
        os2.fsSelection = (os2.fsSelection & ~0x20) | (0x40 if weight == 400 else 0)
        font["head"].macStyle &= ~0x01

    font.save(target)
    print(target)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, help="GoogleSansFlex.ttf variable font")
    parser.add_argument("output", type=Path, help="directory for the static fonts")
    arguments = parser.parse_args()

    arguments.output.mkdir(parents=True, exist_ok=True)

    for weight, style in WEIGHTS:
        instance(
            arguments.source,
            weight,
            style,
            arguments.output / f"GoogleSansFlex-{style}.ttf",
        )


if __name__ == "__main__":
    main()
