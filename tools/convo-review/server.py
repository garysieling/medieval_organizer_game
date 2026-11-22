#!/usr/bin/env python3
"""Conversation review: generate batches of conversations from the game's own
dialogue engine, flip through them in the browser, mark them up, and turn the
feedback into GitHub issues.

    python3 tools/convo-review/server.py [--port 8765]

Then open http://localhost:8765. Standard library only.

Sources:
  generated - dialogue_gen --json: synthetic characters (ideology, traits,
              status), every line tagged with its act and dialogue tree
  village   - talk_dump: a seeded village run forward N days; the chats,
              meetings and meals folk actually had

Everything you enter lives in tools/convo-review/data/ (git-ignored):
batches/<id>.json, feedback.json, tickets.json.
"""

import argparse
import glob
import json
import os
import re
import shutil
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse, parse_qs

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
DATA = BATCHES = FEEDBACK = TICKETS = None


def set_data(path):
    global DATA, BATCHES, FEEDBACK, TICKETS
    DATA = os.path.abspath(path)
    BATCHES = os.path.join(DATA, "batches")
    FEEDBACK = os.path.join(DATA, "feedback.json")
    TICKETS = os.path.join(DATA, "tickets.json")


set_data(os.path.join(HERE, "data"))
LOCK = threading.Lock()

# What each conversation-level tag means - shown in the UI and the tickets
TAGS = {
    "non-sequitur": "A reply doesn't follow from what was just said",
    "out-of-character": "A line doesn't fit the speaker's trade, traits, status or backstory",
    "wrong-address": "Someone is called the wrong thing (title, gender, name, relation)",
    "repetitive": "Lines or beats repeat, within the talk or across talks",
    "wrong-tone": "The tone doesn't fit the relation (rivals too warm, friends too cold...)",
    "register": "Anachronism, or the thee/thou register slips or overdoes it",
    "flat": "Nothing happens - filler, no stakes, no character",
    "ending": "The talk ends abruptly, or drags on past its natural end",
    "good": "A keeper - worth using as the example to aim for",
}

# Short names for ticket titles
TAG_TITLES = {
    "non-sequitur": "replies that don't follow", "out-of-character": "lines out of character",
    "wrong-address": "folk called the wrong thing", "repetitive": "repetition",
    "wrong-tone": "tone wrong for the relation", "register": "register and anachronism slips",
    "flat": "flat talks where nothing happens", "ending": "endings too abrupt or too long",
}

# --- storage -------------------------------------------------------------

def load(path, default):
    try:
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        return default


def save(path, value):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    tmp = path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(value, f, indent=1, ensure_ascii=False)
    os.replace(tmp, path)


def batch_path(bid):
    if not re.fullmatch(r"[A-Za-z0-9_-]+", bid):
        raise ValueError("bad batch id")
    return os.path.join(BATCHES, bid + ".json")


def list_batches():
    fb = load(FEEDBACK, {})
    out = []
    for p in sorted(glob.glob(os.path.join(BATCHES, "*.json")), reverse=True):
        b = load(p, None)
        if not b:
            continue
        ids = [c["id"] for c in b["conversations"]]
        out.append({**b["meta"], "count": len(ids),
                    "reviewed": sum(1 for i in ids if reviewed(fb.get(i)))})
    return out


def reviewed(f):
    return bool(f) and bool(f.get("rating") or f.get("tags") or f.get("note")
                            or f.get("lines") or f.get("skip"))


# --- generation ----------------------------------------------------------

def find_cargo():
    if os.environ.get("CARGO"):
        return os.environ["CARGO"]
    c = shutil.which("cargo")
    if c:
        return c
    # WSL with only the Windows toolchain installed
    hits = glob.glob("/mnt/c/Users/*/.cargo/bin/cargo.exe")
    if hits:
        return hits[0]
    raise RuntimeError("cargo not found - set CARGO=/path/to/cargo")


def run_bin(name, args):
    cargo = find_cargo()
    build = subprocess.run(
        [cargo, "build", "-q", "-p", "game2-core", "--bin", name],
        cwd=ROOT, capture_output=True, text=True)
    if build.returncode != 0:
        raise RuntimeError("build failed:\n" + build.stderr[-4000:])
    exe = os.path.join(ROOT, "target", "debug", name)
    if cargo.endswith(".exe") or os.name == "nt":
        exe += ".exe"
    r = subprocess.run([exe] + args, cwd=ROOT, capture_output=True, text=True, encoding="utf-8")
    if r.returncode != 0:
        raise RuntimeError(f"{name} failed:\n" + r.stderr[-4000:])
    return json.loads(r.stdout)


def generate(source, count, seed, days):
    count = max(1, min(int(count), 500))
    seed = int(seed)
    if source == "generated":
        chars = max(2, min(40, count))
        per = -(-count // chars)
        convs = run_bin("dialogue_gen", ["--json", "-n", str(chars), "-m", str(per), "-s", str(seed)])
    elif source == "village":
        convs = run_bin("talk_dump", ["--seed", str(seed), "--days", str(int(days)), "--max", str(count)])
    else:
        raise ValueError("unknown source " + source)
    convs = convs[:count]
    bid = time.strftime("%Y%m%d-%H%M%S") + f"-{source}-s{seed}"
    meta = {"id": bid, "source": source, "seed": seed, "days": int(days) if source == "village" else None,
            "created": time.strftime("%Y-%m-%d %H:%M")}
    save(batch_path(bid), {"meta": meta, "conversations": convs})
    return {**meta, "count": len(convs), "reviewed": 0}


# --- tickets -------------------------------------------------------------

_SOURCES = None


def source_files():
    global _SOURCES
    if _SOURCES is None:
        _SOURCES = []
        for p in sorted(glob.glob(os.path.join(ROOT, "game2-core", "src", "**", "*.rs"), recursive=True)):
            try:
                with open(p, encoding="utf-8") as f:
                    _SOURCES.append((os.path.relpath(p, ROOT).replace(os.sep, "/"), f.read().split("\n")))
            except OSError:
                pass
    return _SOURCES


def locate(text, names):
    """Where a spoken line is written in the source. Lines are templates with
    names filled in, so look for the longest stretch between the names."""
    t = text
    for n in sorted((n for n in names if n), key=len, reverse=True):
        t = t.replace(n, "\x00")
    chunks = sorted((c.strip(" ,.;:!?-*\"'") for c in t.split("\x00")), key=len, reverse=True)
    needle = next((c for c in chunks if len(c) >= 12), None)
    if not needle:
        return None
    for path, lines in source_files():
        for i, line in enumerate(lines):
            if needle in line:
                return f"{path}:{i + 1}", enclosing(lines, i)
    return None


def enclosing(lines, i):
    """The named block a source line sits in (`const SMALL_GREET: Exchange`)"""
    for j in range(i, -1, -1):
        m = re.match(r"\s*(?:pub\s+)?(?:const|static|fn)\s+(\w+)", lines[j])
        if m:
            return m.group(1)
    return None


def plural(n, word):
    return f"{n} {word}{'' if n == 1 else 's'}"


def transcript(conv, marks=None, edits=None):
    marks, edits = marks or {}, edits or {}
    out = []
    for i, l in enumerate(conv["lines"]):
        who = l.get("speaker") or "(narration)"
        tree = f" [{l['act']} · {l['tree']}]" if l.get("tree") else ""
        flag = " ⚑" if str(i) in marks else ""
        out.append(f"{i:>2} {who}{tree}: {l['text']}{flag}")
        if str(i) in edits:
            out.append(f"   → {edits[str(i)]}")
    return "\n".join(out)


def conv_header(conv, f):
    names = ", ".join(s["name"] for s in conv.get("speakers", []))
    bits = [f"`{conv['id']}`", conv.get("source", ""), conv.get("kind", "")]
    if conv.get("relation"):
        bits.append(conv["relation"])
    if conv.get("day") is not None:
        bits.append(f"day {conv['day']}")
    if f.get("rating"):
        bits.append(f"rated {f['rating']}/5")
    return f"**{names}** - " + " · ".join(b for b in bits if b)


def speaker_notes(conv):
    rows = []
    for s in conv.get("speakers", []):
        if s.get("backstory"):
            extra = ", ".join(s.get("traits", []) + s.get("status", []))
            rows.append(f"- {s['name']} ({s.get('ideology', '')}{'; ' + extra if extra else ''}): {s['backstory']}")
    return "\n".join(rows)


MAX_EXAMPLES = 15


def drafts():
    """Group the feedback not yet ticketed into draft issues: one per
    conversation tag, one per dialogue tree that has rewrites, one for the
    rest of the low-rated notes."""
    fb = load(FEEDBACK, {})
    convs = {}
    for p in glob.glob(os.path.join(BATCHES, "*.json")):
        for c in load(p, {"conversations": []})["conversations"]:
            convs[c["id"]] = c
    by_tag, by_tree, loose, keepers = {}, {}, [], []
    for cid, f in fb.items():
        c = convs.get(cid)
        if not c or f.get("ticketed"):
            continue
        tags = [t for t in f.get("tags", []) if t != "good"]
        if "good" in f.get("tags", []) or (f.get("rating") or 0) >= 5:
            keepers.append((c, f))
        for t in tags:
            by_tag.setdefault(t, []).append((c, f))
        lines = f.get("lines", {})
        names = [s["name"] for s in c.get("speakers", [])]
        for idx, lf in lines.items():
            i = int(idx)
            if i >= len(c["lines"]):
                continue
            line = c["lines"][i]
            if not (lf.get("edit") or lf.get("comment")):
                continue
            found = locate(line["text"], names)
            # village lines don't carry their tree: group by the source block
            tree = line.get("tree") or (found and found[1]) or "unplaced lines"
            by_tree.setdefault(tree, []).append({
                "conv": c, "f": f, "i": i, "line": line, "edit": lf.get("edit"),
                "comment": lf.get("comment"), "where": found and found[0],
                "before": c["lines"][i - 1] if i > 0 else None})
        if not tags and not lines and f.get("note") and (f.get("rating") or 3) <= 3:
            loose.append((c, f))

    out = []
    for tag, items in sorted(by_tag.items(), key=lambda kv: -len(kv[1])):
        body = [f"From the conversation review tool: **{len(items)}** conversation(s) tagged "
                f"`{tag}` - {TAGS.get(tag, tag)}.", "", "## Examples", ""]
        for c, f in items[:MAX_EXAMPLES]:
            body.append(conv_header(c, f))
            if f.get("note"):
                body.append(f"> {f['note']}")
            sn = speaker_notes(c)
            if sn:
                body.append(sn)
            marks = {k: v for k, v in f.get("lines", {}).items() if v.get("comment") or v.get("edit")}
            edits = {k: v["edit"] for k, v in marks.items() if v.get("edit")}
            body += ["```text", transcript(c, marks, edits), "```", ""]
            for k, v in marks.items():
                if v.get("comment"):
                    body.append(f"- line {k}: {v['comment']}")
            body.append("")
        if len(items) > MAX_EXAMPLES:
            body.append(f"...and {len(items) - MAX_EXAMPLES} more in `tools/convo-review/data`.")
        body += ["", "## Done when", f"- The generator no longer produces `{tag}` conversations like these "
                 "(re-run the review tool on a fresh batch to check)."]
        out.append({"key": "tag:" + tag, "title": f"Conversations: {TAG_TITLES.get(tag, tag)} ({len(items)} example{'s' if len(items) != 1 else ''})",
                    "body": "\n".join(body), "labels": ["enhancement", "conversation-review"],
                    "conv_ids": sorted({c["id"] for c, _ in items})})
    for tree, items in sorted(by_tree.items(), key=lambda kv: -len(kv[1])):
        body = [f"From the conversation review tool: **{len(items)}** line(s) to rework in `{tree}`.", ""]
        for it in items[:40]:
            ln = it["line"]
            body.append(f"### {it['conv']['id']} line {it['i']} ({ln.get('speaker') or 'narration'}"
                        f"{', ' + ln['act'] if ln.get('act') else ''})")
            if it["where"]:
                body.append(f"Source: `{it['where']}`")
            if it["before"]:
                body.append(f"After: {it['before'].get('speaker', '')}: \"{it['before']['text']}\"")
            body.append(f"- Was: \"{ln['text']}\"")
            if it["edit"]:
                body.append(f"- Suggested: \"{it['edit']}\"")
            if it["comment"]:
                body.append(f"- Why: {it['comment']}")
            body.append("")
        if len(items) > 40:
            body.append(f"...and {len(items) - 40} more in `tools/convo-review/data`.")
        body += ["## Done when", "- Each line is rewritten as suggested (or better) at its source; "
                 "lines filled from templates keep their placeholders."]
        out.append({"key": "tree:" + tree, "title": f"Dialogue rewrites: {tree} ({plural(len(items), 'line')})",
                    "body": "\n".join(body), "labels": ["enhancement", "conversation-review"],
                    "conv_ids": sorted({it["conv"]["id"] for it in items})})
    if loose:
        body = [f"From the conversation review tool: **{len(loose)}** low-rated conversation(s) with notes.", ""]
        for c, f in loose[:MAX_EXAMPLES]:
            body += [conv_header(c, f), f"> {f['note']}", "```text", transcript(c), "```", ""]
        out.append({"key": "notes", "title": f"Conversations: reviewer notes ({len(loose)})",
                    "body": "\n".join(body), "labels": ["enhancement", "conversation-review"],
                    "conv_ids": sorted({c["id"] for c, _ in loose})})
    return {"drafts": out, "keepers": len(keepers), "created": load(TICKETS, [])}


def gh_ok():
    return shutil.which("gh") is not None


def create_tickets(chosen):
    if not gh_ok():
        raise RuntimeError("the gh CLI isn't on PATH")
    # the label, made once; an existing one is fine
    subprocess.run(["gh", "label", "create", "conversation-review", "--color", "c5def5",
                    "--description", "Filed from the conversation review tool"],
                   cwd=ROOT, capture_output=True, text=True)
    made = []
    for d in chosen:
        args = ["gh", "issue", "create", "--title", d["title"], "--body-file", "-"]
        for lab in d.get("labels", []):
            args += ["--label", lab]
        r = subprocess.run(args, cwd=ROOT, input=d["body"], capture_output=True, text=True)
        if r.returncode != 0:
            made.append({"title": d["title"], "error": r.stderr.strip()[-500:]})
            continue
        url = r.stdout.strip().splitlines()[-1]
        made.append({"title": d["title"], "url": url})
        with LOCK:
            fb = load(FEEDBACK, {})
            for cid in d.get("conv_ids", []):
                if cid in fb:
                    fb[cid].setdefault("ticketed", []).append(url)
            save(FEEDBACK, fb)
            log = load(TICKETS, [])
            log.append({"title": d["title"], "url": url, "created": time.strftime("%Y-%m-%d %H:%M"),
                        "conv_ids": d.get("conv_ids", [])})
            save(TICKETS, log)
    return made


# --- http ----------------------------------------------------------------

class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        sys.stderr.write("%s %s\n" % (self.command, self.path))

    def send(self, code, body, ctype="application/json"):
        data = body if isinstance(body, bytes) else json.dumps(body, ensure_ascii=False).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", ctype + ("; charset=utf-8" if ctype.startswith(("text", "application/json")) else ""))
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(data)

    def body(self):
        n = int(self.headers.get("Content-Length") or 0)
        return json.loads(self.rfile.read(n) or b"{}")

    def guard(self, fn):
        try:
            fn()
        except Exception as e:  # report to the page rather than drop the socket
            self.send(500, {"error": str(e)})

    def do_GET(self):
        self.guard(self._get)

    def do_POST(self):
        self.guard(self._post)

    def do_PUT(self):
        self.guard(self._put)

    def _get(self):
        u = urlparse(self.path)
        p = u.path
        if p in ("/", "/index.html"):
            with open(os.path.join(HERE, "index.html"), "rb") as f:
                return self.send(200, f.read(), "text/html")
        if p == "/api/meta":
            return self.send(200, {"tags": TAGS, "gh": gh_ok()})
        if p == "/api/batches":
            return self.send(200, list_batches())
        m = re.fullmatch(r"/api/batches/([A-Za-z0-9_-]+)", p)
        if m:
            b = load(batch_path(m.group(1)), None)
            if not b:
                return self.send(404, {"error": "no such batch"})
            fb = load(FEEDBACK, {})
            b["feedback"] = {c["id"]: fb[c["id"]] for c in b["conversations"] if c["id"] in fb}
            return self.send(200, b)
        if p == "/api/drafts":
            return self.send(200, drafts())
        self.send(404, {"error": "not found"})

    def _post(self):
        p = urlparse(self.path).path
        if p == "/api/batches":
            b = self.body()
            return self.send(200, generate(b.get("source", "generated"), b.get("count", 100),
                                           b.get("seed", 1), b.get("days", 20)))
        if p == "/api/tickets":
            return self.send(200, {"made": create_tickets(self.body().get("drafts", []))})
        self.send(404, {"error": "not found"})

    def _put(self):
        m = re.fullmatch(r"/api/feedback/(.+)", urlparse(self.path).path)
        if not m:
            return self.send(404, {"error": "not found"})
        cid = m.group(1)
        f = self.body()
        with LOCK:
            fb = load(FEEDBACK, {})
            old = fb.get(cid, {})
            f["ticketed"] = old.get("ticketed", [])
            f["updated"] = time.strftime("%Y-%m-%d %H:%M:%S")
            fb[cid] = f
            save(FEEDBACK, fb)
        self.send(200, {"ok": True})


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--port", type=int, default=8765)
    ap.add_argument("--host", default="127.0.0.1")
    ap.add_argument("--data", default=os.path.join(HERE, "data"), help="where batches and feedback live")
    a = ap.parse_args()
    set_data(a.data)
    os.makedirs(BATCHES, exist_ok=True)
    print(f"Conversation review on http://localhost:{a.port}  (data in {DATA})")
    ThreadingHTTPServer((a.host, a.port), Handler).serve_forever()


if __name__ == "__main__":
    main()
