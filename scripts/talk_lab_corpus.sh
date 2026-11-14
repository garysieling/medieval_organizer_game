#!/usr/bin/env bash
# Talk-lab corpus: a repeatable set of conversation samples for reviewing
# the dialogue system. Run from the repo root:
#   bash scripts/talk_lab_corpus.sh [outdir]
# Default output: evidence/talk-lab/. Re-running regenerates the corpus
# deterministically (everything's seeded) - diff the files to review a
# conversation-system change's effect on output.
set -uo pipefail
cd "$(dirname "$0")/.."

OUT=${1:-evidence/talk-lab}
mkdir -p "$OUT"
RUN="cargo run -q -p game2-core --bin talk_lab --"

echo "== batch sweeps: auto-picked synthetic talks, 5 seeds =="
for seed in 1 7 42 101 777; do
  $RUN batch -n 12 --seed "$seed" --feedback off --out "$OUT/batch-seed$seed.md" \
    > "$OUT/batch-seed$seed.log" 2>&1
done

echo "== synthetic matrix: relation x status =="
for rel in stranger acquaintance friend kin rival; do
  for st in sick hungry grieving drunk elder; do
    $RUN talk --name Probe --status "$st" --rel "$rel" --seed 11 \
      --auto --feedback off --out "$OUT/syn-$rel-$st.txt" \
      > "$OUT/syn-$rel-$st.log" 2>&1
  done
done

echo "== synthetic matrix: ideology sweep =="
for ideo in laborer merchant clergy lord; do
  $RUN talk --name Voice --ideology "$ideo" --trait proud --rel acquaintance \
    --seed 23 --auto --feedback off --out "$OUT/syn-ideology-$ideo.txt" \
    > "$OUT/syn-ideology-$ideo.log" 2>&1
done

echo "== real villager talks: seeds x villagers =="
for seed in 42 7 13; do
  for v in 0 3 8 15 22; do
    $RUN talk --seed "$seed" --villager "$v" --auto --feedback off \
      --out "$OUT/real-s$seed-v$v.txt" > "$OUT/real-s$seed-v$v.log" 2>&1
  done
done

echo "== real talks under stage pressure =="
for stage in famine bereft unrest strongorg; do
  $RUN talk --seed 42 --stage "$stage" --villager 4 --auto --feedback off \
    --out "$OUT/real-stage-$stage.txt" > "$OUT/real-stage-$stage.log" 2>&1
done

echo "== house meetings =="
for seed in 3 9 27; do
  $RUN meeting --seed "$seed" --folk 4 --feedback off \
    --out "$OUT/meeting-$seed.txt" > "$OUT/meeting-$seed.log" 2>&1
done

echo "corpus written to $OUT ($(ls "$OUT" | wc -l) files)"
