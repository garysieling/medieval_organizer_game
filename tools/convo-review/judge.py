#!/usr/bin/env python3
"""The LLM judge for the conversation harness (E0-6, #1415): a local,
offline model compares two transcripts side by side and says which does
better on the three dimensions code can't check - voice, flatness,
implication. Nothing else is scored.

    python3 tools/convo-review/judge.py keepers
    python3 tools/convo-review/judge.py pair <conv-id> <conv-id> [--seed N]
    python3 tools/convo-review/judge.py agreement [--seed N] [--pairs N] [--both]

The protocol is the design's anti-bias shape (procedural-characters.md,
Appendix A): pairwise, order randomized per pair, keeper transcripts
anchoring the rubric, model pinned in judge.config.json so scores stay
comparable over months.

Signals: convo-review marks live in data/feedback.json - a 'good' tag or
rating >= 4 proposes a keeper; negative tags map to the dims they judge:
  out-of-character, wrong-tone -> voice
  flat, repetitive             -> flatness
  (implication has no user tag - reported separately, per the grill)
Structural tags (non-sequitur, ending, wrong-address, register) have
machine checks and stay out of the judge's remit.

'keep it offline': standard library only, one HTTP POST to a local
Ollama endpoint - no key, no network beyond localhost.
"""

import argparse
import glob
import json
import os
import random
import re
import sys
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
CONFIG = os.path.join(HERE, "judge.config.json")
DATA = BATCHES = FEEDBACK = KEEPERS = PROPOSED = None


def set_data(path):
    """Point the tool at a review-data dir (server.py's --data; the
    batches/, feedback.json and keepers.json all live there)."""
    global DATA, BATCHES, FEEDBACK, KEEPERS, PROPOSED
    DATA = os.path.abspath(path)
    BATCHES = os.path.join(DATA, "batches")
    FEEDBACK = os.path.join(DATA, "feedback.json")
    KEEPERS = os.path.join(DATA, "keepers.json")
    PROPOSED = os.path.join(DATA, "keepers.proposed.json")


set_data(os.path.join(HERE, "data"))

# The three dims, in prompt order - the judge sees nothing else
DIMS = ["voice", "flat", "implication"]

# Which review tags count as a negative user signal on each dim (grill,
# #1415: out-of-character -> voice, flat -> flatness; implication has no
# user tag and is reported separately). 'repetitive' rides flatness -
# lines repeating are the flat the reviewer actually marks.
NEGATIVE_TAGS = {
    "voice": {"out-of-character", "wrong-tone"},
    "flat": {"flat", "repetitive"},
    "implication": set(),
}
# A transcript with one of these is clean on no dim's behalf - kept out
# of the 'clean side' pool for pairs on any dim
ANY_NEGATIVE = set().union(*NEGATIVE_TAGS.values())
# A 'good' tag or this rating makes a keeper candidate / a preferred
# clean partner
GOOD_RATING = 4

DIM_LABELS = {
    "voice": "voice fit - the lines sound like the person who says them "
             "(their trade, traits, station, history)",
    "flat": "life - something happens: stakes, character, a thing worth "
            "saying, not filler",
    "implication": "implication - lines mean more than they say: a tell, "
                   "a hedge, a hint a reader can draw the fact from",
}


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


def config():
    cfg = load(CONFIG, {})
    return {
        "model": cfg.get("model", "gemma3:4b"),
        "endpoint": cfg.get("endpoint", "http://localhost:11434"),
        "temperature": cfg.get("temperature", 0),
    }


def conversations():
    """Every conversation in every batch, indexed by id."""
    out = {}
    for p in sorted(glob.glob(os.path.join(BATCHES, "*.json"))):
        b = load(p, None)
        if not b:
            continue
        for c in b.get("conversations", []):
            out[c["id"]] = c
    return out


def transcript_text(conv, max_lines=40):
    """The talk as 'Name: line' rows, capped so the prompt stays small."""
    rows = []
    for ln in conv.get("lines", [])[:max_lines]:
        sp = ln.get("speaker")
        rows.append((sp + ": " if sp else "") + ln.get("text", ""))
    if len(conv.get("lines", [])) > max_lines:
        rows.append("[...]")
    return "\n".join(rows)


def speaker_blurb(conv):
    """Who's talking, when the batch carries persona notes."""
    notes = []
    for s in conv.get("speakers", []):
        parts = [s.get("name", "?")]
        for k in ("ideology", "traits", "status", "vocation"):
            if s.get(k):
                parts.append(str(s[k]))
        notes.append(" ".join(parts))
    return "; ".join(notes)


def negative_dims(feedback_entry):
    """The dims the reviewer marked this transcript down on."""
    tags = set((feedback_entry or {}).get("tags") or [])
    return {d for d, ts in NEGATIVE_TAGS.items() if tags & ts}


def is_good(feedback_entry):
    f = feedback_entry or {}
    return (f.get("rating") or 0) >= GOOD_RATING or "good" in (f.get("tags") or [])


def propose_keepers():
    """Keeper candidates: what the reviewer called good (grill: hybrid -
    proposed here, the user approves into keepers.json)."""
    fb = load(FEEDBACK, {})
    out = sorted(cid for cid, f in fb.items()
                 if is_good(f) and not f.get("skip"))
    return out


def load_keepers(convs):
    """Approved anchor ids -> transcript texts, three at most."""
    ids = load(KEEPERS, [])
    return [transcript_text(convs[i]) for i in ids if i in convs][:3]


def build_prompt(conv_a, conv_b, anchors, flip):
    """The side-by-side prompt. `flip` swaps which transcript shows as
    Conversation 1 - the position-bias counter. Returns (prompt, slot_of_a)
    where slot_of_a is the label '1' or '2' conv_a showed under."""
    first, second = (conv_b, conv_a) if flip else (conv_a, conv_b)
    parts = [
        "You are judging two conversations from a medieval village game.",
        "Compare them on ONLY these three dimensions - nothing else:",
        "1. " + DIM_LABELS["voice"],
        "2. " + DIM_LABELS["flat"],
        "3. " + DIM_LABELS["implication"],
        "",
    ]
    if anchors:
        parts += ["Conversations the reviewer keeps as the standard:"]
        for i, a in enumerate(anchors, 1):
            parts += [f"--- keeper {i} ---", a, ""]
    sa = speaker_blurb(first)
    sb = speaker_blurb(second)
    parts += [
        "--- conversation 1" + (f" ({sa})" if sa else "") + " ---",
        transcript_text(first),
        "",
        "--- conversation 2" + (f" ({sb})" if sb else "") + " ---",
        transcript_text(second),
        "",
        'For each dimension, which conversation does better - "1", "2" '
        'or "tie"? Answer with only this JSON:',
        '{"voice": "...", "flat": "...", "implication": "..."}',
    ]
    return "\n".join(parts), ("2" if flip else "1")


def parse_verdict(text):
    """Pull {dim: '1'|'2'|'tie'} out of the model's reply. The model is
    told to answer JSON only but may wrap it in prose - take the first
    {...} block."""
    m = re.search(r"\{[^{}]*\}", text or "", re.S)
    if not m:
        raise ValueError("no JSON in judge reply: " + (text or "")[:200])
    obj = json.loads(m.group(0))
    out = {}
    for d in DIMS:
        v = str(obj.get(d, "")).strip().lower()
        if v in ("1", "one", "first", "a"):
            out[d] = "1"
        elif v in ("2", "two", "second", "b"):
            out[d] = "2"
        elif v in ("tie", "equal", "same", "0"):
            out[d] = "tie"
        else:
            raise ValueError(f"bad {d} verdict {v!r}")
    return out


def ollama_query(prompt, cfg):
    """One offline generation against the local endpoint."""
    body = json.dumps({
        "model": cfg["model"], "prompt": prompt, "stream": False,
        "options": {"temperature": cfg["temperature"]},
    }).encode()
    req = urllib.request.Request(
        cfg["endpoint"].rstrip("/") + "/api/generate", data=body,
        headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=600) as r:
        return json.loads(r.read())["response"]


def judge_pair(conv_a, conv_b, anchors, cfg, rng, query=ollama_query):
    """Judge one pair, order randomized. Returns {dim: winner} where the
    winner is 'a', 'b' or 'tie' in caller terms, not prompt slots."""
    flip = rng.random() < 0.5
    prompt, slot_of_a = build_prompt(conv_a, conv_b, anchors, flip)
    raw = query(prompt, cfg)
    slots = parse_verdict(raw)
    other = "2" if slot_of_a == "1" else "1"
    return {d: ("a" if v == slot_of_a else "b" if v == other else "tie")
            for d, v in slots.items()}


def agreement_pairs(feedback, convs):
    """(dim, worse_id, clean_id) triples for the agreement run: each
    transcript the reviewer marked down on a dim, paired with a reviewed
    transcript clean on every judged dim - 'good' ones preferred."""
    reviewed = {cid: f for cid, f in feedback.items()
                if cid in convs and not f.get("skip")
                and (f.get("tags") or f.get("rating") or f.get("note"))}
    goods = sorted(cid for cid, f in reviewed.items() if is_good(f))
    cleans = sorted(cid for cid, f in reviewed.items()
                    if not (set(f.get("tags") or []) & ANY_NEGATIVE))
    pool = goods + [c for c in cleans if c not in goods]
    out = []
    for d in DIMS:
        for cid in sorted(reviewed):
            if d not in negative_dims(reviewed[cid]):
                continue
            for partner in pool:
                if partner != cid:
                    out.append((d, cid, partner))
                    break
    return out


def agreement_report(pairs, convs, anchors, cfg, rng, query=ollama_query):
    """Run the pairs, count where the judge sides with the reviewer's
    signal. A tie counts as half agreement."""
    rows = {d: {"agree": 0.0, "n": 0, "ties": 0, "disagree": 0}
            for d in DIMS}
    for d, worse, clean in pairs:
        verdict = judge_pair(convs[worse], convs[clean], anchors, cfg, rng, query)
        v = verdict[d]
        rows[d]["n"] += 1
        if v == "a":
            rows[d]["disagree"] += 1
        elif v == "tie":
            rows[d]["agree"] += 0.5
            rows[d]["ties"] += 1
        else:
            rows[d]["agree"] += 1
    return rows


def print_report(rows):
    total_a, total_n = 0.0, 0
    print("judge agreement (pairwise, order randomized per pair):")
    for d in DIMS:
        r = rows[d]
        total_a += r["agree"]
        total_n += r["n"]
        if r["n"] == 0:
            print(f"  {d:<11} no user signal on this dim - reported separately")
            continue
        pct = 100.0 * r["agree"] / r["n"]
        print(f"  {d:<11} {r['agree']:g}/{r['n']} agreed ({pct:.0f}%)"
              f"  [{r['ties']} tie, {r['disagree']} against]")
    if total_n:
        print(f"  overall     {total_a:g}/{total_n} agreed"
              f" ({100.0 * total_a / total_n:.0f}%)")
    print("threshold: set after this first real run - trust the judge "
          "only above a level the reviewer sets (grill, #1415)")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("mode", choices=["keepers", "pair", "agreement"])
    ap.add_argument("ids", nargs="*", help="pair mode: two conversation ids")
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--pairs", type=int, default=0,
                    help="agreement mode: cap the judged pairs (0 = all)")
    ap.add_argument("--both", action="store_true",
                    help="judge each pair in both orders (position check)")
    ap.add_argument("--data", default=None,
                    help="review-data dir (default: tools/convo-review/data)")
    args = ap.parse_args(argv)
    if args.data:
        set_data(args.data)

    convs = conversations()
    fb = load(FEEDBACK, {})
    cfg = config()
    rng = random.Random(args.seed)

    if args.mode == "keepers":
        cands = propose_keepers()
        save(PROPOSED, cands)
        print(f"{len(cands)} keeper candidates -> {PROPOSED}")
        print("approve them into data/keepers.json to anchor the judge")
        for c in cands:
            print("  " + c)
        return

    anchors = load_keepers(convs)

    if args.mode == "pair":
        if len(args.ids) != 2:
            ap.error("pair mode takes two conversation ids")
        a, b = (convs.get(i) for i in args.ids)
        if a is None or b is None:
            missing = [i for i, c in zip(args.ids, (a, b)) if c is None]
            sys.exit("unknown conversation id(s): " + ", ".join(missing))
        v = judge_pair(a, b, anchors, cfg, rng)
        for d in DIMS:
            win = {"a": args.ids[0], "b": args.ids[1], "tie": "tie"}[v[d]]
            print(f"  {d:<11} {win}")
        return

    pairs = agreement_pairs(fb, convs)
    if args.pairs:
        rng.shuffle(pairs)
        pairs = pairs[:args.pairs]
    print(f"{len(pairs)} agreement pairs (model {cfg['model']}, "
          f"{len(anchors)} keepers)")
    rows = agreement_report(pairs, convs, anchors, cfg, rng)
    print_report(rows)
    if args.both:
        print("\nposition check - same pairs, order forced both ways:")
        steady = 0
        for d, worse, clean in pairs:
            va, vb = 0, 0
            for flip in (False, True):
                prompt, slot_of_a = build_prompt(
                    convs[worse], convs[clean], anchors, flip)
                v = parse_verdict(ollama_query(prompt, cfg))[d]
                # slot verdict -> caller terms ('a' = worse won)
                w = ("a" if v == slot_of_a
                     else "tie" if v == "tie" else "b")
                if flip:
                    vb = w
                else:
                    va = w
            if va == vb:
                steady += 1
        print(f"  {steady}/{len(pairs)} verdicts stable across both orders")


if __name__ == "__main__":
    main()
