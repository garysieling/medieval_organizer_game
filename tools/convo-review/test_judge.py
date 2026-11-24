#!/usr/bin/env python3
"""Unit tests for judge.py - the Ollama call is stubbed throughout, so
these run with no model and no server:

    python3 tools/convo-review/test_judge.py
"""

import json
import os
import random
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import judge


def conv(cid, lines, speakers=None):
    return {"id": cid, "kind": "Chat",
            "lines": [{"speaker": s, "text": t} for s, t in lines],
            "speakers": speakers or []}


CONV_A = conv("a", [("Bram", "Morrow."), ("Tilda", "Good day to thee.")])
CONV_B = conv("b", [("Hob", "The ditch wants digging."), ("Aldous", "Aye.")])

GOOD_FB = {"rating": 4, "tags": ["good"]}
BAD_VOICE = {"rating": 0, "tags": ["out-of-character"]}
BAD_FLAT = {"rating": 0, "tags": ["flat"]}
BAD_REP = {"rating": 0, "tags": ["repetitive"]}
CLEAN_FB = {"rating": 0, "tags": [], "note": "fine"}


class TestPrompt(unittest.TestCase):
    def test_scores_only_the_three_dims(self):
        prompt, _ = judge.build_prompt(CONV_A, CONV_B, [], False)
        for d in ("voice", "flat", "implication"):
            self.assertIn(d, prompt)
        # and nothing outside the remit sneaks in
        for w in ("grammar", "length", "humor", "coherence"):
            self.assertNotIn(w, prompt)

    def test_order_randomization_swaps_the_slots(self):
        p1, slot1 = judge.build_prompt(CONV_A, CONV_B, [], False)
        p2, slot2 = judge.build_prompt(CONV_A, CONV_B, [], True)
        self.assertEqual(slot1, "1")
        self.assertEqual(slot2, "2")
        # conversation 1's body changes with the flip
        self.assertIn("Morrow.", p1.split("--- conversation 2")[0])
        self.assertNotIn("Morrow.", p2.split("--- conversation 2")[0])

    def test_anchors_are_embedded(self):
        prompt, _ = judge.build_prompt(CONV_A, CONV_B,
                                       ["Bram: the good line."], False)
        self.assertIn("keeper 1", prompt)
        self.assertIn("the good line.", prompt)


class TestParse(unittest.TestCase):
    def test_clean_json(self):
        v = judge.parse_verdict(
            '{"voice": "1", "flat": "2", "implication": "tie"}')
        self.assertEqual(v, {"voice": "1", "flat": "2",
                             "implication": "tie"})

    def test_json_wrapped_in_prose(self):
        v = judge.parse_verdict(
            'Sure! {"voice":"tie","flat":"1","implication":"2"} thanks')
        self.assertEqual(v["flat"], "1")

    def test_bad_verdict_raises(self):
        with self.assertRaises(ValueError):
            judge.parse_verdict('{"voice": "maybe", "flat": "1",'
                                ' "implication": "2"}')
        with self.assertRaises(ValueError):
            judge.parse_verdict("no json here")


class TestSignals(unittest.TestCase):
    def test_negative_dims_maps_user_tags(self):
        self.assertEqual(judge.negative_dims(BAD_VOICE), {"voice"})
        self.assertEqual(judge.negative_dims(BAD_FLAT), {"flat"})
        self.assertEqual(judge.negative_dims(BAD_REP), {"flat"})
        self.assertEqual(judge.negative_dims(CLEAN_FB), set())
        self.assertEqual(judge.negative_dims(
            {"tags": ["non-sequitur", "ending"]}), set())

    def test_is_good(self):
        self.assertTrue(judge.is_good(GOOD_FB))
        self.assertTrue(judge.is_good({"rating": 0, "tags": ["good"]}))
        self.assertFalse(judge.is_good(CLEAN_FB))


class TestPairs(unittest.TestCase):
    def test_negative_pairs_with_a_clean_partner(self):
        convs = {"bad": None, "good": None, "ok": None}
        fb = {"bad": BAD_FLAT, "good": GOOD_FB, "ok": CLEAN_FB}
        pairs = judge.agreement_pairs(fb, convs)
        self.assertEqual(pairs, [("flat", "bad", "good")])

    def test_skipped_and_unknown_ids_stay_out(self):
        convs = {"bad": None, "ok": None}
        fb = {"bad": BAD_VOICE, "ok": CLEAN_FB,
              "gone": GOOD_FB, "skip": {**GOOD_FB, "skip": True}}
        pairs = judge.agreement_pairs(fb, convs)
        self.assertEqual(pairs, [("voice", "bad", "ok")])

    def test_implication_never_pairs_no_user_tag(self):
        convs = {"bad": None, "ok": None}
        fb = {"bad": {**BAD_VOICE, "tags": ["out-of-character"]},
              "ok": CLEAN_FB}
        dims = {d for d, _, _ in judge.agreement_pairs(fb, convs)}
        self.assertNotIn("implication", dims)


class TestJudgePair(unittest.TestCase):
    def stub(self, prompt, cfg):
        # whichever slot conv A ("Morrow.") shows in, pick it - the stub
        # judges by content, not position
        slot = "1" if "Morrow." in prompt.split("--- conversation 2")[0] \
            else "2"
        return json.dumps({d: slot for d in judge.DIMS})

    def test_slot_verdict_maps_back_to_caller_terms(self):
        cfg = judge.config()
        for seed in range(6):
            won = judge.judge_pair(CONV_A, CONV_B, [], cfg,
                                   random.Random(seed), query=self.stub)
            for d in judge.DIMS:
                self.assertEqual(won[d], "a")


class TestAgreement(unittest.TestCase):
    CONVS = {"bad": CONV_A, "good": CONV_B}

    def _picks(self, needle):
        # a stub that always sides with the transcript carrying `needle`
        def q(prompt, cfg):
            slot = "1" if needle in prompt.split("--- conversation 2")[0] \
                else "2"
            return json.dumps({d: slot for d in judge.DIMS})
        return q

    def test_clean_winner_counts_agree(self):
        rows = judge.agreement_report(
            [("flat", "bad", "good")] * 3, self.CONVS, [],
            judge.config(), random.Random(0),
            query=self._picks("ditch"))
        self.assertEqual((rows["flat"]["agree"], rows["flat"]["ties"],
                          rows["flat"]["disagree"]), (3, 0, 0))

    def test_ties_count_half_disagrees_count_zero(self):
        ties = lambda p, c: json.dumps({d: "tie" for d in judge.DIMS})
        rows = judge.agreement_report(
            [("flat", "bad", "good")] * 2, self.CONVS, [],
            judge.config(), random.Random(0), query=ties)
        self.assertEqual((rows["flat"]["agree"], rows["flat"]["ties"]),
                         (1.0, 2))
        rows = judge.agreement_report(
            [("flat", "bad", "good")], self.CONVS, [],
            judge.config(), random.Random(0),
            query=self._picks("Morrow"))
        self.assertEqual((rows["flat"]["agree"], rows["flat"]["disagree"]),
                         (0, 1))


class TestKeepers(unittest.TestCase):
    def test_propose_keepers_filters(self):
        old = judge.FEEDBACK
        try:
            with tempfile.TemporaryDirectory() as td:
                judge.FEEDBACK = os.path.join(td, "feedback.json")
                judge.save(judge.FEEDBACK, {
                    "k1": GOOD_FB,
                    "k2": {**GOOD_FB, "skip": True},
                    "k3": CLEAN_FB,
                    "k4": {"rating": 5, "tags": []},
                })
                self.assertEqual(judge.propose_keepers(), ["k1", "k4"])
        finally:
            judge.FEEDBACK = old


if __name__ == "__main__":
    unittest.main()
