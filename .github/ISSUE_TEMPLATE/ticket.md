---
name: Ticket
description: A unit of work an agent (or human) can pick up end to end
about: Fill every section. An issue is only safe for an agent to take when the last box is checked - that check applies the agent-ready label.
title: ""
labels: []
body:
  - type: textarea
    id: summary
    attributes:
      label: Summary
      description: What should change, and why. One paragraph.
    validations:
      required: true
  - type: textarea
    id: acceptance
    attributes:
      label: Acceptance criteria
      description: Concrete, checkable "done" statements. Not "make it better" - what must be true when finished.
      placeholder: |
        - Player can see X in the inspector
        - Villagers do Y on market day
        - cargo test -p game2-core --lib passes
    validations:
      required: true
  - type: textarea
    id: verify
    attributes:
      label: How to verify
      description: The exact commands and dumps that prove it works. Name real repo tooling (cargo test targets, game_ui --dump-frame tick, GAME2_BENCH, etc.) and the tick/frame where the feature is visible.
      placeholder: |
        cargo test -p game2-core --lib <filter>
        cargo run -p game2-client --bin game_ui -- --dump-frame 600 --zoom 2 --focus <kind>
    validations:
      required: true
  - type: textarea
    id: ui-surface
    attributes:
      label: Player / agent surface
      description: What the player sees or does, and whether bots should reach it too. If nothing player-facing changes, say "sim only".
      placeholder: |
        New CtxAct button "Mend Hall" under Build; AgentCmd::Mend for bots
        Sim only - no UI
    validations:
      required: true
  - type: input
    id: depends
    attributes:
      label: Depends on
      description: Other issues, PRs, or docs this needs first. "None" is a fine answer.
      placeholder: "None"
    validations:
      required: false
  - type: checkboxes
    id: ready
    attributes:
      label: Ready for agents
      description: Only check this when every section above is filled and correct. Checking it means an agent may claim this ticket unsupervised and merge its PR.
      options:
        - label: I confirm the acceptance criteria and verify steps are complete, and this ticket is safe for an agent to take end to end.
          required: true
