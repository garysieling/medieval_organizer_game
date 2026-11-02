#!/usr/bin/env python3
"""Build a full ground tile set (grass, every road and river piece) from
three generated Retro Diffusion blocks, using the asset pack's pieces as
shape masks.

The pack's road/river pieces are relief blocks (sunken channels, raised
grass, earth sides) drawn in a tiny flat palette. Generating 32 matching
pieces would be expensive and wouldn't line up; instead every pack pixel
is classified by its palette colour and repainted from the RD blocks:

  top grass / road / water -> that RD block's top-face texture, sampled
                              through a face-to-face diamond mapping
  outer earth sides        -> the RD grass block's own sides, same mapping
  inner walls, outline     -> the RD palette (texture-free, shaded)

Output pieces keep the pack's exact geometry (so tilemap.rs connections,
BLOCK_FACE and FACE_CENTRE keep working) with RD colour and texture.

    python scripts/rd_compose_tiles.py \\
        --grass art-prompts/raw/scale-tile-grass-0b26af48/0.png \\
        --dirt  art-prompts/raw/ground-set-dirt-0ff45e1d/0.png \\
        --water art-prompts/raw/ground-set-water-dc517000/0.png \\
        --out assets/rd/tiles --preview <scratch>/tiles.png

Needs Pillow.
"""
import argparse
import os

from PIL import Image

PACK = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                    "assets", "Isometric Medieval Pack", "Isometric Medieval Pack", "Sprites")

# the pack's flat palette (Spring set)
OUTLINE = (46, 34, 47)
GRASS_TOP = (148, 196, 104)
WALL_LIGHT, WALL_DARK = (59, 175, 113), (33, 144, 99)
ROAD_TOP = (229, 143, 77)
WATER_TOP = (150, 208, 243)
SIDE_LEFT, SIDE_RIGHT = (205, 105, 61), (159, 70, 58)


class Block:
    """A generated iso block and its top-face diamond: centre (cx, cy),
    half width hw, half height hh."""

    def __init__(self, path):
        self.im = Image.open(path).convert("RGBA")
        self.px = self.im.load()
        w, h = self.im.size
        rows = []
        for y in range(h):
            xs = [x for x in range(w) if self.px[x, y][3] >= 128]
            rows.append((min(xs), max(xs)) if xs else None)
        filled = [y for y in range(h) if rows[y]]
        y0, y_end = filled[0], filled[-1]
        widest = max(rows[y][1] - rows[y][0] for y in filled)
        yc = next(y for y in filled if rows[y][1] - rows[y][0] == widest)
        xl, xr = rows[yc]
        self.cx, self.cy = (xl + xr + 1) / 2, yc + 0.5
        self.hw, self.hh = (xr - xl + 1) / 2, yc - y0 + 0.5
        # side depth: from the face's left/right corner down to the bottom edge
        self.depth = max(1.0, y_end - (yc + self.hh))

    def at(self, x, y):
        w, h = self.im.size
        x, y = min(max(int(x), 0), w - 1), min(max(int(y), 0), h - 1)
        return self.px[x, y]

    def face(self, u, v):
        """Top-face pixel at diamond coords u, v in [-1, 1] (|u|+|v| <= 1),
        pulled 1px inside so the outline never gets sampled."""
        k = 0.9
        return self.at(self.cx + u * self.hw * k, self.cy + v * self.hh * k)

    def side(self, u, t):
        """Side pixel: u in [-1, 1] across (left side < 0), t in [0, 1]
        down from the face edge to the bottom."""
        k = 0.92
        x = self.cx + u * self.hw * k
        edge = self.cy + (1 - abs(u)) * self.hh  # the face's lower edge above x
        return self.at(x, edge + 1 + t * (self.depth - 2))

    def mean(self, pick):
        cs = [c for c in self.im.getdata() if c[3] >= 128 and pick(c)]
        n = max(1, len(cs))
        return tuple(sum(c[i] for c in cs) // n for i in range(3))


def lum(c):
    return 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]


def shade(c, k):
    return tuple(max(0, min(255, int(v * k))) for v in c[:3])


def compose(piece, grass, dirt, water, pal):
    """Repaint one pack piece from the RD blocks."""
    src = piece.convert("RGBA")
    w, h = src.size
    sp = src.load()
    out = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    op = out.load()
    # the pack face: top vertex row 0, corners row ~27, centre x ~47
    pcx, pcy, phw, phh = w / 2, 27.0, w / 2, 27.0
    pdepth = h - 2 * phh
    for y in range(h):
        for x in range(w):
            c = sp[x, y]
            if c[3] == 0:
                continue
            rgb = c[:3]
            u, v = (x + 0.5 - pcx) / phw, (y + 0.5 - pcy) / phh
            on_face = abs(u) + abs(v) <= 1.0
            if rgb == GRASS_TOP:
                n = grass.face(u, v) if on_face else (pal["grass"] + (255,))
            elif rgb == ROAD_TOP:
                n = dirt.face(u, v) if on_face else (pal["dirt"] + (255,))
            elif rgb == WATER_TOP:
                n = water.face(u, v) if on_face else (pal["water"] + (255,))
            elif rgb in (SIDE_LEFT, SIDE_RIGHT, WALL_LIGHT, WALL_DARK) and not on_face:
                # outer sides, the pack's green band included: the RD
                # block's own side (thin grass lip over earth)
                t = (y + 0.5 - (pcy + (1 - abs(u)) * phh)) / max(1.0, pdepth)
                n = grass.side(u, min(max(t, 0.0), 1.0))
            elif rgb in (SIDE_LEFT, SIDE_RIGHT):
                n = pal["side_l"] if rgb == SIDE_LEFT else pal["side_r"]
            elif rgb == WALL_LIGHT:
                n = pal["wall_l"]
            elif rgb == WALL_DARK:
                n = pal["wall_d"]
            elif rgb == OUTLINE:
                n = pal["outline"]
            else:
                # odd shades (shadowed earth under a road edge): the nearest
                # palette class, shaded by how much darker this pixel is
                base = min(pal["map"], key=lambda kv: sum((a - b) ** 2 for a, b in zip(kv[0], rgb)))
                n = shade(base[1], lum(rgb) / max(1.0, lum(base[0])))
            op[x, y] = tuple(n[:3]) + (c[3],)
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--grass", required=True)
    ap.add_argument("--dirt", required=True)
    ap.add_argument("--water", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--preview")
    args = ap.parse_args()

    grass, dirt, water = Block(args.grass), Block(args.dirt), Block(args.water)
    g_top = grass.mean(lambda c: c[1] > c[0] and c[1] > c[2])
    side_l = tuple(int(sum(grass.side(-0.5 + i * 0.1, 0.5)[k] for i in range(5)) / 5) for k in range(3))
    side_r = tuple(int(sum(grass.side(0.1 + i * 0.1, 0.5)[k] for i in range(5)) / 5) for k in range(3))
    outline = min((c[:3] for c in grass.im.getdata() if c[3] >= 128), key=lum)
    pal = {
        "grass": g_top,
        "dirt": dirt.face(0, 0)[:3],
        "water": water.face(0, 0)[:3],
        "side_l": side_l, "side_r": side_r,
        "wall_l": shade(g_top, 0.9), "wall_d": shade(g_top, 0.76),
        "outline": outline,
    }
    pal["map"] = [(GRASS_TOP, pal["grass"]), (ROAD_TOP, pal["dirt"]), (WATER_TOP, pal["water"]),
                  (SIDE_LEFT, side_l), (SIDE_RIGHT, side_r), (WALL_LIGHT, pal["wall_l"]),
                  (WALL_DARK, pal["wall_d"]), (OUTLINE, outline)]
    print("palette:", {k: v for k, v in pal.items() if k != "map"})

    os.makedirs(args.out, exist_ok=True)
    jobs = [("Enviroument/Spring/grass.png", "grass.png")]
    jobs += [(f"Roads/Spring/road({i}).png", f"road({i}).png") for i in range(1, 16)]
    jobs += [(f"Rivers/Spring/river({i}).png", f"river({i}).png") for i in range(1, 18)]
    made = []
    for rel, name in jobs:
        path = os.path.join(PACK, rel)
        if not os.path.exists(path):
            continue
        piece = compose(Image.open(path), grass, dirt, water, pal)
        piece.save(os.path.join(args.out, name))
        made.append((Image.open(path).convert("RGBA"), piece))
    print(f"wrote {len(made)} pieces to {args.out}")

    if args.preview:
        cols = 11
        rows = (len(made) + cols - 1) // cols
        sheet = Image.new("RGBA", (cols * 100, rows * 2 * 100), (255, 0, 255, 255))
        for i, (a, b) in enumerate(made):
            x, y = (i % cols) * 100, (i // cols) * 200
            sheet.alpha_composite(a, (x, y))
            sheet.alpha_composite(b, (x, y + 100))
        sheet.save(args.preview)


if __name__ == "__main__":
    main()
