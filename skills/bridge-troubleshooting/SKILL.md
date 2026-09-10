---
name: bridge-troubleshooting
description: Diagnose figma-mcp-rs connection problems. Use when tools fail with "plugin not connected", requests time out, the port is already in use, or the Figma plugin cannot reach the MCP server. Covers installing the companion plugin, port 1994, FigJam limits, and running multiple MCP clients.
---

# Bridge Troubleshooting

figma-mcp-rs talks to Figma through a companion plugin over a local websocket
(default ws://127.0.0.1:1994). Every one of the 75 tools needs that plugin
running in Figma Desktop. Use this checklist when calls fail.

## Error: "plugin not connected"

The MCP server is running but no plugin is attached.

1. Open Figma Desktop (not the browser) and open the file to work on.
2. If the plugin is not installed yet:
   - Download `plugin.zip` from this repo's releases (or run `make plugin` and use `plugin/dist`).
   - In Figma Desktop: Plugins → Development → "Import plugin from manifest"
   - Select `manifest.json` from the extracted plugin.
3. Run the plugin (Plugins → Development → figma-mcp-rs) and keep its window open.
4. Confirm the plugin UI shows a connected state, then retry the tool call.

The plugin window must stay open; closing it disconnects the bridge.

## Error: "request timed out"

The plugin is connected but the operation exceeded its deadline (30s default,
60s for `get_document`).

- Very large pages: prefer `get_design_context` with `detail="minimal"` or
  `"compact"` and a small `depth` instead of `get_document`.
- Scan tools (`scan_text_nodes`, `scan_nodes_by_types`) on huge trees: target a
  smaller subtree `nodeId`.
- Long exports (`export_frames_to_pdf`, `save_screenshots`) send progress frames
  that extend the deadline; if they still time out, export fewer frames per call.
- If Figma itself is frozen or showing a modal dialog, dismiss it and retry.

## Error: port already in use

Another process holds port 1994.

- If it is another figma-mcp-rs instance, that is normal: instances elect a
  leader on the port and the rest become followers that forward requests. No
  action needed.
- If an unrelated app owns the port, start the server with `--port <other>` and
  set the same host/port in the plugin UI.

## FigJam files

`create_sticky` and `create_connector` are FigJam-only. Figma-design-only tools
(components, variables, styles, auto-layout, annotations) return a clear
"not supported in FigJam" error when the open file is a FigJam board — open a
Figma design file instead.

## Multiple MCP clients (Claude Code + Cursor, etc.)

Running several clients at once is supported. The first server process binds the
port and becomes the leader that owns the Figma websocket; later processes
detect the healthy leader and forward their requests to it. If the leader exits,
a follower takes over the port on its next election attempt (3–5s).

## Quick checklist

1. Figma Desktop open, target file open.
2. Companion plugin imported and running, window open.
3. Plugin host/port matches the server (default 127.0.0.1:1994).
4. Retry the failed call; write operations are undoable with Ctrl/Cmd+Z.
