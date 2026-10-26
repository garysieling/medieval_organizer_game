# Literature Survey Skill

Research academic/historical topics and translate them into game mechanics.

## Quick Start

### Run a survey on a specific domain:
```
/literature-survey Agriculture & Food Systems
```

### Run next scheduled survey:
```
/literature-survey
```

## What it does

1. **Searches** for academic sources (papers, books, historical research)
2. **Evaluates** source credibility (A+ to D rating)
3. **Extracts** core mechanisms and models
4. **Translates** to game mechanics (simplified, fun, educational)
5. **Documents** findings in wiki + KB files

## Output artifacts

- `wiki/literature-reviews/{domain}.md` - Full survey writeup
- `.agents/literature-survey-agent/kb/surveys.md` - Survey log entry
- `.agents/literature-survey-agent/kb/source-log.md` - Sources found
- `.agents/literature-survey-agent/kb/mechanism-translations.md` - Mechanic mappings
- `.agents/literature-survey-agent/kb/domain-coverage.md` - Coverage update
- `wiki/backlog.md` - New backlog items (if actionable)

## Current domain rotation

**Cycle 2 (Next):**
1. Agriculture & Food Systems (HIGH)
2. Ecology & Climate (HIGH)
3. Urban Planning (MEDIUM)
4. Philosophy & Ethics (MEDIUM)
5. Anthropology (MEDIUM)
6. Historical Events (MEDIUM)

**Coverage:** 44% of relevant domains (7 well-covered / 16 total)

## KB files to check before/after

- `surveys.md` - 7 surveys completed (Sprints 1-4)
- `mechanism-translations.md` - 8 academic→game mappings
- `source-log.md` - Source credibility ratings
- `domain-coverage.md` - Gap analysis

## Examples

### Example 1: Domain survey
```
/literature-survey Agriculture & Food Systems
```

Expected output:
- Wiki entry with sources, findings, game mechanics
- KB files updated with survey entry, sources, translations
- Backlog items for implementing farming systems

### Example 2: Hot-pursuit research
```
/literature-survey Guild Economics
```

If user requests specific topic outside rotation schedule, do targeted research.

## Execution modes

This skill can also be run via:
- **Ollama** (free, overnight batch) - See `game2-core/src/infra/ollama.rs`
- **Rust** (fast scheduling checks) - See `game2-core/src/agents/literature_survey_agent.rs`

Manager Bot decides which mode based on task criticality and budget.

## Related skills

- **Credibility Analyst** - Validates research claims (coming soon)
- **Emergent Effects Researcher** - Finds emergent mechanics from papers
- **Creative Filter** - Borrows mechanics from other domains

---

See `wiki/agents/hybrid-agent-architecture.md` for full architecture details.
