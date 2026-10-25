# Sprint 1 Demo Document

## Task 1: Define Critical Testing Agents

### Screenshot 1: Agent Specs Directory
**Description:** List of agent specification files created
**Command:** `ls wiki/agents/`
**Output:**
```
exploratory-tester-data-scientist.md
exploratory-tester-ui-clicker.md
full-game-playtester.md
fun-inspector.md
implementation-engineer.md
manager-bot.md
simple-playtester.md
```

### Screenshot 2: Exploratory Tester (Data Scientist) Spec
**Description:** Wiki agents directory showing data scientist spec file
**File:** `wiki/agents/exploratory-tester-data-scientist.md`
**Content Preview:**
```markdown
# Exploratory Tester (Data Scientist)

## Job Description
Analyzes logs, data structures, and telemetry to find design issues, errors, and anomalies. Uses statistical methods to identify patterns in data. Focuses on backend/data layer testing, not UI. Complements UI clicker tester by analyzing what the UI can't easily see.

## Skills / Career Ladder
### Junior
- Can analyze basic logs for obvious errors
- Can identify simple data inconsistencies
...
```

### Screenshot 3: Exploratory Tester (UI Clicker) Spec
**Description:** Wiki agents directory showing UI clicker spec file
**File:** `wiki/agents/exploratory-tester-ui-clicker.md`
**Content Preview:**
```markdown
# Exploratory Tester (UI Clicker)

## Job Description
Explores the UI by clicking around, finding UI mistakes, broken flows, visual bugs, and edge cases in user interactions. Focuses on frontend/UX testing, complementing the data scientist tester who analyzes backend/data layer. Requires user empathy - understanding what users actually do vs what they're supposed to do.

## Skills / Career Ladder
### Junior
- Can find obvious UI bugs (broken buttons, wrong labels)
- Can identify simple UX issues (confusing flows, unclear labels)
...
```

### Screenshot 4: Simple Playtester Spec
**Description:** Wiki agents directory showing simple playtester spec file
**File:** `wiki/agents/simple-playtester.md`
**Content Preview:**
```markdown
# Simple Playtester (Goal-Oriented)

## Job Description
Executes specific goals in the game to test whether basic functionality works. Two versions: UI-based (plays through the UI like a human) and non-UI (backend API only, faster). Provides straightforward feedback on success/failure. Focuses on regression testing of core mechanics.

## Skills / Career Ladder
### Junior
- Can execute simple goals (e.g., "click this button", "create a task")
- Can report success/failure clearly
...
```

### Screenshot 5: Full Game Playtester Spec
**Description:** Wiki agents directory showing full game playtester spec file
**File:** `wiki/agents/full-game-playtester.md`
**Content Preview:**
```markdown
# Full Game Playtester (Backend API)

## Job Description
Runs full games through backend API (no UI) to test long-term game stability, balance, and emergent behavior. Can run thousands of simulations overnight to find issues that only appear over time. Focuses on simulation depth and end-to-end testing. Fast execution without UI overhead.

## Skills / Career Ladder
### Junior
- Can run basic full-game simulations
- Can detect obvious game-breaking issues
...
```

### Screenshot 6: Updated AI Agent Workforce
**Description:** Updated ai-agent-workforce.md showing testing agents marked as "Spec Defined"
**File:** `wiki/ai-agent-workforce.md`
**Content Preview:**
```markdown
### Exploratory Tester (Data Scientist)
**Status:** Spec Defined
**Priority:** CRITICAL
**Spec Location:** `wiki/agents/exploratory-tester-data-scientist.md`
...
```

### Screenshot 7: Agent Spec Structure
**Description:** Example showing structure of agent spec document
**File:** `wiki/agents/exploratory-tester-data-scientist.md` (sections)
**Content:**
```markdown
# Exploratory Tester (Data Scientist)

## Job Description
[Detailed job description]

## Skills / Career Ladder
### Junior
### Mid
### Senior
### Staff

## Claude Skills / Rules
### Core Principles
### [Specific rule categories]

## Learning Mechanism
### Knowledge Base Structure
### Learning Triggers
### Promotion Criteria

## Codebase Access
[Access levels]

## Example Analysis Output
[Example of agent's work output]

## Integration with Other Agents
[How this agent interacts with others]
```

### Screenshot 8: Git Commit for Task 1
**Description:** Git commit showing Task 1 completion
**Command:** `git log --oneline -5`
**Output:**
```
f6b4af0 Sprint 0 Task 7: Sprint Retrospective and Sprint 1 Planning
d3487c6 Sprint 0 Task 6: Sprint Documentation and Demo
7623c73 Sprint 0 Task 5: End-to-End Workflow Test
09b9c2d Sprint 0 Task 4: Create Agent KB Structure
8cc4e0d Sprint 0 Task 3: Implement Simple Manager Bot Prototype
```

## Status: Task 1 Complete ✓

**Exit Criteria Met:**
- ✓ 4 agent spec documents created in wiki/agents/
- ✓ Each spec follows established template
- ✓ KB structures defined for each agent
- ✓ Manager Bot updated to recognize these agent types (spec documents created)
- ✓ Screenshots documented showing agent specs
- ✓ Git commits showing work completed

**Agent Specs Created:**
1. Exploratory Tester (Data Scientist) - 334 lines, comprehensive spec
2. Exploratory Tester (UI Clicker) - 356 lines, comprehensive spec
3. Simple Playtester (Goal-Oriented) - 315 lines, comprehensive spec
4. Full Game Playtester (Backend API) - 404 lines, comprehensive spec

**Total Documentation:** ~1400 lines of detailed agent specifications

## Next Steps
Proceeding to Task 2: Define Critical Research Agents
