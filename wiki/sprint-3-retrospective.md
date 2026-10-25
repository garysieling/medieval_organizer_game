# Sprint 3 Retrospective

## What Went Well
- **Scope delivery** - 6 workstreams all landed: 7 sim domains, 10 game systems, 4 UI, 8 infra, 8 research items
- **Evidence pipeline** - BMP renders + text outputs + PNG screenshots now real software output, not desktop captures
- **Headless rendering** - isometric board + minimap render to BMP; proof the architecture works
- **Emergence validated** - CA fire spread, attachment style emergence, price formation all produce real emergent behavior
- **Lit reviews hit gold** - medieval guilds, QSP compartments, community organizing tactics, religion emergence all directly inform mechanics
- **Ollama works** - gemma3:4b executes tasks in ~15s for agent loops

## What Didn't
- **Context limits** - ran out of context multiple times; had to checkpoint work and continue
- **Windows env** - Application Control still blocks some executions; worked around with std-only code
- **Test debt** - 7 pre-existing filesystem tests still fail (hardcoded paths, need temp dirs)
- **Terrain tuning** - initial render mostly water; fixed via normalization but needs polish
- **Sim integration** - domains are built but not yet wired into a unified world state

## Learnings
- Evidence harness (demo.rs) is the right pattern - software produces its own proof
- BMP encoder works with zero deps - enables real visual evidence
- Compact agent specs scale - template lets us spec 19 agents quickly
- Parallel workstreams keep sprint moving even when one blocks
- Lit reviews continue to yield concrete mechanics, not just validation

## Action Items for Sprint 4
1. Wire sim domains to ECS entities (attach Health/Attachment/Market to villagers)
2. Unify world state - one World holding all components, systems running together
3. Fix filesystem tests (temp dirs instead of hardcoded paths)
4. Agent fleet executes real tasks via Ollama (gemma3:4b)
5. Character sheet + dialogs get entity data behind them
6. Video capture of long sim run
7. Sprint 4 planning with user review

## Status
Sprint 3 substantially exceeded Sprint 2 scope. Foundation is now a real codebase, not just structure.
