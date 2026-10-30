# The authoritative village server: one Game, WebSocket seats, claims.
# The sim needs no assets - it renders nothing - so the image is just the binary.
FROM rust:1.83-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY game2-core ./game2-core
RUN cargo build --release --bin game_server

FROM debian:bookworm-slim
RUN useradd -m game
COPY --from=build /app/target/release/game_server /usr/local/bin/
USER game
EXPOSE 8787
# one process, one world; ~1-5 human seats plus however many agents connect
CMD ["game_server", "--port", "8787", "--idle-secs", "300"]
