# The authoritative village server: one Game, WebSocket seats, claims.
# The sim needs no assets - it renders nothing - so the image is just the binary.
FROM rust:1.83-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
# every workspace member's manifest has to be there for cargo to load the
# workspace, but only the sim crate is built: the server draws nothing
COPY game2-core ./game2-core
COPY game2-client ./game2-client
COPY game2-web ./game2-web
# data the sim compiles in (include_str!): the landform catalog
COPY data/landforms ./data/landforms
RUN cargo build --release -p game2-core --bin game_server

FROM debian:bookworm-slim
RUN useradd -m game
COPY --from=build /app/target/release/game_server /usr/local/bin/
USER game
EXPOSE 8787
# one process, one world; ~1-5 human seats plus however many agents connect
CMD ["game_server", "--port", "8787", "--idle-secs", "300"]
