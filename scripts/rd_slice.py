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
from collections import Counter, deque

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


def drop_clipped(boxes, w, h):
    """A sprite touching the canvas edge was cut off by it (a sheet asked
    for at a small size still draws full-size sprites, just fewer and
    clipped - scale-animals-128): leave those out."""
    keep = [b for b in boxes if b[0] > 0 and b[1] > 0 and b[2] < w and b[3] < h]
    return keep if keep else boxes  # a single full-bleed image: keep it


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


def flush_base(im, max_rows=6):
    """Cut the earth edge off a building's grass slab, so the slab's grass
    is the sprite's bottom: the game stands a building's bottom on the
    tile's front corner, and with the edge left on, the grass (and the
    house) floats that many pixels above the ground.

    The edge is as thick all the way round, so it's measured once - the
    usual distance from a column's lowest pixel up to its first green one
    - and that many rows come off the bottom of every column (a dirt path
    or a corner post on the slab gets the same cut). A sprite where most
    columns find no green in reach has no slab (a stone foundation): it's
    left alone. Returns the re-cropped sprite."""
    im = im.copy()
    px = im.load()
    w, h = im.size
    bottoms, cuts = {}, []
    for x in range(w):
        ys = [y for y in range(h) if px[x, y][3] >= 128]
        if not ys:
            continue
        bottoms[x] = ys[-1]
        for k in range(max_rows + 1):
            r, g, b, a = px[x, ys[-1] - k] if ys[-1] - k >= 0 else (0, 0, 0, 0)
            if a >= 128 and g > r and g >= b and g > 60:
                cuts.append(k)
                break
    if len(cuts) < len(bottoms) / 2:
        return im
    t = sorted(cuts)[len(cuts) // 2]
    for x, bottom in bottoms.items():
        for y in range(max(0, bottom - t + 1), bottom + 1):
            px[x, y] = (0, 0, 0, 0)
    return im.crop(im.getbbox())


def strip_base(im, min_px=15):
    """Take away the little patch of ground a prop (a tree) was drawn
    standing on, so it stands on the game's own grass. The patch is the
    olive/earth ramp (red ~ green, little blue) in the bottom half: foliage
    is bluer, the trunk redder, so both stay. A sprite with no patch is
    returned as it is. Returns the re-cropped sprite."""
    im = im.copy()
    px = im.load()
    w, h = im.size

    def earth(c):
        r, g, b, a = c
        return (a >= 128 and r > 60 and b < 0.7 * min(r, g)
                and abs(r - g) <= 0.15 * max(r, g))

    hits = [(x, y) for y in range(h // 2, h) for x in range(w) if earth(px[x, y])]
    if len(hits) < min_px:
        return im
    for x, y in hits:
        px[x, y] = (0, 0, 0, 0)
    return largest_piece(im)


def strip_slab(im, frac=0.4):
    """Take the grass slab off a stone piece (wall, tower) so it stands on
    whatever ground the map has - grass, rock or snow - with no green
    fringe. Run after flush_base (which takes the slab's earth edge): what's
    left of the slab is grass, olive earth and sandy path, the only colours
    with little blue in the bottom of a grey stone sprite. A door's wood is
    redder (red > 1.38 green) and stays."""
    im = im.copy()
    px = im.load()
    w, h = im.size
    for y in range(int(h * (1 - frac)), h):
        for x in range(w):
            r, g, b, a = px[x, y]
            grass = g > r + 8 and g > b + 4
            ground = b < 0.7 * max(r, g) and g >= 0.72 * r and max(r, g) > 30
            if a and (grass or ground):
                px[x, y] = (0, 0, 0, 0)
    return largest_piece(im)


def strip_fringe(im):
    """Take the same grass/earth/path colours off a loose scatter of pieces
    (a boulder pile, a handful of flat stones) that strip_slab takes off a
    single standing one. strip_slab only looks at the bottom of the sprite
    and keeps one connected piece - wrong here, since a flat scatter's
    stones sit low in the frame as several separate blobs and the slab
    shows between them, not just underneath. This scans the whole sprite
    and keeps every piece, so each stone in the scatter survives."""
    im = im.copy()
    px = im.load()
    w, h = im.size
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            grass = g > r + 8 and g > b + 4
            ground = b < 0.7 * max(r, g) and g >= 0.72 * r and max(r, g) > 30
            if a and (grass or ground):
                px[x, y] = (0, 0, 0, 0)
    return im


def key_cards(im, tol, min_share=0.55):
    """remove_bg sometimes keys only the outer margin, leaving the sprite
    on an opaque light 'card' (a flat rectangle the border flood-fill
    can't reach). For each blob's bounding box: if one light colour
    covers most of the box edge, flood-fill matching pixels from that
    edge inward. A matching colour enclosed by the figure itself (a
    white wimple, pale robes) isn't edge-connected and survives."""
    px = im.load()
    w, h = im.size

    def edge_pixels(x0, y0, x1, y1):
        for x in range(x0, x1):
            for y in (y0, y1 - 1):
                yield x, y
        for y in range(y0, y1):
            for x in (x0, x1 - 1):
                yield x, y

    for (x0, y0, x1, y1) in blobs(im, 24):
        edge = [(x, y) for (x, y) in edge_pixels(x0, y0, x1, y1) if px[x, y][3]]
        if not edge:
            continue
        # the dominant opaque colour on the box edge (16-per-channel buckets)
        (br, bg_, bb), share = Counter(
            ((p[0] >> 4) << 4, (p[1] >> 4) << 4, (p[2] >> 4) << 4)
            for x, y in edge for p in [px[x, y]]).most_common(1)[0]
        if share < len(edge) * min_share or max(br, bg_, bb) < 176:
            continue  # varied or dark edge: a real silhouette, not a card
        card = (br + 8, bg_ + 8, bb + 8)

        def is_card(c):
            return c[3] and all(abs(c[i] - card[i]) <= tol for i in range(3))
        q = deque((x, y) for (x, y) in edge if is_card(px[x, y]))
        seen = set(q)
        while q:
            x, y = q.popleft()
            px[x, y] = (0, 0, 0, 0)
            for n in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
                if x0 <= n[0] < x1 and y0 <= n[1] < y1 and n not in seen and is_card(px[n]):
                    seen.add(n)
                    q.append(n)
    return im


def largest_piece(im):
    px = im.load()
    w, h = im.size
    """Keep only the biggest connected piece, re-cropped: specks of a
    removed patch's dark rim are left hanging otherwise"""
    seen, best = set(), []
    for y0 in range(h):
        for x0 in range(w):
            if (x0, y0) in seen or not px[x0, y0][3]:
                continue
            comp, stack = [], [(x0, y0)]
            seen.add((x0, y0))
            while stack:
                x, y = stack.pop()
                comp.append((x, y))
                for n in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
                    if 0 <= n[0] < w and 0 <= n[1] < h and n not in seen and px[n][3]:
                        seen.add(n)
                        stack.append(n)
            if len(comp) > len(best):
                best = comp
    keep = set(best)
    for y in range(h):
        for x in range(w):
            if (x, y) not in keep:
                px[x, y] = (0, 0, 0, 0)
    return im.crop(im.getbbox())


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
                    help="rRcC=name, rRcC=name@H (resample to H px tall) or rRcC=name@WxH "
                         "(exactly WxH) pairs to export; default: all cells at native size")
    ap.add_argument("--out", help="directory for the exported sprites")
    ap.add_argument("--preview", help="write a labelled, 4x preview PNG here")
    ap.add_argument("--tol", type=int, default=24, help="background colour tolerance per channel")
    ap.add_argument("--min-px", type=int, default=12, help="ignore specks smaller than this")
    ap.add_argument("--gap", type=int, default=1, help="merge blobs this close together")
    ap.add_argument("--flush-base", action="store_true",
                    help="cut a building's grass-slab edge so it stands flush on its tile")
    ap.add_argument("--strip-base", action="store_true",
                    help="remove the patch of ground a tree or prop stands on")
    ap.add_argument("--strip-slab", action="store_true",
                    help="with --flush-base: remove a stone piece's grass slab too")
    ap.add_argument("--strip-fringe", action="store_true",
                    help="remove grass/earth colour from a scatter of loose pieces "
                         "(keeps every piece, unlike --strip-slab)")
    ap.add_argument("--mirror", action="store_true",
                    help="flip left-right (the other wall axis from one generated piece)")
    ap.add_argument("--keep-clipped", action="store_true",
                    help="keep sprites that touch the sheet edge")
    ap.add_argument("--key-card", action="store_true",
                    help="also key a flat light 'card' the sprite sits on "
                         "(remove_bg leaves one inside the transparent margin)")
    ap.add_argument("--half", action="store_true",
                    help="downscale exported sprites 2x (match the game's 2x-drawn pack art)")
    args = ap.parse_intermixed_args()  # names may follow --out

    im = key_background(Image.open(args.sheet), args.tol)
    if args.key_card:
        im = key_cards(im, args.tol)
    boxes = merge_close(blobs(im, args.min_px), args.gap)
    if not args.keep_clipped:
        n = len(boxes)
        boxes = drop_clipped(boxes, im.width, im.height)
        if len(boxes) < n:
            print(f"dropped {n - len(boxes)} sprites clipped by the sheet edge (--keep-clipped keeps them)")
    cells = grid(boxes)
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
        pairs = [p.split("=", 1) for p in args.names] or [(n, n) for n in cells]
        for cell, name in pairs:
            if cell not in cells:
                sys.exit(f"no cell {cell} (see the preview for names)")
            spr = im.crop(cells[cell])
            name, _, height = name.partition("@")
            if args.flush_base:
                spr = flush_base(spr)
            if args.strip_base:
                spr = strip_base(spr)
            if args.strip_slab:
                spr = strip_slab(spr)
            if args.strip_fringe:
                spr = strip_fringe(spr)
            if args.mirror:
                spr = spr.transpose(Image.FLIP_LEFT_RIGHT)
            if args.half:
                spr = halve(spr)
            if "x" in height:
                # exact WxH (e.g. map a ground block onto the pack's block
                # geometry); nearest when enlarging, box when shrinking
                w, h = (int(v) for v in height.split("x"))
                big = w * h >= spr.width * spr.height
                spr = spr.resize((w, h), Image.NEAREST if big else Image.BOX)
                if not big:
                    spr.putalpha(spr.getchannel("A").point(lambda v: 255 if v >= 110 else 0))
            elif height:
                spr = fit_height(spr, int(height))
            spr.save(os.path.join(args.out, name + ".png"))
            print(f"{cell} -> {os.path.join(args.out, name + '.png')} ({spr.width}x{spr.height})")


if __name__ == "__main__":
    main()
