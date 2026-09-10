---
name: figjam-boards
description: Build and read FigJam boards with figma-mcp-rs — sticky grids, shapes, tables, code blocks, connectors, and auto-arrange. Use whenever the open file is a FigJam board, the user mentions stickies, whiteboards, retros, or flow maps in FigJam, or a Figma-only tool just failed in a FigJam file.
---

# FigJam Boards

Eight tools exist only in FigJam. Generic tools (reads, set_text, move,
resize, delete) also work here — see the editor-compatibility skill for the
full matrix.

## Read the board first

- `get_board_contents()` → `{ nodes, connections, total }`: every STICKY,
  SHAPE_WITH_TEXT, TEXT, CODE_BLOCK, and TABLE with its text / code /
  shapeType, plus connector `from`/`to` endpoint IDs. Pass
  `includeConnections: false` to skip connectors. This is the FigJam
  equivalent of `get_document`.
- `get_metadata()` / `get_pages()` for file context.
- `get_screenshot()` on node IDs to visually verify results.

## Create content

Sticky notes:
- One: `create_sticky(text, x?, y?, name?, fillColor?)` — fillColor hex
  e.g. `#FFD54F`.
- Many (max 200 per call): `create_stickies(items, startX?, startY?,
  columns?, spacing?)`. Grid defaults: startX 0, startY 0, columns 5,
  spacing 40. An item with BOTH `x` and `y` is placed exactly; otherwise it
  takes the next row-major grid slot. Per-item overrides: text, x, y, name,
  fillColor.

Shapes with text — `create_shape_with_text(text, shapeType?, x?, y?, width?,
height?, fillColor?)`:
- `shapeType` defaults to `ROUNDED_RECTANGLE`; 29 validated types, notably
  flowchart set (DIAMOND, HEXAGON, PARALLELOGRAM_RIGHT/LEFT, CHEVRON,
  DOCUMENT_SINGLE/MULTIPLE, MANUAL_INPUT, PREDEFINED_PROCESS, OR,
  SUMMING_JUNCTION, INTERNAL_STORAGE, TRAPEZOID), eng set (ENG_DATABASE,
  ENG_QUEUE, ENG_FILE, ENG_FOLDER), basics (SQUARE, ELLIPSE, TRIANGLE_UP,
  TRIANGLE_DOWN, PENTAGON, OCTAGON, STAR, PLUS, ARROW_LEFT, ARROW_RIGHT,
  SHIELD, SPEECH_BUBBLE).
- `width`/`height` apply only when BOTH are given.

Tables — `create_table(rows, columns, cells?, x?, y?, name?)`:
- rows/columns 1–50; `cells` is row-major string arrays, ragged OK — pass
  only the text you have.

Code blocks — `create_code_block(code, language?, x?, y?)`:
- `language` one of TYPESCRIPT, JAVASCRIPT, HTML, CSS, JSON, GRAPHQL,
  PYTHON, GO, SQL, SWIFT, KOTLIN, RUST, BASH, RUBY, CPP (default
  PLAINTEXT).

## Connect

`create_connector(text?, startNodeId?, endNodeId?)` — endpoints optional;
wire them to IDs returned by the create calls. Node IDs use colon format
(`4029:12345`), never hyphens.

## Edit and tidy

- `set_text` works on STICKY nodes, not just TEXT.
- `set_fills`, `move_nodes`, `resize_nodes`, `rename_node`, `clone_node`,
  `group_nodes`, `delete_nodes` all work in FigJam.
- `auto_arrange(nodeIds?, layout?, spacing?, columns?, startX?, startY?)`:
  omit `nodeIds` to arrange every child of the current page. `layout` is
  grid (default, columns default ceil(sqrt(n))), row, or column. Default
  spacing 40.

## Workflow

1. `get_board_contents()` — see what is already there.
2. Create in bulk (`create_stickies`, shapes); capture returned node IDs.
3. `create_connector` between those IDs for relationships.
4. `auto_arrange` if placement was left to defaults.
5. `get_screenshot` to verify. All writes undo with Ctrl/Cmd+Z.

Example — retro board: three `create_stickies` batches (per-item fillColor
red / green / blue) with startX 0 / 600 / 1200 and columns 1 → three
labeled vertical columns; then `create_connector` linking action items back
to their source stickies.
