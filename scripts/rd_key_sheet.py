#!/usr/bin/env python3
"""Cut sprites out of an RD sheet that came back on an opaque ground.

`rd_slice.py` flood-keys the background from the sheet's edge; when the
sheet returns with a painted (parchment) ground - remove_bg not applied,
or a composed scene rather than a loose grid - the ground's gradient and
vignette defeat that. This one takes explicit cell rectangles instead:

    python scripts/rd_key_sheet.py art-prompts/raw/ui-vine-seasons-977b26d6/0.png \
        --out assets/rd/ui-frame --bg 241,225,181 --tol 45 \
        "15,13,98,100=vine-spring" "143,13,98,100=vine-summer" ...

Each RECT is x,y,w,h inside the sheet; its pixels are keyed out where
they sit within --tol of the --bg colour (sampled from the sheet border
when --bg is omitted), the largest connected blobs are kept, trimmed and
saved transparent. `@H` / `@WxH` resample like rd_slice's names.
"""
import argparse
import collections
import sys
from PIL import Image


def key_blobs(im, bg, tol, minpx, corner_mask):
    px = im.load()
    w, h = im.size
    mask = bytearray(w * h)
    for y in range(h):
        for x in range(w):
            if corner_mask and corner_mask(x, y):
                continue
            p = px[x, y]
            if max(abs(p[0] - bg[0]), abs(p[1] - bg[1]), abs(p[2] - bg[2])) > tol:
                mask[y * w + x] = 1
    seen = bytearray(w * h)
    blobs = []
    for i in range(w * h):
        if mask[i] and not seen[i]:
            q = [i]
            seen[i] = 1
            comp = []
            while q:
                j = q.pop()
                comp.append(j)
                x, y = j % w, j // w
                for nj in (j - 1 if x else -1, j + 1 if x < w - 1 else -1,
                           j - w if y else -1, j + w if y < h - 1 else -1):
                    if nj >= 0 and mask[nj] and not seen[nj]:
                        seen[nj] = 1
                        q.append(nj)
            if len(comp) >= minpx:
                blobs.append(comp)
    return blobs


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("sheet")
    ap.add_argument("cells", nargs="+",
                    help="x,y,w,h=name[@H|@WxH][#corners] - #tl#tr#bl#br mask"
                         " corner blocks a frame ornament bleeds into")
    ap.add_argument("--out", required=True)
    ap.add_argument("--bg", help="r,g,b to key out (default: most common edge colour)")
    ap.add_argument("--tol", type=int, default=40)
    ap.add_argument("--min-px", type=int, default=40,
                    help="ignore blobs smaller than this")
    args = ap.parse_args()

    im = Image.open(args.sheet).convert("RGBA")
    if args.bg:
        bg = tuple(int(c) for c in args.bg.split(","))
    else:
        cnt = collections.Counter()
        w, h = im.size
        for x in range(w):
            for y in (0, h - 1):
                cnt[im.getpixel((x, y))[:3]] += 1
        for y in range(h):
            for x in (0, w - 1):
                cnt[im.getpixel((x, y))[:3]] += 1
        bg = cnt.most_common(1)[0][0]
        print("bg sampled:", bg)

    import os
    os.makedirs(args.out, exist_ok=True)
    for spec in args.cells:
        rect, _, rest = spec.partition("=")
        x0, y0, w, h = [int(v) for v in rect.split(",")]
        name, _, size = rest.partition("@")
        name, _, corner = name.partition("#")
        crop = im.crop((x0, y0, x0 + w, y0 + h))
        cm = None
        if corner:
            c = 14
            def cm(x, y, w=w, h=h, c=c, corner=corner):
                return ("tl" in corner and x < c and y < c) \
                    or ("tr" in corner and x >= w - c and y < c) \
                    or ("bl" in corner and x < c and y >= h - c) \
                    or ("br" in corner and x >= w - c and y >= h - c)
        blobs = key_blobs(crop, bg, args.tol, args.min_px, cm)
        if not blobs:
            print(f"{name}: nothing keyed - skipping", file=sys.stderr)
            continue
        ip = crop.load()
        blobs = [b for b in blobs if not all(  # drop pale ornament scraps
            sum(ip[j % w, j // w][k] for j in b) // len(b) > t
            for k, t in ((0, 190), (1, 170), (2, 120)))]
        keep = [j for b in blobs for j in b]
        ip = crop.load()
        xs = [j % w for j in keep]
        ys = [j // w for j in keep]
        bx0, bx1, by0, by1 = min(xs), max(xs) + 1, min(ys), max(ys) + 1
        out = Image.new("RGBA", (bx1 - bx0, by1 - by0), (0, 0, 0, 0))
        op = out.load()
        for j in keep:
            op[j % w - bx0, j // w - by0] = ip[j % w, j // w]
        if size:
            if "x" in size:
                tw, th = [int(v) for v in size.split("x")]
            else:
                th = int(size)
                tw = out.size[0] * th // out.size[1]
            out = out.resize((tw, th), Image.BOX)
        out.save(os.path.join(args.out, name + ".png"))
        print(f"{name}: {out.size[0]}x{out.size[1]} ({len(blobs)} blobs)")


if __name__ == "__main__":
    main()
