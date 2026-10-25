# Game2

A medieval fantasy simulation game with emergent storytelling, complex psych/sociological models, and AI-driven development.

## Project Structure

```
game2/
├── Cargo.toml              # Workspace configuration
├── game2-core/             # Core game engine
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs        # Entry point
│   ├── tests/             # Integration tests
│   └── agents/            # Agent knowledge bases (runtime state)
├── wiki/                  # Knowledge base and documentation
│   ├── README.md
│   ├── backlog.md
│   ├── agents/           # Agent specifications
│   └── ...
└── .agents/              # Agent runtime knowledge bases (git tracked)
```

## Getting Started

### Prerequisites
- Rust toolchain (install via rustup)
- Cargo (comes with Rust)

### Build
```bash
cargo build
```

### Run
```bash
cargo run --package game2-core
```

### Test
```bash
cargo test
```

Note: On Windows, some tests may be blocked by Application Control policies. This is a known issue with the development environment.

## Current Status

**Sprint 0** - Infrastructure Setup (In Progress)

### Completed
- Rust workspace initialized
- Basic project structure created
- Hello world build and run successful

### In Progress
- Task queue implementation
- Manager Bot prototype
- Agent KB structure

## Development Philosophy

This project uses an AI-driven development approach with specialized agents:
- Implementation Engineer
- Manager Bot
- Fun Inspector
- And 17+ more specialized roles

See `wiki/ai-agent-workforce.md` for full details.

## License

TBD
