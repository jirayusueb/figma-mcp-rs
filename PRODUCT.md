# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Developers and designers who drive Figma/FigJam through AI coding tools (Claude Code, Codex CLI, Cursor, GitHub Copilot, any MCP client). They configure the bridge once from inside Figma Desktop, then work in their editor while the AI reads and writes the open file. Plugin-UI situation: a narrow panel inside the Figma Desktop window, opened occasionally to check connection status or re-point host/port.

## Product Purpose

figma-mcp-rs is an open-source MCP server (Rust) with full read/write access to Figma and FigJam via a plugin bridge over a local WebSocket — no REST API, no rate limits, no API token. Success: every tool call round-trips through the plugin instantly and reliably; the user never hits a Figma REST budget.

## Positioning

The only mainstream path to unlimited free writes to the open Figma/FigJam file: a native binary with a bundled self-installing plugin, leader election for multiple instances, and 84 tools including FigJam-specific writes — while official servers pay per REST call.

## Operating Context

- Figma Desktop running a development-imported plugin (`~/.figma-mcp-rs/plugin/manifest.json`), plugin UI is a small web iframe (`dist/index.html`, single-file build).
- Server on `ws://127.0.0.1:1998` (configurable host/port, persisted via `figma.clientStorage`).
- Works in Figma, FigJam, Slides, Dev editors; FigJam writes via eight named tools, Figma-only tools error clearly in FigJam.
- Distributed as npx package, native binaries, Docker image, and Claude Code plugin (ships 15 skills).

## Capabilities and Constraints

- 84 tools; escape hatches `use_figma` / `execute_code` for anything unwrapped.
- Wire protocol kept compatible with the Go reference plugin.
- Plugin UI: TypeScript + SolidJS + Tailwind v4, shadcn-solid components (Kobalte + cva), single-file Vite build inlined into the Rust binary.
- Design system implemented as shadcn-solid themed to the mirrored world (see Brand Commitments).

## Brand Commitments

- The product's visual world is the **"Industrial Brutalist" v2** design system, mirrored exactly from the user's own source system (authorized copy of values, names, both themes, grain overlay, and WCAG-tuned colors).
- Binding: Space Grotesk (display) + JetBrains Mono (mono), 4px spacing scale, warm-black ground with amber accent, radius 0, dark default + light theme, WCAG AA body-text contrast.

## Evidence on Hand

- README.md: full tool catalog, install paths, performance numbers (~6 MB RSS idle, <10 ms startup), Figma REST budget table.
- The mirrored system's source stylesheets (captured this session; owner-supplied). DESIGN.md in this repo is now the system of record.
- No marketing site, screenshots, testimonials, or customer evidence exists; future surfaces must not fabricate any.

## Product Principles

1. The bridge always works: status and recovery paths are first-class UI, not error toasts.
2. Free and unlimited is the product claim; every surface may state it factually (REST budget table), never invent numbers.
3. Native-binary restraint: small, fast, boring — the UI shares that temperament (dense, mono-labeled, zero ornament).
4. Dogfooding: the plugin UI itself proves the design system and the MCP tooling.
5. Figma/FigJam/Slides compatibility is a promise; capability boundaries are communicated, never hidden.

## Accessibility & Inclusion

WCAG AA on body text is a binding token constraint of the mirrored system (source tokens were explicitly bumped for AA; preserve those values). Focus-visible amber outline and reduced-motion guard are part of the base layer.
