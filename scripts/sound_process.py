#!/usr/bin/env python3
"""sound_process.py <incoming-id> --name <asset-id> --kind bed|oneshot|instrument

Process an _incoming/ file through the ffmpeg recipe into
assets/sound/<kind>s/<name>.ogg and append its LEDGER.json entry
(#1495): sha256 computed on the processed file, `processing` recording
the ffmpeg settings. --dry-run prints the pipeline without running it.

Linux uses the system ffmpeg; on Windows run scripts/get-ffmpeg.ps1
first (see assets/sound/README.md).
"""
import argparse
import hashlib
import json
import os
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(__file__), "..")
SOUND = os.path.join(ROOT, "assets", "sound")
LEDGER = os.path.join(SOUND, "LEDGER.json")


def die(msg, code=1):
    print(f"sound_process: {msg}", file=sys.stderr)
    sys.exit(code)


def probe_duration(path):
    try:
        out = subprocess.run(
            ["ffprobe", "-v", "quiet", "-show_entries", "format=duration",
             "-of", "csv=p=0", path],
            capture_output=True, text=True, check=True)
        return float(out.stdout.strip())
    except Exception:
        return None


def recipe(kind, src, dst, dur):
    """The ffmpeg argv for a kind: trim, the kind's normalise, mono for
    point sources, 44.1 kHz Ogg Vorbis q5. Returns (argv, note)."""
    base = ["ffmpeg", "-y", "-v", "error", "-i", src]
    if kind == "bed":
        # loudnorm to -20 LUFS, then crossfade the tail into the head so
        # the loop has no seam (fade = min(2s, dur/4))
        fade = 2.0 if dur is None else min(2.0, dur / 4.0)
        if dur is not None and dur > fade * 2:
            fc = (f"[0:a]atrim=0:{dur - fade:.3f}[h];"
                  f"[0:a]atrim={dur - fade:.3f}[t];"
                  f"[h][t]acrossfade=d={fade:.3f},loudnorm=I=-20:TP=-2:LRA=11")
            return base + ["-filter_complex", fc, "-ar", "44100",
                           "-c:a", "libvorbis", "-q:a", "5", dst], \
                f"trim; loudnorm I=-20 TP=-2; crossfade {fade:.1f}s loop; 44.1kHz ogg q5"
        return base + ["-af", "loudnorm=I=-20:TP=-2:LRA=11", "-ar", "44100",
                       "-c:a", "libvorbis", "-q:a", "5", dst], \
            "trim; loudnorm I=-20 TP=-2; 44.1kHz ogg q5 (too short to loop-blend)"
    if kind == "oneshot":
        return base + ["-af", "loudnorm=I=-14:TP=-3:LRA=11", "-ac", "1",
                       "-ar", "44100", "-c:a", "libvorbis", "-q:a", "5", dst], \
            "trim; peak-normalise -3 dBFS; mono; 44.1kHz ogg q5"
    # instrument: mono note material, sampler pitch-shifts +-3 semitones
    return base + ["-ac", "1", "-ar", "44100",
                   "-c:a", "libvorbis", "-q:a", "5", dst], \
        "trim; mono; 44.1kHz ogg q5"


def main():
    ap = argparse.ArgumentParser(description="ffmpeg an _incoming file into assets/sound/")
    ap.add_argument("incoming_id", help="the _incoming/<id>.ogg to process")
    ap.add_argument("--name", required=True, help="asset id, e.g. church-bell")
    ap.add_argument("--kind", required=True, choices=["bed", "oneshot", "instrument"])
    ap.add_argument("--license", default="CC0-1.0",
                    help="ledger license id (default CC0-1.0; 'unrestricted' needs --grant)")
    ap.add_argument("--grant", default="",
                    help="the grant's own words - required when --license is 'unrestricted'")
    ap.add_argument("--af", default="",
                    help="extra -af chain run BEFORE the kind recipe (e.g. a lowpass to "
                         "tame a crackly source)")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()
    if args.license not in ("CC0-1.0", "unrestricted", "own-synth"):
        die(f"license {args.license!r} is off the allowlist (CC0-1.0 | unrestricted | own-synth)")
    if args.license == "unrestricted" and not args.grant.strip():
        die("'unrestricted' needs --grant quoting the grant's own words")

    src = os.path.join(SOUND, "_incoming", f"{args.incoming_id}.ogg")
    if not os.path.isfile(src):
        die(f"no {src} - fetch it first with scripts/sound_fetch.py")
    sidecar_path = os.path.join(SOUND, "_incoming", f"{args.incoming_id}.json")
    sidecar = {}
    if os.path.isfile(sidecar_path):
        sidecar = json.load(open(sidecar_path))

    sub = {"bed": "beds", "oneshot": "oneshots", "instrument": "instruments"}[args.kind]
    rel = f"{sub}/{args.name}.ogg"
    dst = os.path.join(SOUND, rel)

    if args.dry_run:
        argv, note = recipe(args.kind, src, dst, None)
        print(" ".join(argv))
        print(f"# -> {rel}; ledger entry with sha256 of the output")
        return
    if not shutil_which("ffmpeg"):
        die("ffmpeg not found - on Windows run scripts/get-ffmpeg.ps1")
    if args.af:
        # pre-filter the source into a temp file - safe ahead of either
        # the -af or -filter_complex recipe paths
        pre = src + ".prefilter.ogg"
        r = subprocess.run(["ffmpeg", "-y", "-v", "error", "-i", src,
                            "-af", args.af, pre])
        if r.returncode != 0:
            die(f"prefilter ffmpeg failed ({r.returncode})")
        src, note_pre = pre, f"{args.af}; "
    else:
        note_pre = ""
    argv, note = recipe(args.kind, src, dst, probe_duration(src))
    note = note_pre + note
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    r = subprocess.run(argv)
    if r.returncode != 0:
        die(f"ffmpeg failed ({r.returncode})")

    sha = hashlib.sha256(open(dst, "rb").read()).hexdigest()
    entry = {
        "id": args.name,
        "path": rel,
        "source_url": sidecar.get("source_url", ""),
        "author": sidecar.get("author", ""),
        "license": args.license,
        "license_evidence_url": sidecar.get("license", ""),
        "grant_quote": args.grant,
        "sha256": sha,
        "processing": note,
    }
    ledger = json.load(open(LEDGER)) if os.path.isfile(LEDGER) else []
    ledger = [e for e in ledger if e.get("path") != rel]
    ledger.append(entry)
    with open(LEDGER, "w") as f:
        json.dump(ledger, f, indent=2)
        f.write("\n")
    # the web loader fetches exactly these (A2): one path per asset
    manifest = sorted(e["path"] for e in ledger)
    with open(os.path.join(SOUND, "manifest.json"), "w") as f:
        json.dump(manifest, f, indent=2)
        f.write("\n")
    print(f"{rel} ({os.path.getsize(dst)} bytes) ledgered as {args.name}")


def shutil_which(cmd):
    for d in os.environ.get("PATH", "").split(os.pathsep):
        if os.path.isfile(os.path.join(d, cmd)) or os.path.isfile(os.path.join(d, cmd + ".exe")):
            return True
    return False


if __name__ == "__main__":
    main()
