# Sprint 1 Planning - Agent Expansion & Game Foundation

## Overview

Sprint 1 builds on Sprint 0's infrastructure to expand the agent workforce, enhance coordination capabilities, and begin actual game feature development. This sprint is intentionally more ambitious to leverage the momentum from Sprint 0's success.

## Sprint Goals

**Primary Goal:** Expand agent workforce capabilities and begin game feature development

**Success Criteria:**
- Define specifications for 6+ critical agents
- Enhance Manager Bot with capability matching and better delegation
- Implement task dependencies and ordering
- Implement worktree system for parallel agent work
- Add simple game feature (ECS foundation or basic mechanic)
- Integrate AI (Ollama) with at least one agent for real execution
- Complete literature review sprint with findings
- Sprint retrospective completed
- Sprint 2 plan drafted

## Task Breakdown

### Phase 1: Agent Specification (Days 1-2)

#### Task 1: Define Critical Testing Agents (4 agents)
**Agent:** Implementation Engineer (Mid-Senior)
**Time Estimate:** 6 hours
**Dependencies:** Sprint 0 complete
**Description:**
Define full specifications for:
- Exploratory Tester (Data Scientist)
- Exploratory Tester (UI Clicker)
- Simple Playtester (Goal-Oriented)
- Full Game Playtester (Backend API)

Each spec includes: JD, career ladder, Claude skills/rules, learning mechanism, KB structure, codebase access.

**Exit Criteria:**
- 4 agent spec documents created in wiki/agents/
- Each spec follows established template
- KB structures defined for each agent
- Manager Bot updated to recognize these agent types
- Screenshot: List of agent specs created
- Screenshot: Example of one full agent spec

#### Task 2: Define Critical Research Agents (3 agents)
**Agent:** Implementation Engineer (Mid-Senior)
**Time Estimate:** 6 hours
**Dependencies:** Task 1 (same phase, can parallel)
**Description:**
Define full specifications for:
- Credibility Analyst
- Creative Filter
- Emergent Effects Researcher

Each spec includes: JD, career ladder, Claude skills/rules, learning mechanism, KB structure, codebase access.

**Exit Criteria:**
- 3 agent spec documents created in wiki/agents/
- Each spec follows established template
- KB structures defined for each agent
- Manager Bot updated to recognize these agent types
- Screenshot: List of research agent specs
- Screenshot: Example of one full research agent spec

#### Task 3: Update Manager Bot for All Agent Types
**Agent:** Implementation Engineer (Senior)
**Time Estimate:** 4 hours
**Dependencies:** Tasks 1, 2
**Description:**
Update Manager Bot to recognize all new agent types and route tasks appropriately.

**Exit Criteria:**
- Manager Bot updated with agent type recognition
- Delegation heuristics updated for all agent types
- Manager Bot KB updated with new agent capabilities
- Screenshot: Manager Bot code showing agent type routing
- Screenshot: Updated delegation heuristics

### Phase 2: Coordination Enhancement (Days 2-3)

#### Task 4: Implement Task Dependencies
**Agent:** Implementation Engineer (Senior)
**Time Estimate:** 4 hours
**Dependencies:** Task 3 (Manager Bot updated)
**Description:**
Add task dependency support to TaskQueue. Tasks can specify dependencies on other tasks.

**Exit Criteria:**
- TaskQueue updated to support dependencies
- Manager Bot respects dependencies when assigning
- Dependency resolution algorithm implemented
- Screenshot: Task dependency resolution in action
- Screenshot: Code showing dependency handling

#### Task 5: Implement Worktree System
**Agent:** Implementation Engineer (Staff)
**Time Estimate:** 5 hours
**Dependencies:** Task 4
**Description:**
Implement worktree system for parallel agent work. Each agent gets isolated worktree for their tasks.

**Exit Criteria:**
- Worktree creation/teardown implemented
- Agent isolation in worktrees
- Merge conflict detection (not resolution, just detection)
- Screenshot: Worktree structure
- Screenshot: Worktree creation/deletion in action

#### Task 6: Enhanced Manager Bot Delegation
**Agent:** Implementation Engineer (Staff)
**Time Estimate:** 4 hours
**Dependencies:** Tasks 3, 4, 5
**Description:**
Implement capability matching in Manager Bot. Match tasks to agents based on:
- Agent type (engineer, tester, researcher)
- Agent capability level (junior, mid, senior, staff)
- Task complexity and requirements
- Historical success rates

**Exit Criteria:**
- Capability matching algorithm implemented
- Manager Bot uses capability matching for delegation
- Delegation decisions logged and tracked
- Screenshot: Capability matching in action
- Screenshot: Delegation decision log

### Phase 3: Game Feature Foundation (Days 3-4)

#### Task 7: Implement Basic ECS Foundation
**Agent:** Implementation Engineer (Staff)
**Time Estimate:** 6 hours
**Dependencies:** Task 6
**Description:**
Implement basic Entity Component System foundation:
- Entity management
- Component system
- System scheduler
- Basic query system

This is the foundation for all game features.

**Exit Criteria:**
- ECS can create entities with components
- Systems can query and update components
- Basic ECS operations work (create, read, update, delete)
- Screenshot: ECS entity creation
- Screenshot: ECS system execution
- Screenshot: ECS query results

#### Task 8: Integrate Ollama with One Agent
**Agent:** Implementation Engineer (Senior)
**Time Estimate:** 5 hours
**Dependencies:** Task 7
**Description:**
Connect Implementation Engineer to Ollama for real AI execution. Start with simple task: "write a docstring for a function" or "implement a simple function based on spec."

**Exit Criteria:**
- Ollama integration working
- Implementation Engineer can execute tasks via Ollama
- Task completion tracked and logged
- Screenshot: Ollama integration code
- Screenshot: Task execution via Ollama
- Screenshot: Task completion log

### Phase 4: Research & Review (Days 4-5)

#### Task 9: Literature Review Sprint - Game Design & Simulation
**Agent:** Credibility Analyst + Creative Filter + Emergent Effects Researcher
**Time Estimate:** 4 hours
**Dependencies:** Task 8 (agents operational)
**Description:**
First scheduled literature review focusing on:
- Game design research (emergent narrative, player agency)
- Simulation theory (agent-based modeling, complex systems)

**Exit Criteria:**
- Literature review summary document created
- 10+ sources reviewed
- Key findings documented
- Actions taken (wiki entries, backlog items, proposals)
- Screenshot: Literature review summary
- Screenshot: Example source review

#### Task 10: Sprint Documentation and Demo
**Agent:** Manager Bot + Implementation Engineer
**Time Estimate:** 3 hours
**Dependencies:** Task 9
**Description:**
Create comprehensive Sprint 1 documentation including:
- Sprint summary
- Architecture updates
- Demo compilation (screenshots, videos if possible)
- Lessons learned

**Exit Criteria:**
- Sprint 1 summary complete
- Architecture document updated
- Demo document with screenshots
- User guide updated
- Lessons learned documented
- Screenshot: Sprint summary
- Screenshot: Demo compilation

#### Task 11: Sprint Retrospective and Sprint 2 Planning
**Agent:** Manager Bot + All agents
**Time Estimate:** 2 hours
**Dependencies:** Task 10
**Description:**
Review Sprint 1, update KBs, plan Sprint 2.

**Exit Criteria:**
- Retrospective document complete
- All agent KBs updated
- Sprint 2 plan drafted
- Screenshot: Retrospective findings
- Screenshot: Sprint 2 plan outline

## Total Sprint Estimate

- **Total Time:** 49 hours
- **Buffer:** 12 hours (25%)
- **Sprint Duration:** 5 days

## Screenshot Requirements

For each task, capture:
1. **Code Screenshots** - Terminal showing code being written
2. **Execution Screenshots** - Terminal showing code running/tests passing
3. **Output Screenshots** - Actual output/files created
4. **File Screenshots** - Directory structure, file contents
5. **Process Screenshots** - Git commits, KB updates, etc.

### Screenshot Naming Convention
```
sprint-1-task-{N}-{description}-{timestamp}.png
```

Example:
- `sprint-1-task-1-agent-spec-created-20261025.png`
- `sprint-1-task-7-ecs-entity-created-20261025.png`

### Video Requirements
- Sprint-end video: 5-10 minutes showing all major achievements
- In-progress videos: Only for major milestones (ECS foundation, AI integration)

## Dependencies & Parallelization

### Critical Path (Sequential)
Task 1 → Task 3 → Task 4 → Task 5 → Task 6 → Task 7 → Task 8 → Task 9 → Task 10 → Task 11

### Parallel Work
- Tasks 1 & 2 can run in parallel (different agent specs)
- Tasks 4 & 5 can partially overlap (dependency resolution in worktrees)
- Tasks 9 & 10 can overlap (documentation as research concludes)

## Risks

### High Risk
- **AI Integration** - Ollama integration may be complex, need fallback plan
- **ECS Complexity** - ECS implementation may reveal architecture issues

### Medium Risk
- **Worktree System** - Parallel work may introduce merge conflicts
- **Literature Review** - May not find useful sources, need alternative

### Low Risk
- **Agent Specs** - Straightforward, just time-consuming
- **Task Dependencies** - Well-understood pattern

## Mitigation Strategies

### AI Integration Risk
- Start with simplest possible integration (one agent, one task type)
- Have fallback: if Ollama doesn't work, continue with simulated execution
- Document lessons learned for future AI integration

### ECS Complexity Risk
- Start with simplest possible ECS (entities, components, basic queries)
- Don't over-engineer - just foundation
- Test incrementally, don't build all at once

### Worktree Risk
- Start with simple worktree management
- Don't implement full merge conflict resolution initially
- Focus on isolation, not coordination

## Success Metrics

### Quantitative
- 6+ agent specs defined
- 3+ new modules implemented (dependencies, worktrees, ECS)
- 1+ AI integration successful
- 10+ literature sources reviewed
- 20+ screenshots captured
- 1+ video captured

### Qualitative
- Agent specs follow established template
- Manager Bot delegation is intelligent (not just round-robin)
- ECS foundation is clean and extensible
- AI integration is functional and demonstrates real execution
- Literature review finds useful insights
- User can see clear progress and understand what was built

## Questions for User

1. Is this level of ambition appropriate for Sprint 1?
2. Should we prioritize different agents (e.g., focus on fewer but more critical)?
3. For game features: prefer ECS foundation or a different mechanic?
4. For AI integration: prefer Ollama or Claude API (if available)?
5. Any concerns about the parallel work approach?
6. Screenshot/video format preferences?

## Next Steps

Once approved:
1. Begin Phase 1: Agent Specification
2. Start with Task 1: Define Critical Testing Agents
3. Capture screenshots as work progresses
4. Regular check-ins (daily or every 2 days)
5. Sprint end: Demo video + retrospective
