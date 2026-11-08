## Summary

- **Same root cause, two tests (#831, events):** `test_the_pepper_eases_and_cures` and `test_the_strange_thing_sells` teleport a villager next to a visitor's pitch and let a day run — but if she happens to stand within `TAVERN_TABLE_REACH` at the day tick, `tavern_bench` sells her a cup off the bar (it gates on `Want::Drink` satisfaction, not her wants list). That's deliberate design — the bench fills with whoever's dry — but it made the assertions flaky by stage:
  - **pepper:** in `Famine` the cup cost ~4g on top of the pepper's 2g, so her purse read 4.0 not 8.0.
  - **strange_thing:** the counter-poured cup left an `"ale": 0` residue in her satchel, so it wasn't `is_empty()` after the alchemist's purchase.
- **Fix is test-side isolation:** satisfy `Want::Drink` for the test villager ("she's not thirsty today") so the event transaction is the only spend/movement; the strange-thing test also clears the satchel before planting the hazel so the `is_empty` assertion measures the sale, not leftovers. No sim changes — per the grill, current deliberate design wins.

#### Test plan
- [x] `cargo test -p game2-core --lib -- session::tests::events::test_the_pepper_eases_and_cures session::tests::events::test_the_strange_thing_sells` — 2/2 green (both stages each)
- [x] Debugged with a scratch harness: gold/satchel/hunger traced per frame — the only extra spend was the bench cup
- [ ] `cargo test -p game2-core --lib` — remaining red is the log_priority tranche, sibling PR

Part of #831

Generated with [Devin](https://devin.ai)
