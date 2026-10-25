# Sprint 1 Retrospective

## What Went Well
- **7 agent specs written** (~2,600 lines) - template proved reusable and fast
- **Capability matching** - tasks now route to correct agent types (Design → pedantic-senior-engineer, etc.)
- **Task dependencies** - dependency chains correctly block until deps complete
- **ECS foundation** - clean, minimal, tested; ready for game systems
- **Ollama integration** - zero-dependency CLI wrapper works with installed 0.40.11
- **Lit review found real validation** - EA Sims paper validates our playtester architecture; Machinations + cellular automata are concrete next mechanics

## What Didn't
- Windows Application Control intermittently blocked exe runs and build scripts → worked around with std-only code and line-based formats, but real screenshots/executables remain flaky in this env
- Capability inference is keyword-based and crude - task_10 "Write integration tests" routed to implementation-engineer instead of a tester (keyword "tests" matched "testing" capability which impl engineer also has)

## Learnings
- Agent spec template works - specs now take ~10 min each, not hours
- Backwards compatibility matters - legacy 5-field task format broke parser until handled
- Std-only Rust is more robust in this environment than external deps
- Lit reviews pay off - first one found 2 concrete mechanics + architecture validation

## Sprint 2 Recommendations
1. **Real Ollama test** - pull a model, execute an actual agent task end-to-end
2. **Improve inference** - priority ordering so specialized agents win over impl-engineer
3. **First game mechanic** - villager on ECS (hunger → seek food → eat)
4. **Remaining agent specs** - Pedantic, Retrospective, PR, Conflict resolution
5. **Lit review** - Economics + Psychology/Sociology domains

## Status
Sprint 1 objectives substantially met. Foundation is ready for first real game mechanic.
