# PR #180 — One build away from a browser tab

https://github.com/garysieling/game2/pull/180

## What changed
- `cargo build -p game2-web --target wasm32-unknown-unknown` passes — `game2-core` compiles for wasm32, `game2-web` links
- `game2-web` joined workspace members; `.cargo/config.toml` adds `getrandom_backend="wasm_js"` (wasm-scoped); `game2-web/Cargo.toml` gets a target-scoped `getrandom = { version = "0.3", features = ["wasm_js"] }`
- `WasmGame` exports: `new()` / `scenario(kind)` / `tick()` / `scene()` (serialized UiModel) / `day()` / `telemetry()`

## Quality gates
- `cargo build -p game2-web --target wasm32-unknown-unknown`: links
- `cargo test --lib -p game2-core`: 450/450 (native unaffected)

## Deferred
- Canvas/ws browser shims (per the scaffold note); IndexedDB persistence; audio
