# Sprint 0 Planning - Foundation Setup

## Current State Review

### Completed Specifications
- **Implementation Engineer** - Full spec with career ladder, skills, learning mechanism
- **Manager Bot** - Full spec with delegation, sprint planning, retrospectives
- **Fun Inspector** - Full spec with fun analysis, playtesting
- **Agent Knowledge Base Architecture** - Full design for KB structure and processes
- **Literature Review Scheduling** - Full system for periodic research

### Still Need Specifications (16 agents)
- Pedantic Senior Engineer
- Performance Engineer
- Exploratory Tester (Data Scientist)
- Exploratory Tester (UI Clicker)
- Simple Playtester (Goal-Oriented)
- Full Game Playtesters (Backend API)
- Retrospective Agent
- Economist Testers
- Credibility Analyst
- Creative Filter
- Emergent Effects Researcher
- Aesthetic Consultant
- Gameplay Consultant
- PR Agent
- Conflict Resolution Agent
- Agent Coordination System (infrastructure)

### Infrastructure Missing
- No actual Rust project set up yet
- No agent coordination system implemented
- No way for agents to actually run or communicate
- No game code to work on

## Strategy for First Sprint

Since we want to start with the "world's simplest tasks" and let the system learn, Sprint 0 should focus on:

### Option A: Project Infrastructure (Recommended)
Set up the basic technical foundation so agents can actually work:

1. **Initialize Rust Project**
   - Create Cargo.toml
   - Set up basic workspace structure
   - Add initial dependencies
   - Create basic directory layout

2. **Implement Simple Agent Coordination Prototype**
   - Basic task queue (text file initially)
   - Simple agent "execution" (maybe just scripts that run)
   - Basic worktree management
   - Simple KB initialization

3. **Implement Manager Bot Prototype**
   - Very basic: can read tasks, assign to agents
   - Doesn't need full AI yet, just basic coordination
   - Can track task completion

4. **Test the Workflow**
   - Create a simple task (e.g., "add a README to the project")
   - Have Manager Bot assign it
   - Have Implementation Engineer "execute" it
   - Update KBs
   - Learn from the process

**Pros:** Establishes the foundation for everything else
**Cons:** Not building game features yet

### Option B: Agent Specification Sprint
Finish specifying all the agents before building anything:

1. **Specify remaining 16 agents**
   - Follow the template we established
   - Focus on the most critical ones first

2. **Refine existing specs**
   - Review the 3 we have
   - Adjust based on what we learned

3. **Create agent spec testing framework**
   - Can we validate agent specs?
   - Can we simulate agent behavior?

**Pros:** We have full agent workforce designed before building
**Cons:** No actual working system yet

### Option C: Hybrid (Balance)
Do a little of both:

1. **Set up basic Rust project** (1 day)
2. **Specify 3-5 more critical agents** (2 days)
3. **Implement a tiny prototype of the workflow** (2 days)

**Pros:** Making progress on both fronts
**Cons:** Spreads focus thin

## Recommendation

**Option A** - Focus on infrastructure first. Here's why:

1. **We need a working system** - All the agent specs in the world don't help if there's no system to run them
2. **Simple tasks are about the system, not the game** - The "world's simplest tasks" should be about setting up the coordination system, not game features
3. **We can learn by doing** - Actually implementing a simple version of Manager Bot will teach us what we actually need vs what we thought we needed
4. **Iterate on specs** - Once we have a working prototype, we can refine agent specs based on reality

## Proposed Sprint 0 Tasks

### Task 1: Initialize Rust Project
**Agent:** Implementation Engineer (Junior - this is straightforward)
**Time Estimate:** 2 hours
**Dependencies:** None
**Description:**
- Create `Cargo.toml` with workspace structure
- Set up directories: `src/`, `tests/`, `agents/`
- Add basic dependencies (maybe just basic ones for now)
- Create initial `main.rs` (just prints "hello world")
- Write basic test

**Exit Criteria:**
- `cargo build` succeeds
- `cargo test` passes
- `cargo run` prints "hello world"

### Task 2: Implement Basic Task Queue
**Agent:** Implementation Engineer (Mid - simple data structure)
**Time Estimate:** 3 hours
**Dependencies:** Task 1
**Description:**
- Create simple task queue (text file: `tasks/todo.txt`)
- Task format: JSON or simple text
- Functions to add task, mark complete, list tasks
- Basic tests

**Exit Criteria:**
- Can add tasks to queue
- Can mark tasks complete
- Can list pending tasks
- Tests pass

### Task 3: Implement Simple Manager Bot Prototype
**Agent:** Implementation Engineer (Senior - needs some logic)
**Time Estimate:** 4 hours
**Dependencies:** Task 2
**Description:**
- Create `manager_bot.rs` module
- Can read tasks from queue
- Can assign tasks to "agents" (just log assignments for now)
- Can track task completion
- Basic tests

**Exit Criteria:**
- Manager Bot can read pending tasks
- Manager Bot can assign task to agent (logged)
- Manager Bot can mark task complete
- Tests pass

### Task 4: Create Agent KB Structure
**Agent:** Implementation Engineer (Mid - file operations)
**Time Estimate:** 2 hours
**Dependencies:** Task 1
**Description:**
- Create `.agents/` directory structure
- Create KB files for Implementation Engineer and Manager Bot
- Implement basic KB read/write functions
- Tests

**Exit Criteria:**
- `.agents/implementation-engineer/kb/` exists with template files
- `.agents/manager-bot/kb/` exists with template files
- Can read and write KB entries
- Tests pass

### Task 5: End-to-End Workflow Test
**Agent:** Manager Bot (orchestration) + Implementation Engineer (execution)
**Time Estimate:** 3 hours
**Dependencies:** Tasks 1-4
**Description:**
- Create a test task: "Add a comment to main.rs"
- Manager Bot assigns task
- Implementation Engineer executes task
- Update KBs
- Verify task complete

**Exit Criteria:**
- Task assigned successfully
- Task completed successfully
- KBs updated
- Full workflow demonstrated

### Task 6: Sprint Retrospective
**Agent:** Manager Bot + All agents
**Time Estimate:** 2 hours
**Dependencies:** Task 5
**Description:**
- Review what worked
- Review what didn't work
- Update KBs with learnings
- Plan Sprint 1

**Exit Criteria:**
- Retrospective document created
- KBs updated
- Sprint 1 plan drafted

## Total Sprint Estimate
- **Total Time:** 16 hours (2 days)
- **Buffer:** 4 hours (25%)
- **Sprint Duration:** 3 days

## Sprint 0 Exit Criteria
- Rust project builds and tests pass
- Basic task queue functional
- Manager Bot prototype can assign and track tasks
- Agent KB structure created and functional
- End-to-end workflow demonstrated (task assigned → completed → KB updated)
- Retrospective completed
- Sprint 1 plan drafted

## Risks
- **Over-engineering:** Might try to build too much too soon. Mitigation: Keep it VERY simple.
- **Wrong foundation:** Might build the wrong thing. Mitigation: Prototype is intentionally simple, we'll refactor.
- **No actual agents:** We're simulating agents with code, not actually running AI agents. Mitigation: That's fine for Sprint 0 - we're building the infrastructure first.

## After Sprint 0
We'll have:
- A working (if simple) system
- Real data on what works and what doesn't
- Updated KBs with actual learnings
- A foundation to build on for Sprint 1

Sprint 1 could then focus on:
- Specifying more agents
- Building more sophisticated coordination
- OR starting on actual game features (ECS foundation, etc.)

## User Feedback Requirements

### Demos
- **Screenshots:** Capture terminal output, directory structures, file contents
- **Videos:** If possible, record workflow demonstrations (task assignment → execution → completion)
- **Interactive Demo:** User should be able to run the system themselves after Sprint 0

### Documentation
- Each task must include:
  - In-code documentation (comments, doc comments)
  - README for each module/component
  - Architecture diagram (even if simple ASCII art)
  - How-to guide for running the system
- Sprint deliverables:
  - Sprint 0 summary document
  - Architecture overview
  - User guide for the prototype system
  - Lessons learned document

## Updated Sprint 0 Tasks with Demo/Doc Requirements

### Task 1: Initialize Rust Project
**Agent:** Implementation Engineer (Junior)
**Time Estimate:** 2 hours
**Dependencies:** None
**Description:**
- Create `Cargo.toml` with workspace structure
- Set up directories: `src/`, `tests/`, `agents/`
- Add basic dependencies
- Create initial `main.rs` (just prints "hello world")
- Write basic test
- **Doc:** Create README.md explaining project structure
- **Demo:** Screenshot of `cargo build`, `cargo test`, `cargo run` output

**Exit Criteria:**
- `cargo build` succeeds
- `cargo test` passes
- `cargo run` prints "hello world"
- README.md created with project structure explanation
- Screenshots captured

### Task 2: Implement Basic Task Queue
**Agent:** Implementation Engineer (Mid)
**Time Estimate:** 3 hours
**Dependencies:** Task 1
**Description:**
- Create simple task queue (text file: `tasks/todo.txt`)
- Task format: JSON or simple text
- Functions to add task, mark complete, list tasks
- Basic tests
- **Doc:** Doc comments on all functions, README for task queue module
- **Demo:** Screenshot showing task queue operations (add, list, complete)

**Exit Criteria:**
- Can add tasks to queue
- Can mark tasks complete
- Can list pending tasks
- Tests pass
- Documentation complete
- Demo screenshots captured

### Task 3: Implement Simple Manager Bot Prototype
**Agent:** Implementation Engineer (Senior)
**Time Estimate:** 4 hours
**Dependencies:** Task 2
**Description:**
- Create `manager_bot.rs` module
- Can read tasks from queue
- Can assign tasks to "agents" (just log assignments for now)
- Can track task completion
- Basic tests
- **Doc:** Doc comments, architecture diagram (ASCII), usage guide
- **Demo:** Video or screenshot sequence showing Manager Bot assigning and tracking tasks

**Exit Criteria:**
- Manager Bot can read pending tasks
- Manager Bot can assign task to agent (logged)
- Manager Bot can mark task complete
- Tests pass
- Documentation complete
- Demo captured

### Task 4: Create Agent KB Structure
**Agent:** Implementation Engineer (Mid)
**Time Estimate:** 2 hours
**Dependencies:** Task 1
**Description:**
- Create `.agents/` directory structure
- Create KB files for Implementation Engineer and Manager Bot
- Implement basic KB read/write functions
- Tests
- **Doc:** KB structure README, API documentation for KB functions
- **Demo:** Screenshot of KB directory structure, example KB entry

**Exit Criteria:**
- `.agents/implementation-engineer/kb/` exists with template files
- `.agents/manager-bot/kb/` exists with template files
- Can read and write KB entries
- Tests pass
- Documentation complete
- Demo captured

### Task 5: End-to-End Workflow Test
**Agent:** Manager Bot (orchestration) + Implementation Engineer (execution)
**Time Estimate:** 3 hours
**Dependencies:** Tasks 1-4
**Description:**
- Create a test task: "Add a comment to main.rs"
- Manager Bot assigns task
- Implementation Engineer executes task
- Update KBs
- Verify task complete
- **Doc:** Workflow documentation, step-by-step guide
- **Demo:** Video of full workflow (recommended) or detailed screenshot sequence

**Exit Criteria:**
- Task assigned successfully
- Task completed successfully
- KBs updated
- Full workflow demonstrated
- Documentation complete
- Demo captured

### Task 6: Sprint Documentation and Demo
**Agent:** Manager Bot + Implementation Engineer
**Time Estimate:** 3 hours
**Dependencies:** Task 5
**Description:**
- Create Sprint 0 summary document
- Create architecture overview document
- Create user guide for the prototype system
- Create lessons learned document
- Compile all demos into a single presentation
- **Doc:** Comprehensive documentation suite
- **Demo:** Video walkthrough of entire system (if possible) or compiled demo document

**Exit Criteria:**
- Sprint 0 summary complete
- Architecture overview complete
- User guide complete
- Lessons learned complete
- All demos compiled and accessible
- User can run the system independently

### Task 7: Sprint Retrospective
**Agent:** Manager Bot + All agents
**Time Estimate:** 2 hours
**Dependencies:** Task 6
**Description:**
- Review what worked
- Review what didn't work
- Update KBs with learnings
- Plan Sprint 1
- **Doc:** Retrospective document
- **Demo:** Presentation of retrospective findings

**Exit Criteria:**
- Retrospective document created
- KBs updated
- Sprint 1 plan drafted
- Documentation complete

## Updated Total Sprint Estimate
- **Total Time:** 19 hours (added doc/demo tasks)
- **Buffer:** 5 hours (26%)
- **Sprint Duration:** 3 days

## Updated Sprint 0 Exit Criteria
- Rust project builds and tests pass
- Basic task queue functional
- Manager Bot prototype can assign and track tasks
- Agent KB structure created and functional
- End-to-end workflow demonstrated (task assigned → completed → KB updated)
- **Documentation complete** (READMEs, architecture docs, user guides)
- **Demos captured** (screenshots, videos, or compiled demo document)
- User can run the system independently
- Retrospective completed
- Sprint 1 plan drafted

## Demo Capture Strategy

### Screenshots (Frequent)
- Capture screenshots at key milestones in each task
- Include in-progress screenshots (not just final state)
- Terminal screenshots for command-line operations
- Directory tree screenshots for structure
- Code editor screenshots for code examples
- Compile into Sprint 0 demo document with explanations

### Videos (Occasional)
- Sprint end video: full walkthrough of completed system
- Record only at major milestones (end of sprint, after major features)
- Upload to wiki or provide link
- Not required for every task

### Interactive Demo
- User should be able to:
  - Clone the repo
  - Run `cargo build`
  - Run the prototype system
  - See the workflow in action
- User guide will walk them through this

## Documentation Deliverables

1. **Project README** - Overview, how to run, structure
2. **Module READMEs** - Each major component (task queue, manager bot, KB system)
3. **Architecture Document** - Mermaid diagrams for component relationships, data flow
4. **User Guide** - How to use the prototype system
5. **API Documentation** - Doc comments compiled if possible
6. **Sprint 0 Summary** - What was built, what was learned
7. **Lessons Learned** - What worked, what didn't, what to change
8. **Demo Document** - All screenshots/videos compiled with explanations

## Diagram Format
- Use **Mermaid** for diagrams (supported in GitHub markdown, many viewers)
- Alternative: SVG if Mermaid insufficient
- Text-based diagrams (ASCII) as fallback if needed
- Examples: flowcharts, sequence diagrams, architecture diagrams

## Questions for User

1. Does this updated plan with demos and docs look good?
2. Screenshots (frequent) + sprint-end video (occasional) - is this the right balance?
3. Mermaid diagrams for architecture - is this format acceptable?
4. Is the additional time for docs/demos acceptable?
5. Any other feedback or adjustments?
