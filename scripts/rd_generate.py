#!/usr/bin/env python3
"""Run Retro Diffusion generations for entries in the art-prompt manifest.

generate_art_prompts.py decides WHAT to draw (prompts.json); this script
spends the money: it POSTs each picked entry to the RD API, saves every
returned image under art-prompts/raw/<id>/, and records the id as done in
art-prompts/generated.json so the next manifest run skips it.

    # free dry run: what would these cost?
    python scripts/rd_generate.py sheet-farm-animals --dry-run
    # generate (ids may be prefixes; the manifest is rebuilt if missing)
    python scripts/rd_generate.py sheet-farm-animals
    # regenerate something already done (new seed, new output dir suffix)
    python scripts/rd_generate.py sheet-farm-animals --force

Token: $RETRODIFFUSION_API_KEY, else ~/.config/retrodiffusion/token.
Never commit the token.

Each run writes art-prompts/raw/<id>/<n>.png plus meta.json (prompt,
request, seed-free response metadata, cost, timestamp) so a sheet can be
re-sliced later without paying again. Stdlib only.
"""
import argparse
import base64
import json
import os
import sys
import time
import urllib.error
import urllib.request


ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STATE = os.path.join(ROOT, "art-prompts", "generated.json")
RAW = os.path.join(ROOT, "art-prompts", "raw")
API = "https://api.retrodiffusion.ai/v1/inferences"
CREDITS = "https://api.retrodiffusion.ai/v1/inferences/credits"
# Hard stop per invocation unless --max-usd raises it: a typo in the id
# filter shouldn't be able to drain the balance.
DEFAULT_MAX_USD = 2.00


def token():
    t = os.environ.get("RETRODIFFUSION_API_KEY")
    if t:
        return t.strip()
    path = os.path.expanduser("~/.config/retrodiffusion/token")
    try:
        with open(path, encoding="utf-8") as f:
            return f.read().strip()
    except OSError:
        sys.exit("no token: set RETRODIFFUSION_API_KEY or write it to " + path)


def call(url, body=None, idem=None):
    headers = {"X-RD-Token": token(), "Content-Type": "application/json"}
    if idem:
        headers["Idempotency-Key"] = idem
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, headers=headers,
                                 method="POST" if data else "GET")
    try:
        with urllib.request.urlopen(req, timeout=300) as r:
            return json.load(r)
    except urllib.error.HTTPError as e:
        raise RuntimeError(f"RD API {e.code}: {e.read().decode(errors='replace')[:500]}") from None


def load_manifest():
    """Every entry, straight from the generator (prompts.json on disk may
    be a filtered batch, or stale)."""
    sys.path.insert(0, os.path.join(ROOT, "scripts"))
    import generate_art_prompts as g
    return g.ENTRIES


def load_done():
    try:
        with open(STATE, encoding="utf-8") as f:
            d = json.load(f)
        return set(d.get("done", d) if isinstance(d, dict) else d)
    except (OSError, json.JSONDecodeError):
        return set()


def save_done(done):
    with open(STATE, "w", encoding="utf-8") as f:
        json.dump({"done": sorted(done)}, f, indent=1)
        f.write("\n")


def body_for(e, check_cost=False):
    b = dict(e["request"])
    b["prompt"] = e["prompt"]
    if check_cost:
        b["check_cost"] = True
    return b


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("ids", nargs="+", help="manifest ids or id prefixes (e.g. sheet-farm-animals)")
    ap.add_argument("--dry-run", action="store_true", help="check_cost only, spend nothing")
    ap.add_argument("--force", action="store_true", help="regenerate ids already marked done")
    ap.add_argument("--max-usd", type=float, default=DEFAULT_MAX_USD,
                    help="refuse if the checked cost exceeds this (default %(default)s)")
    args = ap.parse_args()

    entries = load_manifest()
    done = load_done()
    picked = []
    for want in args.ids:
        hits = [e for e in entries if e["id"] == want or e["id"].startswith(want + "-")]
        if not hits:
            sys.exit(f"no manifest entry matches {want!r} (see generate_art_prompts.py)")
        for e in hits:
            if e["id"] in done and not args.force:
                print(f"skip {e['id']}: already generated (--force to redo)")
                continue
            picked.append(e)
    if not picked:
        return

    # price the whole batch first: check_cost is free
    total = 0.0
    for e in picked:
        c = call(API, body_for(e, check_cost=True))
        e["_cost"] = c.get("balance_cost") or c.get("credit_cost") or 0
        total += e["_cost"]
        print(f"{e['id']:44s} {e['request']['prompt_style']:32s} ${e['_cost']:.2f}"
              f" (manifest est ${e['est_cost_usd']:.2f})")
    bal = call(CREDITS).get("balance")
    print(f"total ${total:.2f}; balance ${bal}")
    if args.dry_run:
        return
    if total > args.max_usd:
        sys.exit(f"batch ${total:.2f} > --max-usd {args.max_usd:.2f}; raise it to proceed")

    failed = []
    for e in picked:
        out = os.path.join(RAW, e["id"])
        if os.path.exists(out) and args.force:
            out += time.strftime("-%Y%m%d%H%M%S")
        os.makedirs(out, exist_ok=True)
        t0 = time.time()
        # no Idempotency-Key: the API rejects it on synchronous calls
        # ("idempotency_async_required")
        try:
            r = call(API, body_for(e))
        except RuntimeError as err:
            # one bad entry (e.g. a size the style can't do - check_cost
            # doesn't validate sizes) shouldn't sink the rest of the batch
            print(f"{e['id']}: FAILED {err}")
            failed.append(e["id"])
            os.rmdir(out) if not os.listdir(out) else None
            continue
        imgs = r.get("base64_images") or []
        if not imgs:
            sys.exit(f"{e['id']}: no images in response: "
                     + json.dumps({k: v for k, v in r.items() if k != 'base64_images'})[:500])
        for i, b64 in enumerate(imgs):
            raw = base64.b64decode(b64)
            # animation styles can answer with a GIF; name files by content
            ext = "gif" if raw[:4] == b"GIF8" else "webp" if raw[8:12] == b"WEBP" else "png"
            with open(os.path.join(out, f"{i}.{ext}"), "wb") as f:
                f.write(raw)
        meta = {k: v for k, v in r.items() if k not in ("base64_images", "output_images")}
        meta.update({"id": e["id"], "prompt": e["prompt"], "request": e["request"],
                     "generated_at": time.strftime("%Y-%m-%dT%H:%M:%S"),
                     "seconds": round(time.time() - t0, 1)})
        with open(os.path.join(out, "meta.json"), "w", encoding="utf-8") as f:
            json.dump(meta, f, indent=1)
            f.write("\n")
        done.add(e["id"])
        save_done(done)
        print(f"{e['id']}: {len(imgs)} image(s) -> {os.path.relpath(out, ROOT)} "
              f"(${r.get('balance_cost', '?')}, {meta['seconds']}s, balance left "
              f"${r.get('remaining_balance', '?')})")

    if failed:
        sys.exit(f"{len(failed)} failed (not recorded as done): {', '.join(failed)}")


if __name__ == "__main__":
    main()
