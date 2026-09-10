//! MCP server: tool definitions and relay helpers.
//!
//! One module per reference Go tool file. Each module owns its args structs
//! and handler fns; this file wires them into the rmcp tool router, mirroring
//! the tool names/descriptions from figma-mcp-go's internal/tools_*.go verbatim.

pub mod figjam;
pub mod read_document;
pub mod read_export;
pub mod read_styles;
pub mod status;
pub mod use_figma;
pub mod write_components;
pub mod write_create;
pub mod write_modify;
pub mod write_page;
pub mod write_prototype;
pub mod write_styles;
pub mod write_variables;

use std::sync::Arc;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo};
use rmcp::{ErrorData, prompt_handler, tool, tool_handler, tool_router, ServerHandler};
use serde::Serialize;

use crate::node::Node;
use crate::types::BridgeResponse;

pub(crate) type McpError = ErrorData;

#[derive(Clone)]
pub struct FigmaServer {
    pub node: Arc<Node>,
}

impl FigmaServer {
    pub fn new(node: Arc<Node>) -> Self {
        Self { node }
    }
}

/// Relay a parsed args struct to the plugin.
///
/// Extracts `nodeIds` (string array) or `nodeId` (single string) from the
/// serialized args into the request's `nodeIds` field and keeps the rest in
/// `params` — matching the reference handlers.
pub(crate) async fn relay<T: Serialize>(
    node: &Node,
    tool: &'static str,
    args: &T,
) -> Result<CallToolResult, McpError> {
    let value = serde_json::to_value(args)
        .map_err(|e| McpError::internal_error(format!("marshal args: {e}"), None))?;
    let resp = relay_value(node, tool, value).await;
    render(resp)
}

/// Relay pre-built params (server-side tools that don't pass args structs).
pub(crate) async fn relay_params(
    node: &Node,
    tool: &'static str,
    params: serde_json::Value,
) -> Result<CallToolResult, McpError> {
    let resp = relay_value(node, tool, params).await;
    render(resp)
}

async fn relay_value(node: &Node, tool: &str, value: serde_json::Value) -> Result<BridgeResponse, String> {
    let node_ids = match value.get("nodeIds") {
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        _ => match value.get("nodeId").and_then(|v| v.as_str()) {
            Some(id) => vec![id.to_string()],
            None => vec![],
        },
    };
    node.send(tool, node_ids, value).await
}

/// Convert a BridgeResponse into an MCP tool result (mirrors renderResponse).
fn render(resp: Result<BridgeResponse, String>) -> Result<CallToolResult, McpError> {
    match resp {
        Err(err) => Ok(CallToolResult::error(vec![ContentBlock::text(err)])),
        Ok(resp) => {
            if !resp.error_text().is_empty() {
                return Ok(CallToolResult::error(vec![ContentBlock::text(
                    resp.error_text().to_string(),
                )]));
            }
            let text = serde_json::to_string(&resp.data.unwrap_or(serde_json::Value::Null))
                .unwrap_or_else(|e| format!("{{\"error\":\"marshal response: {e}\"}}"));
            Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
        }
    }
}

// ── Tool router ──────────────────────────────────────────────────────────────
// Each entry mirrors an `s.AddTool(mcp.NewTool(...))` block in the reference;
// descriptions are ported verbatim (they are LLM-facing prompt engineering).

#[tool_router]
impl FigmaServer {
    #[tool(description = r##"Check whether the Figma plugin bridge is connected and get setup instructions when it is not. Call this first if any tool fails with 'plugin not connected'. Does not touch the Figma document."##)]
    async fn get_status(&self) -> Result<CallToolResult, McpError> {
        status::get_status(std::sync::Arc::clone(&self.node)).await
    }

    #[tool(description = r##"Get the full node tree of the current page (not the whole file — only the active page). Returns all nodes recursively and can be very large. Prefer get_design_context for exploration or when token efficiency matters."##)]
    async fn get_document(&self) -> Result<CallToolResult, McpError> {
        read_document::get_document(std::sync::Arc::clone(&self.node)).await
    }

    #[tool(description = r##"List all pages in the document with their IDs and names. Lightweight alternative to get_document."##)]
    async fn get_pages(&self) -> Result<CallToolResult, McpError> {
        read_document::get_pages(std::sync::Arc::clone(&self.node)).await
    }

    #[tool(description = r##"Get metadata about the current Figma document: file name, pages, current page"##)]
    async fn get_metadata(&self) -> Result<CallToolResult, McpError> {
        read_document::get_metadata(std::sync::Arc::clone(&self.node)).await
    }

    #[tool(description = r##"Get the nodes currently selected in Figma. Returns an empty array if nothing is selected. Use get_design_context or get_node to retrieve deeper detail about a specific node by ID."##)]
    async fn get_selection(&self) -> Result<CallToolResult, McpError> {
        read_document::get_selection(std::sync::Arc::clone(&self.node)).await
    }

    #[tool(description = r##"Get a single node by ID with full detail. Use get_nodes_info to fetch multiple nodes in one round-trip instead of calling this repeatedly. Node ID must be colon format e.g. '4029:12345', never hyphens."##)]
    async fn get_node(&self, Parameters(args): Parameters<read_document::GetNodeArgs>) -> Result<CallToolResult, McpError> {
        read_document::get_node(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Get full details for multiple nodes by ID in one round-trip. Prefer this over calling get_node repeatedly when you need several nodes."##)]
    async fn get_nodes_info(&self, Parameters(args): Parameters<read_document::GetNodesInfoArgs>) -> Result<CallToolResult, McpError> {
        read_document::get_nodes_info(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Get a depth-limited, token-efficient tree of the current selection or page. Use this instead of get_document when exploring large files. Supports detail levels (minimal/compact/full) and dedupe_components for pages heavy with repeated component instances."##)]
    async fn get_design_context(&self, Parameters(args): Parameters<read_document::GetDesignContextArgs>) -> Result<CallToolResult, McpError> {
        read_document::get_design_context(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Search for nodes by name substring and/or type within a subtree. Use this when you know (part of) the node name. Use scan_nodes_by_types when you want all nodes of a type regardless of name."##)]
    async fn search_nodes(&self, Parameters(args): Parameters<read_document::SearchNodesArgs>) -> Result<CallToolResult, McpError> {
        read_document::search_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Scan all TEXT nodes in a subtree and return their content. Shorthand for scan_nodes_by_types with ['TEXT'] — use when you only need text copy from a component or frame."##)]
    async fn scan_text_nodes(&self, Parameters(args): Parameters<read_document::ScanTextNodesArgs>) -> Result<CallToolResult, McpError> {
        read_document::scan_text_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Find all nodes of specific types in a subtree, regardless of name. Use search_nodes instead when you need to filter by name."##)]
    async fn scan_nodes_by_types(&self, Parameters(args): Parameters<read_document::ScanNodesByTypesArgs>) -> Result<CallToolResult, McpError> {
        read_document::scan_nodes_by_types(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Get the prototype reactions defined on a node. Returns an array of reaction objects — each has a trigger (e.g. ON_CLICK, ON_HOVER, AFTER_TIMEOUT) and an actions array (navigate to node, open URL, go back, etc.). Use set_reactions to add or replace reactions, remove_reactions to delete them."##)]
    async fn get_reactions(&self, Parameters(args): Parameters<read_document::GetReactionsArgs>) -> Result<CallToolResult, McpError> {
        read_document::get_reactions(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Get the current Figma viewport: scroll center, zoom level, and visible bounds."##)]
    async fn get_viewport(&self) -> Result<CallToolResult, McpError> {
        read_document::get_viewport(std::sync::Arc::clone(&self.node)).await
    }

    #[tool(description = r##"List all fonts used in the current page, sorted by usage frequency. Useful for understanding typography without scanning all text nodes."##)]
    async fn get_fonts(&self) -> Result<CallToolResult, McpError> {
        read_document::get_fonts(std::sync::Arc::clone(&self.node)).await
    }

    #[tool(description = r##"Export a screenshot of one or more nodes as base64-encoded image data (held in memory). Use save_screenshots instead when you want to write images directly to disk without base64 in the response."##)]
    async fn get_screenshot(&self, Parameters(args): Parameters<read_export::GetScreenshotArgs>) -> Result<CallToolResult, McpError> {
        read_export::get_screenshot(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Export multiple frames as a single multi-page PDF file. Each frame becomes one page in order. Ideal for pitch decks, proposals, and slide exports."##)]
    async fn export_frames_to_pdf(&self, Parameters(args): Parameters<read_export::ExportFramesToPdfArgs>) -> Result<CallToolResult, McpError> {
        read_export::export_frames_to_pdf(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Export screenshots for multiple nodes and write them to the local filesystem. Returns file metadata (path, size, dimensions) — no base64 in the response. Use get_screenshot instead when you need the image data in memory."##)]
    async fn save_screenshots(&self, Parameters(args): Parameters<read_export::SaveScreenshotsArgs>) -> Result<CallToolResult, McpError> {
        read_export::save_screenshots(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Get all local styles in the document (paint, text, effect, and grid). Returns each style's ID, name, type, and properties. Use the style ID with apply_style_to_node or update_paint_style. For design tokens (variables), use get_variable_defs instead."##)]
    async fn get_styles(&self) -> Result<CallToolResult, McpError> {
        read_styles::get_styles(std::sync::Arc::clone(&self.node)).await
    }

    #[tool(description = r##"Get all local variable definitions: collections, modes, and values. Variables are Figma's design token system."##)]
    async fn get_variable_defs(&self) -> Result<CallToolResult, McpError> {
        read_styles::get_variable_defs(std::sync::Arc::clone(&self.node)).await
    }

    #[tool(description = r##"Get all components defined in the current Figma file."##)]
    async fn get_local_components(&self) -> Result<CallToolResult, McpError> {
        read_styles::get_local_components(std::sync::Arc::clone(&self.node)).await
    }

    #[tool(description = r##"Get dev-mode annotations in the current document or scoped to a specific node. Returns annotation objects with label text, measurement type, and the ID of the annotated node. Omit nodeId to retrieve all annotations on the current page."##)]
    async fn get_annotations(&self, Parameters(args): Parameters<read_styles::GetAnnotationsArgs>) -> Result<CallToolResult, McpError> {
        read_styles::get_annotations(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Export all design tokens (variables and paint styles) as JSON or CSS custom properties. Ideal for bridging Figma variables into your codebase."##)]
    async fn export_tokens(&self, Parameters(args): Parameters<read_styles::ExportTokensArgs>) -> Result<CallToolResult, McpError> {
        read_styles::export_tokens(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Switch the active Figma page. Provide either pageId or pageName."##)]
    async fn navigate_to_page(&self, Parameters(args): Parameters<write_components::NavigateToPageArgs>) -> Result<CallToolResult, McpError> {
        write_components::navigate_to_page(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Group two or more nodes into a GROUP. All nodes must share the same parent."##)]
    async fn group_nodes(&self, Parameters(args): Parameters<write_components::GroupNodesArgs>) -> Result<CallToolResult, McpError> {
        write_components::group_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Ungroup one or more GROUP nodes, moving their children to the parent and removing the group."##)]
    async fn ungroup_nodes(&self, Parameters(args): Parameters<write_components::UngroupNodesArgs>) -> Result<CallToolResult, McpError> {
        write_components::ungroup_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Swap the main component of an existing INSTANCE node, replacing it with a different component while keeping position and size."##)]
    async fn swap_component(&self, Parameters(args): Parameters<write_components::SwapComponentArgs>) -> Result<CallToolResult, McpError> {
        write_components::swap_component(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Detach one or more component instances, converting them to plain frames. The link to the main component is broken; all visual properties are preserved."##)]
    async fn detach_instance(&self, Parameters(args): Parameters<write_components::DetachInstanceArgs>) -> Result<CallToolResult, McpError> {
        write_components::detach_instance(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Create a new frame on the current page or inside a parent node."##)]
    async fn create_frame(&self, Parameters(args): Parameters<write_create::CreateFrameArgs>) -> Result<CallToolResult, McpError> {
        write_create::create_frame(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Create a new rectangle on the current page or inside a parent node."##)]
    async fn create_rectangle(&self, Parameters(args): Parameters<write_create::CreateRectangleArgs>) -> Result<CallToolResult, McpError> {
        write_create::create_rectangle(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Create a new ellipse (circle/oval) on the current page or inside a parent node."##)]
    async fn create_ellipse(&self, Parameters(args): Parameters<write_create::CreateEllipseArgs>) -> Result<CallToolResult, McpError> {
        write_create::create_ellipse(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Create a new text node on the current page or inside a parent node. The font is loaded automatically before insertion. Returns the created node ID and bounds. Use set_text to update the content of an existing text node."##)]
    async fn create_text(&self, Parameters(args): Parameters<write_create::CreateTextArgs>) -> Result<CallToolResult, McpError> {
        write_create::create_text(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Import a base64-encoded image into Figma as a rectangle with an image fill. Use get_screenshot to capture images or provide your own base64 PNG/JPG."##)]
    async fn import_image(&self, Parameters(args): Parameters<write_create::ImportImageArgs>) -> Result<CallToolResult, McpError> {
        write_create::import_image(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Convert an existing FRAME node into a reusable COMPONENT. The frame is replaced in place by the new component."##)]
    async fn create_component(&self, Parameters(args): Parameters<write_create::CreateComponentArgs>) -> Result<CallToolResult, McpError> {
        write_create::create_component(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Create a Figma Section node on the current page. Sections are the modern way to organize frames and groups on a page."##)]
    async fn create_section(&self, Parameters(args): Parameters<write_create::CreateSectionArgs>) -> Result<CallToolResult, McpError> {
        write_create::create_section(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Update the text content of an existing TEXT node."##)]
    async fn set_text(&self, Parameters(args): Parameters<write_modify::SetTextArgs>) -> Result<CallToolResult, McpError> {
        write_modify::set_text(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Set the fill color on a single node (takes one nodeId, not an array). Use mode='append' to stack a new fill on top of existing fills instead of replacing them."##)]
    async fn set_fills(&self, Parameters(args): Parameters<write_modify::SetFillsArgs>) -> Result<CallToolResult, McpError> {
        write_modify::set_fills(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Set the stroke color and weight on a single node (takes one nodeId, not an array). Use mode='append' to stack a new stroke on top of existing strokes instead of replacing them."##)]
    async fn set_strokes(&self, Parameters(args): Parameters<write_modify::SetStrokesArgs>) -> Result<CallToolResult, McpError> {
        write_modify::set_strokes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Move one or more nodes to an absolute canvas position. The same x/y is applied to every node independently (not a relative offset from current position)."##)]
    async fn move_nodes(&self, Parameters(args): Parameters<write_modify::MoveNodesArgs>) -> Result<CallToolResult, McpError> {
        write_modify::move_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Resize one or more nodes. The same width/height is applied to every node in the list independently. Provide width, height, or both."##)]
    async fn resize_nodes(&self, Parameters(args): Parameters<write_modify::ResizeNodesArgs>) -> Result<CallToolResult, McpError> {
        write_modify::resize_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Rename a single node by ID. Returns the updated node with its new name. Use batch_rename_nodes to rename multiple nodes at once or to apply find/replace patterns across many nodes."##)]
    async fn rename_node(&self, Parameters(args): Parameters<write_modify::RenameNodeArgs>) -> Result<CallToolResult, McpError> {
        write_modify::rename_node(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Clone an existing node, optionally repositioning it or placing it in a new parent."##)]
    async fn clone_node(&self, Parameters(args): Parameters<write_modify::CloneNodeArgs>) -> Result<CallToolResult, McpError> {
        write_modify::clone_node(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Set the opacity of one or more nodes (0 = fully transparent, 1 = fully opaque)."##)]
    async fn set_opacity(&self, Parameters(args): Parameters<write_modify::SetOpacityArgs>) -> Result<CallToolResult, McpError> {
        write_modify::set_opacity(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Set corner radius on one or more nodes. Provide a uniform cornerRadius or individual per-corner values."##)]
    async fn set_corner_radius(&self, Parameters(args): Parameters<write_modify::SetCornerRadiusArgs>) -> Result<CallToolResult, McpError> {
        write_modify::set_corner_radius(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Set or update auto-layout (flex) properties on an existing frame."##)]
    async fn set_auto_layout(&self, Parameters(args): Parameters<write_modify::SetAutoLayoutArgs>) -> Result<CallToolResult, McpError> {
        write_modify::set_auto_layout(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Delete one or more nodes. This cannot be undone via MCP — use with care."##)]
    async fn delete_nodes(&self, Parameters(args): Parameters<write_modify::DeleteNodesArgs>) -> Result<CallToolResult, McpError> {
        write_modify::delete_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Show or hide one or more nodes by setting their visibility."##)]
    async fn set_visible(&self, Parameters(args): Parameters<write_modify::SetVisibleArgs>) -> Result<CallToolResult, McpError> {
        write_modify::set_visible(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Lock one or more nodes to prevent accidental edits in Figma."##)]
    async fn lock_nodes(&self, Parameters(args): Parameters<write_modify::LockNodesArgs>) -> Result<CallToolResult, McpError> {
        write_modify::lock_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Unlock one or more nodes, allowing them to be edited again."##)]
    async fn unlock_nodes(&self, Parameters(args): Parameters<write_modify::UnlockNodesArgs>) -> Result<CallToolResult, McpError> {
        write_modify::unlock_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Rotate one or more nodes to an absolute angle in degrees."##)]
    async fn rotate_nodes(&self, Parameters(args): Parameters<write_modify::RotateNodesArgs>) -> Result<CallToolResult, McpError> {
        write_modify::rotate_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Change the z-order (layer stack position) of one or more nodes."##)]
    async fn reorder_nodes(&self, Parameters(args): Parameters<write_modify::ReorderNodesArgs>) -> Result<CallToolResult, McpError> {
        write_modify::reorder_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Set the blend mode of one or more nodes (e.g. MULTIPLY, SCREEN, OVERLAY)."##)]
    async fn set_blend_mode(&self, Parameters(args): Parameters<write_modify::SetBlendModeArgs>) -> Result<CallToolResult, McpError> {
        write_modify::set_blend_mode(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Set layout constraints (pinning behaviour) on one or more nodes relative to their parent."##)]
    async fn set_constraints(&self, Parameters(args): Parameters<write_modify::SetConstraintsArgs>) -> Result<CallToolResult, McpError> {
        write_modify::set_constraints(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Move one or more nodes to a different parent frame, group, or section."##)]
    async fn reparent_nodes(&self, Parameters(args): Parameters<write_modify::ReparentNodesArgs>) -> Result<CallToolResult, McpError> {
        write_modify::reparent_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Rename multiple nodes using find/replace, regex substitution, or prefix/suffix addition."##)]
    async fn batch_rename_nodes(&self, Parameters(args): Parameters<write_modify::BatchRenameNodesArgs>) -> Result<CallToolResult, McpError> {
        write_modify::batch_rename_nodes(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Find and replace text content across all TEXT nodes in a subtree. Searches the entire current page if no nodeId is given."##)]
    async fn find_replace_text(&self, Parameters(args): Parameters<write_modify::FindReplaceTextArgs>) -> Result<CallToolResult, McpError> {
        write_modify::find_replace_text(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Add a new page to the Figma document."##)]
    async fn add_page(&self, Parameters(args): Parameters<write_page::AddPageArgs>) -> Result<CallToolResult, McpError> {
        write_page::add_page(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Delete a page from the Figma document. Cannot delete the only remaining page."##)]
    async fn delete_page(&self, Parameters(args): Parameters<write_page::DeletePageArgs>) -> Result<CallToolResult, McpError> {
        write_page::delete_page(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Rename an existing page in the Figma document."##)]
    async fn rename_page(&self, Parameters(args): Parameters<write_page::RenamePageArgs>) -> Result<CallToolResult, McpError> {
        write_page::rename_page(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##""##)]
    async fn set_reactions(&self, Parameters(args): Parameters<write_prototype::SetReactionsArgs>) -> Result<CallToolResult, McpError> {
        write_prototype::set_reactions(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Remove prototype reactions from a node. Omit indices to remove all reactions. Provide a zero-based indices array to remove specific reactions (use get_reactions first to see current indices)."##)]
    async fn remove_reactions(&self, Parameters(args): Parameters<write_prototype::RemoveReactionsArgs>) -> Result<CallToolResult, McpError> {
        write_prototype::remove_reactions(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Create a new local paint style with a solid fill color."##)]
    async fn create_paint_style(&self, Parameters(args): Parameters<write_styles::CreatePaintStyleArgs>) -> Result<CallToolResult, McpError> {
        write_styles::create_paint_style(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Create a new local text style (typography preset). Returns the new style's ID. Apply it to nodes with apply_style_to_node. Use get_styles to list existing text styles."##)]
    async fn create_text_style(&self, Parameters(args): Parameters<write_styles::CreateTextStyleArgs>) -> Result<CallToolResult, McpError> {
        write_styles::create_text_style(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Create a new local effect style (drop shadow, inner shadow, or blur)."##)]
    async fn create_effect_style(&self, Parameters(args): Parameters<write_styles::CreateEffectStyleArgs>) -> Result<CallToolResult, McpError> {
        write_styles::create_effect_style(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Create a new local layout grid style."##)]
    async fn create_grid_style(&self, Parameters(args): Parameters<write_styles::CreateGridStyleArgs>) -> Result<CallToolResult, McpError> {
        write_styles::create_grid_style(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Update an existing paint style's name, color, or description. Only paint styles support in-place updates — to modify text, effect, or grid styles, use delete_style and recreate them."##)]
    async fn update_paint_style(&self, Parameters(args): Parameters<write_styles::UpdatePaintStyleArgs>) -> Result<CallToolResult, McpError> {
        write_styles::update_paint_style(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Delete a style (paint, text, effect, or grid) by its ID."##)]
    async fn delete_style(&self, Parameters(args): Parameters<write_styles::DeleteStyleArgs>) -> Result<CallToolResult, McpError> {
        write_styles::delete_style(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Apply an existing local style (paint, text, effect, or grid) to a node, linking the node to that style."##)]
    async fn apply_style_to_node(&self, Parameters(args): Parameters<write_styles::ApplyStyleToNodeArgs>) -> Result<CallToolResult, McpError> {
        write_styles::apply_style_to_node(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Apply one or more effects (drop shadow, inner shadow, layer blur, background blur) directly to a node. Replaces all existing effects. Pass an empty array to clear all effects."##)]
    async fn set_effects(&self, Parameters(args): Parameters<write_styles::SetEffectsArgs>) -> Result<CallToolResult, McpError> {
        write_styles::set_effects(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Bind a local variable to a node property so the property is driven by the variable's value. COLOR variables: use fillColor or strokeColor. BOOLEAN variables: use visible. FLOAT variables: use opacity, rotation, width, height, cornerRadius, topLeftRadius, topRightRadius, bottomLeftRadius, bottomRightRadius, strokeWeight, itemSpacing, paddingTop, paddingRight, paddingBottom, paddingLeft."##)]
    async fn bind_variable_to_node(&self, Parameters(args): Parameters<write_styles::BindVariableToNodeArgs>) -> Result<CallToolResult, McpError> {
        write_styles::bind_variable_to_node(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##""##)]
    async fn create_variable_collection(&self, Parameters(args): Parameters<write_variables::CreateVariableCollectionArgs>) -> Result<CallToolResult, McpError> {
        write_variables::create_variable_collection(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##""##)]
    async fn add_variable_mode(&self, Parameters(args): Parameters<write_variables::AddVariableModeArgs>) -> Result<CallToolResult, McpError> {
        write_variables::add_variable_mode(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Create a new variable (design token) inside an existing collection. Returns the new variable's ID. Use get_variable_defs to find collection IDs, set_variable_value to set values per mode, and bind_variable_to_node to apply the variable to a node property."##)]
    async fn create_variable(&self, Parameters(args): Parameters<write_variables::CreateVariableArgs>) -> Result<CallToolResult, McpError> {
        write_variables::create_variable(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Set a variable's value for a specific mode."##)]
    async fn set_variable_value(&self, Parameters(args): Parameters<write_variables::SetVariableValueArgs>) -> Result<CallToolResult, McpError> {
        write_variables::set_variable_value(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = r##"Delete a single variable (provide variableId) or an entire collection and all its variables (provide collectionId). Provide exactly one of the two — not both."##)]
    async fn delete_variable(&self, Parameters(args): Parameters<write_variables::DeleteVariableArgs>) -> Result<CallToolResult, McpError> {
        write_variables::delete_variable(std::sync::Arc::clone(&self.node), args).await
    }

    // ── FigJam additions (not in the Go reference) ───────────────────────────
    #[tool(description = "Create a sticky note on the current FigJam page. FigJam only.")]
    async fn create_sticky(&self, Parameters(args): Parameters<figjam::CreateStickyArgs>) -> Result<CallToolResult, McpError> {
        figjam::create_sticky(std::sync::Arc::clone(&self.node), args).await
    }

    #[tool(description = "Create a connector on the current FigJam page, optionally linked between two nodes. FigJam only.")]
    async fn create_connector(&self, Parameters(args): Parameters<figjam::CreateConnectorArgs>) -> Result<CallToolResult, McpError> {
        figjam::create_connector(std::sync::Arc::clone(&self.node), args).await
    }

    // ── Escape hatch ─────────────────────────────────────────────────────────
    #[tool(description = r##"Run JavaScript against the Figma Plugin API inside the file. Use this for anything the named tools don't cover: FigJam tables/shapes/code blocks/labels, Slides, component variants and properties, vector networks, styled text ranges, variable scopes, library imports, bulk edits in one round-trip.

Code runs as the body of `async () => { ... }`: top-level `await` works, and only the value you `return` comes back (console.log is discarded, so return IDs, counts, or names — never node objects, the response must be JSON-serializable).

Sandbox rules that cause most failures: colors are 0–1 floats, not 0–255; `fills`/`strokes` are read-only arrays, so clone-modify-reassign; load fonts with `await figma.loadFontAsync(node.fontName)` before touching `characters` on existing text; switch pages with `await figma.setCurrentPageAsync(page)` (the sync setter throws) and note the page resets to the first page on every call; `figma.notify()` is unavailable; append a node to an auto-layout parent before setting `layoutSizingHorizontal/Vertical` to HUG or FILL; `figma.createPage()` is Design-file only. Work in small steps and verify between calls."##)]
    async fn use_figma(&self, Parameters(args): Parameters<use_figma::UseFigmaArgs>) -> Result<CallToolResult, McpError> {
        use_figma::use_figma(std::sync::Arc::clone(&self.node), args).await
    }
}

// ── Prompts ──────────────────────────────────────────────────────────────────
// Registered in prompts.rs via #[prompt_router] on FigmaServer.

#[tool_handler]
#[prompt_handler]
impl ServerHandler for FigmaServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
        )
        .with_server_info(Implementation::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))
    }
}
