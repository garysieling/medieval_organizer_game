#!/usr/bin/env python3
"""Rebuild the castles' grand curtain walls (assets/rd/castle/grand_wall_x
and its mirror grand_wall_y) as tileable curtain segments (#1158).

The `castle-grand-wall-run` generation came back as a standalone tower
block - door, front-corner point and a battlement cap of its own - so
every curtain tile drew a little tower and a run read as a staircase of
keeps with seams between them.

This composes a straight curtain from `grand_postern_x` instead: its cap
and flat stone face are right, only the door below is wrong. The sprite
keeps the postern's top (cap + upper face), repeats the clean face band
to reach the grand set's 109px height, and synthesises the base taper as
a fresh isometric point filled with face brick - the door never appears.
`grand_wall_y.png` is the horizontal mirror, like every `_y` in the set.

The postern pieces are grown to the same 109px by inserting the face band
above the door, so a door stretch keeps its cap level with the curtains';
the generated postern is preserved as `grand_postern_src.png`.

    python scripts/rd_compose_castle_curtain.py
"""

import os

from PIL import Image, ImageOps

CASTLE = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                      "assets", "rd", "castle")

SRC = "grand_postern_x.png"   # 63x78: cap rows 0-20, clean face to ~44
BACKUP = "grand_postern_src.png"  # the door piece's untouched source
OUT_X, OUT_Y = "grand_wall_x.png", "grand_wall_y.png"
POST_X, POST_Y = "grand_postern_x.png", "grand_postern_y.png"

W, H = 63, 109                # the grand set's curtain height
KEEP_ROWS = 44                # cap + upper face, copied untouched
BAND = (28, 44)               # clean flat-face rows, repeated downward
TAPER_TOP = 81                # the base point starts here
XL0, XR0 = 5, 61              # face width where the taper begins
POINT_X = 39                  # the front corner lands here
DOOR_TOP = 50                 # the postern's door region starts here


def band_rows(px, po, y0, y1):
    """Fill [y0, y1) with the face band repeated, so a face or a region
    above the door can stretch to the grand set's height."""
    span = BAND[1] - BAND[0]
    for y in range(y0, y1):
        src_y = BAND[0] + (y - y0) % span
        for x in range(W):
            po[x, y] = px[x, src_y]


def compose():
    src = Image.open(os.path.join(CASTLE, BACKUP)).convert("RGBA")
    px = src.load()
    out = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    po = out.load()
    for y in range(KEEP_ROWS):
        for x in range(W):
            po[x, y] = px[x, y]
    # the face stretched down by repeating its clean band
    band_rows(px, po, KEEP_ROWS, TAPER_TOP)
    # the base taper: a fresh iso point filled with face brick, edges
    # outlined like the sprite's own silhouette
    for y in range(TAPER_TOP, H):
        xl = round(XL0 + (y - TAPER_TOP) * (POINT_X - XL0) / (H - TAPER_TOP))
        xr = round(XR0 - (y - TAPER_TOP) * (XR0 - POINT_X) / (H - TAPER_TOP))
        for x in range(xl, min(xr + 1, W)):
            r, g, b, a = px[min(x, 56), 36 + (y - TAPER_TOP) % 9]
            if x == xl or x == xr or y >= H - 3:
                r, g, b = int(r * 0.45), int(g * 0.45), int(b * 0.45)
            po[x, y] = (r, g, b, 255)
    return out


def compose_postern():
    """The door piece stays a door, grown to curtain height: the same
    face-band insert goes in above the door, so its cap rides as high
    as the curtain's and the door keeps its own base point."""
    src = Image.open(os.path.join(CASTLE, BACKUP)).convert("RGBA")
    px = src.load()
    grow = H - src.height          # 31 rows of face above the door
    out = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    po = out.load()
    for y in range(DOOR_TOP):
        for x in range(W):
            po[x, y] = px[x, y]
    band_rows(px, po, DOOR_TOP, DOOR_TOP + grow)
    for y in range(DOOR_TOP, src.height):
        for x in range(W):
            po[x, y + grow] = px[x, y]
    return out


def main():
    # keep the generated postern under a source name so reruns stay
    # idempotent and the door piece's origin is still on disk
    src_path = os.path.join(CASTLE, SRC)
    bak_path = os.path.join(CASTLE, BACKUP)
    if not os.path.exists(bak_path):
        os.replace(src_path, bak_path)

    for built, name_x, name_y in (
        (compose(), OUT_X, OUT_Y),
        (compose_postern(), POST_X, POST_Y),
    ):
        built.save(os.path.join(CASTLE, name_x))
        ImageOps.mirror(built).save(os.path.join(CASTLE, name_y))
        print(f"wrote {name_x} and {name_y} ({built.width}x{built.height})")


if __name__ == "__main__":
    main()
