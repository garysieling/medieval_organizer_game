#!/usr/bin/env python3
"""Download the Creativity Generator corpus from Project Gutenberg.

Reads wiki/creativity-corpus/sources.tsv and writes plain-text files to
data/creativity-corpus/<tradition>/<gutenberg_id>.txt with the Gutenberg
license header/footer stripped. Stdlib only. Re-running skips files that exist.

Usage: python scripts/fetch_creativity_corpus.py [--out DIR] [--only TRADITION,...]
"""
import argparse
import os
import shutil
import subprocess
import sys
import time
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MANIFEST = os.path.join(ROOT, "wiki", "creativity-corpus", "sources.tsv")
URLS = [
    "https://www.gutenberg.org/cache/epub/{id}/pg{id}.txt",
    "https://www.gutenberg.org/files/{id}/{id}-0.txt",
    "https://www.gutenberg.org/files/{id}/{id}.txt",
]
UA = "game2-creativity-generator/0.1 (personal research corpus)"


def read_manifest(path):
    for line in open(path, encoding="utf-8"):
        if not line.strip() or line.startswith("#"):
            continue
        gid, tradition, lang, title = line.rstrip("\n").split("\t", 3)
        yield gid, tradition, lang, title


def strip_boilerplate(text):
    start = text.find("*** START OF")
    if start != -1:
        text = text[text.find("\n", start) + 1:]
    end = text.find("*** END OF")
    if end != -1:
        text = text[:end]
    return text.strip()


def download(gid):
    last = None
    for tmpl in URLS:
        url = tmpl.format(id=gid)
        try:
            req = urllib.request.Request(url, headers={"User-Agent": UA})
            with urllib.request.urlopen(req, timeout=90) as resp:
                return resp.read().decode("utf-8", errors="replace")
        except Exception as e:  # try next mirror path
            last = e
            # Python's CA bundle can reject TLS-intercepting proxies; system curl uses the OS store
            if "CERTIFICATE_VERIFY_FAILED" in str(e) and shutil.which("curl"):
                r = subprocess.run(["curl", "-sfL", "-m", "120", "-A", UA, url], capture_output=True)
                if r.returncode == 0 and r.stdout:
                    return r.stdout.decode("utf-8", errors="replace")
                last = RuntimeError(f"{e}; curl exit {r.returncode}")
    raise RuntimeError(f"all URLs failed for {gid}: {last}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=os.path.join(ROOT, "data", "creativity-corpus"))
    ap.add_argument("--only", default="", help="comma-separated traditions")
    ap.add_argument("--delay", type=float, default=1.0, help="seconds between downloads")
    args = ap.parse_args()
    only = {t.strip() for t in args.only.split(",") if t.strip()}

    ok = skipped = failed = 0
    for gid, tradition, lang, title in read_manifest(MANIFEST):
        if only and tradition not in only:
            continue
        dest_dir = os.path.join(args.out, tradition)
        dest = os.path.join(dest_dir, f"{gid}.txt")
        if os.path.exists(dest) and os.path.getsize(dest) > 0:
            skipped += 1
            continue
        os.makedirs(dest_dir, exist_ok=True)
        try:
            body = strip_boilerplate(download(gid))
            with open(dest, "w", encoding="utf-8", newline="\n") as f:
                f.write(f"Title: {title}\nTradition: {tradition}\nLanguage: {lang}\nSource: https://www.gutenberg.org/ebooks/{gid}\n\n")
                f.write(body)
            ok += 1
            print(f"[ok] {tradition:14} {gid:>6} {title} ({len(body)//1024} KB)")
            time.sleep(args.delay)
        except Exception as e:
            failed += 1
            print(f"[fail] {gid} {title}: {e}", file=sys.stderr)
    print(f"downloaded={ok} skipped={skipped} failed={failed} -> {args.out}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
