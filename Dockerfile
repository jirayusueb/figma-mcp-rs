# Build the MCP server binary. plugin/dist is committed and embedded via
# include_str! at compile time, so no Node/Bun stage is needed.
FROM rust:1-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY plugin/manifest.json plugin/manifest.json
COPY plugin/dist ./plugin/dist
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked \
    && cp target/release/figma-mcp-rs /usr/local/bin/figma-mcp-rs

FROM debian:bookworm-slim
COPY --from=build /usr/local/bin/figma-mcp-rs /usr/local/bin/figma-mcp-rs
# Plugin WebSocket bridge + leader election port.
EXPOSE 1998
# 0.0.0.0 so the mapped port is reachable from the host running Figma Desktop.
ENTRYPOINT ["/usr/local/bin/figma-mcp-rs"]
CMD ["--ip", "0.0.0.0"]
