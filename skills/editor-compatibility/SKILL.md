---
name: editor-compatibility
description: Which figma-mcp-rs tools work in Figma, FigJam, and Slides. Use when a tool fails with "not available in FigJam" or "only available in FigJam", when planning edits to a FigJam or Slides file, or when choosing tools for a file whose editor type is unclear.
---

# Editor Compatibility

The plugin runs in every editor (figma, figjam, slides, dev). Tools check
the editor type at runtime and fail fast with a clear error — this matrix
tells you which call to make up front.

## Works everywhere

- Reads: get_metadata, get_pages, get_document, get_design_context,
  get_selection, get_node, get_nodes_info, search_nodes, scan_text_nodes,
  scan_nodes_by_types, get_viewport, get_fonts, get_styles,
  get_variable_defs, get_local_components, get_annotations, export_tokens.
- Export: get_screenshot, save_screenshots, export_frames_to_pdf.
- Generic writes: create_frame, create_rectangle, create_ellipse,
  create_text, import_image, create_section, set_text, set_fills,
  set_strokes, set_opacity, set_corner_radius, set_visible, lock_nodes,
  unlock_nodes, rotate_nodes, reorder_nodes, set_blend_mode,
  set_constraints, move_nodes, resize_nodes, rename_node, clone_node,
  reparent_nodes, group_nodes, ungroup_nodes, batch_rename_nodes,
  find_replace_text, delete_nodes.
- Pages: add_page, delete_page, rename_page, navigate_to_page.
- `use_figma` always works — it runs Plugin API JS in whatever editor is
  open.

## FigJam only

create_sticky, create_stickies, create_connector, create_shape_with_text,
create_table, create_code_block, auto_arrange, get_board_contents.
Calling these in a Figma design file errors "only available in FigJam".
Workflows: see the figjam-boards skill.

## Figma only

Calling these in a FigJam file errors "not available in FigJam":
- Styles: create_paint_style, create_text_style, create_effect_style,
  create_grid_style, update_paint_style, delete_style, apply_style_to_node,
  set_effects.
- Variables: create_variable_collection, add_variable_mode,
  create_variable, set_variable_value, bind_variable_to_node,
  delete_variable.
- Components: create_component, swap_component, detach_instance.
- Prototype: set_reactions, remove_reactions.
- Layout: set_auto_layout.

## Slides

No editor-specific tools. Generic node tools and `use_figma` apply; decks
and slide properties are reached via `use_figma`.

## When a tool errors on editor type

1. Check the open file with `get_metadata`.
2. Substitute a supported tool from the matrix above, or
3. Fall back to `use_figma` — Plugin API JS often reaches what named tools
   don't wrap (styled text ranges, widget properties, slide structure).
   Remember: only the `return` value comes back and it must be
   JSON-serializable.
