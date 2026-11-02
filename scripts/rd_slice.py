#!/usr/bin/env python3
"""Slice a Retro Diffusion sprite sheet into individual transparent sprites.

rd_pro__spritesheet (and inventory_items etc.) return a loose grid of
sprites on a flat background, not a fixed cell size. This keys the
background out by flood-filling from the image border (so white areas
INSIDE a sprite, like a sheep's fleece, survive), finds each sprite as a
connected blob, and orders blobs into rows/cols by position.

    # look first: writes every cell + a labelled 4x preview
    python scripts/rd_slice.py art-prompts/raw/sheet-farm-animals-a628136d/0.png \
        --preview scratch/farm-preview.png
    # then export the cells you want under game names
    python scripts/rd_slice.py art-prompts/raw/sheet-farm-animals-a628136d/0.png \
        --out assets/rd/farm-animals r0c0=dog r1c1=cat r2c0=hen

Cell names are r<row>c<col>, 0-based, as labelled on the preview.
Without NAME args every cell is exported as r<row>c<col>.png.
Needs Pillow.
"""
import argparse
import os
import sys
from collections import deque

from PIL import Image, ImageDraw


def key_background(im, tol):
    """RGBA copy with the border-connected background made transparent.
    Background colour = the most common border pixel."""
    im = im.convert("RGBA")
    w, h = im.size
    px = im.load()
    border = [px[x, y] for x in range(w) for y in (0, h - 1)] + \
             [px[x, y] for y in range(h) for x in (0, w - 1)]
    bg = max(set(border), key=border.count)
    if bg[3] == 0:
        return im  # already transparent (remove_bg was on)

    def is_bg(c):
        return c[3] == 0 or all(abs(c[i] - bg[i]) <= tol for i in range(3))

    seen = bytearray(w * h)
    q = deque()
    for x in range(w):
        q.append((x, 0)); q.append((x, h - 1))
    for y in range(h):
        q.append((0, y)); q.append((w - 1, y))
    while q:
        x, y = q.popleft()
        if not (0 <= x < w and 0 <= y < h) or seen[y * w + x]:
            continue
        seen[y * w + x] = 1
        if not is_bg(px[x, y]):
            continue
        px[x, y] = (0, 0, 0, 0)
        q.extend(((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)))
    return im


def blobs(im, min_px):
    """Bounding boxes of 8-connected opaque regions with >= min_px pixels."""
    w, h = im.size
    a = im.getchannel("A").load()
    seen = bytearray(w * h)
    out = []
    for sy in range(h):
        for sx in range(w):
            if seen[sy * w + sx] or a[sx, sy] == 0:
                continue
            q = deque([(sx, sy)])
            seen[sy * w + sx] = 1
            n, x0, y0, x1, y1 = 0, sx, sy, sx, sy
            while q:
                x, y = q.popleft()
                n += 1
                x0, y0, x1, y1 = min(x0, x), min(y0, y), max(x1, x), max(y1, y)
                for dx in (-1, 0, 1):
                    for dy in (-1, 0, 1):
                        nx, ny = x + dx, y + dy
                        if 0 <= nx < w and 0 <= ny < h and not seen[ny * w + nx] and a[nx, ny]:
                            seen[ny * w + nx] = 1
                            q.append((nx, ny))
            if n >= min_px:
                out.append([x0, y0, x1 + 1, y1 + 1])
    return out


def merge_close(boxes, gap):
    """Union boxes that overlap or sit within `gap` px (a sprite whose
    parts don't touch, e.g. a chick's legs)."""
    changed = True
    while changed:
        changed = False
        for i in range(len(boxes)):
            for j in range(i + 1, len(boxes)):
                a, b = boxes[i], boxes[j]
                if a[0] - gap < b[2] and b[0] - gap < a[2] and a[1] - gap < b[3] and b[1] - gap < a[3]:
                    boxes[i] = [min(a[0], b[0]), min(a[1], b[1]), max(a[2], b[2]), max(a[3], b[3])]
                    del boxes[j]
                    changed = True
                    break
            if changed:
                break
    return boxes


def grid(boxes):
    """Assign (row, col): a new row starts when a box's vertical centre is
    below the current row's bottom edge."""
    boxes = sorted(boxes, key=lambda b: (b[1] + b[3]) / 2)
    rows, cur, cur_bottom = [], [], None
    for b in boxes:
        cy = (b[1] + b[3]) / 2
        if cur and cy > cur_bottom:
            rows.append(cur)
            cur = []
        cur.append(b)
        cur_bottom = max(bb[3] for bb in cur) if len(cur) > 1 else b[3]
    if cur:
        rows.append(cur)
    cells = {}
    for r, row in enumerate(rows):
        for c, b in enumerate(sorted(row, key=lambda b: b[0])):
            cells[f"r{r}c{c}"] = b
    return cells


def halve(im):
    """2x pixel-art downscale: each 2x2 block becomes its most common
    opaque colour, ties going to the darkest (keeps outlines); a block
    with 3+ transparent pixels goes transparent. The game draws pack art
    at 2x, so RD sprites drawn on a 1px grid need this to match it."""
    w, h = (im.width + 1) // 2, (im.height + 1) // 2
    src = im.load()
    out = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    dst = out.load()
    for y in range(h):
        for x in range(w):
            blk = [src[min(2 * x + dx, im.width - 1), min(2 * y + dy, im.height - 1)]
                   for dx in (0, 1) for dy in (0, 1)]
            solid = [c for c in blk if c[3] >= 128]
            if len(solid) < 2:
                continue
            dst[x, y] = max(set(solid), key=lambda c: (solid.count(c), -sum(c[:3])))[:3] + (255,)
    return out


def fit_height(im, h):
    """Box-resample a sprite down to `h` px tall and re-harden the alpha.
    Reads far better than nearest-neighbour or halve() at the small sizes
    the game draws animals and props at (a villager is ~24px tall)."""
    w = max(1, round(im.width * h / im.height))
    r = im.resize((w, h), Image.BOX)
    r.putalpha(r.getchannel("A").point(lambda v: 255 if v >= 110 else 0))
    return r


def preview(im, cells, path, scale=4):
    big = Image.new("RGBA", im.size, (60, 70, 60, 255))
    big.alpha_composite(im)
    big = big.resize((im.width * scale, im.height * scale), Image.NEAREST)
    d = ImageDraw.Draw(big)
    for name, (x0, y0, x1, y1) in cells.items():
        d.rectangle([x0 * scale, y0 * scale, x1 * scale - 1, y1 * scale - 1], outline=(255, 0, 255))
        d.text((x0 * scale + 2, y0 * scale + 1), name, fill=(255, 255, 0))
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    big.save(path)


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("sheet")
    ap.add_argument("names", nargs="*",
                    help="rRcC=name or rRcC=name@H pairs to export (H: resample to H px "
                         "tall); default: all cells at native size")
    ap.add_argument("--out", help="directory for the exported sprites")
    ap.add_argument("--preview", help="write a labelled, 4x preview PNG here")
    ap.add_argument("--tol", type=int, default=24, help="background colour tolerance per channel")
    ap.add_argument("--min-px", type=int, default=12, help="ignore specks smaller than this")
    ap.add_argument("--gap", type=int, default=1, help="merge blobs this close together")
    ap.add_argument("--half", action="store_true",
                    help="downscale exported sprites 2x (match the game's 2x-drawn pack art)")
    args = ap.parse_intermixed_args()  # names may follow --out
    if args.names and not args.out:
        sys.exit("cell=name args need --out DIR")
    pairs = []
    for p in args.names:
        cell, eq, name = p.partition("=")
        name, at, height = name.partition("@")
        if not eq or not name or "/" in name or os.sep in name:
            sys.exit(f"expected rRcC=name or rRcC=name@H, got {p!r}")
        if at and not (height.isdigit() and int(height) > 0):
            sys.exit(f"{p!r}: @H must be a positive pixel height")
        pairs.append((cell, name, int(height) if at else None))
    if not os.path.isfile(args.sheet):
        sys.exit(f"{args.sheet}: no such file")

    im = key_background(Image.open(args.sheet), args.tol)
    cells = grid(merge_close(blobs(im, args.min_px), args.gap))
    rows = sorted({int(n[1:].split("c")[0]) for n in cells})
    for r in rows:
        row = [n for n in cells if n.startswith(f"r{r}c")]
        print(f"row {r}: {len(row)} cells  " + " ".join(
            f"{n}:{cells[n][2]-cells[n][0]}x{cells[n][3]-cells[n][1]}" for n in row))
    if args.preview:
        preview(im, cells, args.preview)
        print("preview ->", args.preview)
    if args.out:
        os.makedirs(args.out, exist_ok=True)
        pairs = pairs or [(n, n, None) for n in cells]
        missing = [cell for cell, _, _ in pairs if cell not in cells]
        if missing:
            sys.exit(f"no cell {' '.join(missing)} (see the preview for names)")
        for cell, name, height in pairs:
            spr = im.crop(cells[cell])
            if args.half:
                spr = halve(spr)
            if height:
                spr = fit_height(spr, height)
            spr.save(os.path.join(args.out, name + ".png"))
            print(f"{cell} -> {os.path.join(args.out, name + '.png')} ({spr.width}x{spr.height})")


if __name__ == "__main__":
    main()
