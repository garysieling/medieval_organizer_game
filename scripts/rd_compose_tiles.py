#!/usr/bin/env python3
"""Build the ground tile sets (grass, roads, rivers, crop fields, and the
winter snow/road/river/ice set) from a few generated Retro Diffusion
blocks, using the asset pack's pieces as shape masks.

The pack's pieces are relief blocks (sunken channels, raised grass, inset
crop beds, earth sides) drawn in a tiny flat palette. Generating each of
the ~70 pieces would be expensive and wouldn't line up; instead every
pack pixel is classified by its palette colour and repainted:

  top surfaces (grass, road, water, soil, crop, snow, ice)
                     -> that RD block's top-face texture, sampled through
                        a face-to-face diamond mapping
  outer sides        -> the group's base RD block's own sides
  inner walls, edges -> shades of the RD colours (texture-free)

The same pack colour can mean different things in different pieces (the
light green that is a grass wall on a road is the crop on a corn field),
so each GROUP of pieces carries its own colour table.

Output pieces keep the pack's exact geometry (so tilemap.rs connections,
BLOCK_FACE and FACE_CENTRE keep working) with RD colour and texture.
Groups whose blocks aren't given are skipped. Current set:

    python scripts/rd_compose_tiles.py --out assets/rd/tiles \\
        --grass art-prompts/raw/scale-tile-grass-0b26af48/0.png \\
        --dirt  art-prompts/raw/ground-set-dirt-0ff45e1d/0.png \\
        --water art-prompts/raw/ground-set-water-dc517000/0.png \\
        --soil  art-prompts/raw/ground-set-soil-4fbbfde4/0.png \\
        --wheat art-prompts/raw/ground-set-wheat-0bdc40c3/0.png \\
        --greencrop art-prompts/raw/ground-set-greencrop-228d4299/0.png \\
        --snow  art-prompts/raw/ground-set-snow-52322a18/0.png \\
        --ice   art-prompts/raw/ground-set-ice-87f74ff0/0.png

Needs Pillow.
"""
import argparse
import os

from PIL import Image

PACK = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                    "assets", "Isometric Medieval Pack", "Isometric Medieval Pack", "Sprites")

# the pack's flat palette
OUTLINE = (46, 34, 47)
GRASS_TOP = (148, 196, 104)
WALL_LIGHT, WALL_DARK = (59, 175, 113), (33, 144, 99)
ROAD_TOP = (229, 143, 77)
WATER_TOP = (150, 208, 243)
SIDE_LEFT, SIDE_RIGHT = (205, 105, 61), (159, 70, 58)
SOIL_TOP = (230, 144, 78)
WHEAT_TOP, WHEAT_SIDE_L, WHEAT_SIDE_R = (249, 194, 45), (244, 150, 30), (237, 108, 41)
CORN_TOP, CORN_SIDE_L, CORN_SIDE_R = (59, 175, 113), (33, 144, 99), (19, 90, 76)
SNOW_TOP, SNOW_HI = (255, 255, 255), (242, 241, 242)
SNOW_SIDE_L, SNOW_SIDE_R = (221, 221, 221), (179, 179, 178)
ICE_TOP = (157, 211, 205)


class Block:
    """A generated iso block and its top-face diamond: centre (cx, cy),
    half width hw, half height hh. The face is measured at its widest row
    (the left/right corners) and taken as 2:1 - RD blocks are, and crops
    or snow standing above the face would throw off a top-down measure."""

    def __init__(self, path):
        self.im = Image.open(path).convert("RGBA")
        self.px = self.im.load()
        w, h = self.im.size
        rows = []
        for y in range(h):
            xs = [x for x in range(w) if self.px[x, y][3] >= 128]
            rows.append((min(xs), max(xs)) if xs else None)
        filled = [y for y in range(h) if rows[y]]
        y_end = filled[-1]
        widest = max(rows[y][1] - rows[y][0] for y in filled)
        yc = next(y for y in filled if rows[y][1] - rows[y][0] == widest)
        xl, xr = rows[yc]
        self.cx, self.cy = (xl + xr + 1) / 2, yc + 0.5
        self.hw = (xr - xl + 1) / 2
        self.hh = self.hw / 2
        # side depth: from the face's front corner down to the bottom edge
        self.depth = max(1.0, y_end - (self.cy + self.hh))

    def at(self, x, y):
        w, h = self.im.size
        x, y = min(max(int(x), 0), w - 1), min(max(int(y), 0), h - 1)
        return self.px[x, y]

    def face(self, u, v):
        """Top-face pixel at diamond coords u, v in [-1, 1] (|u|+|v| <= 1),
        pulled inside so the outline never gets sampled."""
        k = 0.88
        return self.at(self.cx + u * self.hw * k, self.cy + v * self.hh * k)

    def side(self, u, t):
        """Side pixel: u in [-1, 1] across (left side < 0), t in [0, 1]
        down from the face edge to the bottom."""
        k = 0.92
        x = self.cx + u * self.hw * k
        edge = self.cy + (1 - abs(u)) * self.hh  # the face's lower edge above x
        return self.at(x, edge + 1 + t * (self.depth - 2))

    def face_mean(self):
        cs = [self.face(u / 5, v / 5) for u in range(-3, 4) for v in range(-3, 4) if abs(u) + abs(v) <= 4]
        cs = [c for c in cs if c[3] >= 128]
        return tuple(sum(c[i] for c in cs) // max(1, len(cs)) for i in range(3))

    def side_mean(self, left):
        us = [-0.6, -0.5, -0.4, -0.3] if left else [0.3, 0.4, 0.5, 0.6]
        cs = [self.side(u, t) for u in us for t in (0.4, 0.6, 0.8)]
        return tuple(sum(c[i] for c in cs) // len(cs) for i in range(3))

    def outline(self):
        return min((c[:3] for c in self.im.getdata() if c[3] >= 128), key=lum)


def lum(c):
    return 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]


def shade(c, k):
    return tuple(max(0, min(255, int(v * k))) for v in c[:3])


class Group:
    """Pieces that share one colour table.
    faces:  pack colour -> RD block whose top-face texture fills it
    sides:  pack colours that, off the top face, are the block's outer
            sides: sampled from the `base` block's sides
    flat:   pack colour -> RD colour (inner walls, crop-bed edges, ...)"""

    def __init__(self, name, base, faces, sides, flat, jobs):
        self.name, self.base, self.faces, self.sides, self.flat, self.jobs = name, base, faces, sides, flat, jobs
        self.flat.setdefault(OUTLINE, base.outline())
        # fallback for odd shades: nearest known class, shaded to match
        self.known = [(c, b.face_mean()) for c, b in faces.items()] + list(flat.items())

    def paint(self, piece):
        src = piece.convert("RGBA")
        w, h = src.size
        sp = src.load()
        out = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        op = out.load()
        # the pack face: top vertex row 0, corners row ~27, centred
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
                if rgb in self.faces:
                    b = self.faces[rgb]
                    n = b.face(u, v) if on_face else b.face_mean()
                elif rgb in self.sides and not on_face:
                    t = (y + 0.5 - (pcy + (1 - abs(u)) * phh)) / max(1.0, pdepth)
                    n = self.base.side(u, min(max(t, 0.0), 1.0))
                elif rgb in self.flat:
                    n = self.flat[rgb]
                else:
                    base = min(self.known, key=lambda kv: sum((a - b) ** 2 for a, b in zip(kv[0], rgb)))
                    n = shade(base[1], lum(rgb) / max(1.0, lum(base[0])))
                op[x, y] = tuple(n[:3]) + (c[3],)
        return out


def groups(b):
    """The piece groups the given blocks can build. b: name -> Block."""
    out = []
    if all(k in b for k in ("grass", "dirt", "water")):
        g = b["grass"].face_mean()
        out.append(Group("green", b["grass"],
            faces={GRASS_TOP: b["grass"], ROAD_TOP: b["dirt"], WATER_TOP: b["water"]},
            sides={SIDE_LEFT, SIDE_RIGHT, WALL_LIGHT, WALL_DARK},
            flat={SIDE_LEFT: b["grass"].side_mean(True), SIDE_RIGHT: b["grass"].side_mean(False),
                  WALL_LIGHT: shade(g, 0.9), WALL_DARK: shade(g, 0.76)},
            jobs=[("Enviroument/Spring/grass.png", "grass.png")]
                 + [(f"Roads/Spring/road({i}).png", f"road({i}).png") for i in range(1, 16)]
                 + [(f"Rivers/Spring/river({i}).png", f"river({i}).png") for i in range(1, 18)]))
    if all(k in b for k in ("soil", "wheat")):
        wm = b["wheat"].face_mean()
        out.append(Group("wheat", b["soil"],
            faces={SOIL_TOP: b["soil"], WHEAT_TOP: b["wheat"]},
            sides={SIDE_LEFT, SIDE_RIGHT},
            flat={SIDE_LEFT: b["soil"].side_mean(True), SIDE_RIGHT: b["soil"].side_mean(False),
                  WHEAT_SIDE_L: shade(wm, 0.85), WHEAT_SIDE_R: shade(wm, 0.7)},
            jobs=[(f"Buildings/wheat/wheat({i}).png", f"wheat({i}).png") for i in (1, 2)]))
    if all(k in b for k in ("soil", "greencrop")):
        cm = b["greencrop"].face_mean()
        out.append(Group("corn", b["soil"],
            faces={SOIL_TOP: b["soil"], CORN_TOP: b["greencrop"]},
            sides={SIDE_LEFT, SIDE_RIGHT},
            flat={SIDE_LEFT: b["soil"].side_mean(True), SIDE_RIGHT: b["soil"].side_mean(False),
                  CORN_SIDE_L: shade(cm, 0.8), CORN_SIDE_R: shade(cm, 0.6)},
            jobs=[(f"Buildings/corn/corn({i}).png", f"corn({i}).png") for i in (1, 2)]))
    if all(k in b for k in ("snow", "dirt", "water", "ice")):
        sm = b["snow"].face_mean()
        out.append(Group("winter", b["snow"],
            faces={SNOW_TOP: b["snow"], SNOW_HI: b["snow"], ROAD_TOP: b["dirt"],
                   WATER_TOP: b["water"], ICE_TOP: b["ice"]},
            sides={SIDE_LEFT, SIDE_RIGHT, SNOW_SIDE_L, SNOW_SIDE_R},
            flat={SIDE_LEFT: b["snow"].side_mean(True), SIDE_RIGHT: b["snow"].side_mean(False),
                  SNOW_SIDE_L: shade(sm, 0.88), SNOW_SIDE_R: shade(sm, 0.72)},
            jobs=[("Enviroument/Winter/snow.png", "winter/snow.png")]
                 + [(f"Roads/Winter/road_winter({i}).png", f"winter/road_winter({i}).png") for i in range(1, 16)]
                 + [(f"Rivers/Winter/river_winter({i}).png", f"winter/river_winter({i}).png") for i in range(1, 18)]
                 + [(f"Rivers/Winter/Ice/river_ice_winter({i}).png", f"winter/river_ice_winter({i}).png")
                    for i in range(1, 18)]))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    names = ["grass", "dirt", "water", "soil", "wheat", "greencrop", "snow", "ice"]
    for n in names:
        ap.add_argument("--" + n, help=f"the generated {n} block")
    ap.add_argument("--out", required=True)
    ap.add_argument("--preview", help="pack piece over repainted piece, every piece")
    args = ap.parse_args()

    blocks = {n: Block(getattr(args, n)) for n in names if getattr(args, n)}
    made = []
    for g in groups(blocks):
        n = 0
        for rel, name in g.jobs:
            path = os.path.join(PACK, rel)
            if not os.path.exists(path):
                continue
            dst = os.path.join(args.out, name)
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            piece = g.paint(Image.open(path))
            piece.save(dst)
            made.append((Image.open(path).convert("RGBA"), piece))
            n += 1
        print(f"{g.name}: {n} pieces")
    print(f"wrote {len(made)} pieces to {args.out}")

    if args.preview:
        cols = 12
        rows = (len(made) + cols - 1) // cols
        sheet = Image.new("RGBA", (cols * 100, rows * 2 * 100), (255, 0, 255, 255))
        for i, (a, b) in enumerate(made):
            x, y = (i % cols) * 100, (i // cols) * 200
            sheet.alpha_composite(a, (x, y))
            sheet.alpha_composite(b, (x, y + 100))
        sheet.save(args.preview)


if __name__ == "__main__":
    main()
