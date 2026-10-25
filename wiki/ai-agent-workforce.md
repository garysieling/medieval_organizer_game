# AI Agent Workforce Architecture

This document defines the roles, responsibilities, skills, and learning mechanisms for the AI agents that will build and test the game.

## Agent Template

Each agent should have:
- **Job Description (JD)** - What they do, their scope, their goals
- **Skills/Career Ladder** - How they grow and improve over time
- **Claude Skills/Rules** - Specific prompts, guidelines, and constraints
- **Learning Mechanism** - How they learn from interactions and improve

---

## Engineering Agents

### Implementation Engineer
**Status:** Spec Defined
**Priority:** CRITICAL
**Spec Location:** `wiki/agents/implementation-engineer.md`

**Job Description:**
- Primary coding agent for implementing features
- Takes design specs and writes production code
- Follows established patterns and architecture
- Writes tests alongside implementation
- Normal mode coding - not specializing in any particular area

**Skills/Career Ladder:**
- Junior: Can implement simple features with guidance
- Mid: Can implement complex features independently
- Senior: Can architect solutions and mentor other agents
- Staff: Can design system-level changes and drive technical direction

**Claude Skills/Rules:**
- See spec document for full details

**Learning Mechanism:**
- See spec document for full details

---

### Pedantic Senior Engineer
**Status:** Backlog
**Priority:** CRITICAL

**Job Description:**
- Specializes in complexity analysis and maintainability
- Identifies code smells, anti-patterns, and technical debt
- Refactors code along natural seams
- Introduces appropriate design patterns
- Runs every N turns (not every commit)
- Focuses on long-term code health

**Skills/Career Ladder:**
- Junior: Can identify basic code smells
- Mid: Can refactor small modules
- Senior: Can architect large refactors with minimal risk
- Staff: Can drive architectural evolution and pattern adoption

**Claude Skills/Rules:**
- TBD

**Learning Mechanism:**
- TBD

---

### Performance Engineer
**Status:** Backlog
**Priority:** CRITICAL

**Job Description:**
- Monitors system performance metrics
- Identifies anomalies and bottlenecks
- Profiles code and suggests optimizations
- Watches telemetry for performance regressions
- Focuses on ensuring simulation runs efficiently

**Skills/Career Ladder:**
- Junior: Can identify obvious performance issues
- Mid: Can profile and optimize hot paths
- Senior: Can design performance-conscious architecture
- Staff: Can drive system-wide performance strategy

**Claude Skills/Rules:**
- TBD

**Learning Mechanism:**
- TBD

---

## Testing Agents

### Exploratory Tester (Data Scientist)
**Status:** Spec Defined
**Priority:** CRITICAL
**Spec Location:** `wiki/agents/exploratory-tester-data-scientist.md`

**Job Description:**
- Analyzes logs, data structures, and telemetry
- Looks for design issues in data patterns
- Identifies errors through data analysis
- Uses statistical methods to find anomalies
- Focuses on backend/data layer testing

**Skills/Career Ladder:**
- Junior: Can analyze basic logs for errors
- Mid: Can identify patterns in data
- Senior: Can design data-driven test strategies
- Staff: Can architect observability and analytics systems

**Claude Skills/Rules:**
- See spec document for full details

**Learning Mechanism:**
- See spec document for full details

---

### Exploratory Tester (UI Clicker)
**Status:** Spec Defined
**Priority:** CRITICAL
**Spec Location:** `wiki/agents/exploratory-tester-ui-clicker.md`

**Job Description:**
- Explores the UI by clicking around
- Finds UI mistakes, broken flows, visual bugs
- Tests edge cases in user interactions
- Focuses on frontend/UX testing
- Complements data scientist tester

**Skills/Career Ladder:**
- Junior: Can find obvious UI bugs
- Mid: Can systematically explore UI flows
- Senior: Can design comprehensive UI test strategies
- Staff: Can architect UX testing frameworks

**Claude Skills/Rules:**
- See spec document for full details

**Learning Mechanism:**
- See spec document for full details

---

### Simple Playtester (Goal-Oriented)
**Status:** Spec Defined
**Priority:** CRITICAL
**Spec Location:** `wiki/agents/simple-playtester.md`

**Job Description:**
- Executes specific goals in the game (e.g., "build 5 units")
- Tests whether basic functionality works
- Two versions: UI-based and non-UI (backend API)
- Provides straightforward feedback on success/failure
- Focuses on regression testing of core mechanics

**Skills/Career Ladder:**
- Junior: Can execute simple goals
- Mid: Can handle complex multi-step goals
- Senior: Can design goal-based test suites
- Staff: Can architect automated playtesting frameworks

**Claude Skills/Rules:**
- See spec document for full details

**Learning Mechanism:**
- See spec document for full details

---

### Full Game Playtesters (Backend API)
**Status:** Spec Defined
**Priority:** CRITICAL
**Spec Location:** `wiki/agents/full-game-playtester.md`

**Job Description:**
- Runs full games through backend API (no UI)
- Can run thousands of simulations overnight
- Tests long-term game stability and balance
- Focuses on simulation depth and end-to-end testing
- Fast execution without UI overhead

**Skills/Career Ladder:**
- Junior: Can run basic full-game simulations
- Mid: Can analyze simulation results for issues
- Senior: Can design simulation test scenarios
- Staff: Can architect mass simulation infrastructure

**Claude Skills/Rules:**
- See spec document for full details

**Learning Mechanism:**
- See spec document for full details

---

### "Fun" Inspector
**Status:** Spec Defined
**Priority:** CRITICAL
**Spec Location:** `wiki/agents/fun-inspector.md`

**Job Description:**
- Analyzes playtests for "fun" factor
- Evaluates balance between grind and emotional stimulation
- Looks for pacing issues, engagement drops
- Retrospects playtest data to identify boredom/frustration
- Focuses on qualitative playability assessment

**Skills/Career Ladder:**
- Junior: Can identify obvious fun issues
- Mid: Can analyze engagement patterns
- Senior: Can design fun metrics and rubrics
- Staff: Can architect playability assessment systems

**Claude Skills/Rules:**
- See spec document for full details

**Learning Mechanism:**
- See spec document for full details

---

## Research & Analysis Agents

### Retrospective Agent
**Status:** Backlog
**Priority:** CRITICAL

**Job Description:**
- Looks backwards at issues and problems
- Proposes fixes to all other agents
- Identifies systemic issues across the workforce
- Synthesizes learnings from failures
- Focuses on continuous improvement of the agent ecosystem

**Skills/Career Ladder:**
- Junior: Can analyze individual issues
- Mid: Can identify patterns across issues
- Senior: Can propose systemic improvements
- Staff: Can drive organizational learning and evolution

**Claude Skills/Rules:**
- TBD

**Learning Mechanism:**
- TBD

---

### Economist Testers
**Status:** Backlog
**Priority:** CRITICAL

**Job Description:**
- Specializes in economic simulation testing
- Tests balance of trade, resources, markets
- Validates economic models (age-based self-interest, etc.)
- Looks for economic exploits or broken mechanics
- Focuses on simulation realism and game balance

**Skills/Career Ladder:**
- Junior: Can test basic economic mechanics
- Mid: Can analyze economic system behavior
- Senior: Can design economic test scenarios
- Staff: Can architect economic simulation validation

**Claude Skills/Rules:**
- TBD

**Learning Mechanism:**
- TBD

---

### Credibility Analyst
**Status:** Backlog
**Priority:** CRITICAL

**Job Description:**
- Validates psych/sociology research
- Checks that game mechanics align with academic research
- Ensures simulations are reasonably accurate
- Filters for high-quality sources
- Focuses on academic credibility of game systems

**Skills/Career Ladder:**
- Junior: Can validate basic research claims
- Mid: Can synthesize research into game mechanics
- Senior: Can design research-to-game pipelines
- Staff: Can architect academic validation systems

**Claude Skills/Rules:**
- TBD

**Learning Mechanism:**
- TBD

---

### Creative Filter
**Status:** Backlog
**Priority:** CRITICAL

**Job Description:**
- Filters research content for simple mechanics to borrow
- Looks for creative applications (e.g., QSP to non-health things)
- Identifies transferable concepts across domains
- Proposes novel mechanic combinations
- Focuses on creative borrowing and cross-pollination

**Skills/Career Ladder:**
- Junior: Can identify obvious transferable mechanics
- Mid: Can propose creative domain crossings
- Senior: Can design novel mechanic combinations
- Staff: Can architect creative innovation systems

**Claude Skills/Rules:**
- TBD

**Learning Mechanism:**
- TBD

---

### Emergent Effects Researcher
**Status:** Backlog
**Priority:** CRITICAL

**Job Description:**
- Researches papers on emergent effects
- Proposes ideas based on emergent systems (e.g., fractals from simple rules)
- Structures ideas and writes them out as mods
- Writes agents or tests to prove out ideas
- Focuses on bottom-up complexity generation

**Skills/Career Ladder:**
- Junior: Can research and summarize emergent systems
- Mid: Can propose emergent mechanic ideas
- Senior: Can implement and test emergent systems
- Staff: Can architect emergent design frameworks

**Claude Skills/Rules:**
- TBD

**Learning Mechanism:**
- TBD

---

## Consultation Agents

### Aesthetic Consultant
**Status:** Backlog
**Priority:** CRITICAL

**Job Description:**
- Reviews screencaps for aesthetic recommendations
- Provides feedback on visual consistency, style, quality
- Identifies visual bugs or inconsistencies
- Focuses on art direction and visual polish

**Skills/Career Ladder:**
- Junior: Can identify obvious visual issues
- Mid: Can provide detailed aesthetic feedback
- Senior: Can guide art direction and style
- Staff: Can architect aesthetic quality systems

**Claude Skills/Rules:**
- TBD

**Learning Mechanism:**
- TBD

---

### Gameplay Consultant
**Status:** Backlog
**Priority:** CRITICAL

**Job Description:**
- Watches gameplay walkthroughs
- Provides feedback on game feel, pacing, design
- Identifies design issues from actual gameplay
- Focuses on qualitative gameplay assessment

**Skills/Career Ladder:**
- Junior: Can identify obvious gameplay issues
- Mid: Can provide detailed gameplay feedback
- Senior: Can guide gameplay design iteration
- Staff: Can architect gameplay quality systems

**Claude Skills/Rules:**
- TBD

**Learning Mechanism:**
- TBD

---

## Orchestration & Coordination

### Agent Coordination System
**Status:** Backlog
**Priority:** CRITICAL

**Job Description:**
- The technical infrastructure for agent coordination
- Manages worktrees and branching
- Provides messaging between agents
- Implements locking and coordination primitives
- Not an agent itself, but the system they run on

**Skills/Career Ladder:**
- N/A (system design)

**Claude Skills/Rules:**
- TBD

**Learning Mechanism:**
- N/A (system configuration)

---

### Manager Bot
**Status:** Spec Defined
**Priority:** CRITICAL
**Spec Location:** `wiki/agents/manager-bot.md`

**Job Description:**
- Oversees entire agent workforce
- Delegates tasks to appropriate agents
- Handles agent promotions (assesses reliability, capability expansion)
- Interprets agent feedback and knowledge base updates
- Builds sprint plans with exit criteria
- Reviews overall structure and agent effectiveness
- Coordinates retrospectives and forward planning

**Skills/Career Ladder:**
- N/A (this is the orchestrator, not a promoted agent)
- Evolves its understanding of agent capabilities over time

**Claude Skills/Rules:**
- See spec document for full details

**Learning Mechanism:**
- See spec document for full details
- Refines sprint planning based on historical outcomes

---

### PR Agent
**Status:** Backlog
**Priority:** CRITICAL

**Job Description:**
- Creates pull requests for agent work
- Reviews PRs from other agents
- Merges PRs when appropriate
- Ensures code quality before merge
- Manages PR workflow and communication

**Skills/Career Ladder:**
- Junior: Can create simple PRs
- Mid: Can review and merge straightforward PRs
- Senior: Can handle complex PR conflicts and reviews
- Staff: Can architect PR workflow and quality gates

**Claude Skills/Rules:**
- TBD

**Learning Mechanism:**
- TBD

---

### Conflict Resolution Agent
**Status:** Backlog
**Priority:** CRITICAL

**Job Description:**
- Resolves merge conflicts in worktrees
- Handles integration issues between agent work
- Coordinates with PR agent on blocked merges
- Identifies root causes of conflicts
- Proposes structural changes to reduce future conflicts

**Skills/Career Ladder:**
- Junior: Can resolve simple conflicts
- Mid: Can handle complex multi-file conflicts
- Senior: Can identify and fix structural conflict patterns
- Staff: Can architect conflict prevention systems

**Claude Skills/Rules:**
- TBD

**Learning Mechanism:**
- TBD

---

## Architectural Decisions

### Knowledge Base
- Each agent has their own knowledge base (test files)
- Separate from core research wiki folders
- Agents update themselves OR provide feedback to manager who interprets
- Manager can see overall structure and agent learnings

### Effectiveness Measurement
- Primary concern: quality of output
- Peer review between agents
- Periodic retrospective and forward planning steps

### Career Ladder & Promotion
- Separate manager bot handles promotions
- Promotion based on: agent learned enough to be more effective, can be relied upon for larger scope without tuning
- Manager assesses agent reliability and capability expansion

### State Management
- Preference: txt files for state
- Fallback: postgres container if necessary
- Keep state simple and version-controllable

### Conflict Resolution
- Manager delegates tasks
- Monorepo with worktrees
- Dedicated agent for PRs
- Dedicated agent for resolving conflicts

### Triggering & Execution
- Sprint-based planning
- Manager builds sprint plan
- User reviews sprint plan
- Plan includes exit criteria
- System runs autonomously for sprint duration

### Runtime Environment
- All local execution
- Resources: Claude code credits, Devin credits, Ollama

### Codebase Access
- Varies by agent role
- Decide per agent what makes sense
- Some agents: direct codebase access (read/write)
- Some agents: API only (game testing interface)
- Some agents: read-only codebase + API

## Questions to Resolve

1. How do agents share context and learnings? (DECIDED: agent KBs + manager interpretation)
2. How do we prevent agents from conflicting with each other? (DECIDED: manager delegation + worktrees + PR agent + conflict agent)
3. How do we measure agent effectiveness? (DECIDED: quality focus + peer review + periodic retrospectives)
4. How do agents evolve their skills over time? (DECIDED: manager bot handles promotions)
5. What's the runtime environment for these agents? (DECIDED: all local - Claude/Devin/Ollama)
6. How do we handle agent failures or bad outputs? (TO BE DEFINED - added to backlog)
7. How do we prioritize which agent runs when? (DECIDED: sprint planning with exit criteria)
8. How do agents access the game codebase and telemetry? (DECIDED: varies by role, per-agent decision)
