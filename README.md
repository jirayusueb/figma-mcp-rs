# figma-mcp-rs

Figma/FigJam MCP server in Rust — free, no rate limits. Rust port of [vkhanhqui/figma-mcp-go](https://github.com/vkhanhqui/figma-mcp-go) with a rewritten SolidJS plugin and FigJam support.

Open-source MCP server with full read/write access to **Figma and FigJam** via plugin — no REST API, no rate limits. Turn text into designs and designs into real code. Works with Claude, Cursor, GitHub Copilot, and any MCP-compatible AI tool.

[![npm](https://img.shields.io/npm/v/figma-mcp-rs)](https://www.npmjs.com/package/figma-mcp-rs)
[![license](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

**Highlights**
- No Figma API token required
- No rate limits — free plan friendly
- **Read and Write** live Figma/FigJam data via plugin bridge — 76 tools total (73 from the Go reference, `create_sticky` / `create_connector` for FigJam, and `use_figma` for everything else)
- Full design automation — styles, variables, components, prototypes, and content
- Design strategies included — 12 MCP prompts built in
- Native binary, fast startup, low memory
- Speaks MCP spec revisions up to 2025-11-25 (newest in rmcp 3.2); negotiates down for older clients

## Why this exists

The official Figma MCP servers go through the REST API, which is rate limited per plan:

| Figma plan | REST budget |
|------------|-------------|
| Starter | 6 calls/month |
| Professional | 200 calls/day |
| Enterprise | 600 calls/day |

This server never calls the REST API. It talks to a plugin running inside Figma Desktop over a local WebSocket, so every read and write is unlimited and free — and unlike REST, the plugin can *write* to the open file.

## Performance

Measured on Apple M4 Pro: ~6 MB RSS idle, ~7 MB with a plugin connected during tool round trips; startup < 10 ms. The Go reference runs ~15–25 MB.

## Installation & Setup

### 1. Configure your AI tool

**Claude Code CLI**
```bash
claude mcp add -s project figma-mcp-rs -- npx -y figma-mcp-rs@latest
```

Or build from source:
```bash
make build   # cargo build --release + plugin build
claude mcp add -s project figma-mcp-rs -- /path/to/figma-mcp-rs/target/release/figma-mcp-rs
```

**Codex CLI**
```bash
codex mcp add figma-mcp-rs -- npx -y figma-mcp-rs@latest
```

**Claude Code plugin marketplace** (also installs the 15 skills)
```
/plugin marketplace add <this-repo>
/plugin install figma-mcp-rs@figma-mcp-rs
```

**.mcp.json** (Claude and other MCP-compatible tools)
```json
{
  "mcpServers": {
    "figma-mcp-rs": {
      "command": "npx",
      "args": ["-y", "figma-mcp-rs"]
    }
  }
}
```

**.vscode/mcp.json** (Cursor / VS Code / GitHub Copilot)
```json
{
  "servers": {
    "figma-mcp-rs": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "figma-mcp-rs"]
    }
  }
}
```

Server flags: `--ip` (default `127.0.0.1`), `--port` (default `1998`).

### 2. Install the Figma plugin

1. In Figma Desktop: **Plugins → Development → Import plugin from manifest**
2. Select `manifest.json` from the release `plugin.zip` (or `plugin/manifest.json` after `make build-ts`)
3. Run the plugin inside any **Figma or FigJam** file — it connects to the server over `ws://127.0.0.1:1998` (host/port configurable in the plugin UI)

Multiple server instances are safe: the first to bind the port becomes leader and owns the plugin connection; others become followers that proxy tool calls and take over automatically if the leader dies.

## FigJam & Slides support

The plugin loads in every editor (`editorType: ["figma", "figjam", "slides", "dev"]`):
- **Reads** work in both: `get_document`/`get_design_context` serialize FigJam node types (STICKER, CONNECTOR, MARKER, WIDGET, EMBED, MEDIA…)
- **FigJam-only writes**: `create_sticky`, `create_connector`
- **Figma-only tools** (styles, variables, components, prototype reactions, auto-layout) return a clear error in FigJam files
- **Slides, FigJam tables / shapes-with-text / code blocks / labels**: no named tool — use `use_figma`

## Available Tools

### Escape hatch

| Tool | Description |
|------|-------------|
| `use_figma` | Run Plugin API JavaScript inside the file. The code is the body of `async () => { … }`: top-level `await` works and only the `return` value comes back (console.log is discarded; the value must be JSON-serializable, so return IDs/counts/names, not node objects). Covers what the named tools don't wrap: Slides, FigJam tables/shapes/code blocks/labels, component variants and properties, vector networks, styled text ranges, variable scopes, bulk edits in one round-trip. |

### Write — Create

| Tool | Description |
|------|-------------|
| `create_frame` | Create a frame with optional auto-layout, fill, and parent |
| `create_rectangle` | Create a rectangle with optional fill and corner radius |
| `create_ellipse` | Create an ellipse or circle |
| `create_text` | Create a text node (font loaded automatically) |
| `import_image` | Decode base64 image and place it as a rectangle fill |
| `create_component` | Convert an existing FRAME node into a reusable component |
| `create_section` | Create a Figma Section node to organise frames on a page |

### Write — Modify

| Tool | Description |
|------|-------------|
| `set_text` | Update text content of an existing TEXT or STICKER node |
| `set_fills` | Set solid fill color (hex) on a node |
| `set_strokes` | Set solid stroke color and weight on a node |
| `set_opacity` | Set opacity of one or more nodes (0 = transparent, 1 = opaque) |
| `set_corner_radius` | Set corner radius — uniform or per-corner |
| `set_auto_layout` | Set or update auto-layout (flex) properties on a frame (Figma only) |
| `set_visible` | Show or hide one or more nodes |
| `lock_nodes` / `unlock_nodes` | Lock/unlock one or more nodes |
| `rotate_nodes` | Set absolute rotation in degrees on one or more nodes |
| `reorder_nodes` | Change z-order: `bringToFront`, `sendToBack`, `bringForward`, `sendBackward` |
| `set_blend_mode` | Set blend mode (MULTIPLY, SCREEN, OVERLAY, …) |
| `set_constraints` | Set responsive constraints on one or more nodes |
| `move_nodes` | Move nodes to an absolute x/y position |
| `resize_nodes` | Resize nodes by width and/or height |
| `rename_node` | Rename a node |
| `clone_node` | Clone a node, optionally repositioning or reparenting |
| `reparent_nodes` | Move nodes to a different parent frame, group, or section |
| `group_nodes` / `ungroup_nodes` | Group nodes into a GROUP / dissolve groups |
| `batch_rename_nodes` | Bulk rename via find/replace, regex, or prefix/suffix |
| `find_replace_text` | Find and replace text across TEXT nodes in a subtree or page; supports regex |
| `delete_nodes` | Delete one or more nodes permanently |

### Write — Prototype (Figma only)

| Tool | Description |
|------|-------------|
| `set_reactions` | Set prototype reactions (triggers + actions); mode `replace` or `append` |
| `remove_reactions` | Remove all or specific reactions by index |

### Write — Styles (Figma only)

| Tool | Description |
|------|-------------|
| `set_effects` | Apply drop shadow / blur effects directly on a node |
| `create_paint_style` / `create_text_style` / `create_effect_style` / `create_grid_style` | Create named local styles |
| `update_paint_style` | Rename or recolor an existing paint style |
| `apply_style_to_node` | Apply an existing local style to a node |
| `delete_style` | Delete any style by ID |

### Write — Variables (Figma only)

| Tool | Description |
|------|-------------|
| `create_variable_collection` | Create a new local variable collection with an optional initial mode |
| `add_variable_mode` | Add a new mode to an existing collection (e.g. Light/Dark) |
| `create_variable` | Create a variable (COLOR/FLOAT/STRING/BOOLEAN) in a collection |
| `set_variable_value` | Set a variable's value for a specific mode |
| `bind_variable_to_node` | Bind a variable to a node property (fills, strokes, size, spacing, …) |
| `delete_variable` | Delete a variable or an entire collection |

### Write — Pages

| Tool | Description |
|------|-------------|
| `add_page` | Add a new page (optional name and index) |
| `delete_page` | Delete a page by ID or name |
| `rename_page` | Rename a page by ID or current name |

### Write — Components & Navigation

| Tool | Description |
|------|-------------|
| `navigate_to_page` | Switch the active page by ID or name |
| `swap_component` | Swap the main component of an INSTANCE node |
| `detach_instance` | Detach component instances to plain frames |

### Read — Document & Selection

| Tool | Description |
|------|-------------|
| `get_document` | Full current page tree |
| `get_metadata` | File name, pages, current page |
| `get_pages` | All pages (IDs + names) — lightweight |
| `get_selection` | Currently selected nodes |
| `get_node` / `get_nodes_info` | One node / multiple nodes by ID |
| `get_design_context` | Depth-limited tree with `detail` level (minimal/compact/full) |
| `search_nodes` | Find nodes by name substring and/or type |
| `scan_text_nodes` | All text nodes in a subtree |
| `scan_nodes_by_types` | Nodes matching a type list |
| `get_viewport` | Current viewport center, zoom, visible bounds |
| `get_reactions` | Prototype reactions on a node |
| `get_fonts` | Fonts used on the current page, sorted by frequency |

### Read — Styles & Variables

| Tool | Description |
|------|-------------|
| `get_styles` | Paint, text, effect, and grid styles |
| `get_variable_defs` | Variable collections and values |
| `get_local_components` | All components + component sets |
| `get_annotations` | Dev-mode annotations |
| `export_tokens` | Design tokens (variables + paint styles) as JSON or CSS |

### Export

| Tool | Description |
|------|-------------|
| `get_screenshot` | Base64 image export of any node |
| `save_screenshots` | Export images to disk (server-side write) |
| `export_frames_to_pdf` | Multiple frames as one multi-page PDF (server-side merge) |

### FigJam (new)

| Tool | Description |
|------|-------------|
| `create_sticky` | Create a sticky note with text (FigJam only) |
| `create_connector` | Create a connector, optionally between two nodes (FigJam only) |

### MCP Prompts (12)

`read_design_strategy`, `design_strategy`, `text_replacement_strategy`, `annotation_conversion_strategy`, `swap_overrides_instances`, `reaction_to_connector_strategy`, `style_audit_strategy`, `bulk_rename_strategy`, `design_token_generation_strategy`, `generate_color_palette`, `generate_type_scale`, `generate_component_variants`

### Claude Skills (15)

`skills/` ships the 12 prompts as Claude Code skills plus `bridge-troubleshooting` (connection diagnostics), `figjam-boards` (FigJam stickies, shapes, tables, connectors, auto-arrange), and `editor-compatibility` (Figma/FigJam/Slides tool matrix). Installed automatically via the plugin marketplace; otherwise copy `skills/` into your project's `.claude/skills/`.

## Development

```bash
make test      # cargo test + plugin bun test
make build     # release binary + plugin dist
```

- Server: Rust (tokio, axum, rmcp). Plugin: TypeScript + SolidJS, built with Vite (UI inlined to `dist/index.html`, core IIFE `dist/code.js`).
- Toolchain: Bun 1.4.2, TypeScript 7.
- The wire protocol (bridge WebSocket frames, follower `/rpc`) is kept compatible with the Go reference, so this server also works with the original [figma-mcp-go](https://github.com/vkhanhqui/figma-mcp-go) plugin.

## Credits

Fork of the architecture and tool design of [figma-mcp-go](https://github.com/vkhanhqui/figma-mcp-go) by [vkhanhqui](https://github.com/vkhanhqui) — ported to Rust with a SolidJS plugin and FigJam support.

## Related Projects

- [vkhanhqui/figma-mcp-go](https://github.com/vkhanhqui/figma-mcp-go) — the Go reference this port follows
- [alvinindra/figma-mcp-rust](https://github.com/alvinindra/figma-mcp-rust) — another Rust port of the same server
- [grab/cursor-talk-to-figma-mcp](https://github.com/grab/cursor-talk-to-figma-mcp) — the plugin-bridge approach this lineage comes from
- [gethopp/figma-mcp-bridge](https://github.com/gethopp/figma-mcp-bridge) — alternative plugin bridge

## License

MIT
