#!/usr/bin/env python3
"""sound_fetch.py <query> [--max N] [--dry-run]

Search Freesound's CC0 corpus and download HQ OGG previews into
assets/sound/_incoming/ (#1495). The API key comes from
$FREESOUND_API_KEY and is never printed or logged (decision 11).
Each file lands as <id>.ogg plus a <id>.json sidecar holding the
source URL, author and license URL - the ledger's evidence.
"""
import argparse
import json
import os
import sys
import urllib.request
import urllib.parse

API = "https://freesound.org/apiv2/search/text/"
INCOMING = os.path.join(os.path.dirname(__file__), "..", "assets", "sound", "_incoming")


def die(msg, code=1):
    print(f"sound_fetch: {msg}", file=sys.stderr)
    sys.exit(code)


def api_key():
    key = os.environ.get("FREESOUND_API_KEY", "").strip()
    if not key:
        die("FREESOUND_API_KEY is not set - get a Freesound API key and export it "
            "(the key is never committed or printed)", 2)
    return key


def get(url, key, binary=False):
    req = urllib.request.Request(url, headers={"Authorization": f"Token {key}"})
    with urllib.request.urlopen(req) as r:
        return r.read() if binary else json.loads(r.read().decode())


def main():
    ap = argparse.ArgumentParser(description="Fetch CC0 previews from Freesound into _incoming/")
    ap.add_argument("query")
    ap.add_argument("--max", type=int, default=5, help="results to fetch (default 5)")
    ap.add_argument("--dry-run", action="store_true", help="list candidates, download nothing")
    args = ap.parse_args()

    key = api_key()
    params = urllib.parse.urlencode({
        "query": args.query,
        "filter": 'license:"Creative Commons 0"',
        "fields": "id,name,username,license,url,previews",
        "page_size": max(args.max, 1),
    })
    try:
        data = get(f"{API}?{params}", key)
    except Exception as e:
        die(f"search failed: {e}")

    results = data.get("results", [])[: args.max]
    if not results:
        print(f"sound_fetch: no CC0 results for {args.query!r}")
        return
    for s in results:
        preview = (s.get("previews") or {}).get("preview-hq-ogg")
        print(f"  {s['id']:>8}  {s.get('name','?')[:60]:60}  by {s.get('username','?')}")
        if args.dry_run:
            continue
        if not preview:
            print(f"    skip {s['id']}: no HQ OGG preview")
            continue
        os.makedirs(INCOMING, exist_ok=True)
        try:
            blob = get(preview, key, binary=True)
        except Exception as e:
            print(f"    skip {s['id']}: download failed ({e})", file=sys.stderr)
            continue
        with open(os.path.join(INCOMING, f"{s['id']}.ogg"), "wb") as f:
            f.write(blob)
        sidecar = {
            "source_url": s.get("url", ""),
            "author": s.get("username", ""),
            "license": s.get("license", ""),
            "freesound_id": s["id"],
        }
        with open(os.path.join(INCOMING, f"{s['id']}.json"), "w") as f:
            json.dump(sidecar, f, indent=2)
        print(f"    -> _incoming/{s['id']}.ogg ({len(blob)} bytes) + {s['id']}.json")


if __name__ == "__main__":
    main()
