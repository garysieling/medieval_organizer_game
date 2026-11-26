#!/usr/bin/env python3
"""sound_fetch_commons.py <query> [--max N] [--dry-run]

Wikimedia Commons CC0 fallback for sound_fetch.py (#1500/#1577): search
Commons' audio, keep only files whose own metadata says CC0 or public
domain, download into assets/sound/_incoming/ as <id>.ogg plus a
<id>.json sidecar holding the source URL, author and license evidence -
the ledger's evidence. Non-Ogg originals (flac/wav) are transcoded via
ffmpeg so sound_process.py finds them.

  python3 scripts/sound_fetch_commons.py "birdsong forest" --max 3
"""
import argparse
import html
import json
import os
import re
import subprocess
import sys
import tempfile
import time
import urllib.parse
import urllib.request

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

API = "https://commons.wikimedia.org/w/api.php"
INCOMING = os.path.join(os.path.dirname(__file__), "..", "assets", "sound", "_incoming")
UA = {"User-Agent": "game2-asset-fetch/1.0 (asset pipeline; contact: repo owner)"}
CC0 = re.compile(r"cc0|cc.zero|public.domain", re.I)
TAG = re.compile(r"<[^>]+>")


def api(**params):
    # maxlag asks the server to tell us when it's loaded instead of
    # silently queueing; back off on that and on 429/5xx.
    params.update({"format": "json", "maxlag": "5"})
    url = f"{API}?{urllib.parse.urlencode(params)}"
    for attempt in range(6):
        req = urllib.request.Request(url, headers=UA)
        try:
            with urllib.request.urlopen(req, timeout=30) as r:
                data = json.loads(r.read().decode())
        except urllib.error.HTTPError as e:
            if e.code == 429 or e.code >= 500:
                time.sleep(2 ** attempt + 1)
                continue
            raise
        err = data.get("error") or {}
        if err.get("code") == "maxlag":
            time.sleep(2 ** attempt + 1)
            continue
        if err:
            # a real API error (e.g. "toomanyvalues") - surface it instead
            # of letting the caller read the missing "query" key as "no
            # results"
            die(f"Commons API error ({err.get('code', '?')}): {err.get('info', '')}")
        return data
    die("Commons API kept us waiting too long - try again later")


def die(msg, code=1):
    print(f"sound_fetch_commons: {msg}", file=sys.stderr)
    sys.exit(code)


def text(meta, key):
    v = (meta.get(key) or {}).get("value", "")
    return html.unescape(TAG.sub("", v)).strip()


def search(query, limit):
    data = api(action="query", list="search", srsearch=f"{query} filetype:audio",
               srnamespace=6, srlimit=limit * 3)
    return [h["title"] for h in data.get("query", {}).get("search", [])]


def info(titles):
    """Metadata for every candidate title, batched to Commons' 50-title
    cap per imageinfo request (over that the API answers "toomanyvalues",
    which api() now surfaces instead of looking like an empty result).
    Indexes results under both the canonical (returned) title and
    whatever title was actually asked for, since MediaWiki normalises
    titles (underscores -> spaces, first letter) before answering, so a
    title copied verbatim from a Commons URL must still find its match."""
    out = {}
    for i in range(0, len(titles), 50):
        batch = titles[i:i + 50]
        data = api(action="query", titles="|".join(batch), prop="imageinfo",
                   iiprop="url|size|mime|extmetadata")
        alias = {n.get("to"): n.get("from")
                 for n in data.get("query", {}).get("normalized", [])}
        for page in data.get("query", {}).get("pages", {}).values():
            canonical = page.get("title", "")
            for ii in page.get("imageinfo", []):
                out[canonical] = (page["pageid"], ii)
                original = alias.get(canonical)
                if original:
                    out[original] = (page["pageid"], ii)
    return out


def is_audio(ii):
    """Reject known non-audio media (an image title slipping through
    --title, say) while still allowing audio reported as the generic
    "application/ogg" container mime that Commons gives some Ogg
    Vorbis uploads."""
    mime = (ii.get("mime") or "").lower()
    return mime.startswith("audio/") or mime == "application/ogg"


def has_audio_stream(path):
    try:
        out = subprocess.run(
            ["ffprobe", "-v", "quiet", "-select_streams", "a", "-show_entries",
             "stream=codec_type", "-of", "csv=p=0", path],
            capture_output=True, text=True, check=True)
        return bool(out.stdout.strip())
    except Exception:
        return False


def cc0_only(ii):
    meta = ii.get("extmetadata", {})
    return (CC0.search(text(meta, "LicenseShortName"))
            or CC0.search(text(meta, "License"))
            or CC0.search(text(meta, "UsageTerms")))


def save(idd, ii):
    url, meta = ii["url"], ii.get("extmetadata", {})
    ext = os.path.splitext(urllib.parse.urlparse(url).path)[1] or ".ogg"
    os.makedirs(INCOMING, exist_ok=True)
    req = urllib.request.Request(url, headers=UA)
    blob = urllib.request.urlopen(req, timeout=60).read()
    dst = os.path.join(INCOMING, f"{idd}.ogg")
    if ext.lower() in (".ogg", ".oga"):
        with open(dst, "wb") as f:
            f.write(blob)
    else:  # sound_process.py reads <id>.ogg; transcode flac/wav/etc.
        with tempfile.NamedTemporaryFile(suffix=ext, delete=False) as t:
            t.write(blob)
        try:
            subprocess.run(["ffmpeg", "-y", "-v", "error", "-i", t.name, dst], check=True)
        finally:
            os.unlink(t.name)
    if not has_audio_stream(dst):
        os.unlink(dst)
        raise RuntimeError(f"no audio stream in the downloaded file (mime {ii.get('mime', '?')})")
    sidecar = {
        "source_url": ii.get("descriptionurl", ""),
        "author": text(meta, "Artist") or text(meta, "Attribution"),
        "license": ii.get("descriptionurl", ""),
        "license_claim": text(meta, "LicenseShortName"),
        "commons_title": ii.get("commons_title", ""),
    }
    with open(os.path.join(INCOMING, f"{idd}.json"), "w") as f:
        json.dump(sidecar, f, indent=2)
    print(f"    -> _incoming/{idd}.ogg ({len(blob)} bytes) + {idd}.json")


def main():
    ap = argparse.ArgumentParser(description="Fetch CC0 audio from Wikimedia Commons into _incoming/")
    ap.add_argument("query", nargs="?", help="search text, or omit with --title")
    ap.add_argument("--title", help="fetch this exact 'File:...' instead of searching")
    ap.add_argument("--max", type=int, default=5, help="files to fetch (default 5)")
    ap.add_argument("--dry-run", action="store_true", help="list candidates, download nothing")
    args = ap.parse_args()
    if not args.title and not args.query:
        die("give a query or --title File:...")

    got = 0
    titles = [args.title] if args.title else search(args.query, args.max)
    info_map = info(titles)
    for title in titles:
        pageid, ii = info_map.get(title, (None, None))
        if ii is None or not cc0_only(ii):
            continue
        if not is_audio(ii):
            print(f"    skip {title}: not audio (mime {ii.get('mime', '?')})", file=sys.stderr)
            continue
        meta = ii.get("extmetadata", {})
        print(f"  {pageid:>9}  {title[5:][:56]:56}  [{text(meta, 'LicenseShortName')}]")
        if not args.dry_run:
            ii["commons_title"] = title
            try:
                save(f"commons_{pageid}", ii)
            except Exception as e:
                print(f"    skip {title}: {e}", file=sys.stderr)
                continue
        got += 1
        if got >= args.max:
            return
    if not got:
        print(f"sound_fetch_commons: no CC0 results for {args.title or args.query!r}")


if __name__ == "__main__":
    main()
