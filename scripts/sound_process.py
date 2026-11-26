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
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.join(os.path.dirname(__file__), "..")
SOUND = os.path.join(ROOT, "assets", "sound")
LEDGER = os.path.join(SOUND, "LEDGER.json")

# Strictly CC0 (not "public domain" generally - that still needs a --grant
# quote), matched against a fetcher sidecar's own "license_claim" field.
CC0_CLAIM = re.compile(r"cc0|cc.?zero", re.I)


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


def oneshot_steps(src, dst):
    return [(["ffmpeg", "-y", "-v", "error", "-i", src,
              "-af", "loudnorm=I=-14:TP=-3:LRA=11", "-ac", "1",
              "-ar", "44100", "-c:a", "libvorbis", "-q:a", "5", dst])], \
        "trim; peak-normalise -3 dBFS; mono; 44.1kHz ogg q5"


def instrument_steps(src, dst):
    return [(["ffmpeg", "-y", "-v", "error", "-i", src,
              "-ac", "1", "-ar", "44100", "-c:a", "libvorbis", "-q:a", "5", dst])], \
        "trim; mono; 44.1kHz ogg q5"


def bed_steps(src, dst, dur, tmp):
    """The bed pipeline: loudnorm to -20 LUFS, then (if long enough) shift
    the loop point by `fade` seconds and crossfade the true tail into the
    true head there, so the seam sits mid-file and the end-to-start wrap
    is continuous (fade = min(2s, dur/4)). Crossfading the end of the file
    against itself, as an earlier version of this recipe did, never
    touches that join at all.

    Each stage is its own ffmpeg process rather than one -filter_complex
    graph: feeding several atrim branches of the same input into an
    acrossfade followed by a concat, all in one graph, made ffmpeg's
    filter scheduler emit frames out of order and truncate the output
    (reproducible: "Non-monotonic DTS" warnings and a short file).

    Returns (list of argv, note). The last step's output is `dst`."""
    norm = os.path.join(tmp, "norm.ogg")
    fade = 2.0 if dur is None else min(2.0, dur / 4.0)
    norm_step = ["ffmpeg", "-y", "-v", "error", "-i", src,
                 "-af", "loudnorm=I=-20:TP=-2:LRA=11", "-ar", "44100",
                 "-c:a", "libvorbis", "-q:a", "5"]
    if dur is None or dur <= fade * 2:
        return [norm_step + [dst]], \
            "trim; loudnorm I=-20 TP=-2; 44.1kHz ogg q5 (too short to loop-blend)"

    mid_end = dur - fade
    head, tail, mid, seam = (os.path.join(tmp, n) for n in
                              ("head.ogg", "tail.ogg", "mid.ogg", "seam.ogg"))
    steps = [
        norm_step + [norm],
        ["ffmpeg", "-y", "-v", "error", "-i", norm, "-af",
         f"atrim=0:{fade:.3f},asetpts=PTS-STARTPTS", "-c:a", "libvorbis", "-q:a", "5", head],
        ["ffmpeg", "-y", "-v", "error", "-i", norm, "-af",
         f"atrim={mid_end:.3f}:{dur:.3f},asetpts=PTS-STARTPTS", "-c:a", "libvorbis", "-q:a", "5", tail],
        ["ffmpeg", "-y", "-v", "error", "-i", norm, "-af",
         f"atrim={fade:.3f}:{mid_end:.3f},asetpts=PTS-STARTPTS", "-c:a", "libvorbis", "-q:a", "5", mid],
        ["ffmpeg", "-y", "-v", "error", "-i", tail, "-i", head, "-filter_complex",
         f"[0:a][1:a]acrossfade=d={fade:.3f}", "-c:a", "libvorbis", "-q:a", "5", seam],
        ["ffmpeg", "-y", "-v", "error", "-i", mid, "-i", seam, "-filter_complex",
         "[0:a][1:a]concat=n=2:v=0:a=1", "-ar", "44100", "-c:a", "libvorbis", "-q:a", "5", dst],
    ]
    note = (f"trim; loudnorm I=-20 TP=-2; 44.1kHz ogg q5; loop point shifted "
            f"by {fade:.1f}s with tail-into-head crossfade")
    return steps, note


def recipe_steps(kind, src, dst, dur, tmp):
    if kind == "bed":
        return bed_steps(src, dst, dur, tmp)
    if kind == "oneshot":
        return oneshot_steps(src, dst)
    return instrument_steps(src, dst)  # instrument: mono note material


def resolve_license(args, sidecar):
    """Infer --license from the _incoming sidecar's own "license_claim"
    (written by fetchers like sound_fetch_commons.py) when --license
    wasn't given explicitly, reject it if it conflicts with one that was,
    and auto-fill --grant with the claim's own words for a non-CC0 import
    - the legacy default (CC0-1.0) is unchanged for sidecars, or fetchers,
    that carry no such claim."""
    claim = sidecar.get("license_claim", "")
    inferred = None
    if claim:
        inferred = "CC0-1.0" if CC0_CLAIM.search(claim) else "unrestricted"
    if args.license is None:
        args.license = inferred or "CC0-1.0"
    elif inferred is not None and args.license != inferred:
        die(f"--license {args.license!r} conflicts with the _incoming sidecar's "
            f"license_claim {claim!r} (expected {inferred!r})")
    if args.license == "unrestricted" and not args.grant.strip() and claim:
        args.grant = claim


def main():
    ap = argparse.ArgumentParser(description="ffmpeg an _incoming file into assets/sound/")
    ap.add_argument("incoming_id", help="the _incoming/<id>.ogg to process")
    ap.add_argument("--name", required=True, help="asset id, e.g. church-bell")
    ap.add_argument("--kind", required=True, choices=["bed", "oneshot", "instrument"])
    ap.add_argument("--license", default=None,
                    help="ledger license id (default: inferred from the _incoming sidecar's "
                         "license_claim, else CC0-1.0; 'unrestricted' needs --grant)")
    ap.add_argument("--grant", default="",
                    help="the grant's own words - required when --license is 'unrestricted' "
                         "(auto-filled from the sidecar's license_claim when inferred)")
    ap.add_argument("--af", default="",
                    help="extra -af chain run BEFORE the kind recipe (e.g. a lowpass to "
                         "tame a crackly source)")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    src = os.path.join(SOUND, "_incoming", f"{args.incoming_id}.ogg")
    if not os.path.isfile(src) and not args.dry_run:
        die(f"no {src} - fetch it first with scripts/sound_fetch.py")
    sidecar_path = os.path.join(SOUND, "_incoming", f"{args.incoming_id}.json")
    sidecar = {}
    if os.path.isfile(sidecar_path):
        sidecar = json.load(open(sidecar_path))

    resolve_license(args, sidecar)
    if args.license not in ("CC0-1.0", "unrestricted", "own-synth"):
        die(f"license {args.license!r} is off the allowlist (CC0-1.0 | unrestricted | own-synth)")
    if args.license == "unrestricted" and not args.grant.strip():
        die("'unrestricted' needs --grant quoting the grant's own words")

    sub = {"bed": "beds", "oneshot": "oneshots", "instrument": "instruments"}[args.kind]
    rel = f"{sub}/{args.name}.ogg"
    dst = os.path.join(SOUND, rel)

    with tempfile.TemporaryDirectory() as tmp:
        pre_steps = []
        note_pre = ""
        effective_src = src
        if args.af:
            # pre-filter the source into a temp file before the kind recipe
            pre = os.path.join(tmp, "prefilter.ogg")
            pre_steps = [["ffmpeg", "-y", "-v", "error", "-i", src, "-af", args.af, pre]]
            effective_src = pre
            note_pre = f"{args.af}; "

        if args.dry_run:
            for argv in pre_steps:
                print(" ".join(argv))
            steps, note = recipe_steps(args.kind, effective_src, dst,
                                        probe_duration(src) if os.path.isfile(src) else None, tmp)
            for argv in steps:
                print(" ".join(argv))
            print(f"# -> {rel}; ledger entry with sha256 of the output")
            return

        if not shutil_which("ffmpeg"):
            die("ffmpeg not found - on Windows run scripts/get-ffmpeg.ps1")
        for argv in pre_steps:
            r = subprocess.run(argv)
            if r.returncode != 0:
                die(f"prefilter ffmpeg failed ({r.returncode})")

        steps, note = recipe_steps(args.kind, effective_src, dst, probe_duration(effective_src), tmp)
        note = note_pre + note
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        for argv in steps:
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
    return shutil.which(cmd) is not None


if __name__ == "__main__":
    main()
