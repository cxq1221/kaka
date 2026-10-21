#!/usr/bin/env python3
"""Generate the deterministic 1024px Kaka application icon source."""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "src-tauri" / "icons" / "app-icon-source.png"
CANVAS = 1024
SCALE = 4
FONT_PATH = "/System/Library/Fonts/Avenir Next Condensed.ttc"

INK = "#171512"
PAPER = "#F2EADB"
ORANGE = "#E9542F"


def scaled(value: int) -> int:
    return value * SCALE


def main() -> None:
    image = Image.new("RGBA", (scaled(CANVAS), scaled(CANVAS)), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)

    # The 840px body occupies 82% of the canvas, matching the Tauri icon
    # validator while leaving genuinely transparent space at every corner.
    inset = 112
    body = (
        scaled(inset),
        scaled(inset),
        scaled(CANVAS - inset),
        scaled(CANVAS - inset),
    )
    draw.rounded_rectangle(body, radius=scaled(132), fill=INK)

    # Contact-sheet registration marks and a single orange frame line give the
    # mark a capture-tool identity without relying on tiny decorative detail.
    draw.rounded_rectangle(
        (scaled(168), scaled(160), scaled(560), scaled(174)),
        radius=scaled(7),
        fill=ORANGE,
    )
    draw.rounded_rectangle(
        (scaled(170), scaled(854), scaled(284), scaled(868)),
        radius=scaled(7),
        fill=PAPER,
    )
    draw.rounded_rectangle(
        (scaled(740), scaled(710), scaled(854), scaled(854)),
        radius=scaled(20),
        fill=ORANGE,
    )

    font = ImageFont.truetype(FONT_PATH, scaled(540), index=8)
    bounds = draw.textbbox((0, 0), "k", font=font)
    text_width = bounds[2] - bounds[0]
    text_height = bounds[3] - bounds[1]
    text_x = (scaled(CANVAS) - text_width) // 2 - bounds[0] - scaled(20)
    text_y = (scaled(CANVAS) - text_height) // 2 - bounds[1] - scaled(34)
    draw.text((text_x, text_y), "k", font=font, fill=PAPER)

    icon = image.resize((CANVAS, CANVAS), Image.Resampling.LANCZOS)
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    icon.save(OUTPUT, "PNG", optimize=True)
    print(OUTPUT)


if __name__ == "__main__":
    main()
