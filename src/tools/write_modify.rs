// MANIFEST: set_text | SetTextArgs | Update the text content of an existing TEXT node.
// MANIFEST: set_fills | SetFillsArgs | Set the fill color on a single node (takes one nodeId, not an array). Use mode='append' to stack a new fill on top of existing fills instead of replacing them.
// MANIFEST: set_strokes | SetStrokesArgs | Set the stroke color and weight on a single node (takes one nodeId, not an array). Use mode='append' to stack a new stroke on top of existing strokes instead of replacing them.
// MANIFEST: move_nodes | MoveNodesArgs | Move one or more nodes to an absolute canvas position. The same x/y is applied to every node independently (not a relative offset from current position).
// MANIFEST: resize_nodes | ResizeNodesArgs | Resize one or more nodes. The same width/height is applied to every node in the list independently. Provide width, height, or both.
// MANIFEST: rename_node | RenameNodeArgs | Rename a single node by ID. Returns the updated node with its new name. Use batch_rename_nodes to rename multiple nodes at once or to apply find/replace patterns across many nodes.
// MANIFEST: clone_node | CloneNodeArgs | Clone an existing node, optionally repositioning it or placing it in a new parent.
// MANIFEST: set_opacity | SetOpacityArgs | Set the opacity of one or more nodes (0 = fully transparent, 1 = fully opaque).
// MANIFEST: set_corner_radius | SetCornerRadiusArgs | Set corner radius on one or more nodes. Provide a uniform cornerRadius or individual per-corner values.
// MANIFEST: set_auto_layout | SetAutoLayoutArgs | Set or update auto-layout (flex) properties on an existing frame.
// MANIFEST: delete_nodes | DeleteNodesArgs | Delete one or more nodes. This cannot be undone via MCP — use with care.
// MANIFEST: set_visible | SetVisibleArgs | Show or hide one or more nodes by setting their visibility.
// MANIFEST: lock_nodes | LockNodesArgs | Lock one or more nodes to prevent accidental edits in Figma.
// MANIFEST: unlock_nodes | UnlockNodesArgs | Unlock one or more nodes, allowing them to be edited again.
// MANIFEST: rotate_nodes | RotateNodesArgs | Rotate one or more nodes to an absolute angle in degrees.
// MANIFEST: reorder_nodes | ReorderNodesArgs | Change the z-order (layer stack position) of one or more nodes.
// MANIFEST: set_blend_mode | SetBlendModeArgs | Set the blend mode of one or more nodes (e.g. MULTIPLY, SCREEN, OVERLAY).
// MANIFEST: set_constraints | SetConstraintsArgs | Set layout constraints (pinning behaviour) on one or more nodes relative to their parent.
// MANIFEST: reparent_nodes | ReparentNodesArgs | Move one or more nodes to a different parent frame, group, or section.
// MANIFEST: batch_rename_nodes | BatchRenameNodesArgs | Rename multiple nodes using find/replace, regex substitution, or prefix/suffix addition.
// MANIFEST: find_replace_text | FindReplaceTextArgs | Find and replace text content across all TEXT nodes in a subtree. Searches the entire current page if no nodeId is given.

use std::sync::Arc;

use rmcp::model::CallToolResult;
use super::McpError;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetTextArgs {
    /// TEXT node ID in colon format e.g. '4029:12345'
    pub node_id: String,
    /// New text content
    pub text: String,
}

pub(crate) async fn set_text(node: Arc<Node>, args: SetTextArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_text", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetFillsArgs {
    /// Node ID in colon format e.g. '4029:12345'
    pub node_id: String,
    /// Fill color as hex: #RRGGBB e.g. #FF5733 or #RRGGBBAA e.g. #FF573380 for 50% alpha
    pub color: String,
    /// Fill opacity 0–1 (default 1). Combines multiplicatively with any alpha in the color hex.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    /// 'replace' (default) overwrites all existing fills; 'append' stacks this fill on top of existing ones
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

pub(crate) async fn set_fills(node: Arc<Node>, args: SetFillsArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_fills", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetStrokesArgs {
    /// Node ID in colon format e.g. '4029:12345'
    pub node_id: String,
    /// Stroke color as hex e.g. #000000
    pub color: String,
    /// Stroke weight in pixels (default 1)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_weight: Option<f64>,
    /// 'replace' (default) overwrites all strokes; 'append' stacks on top of existing strokes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

pub(crate) async fn set_strokes(node: Arc<Node>, args: SetStrokesArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_strokes", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MoveNodesArgs {
    /// Node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
    /// Target X position
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Target Y position
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
}

pub(crate) async fn move_nodes(node: Arc<Node>, args: MoveNodesArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "move_nodes", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResizeNodesArgs {
    /// Node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
    /// New width in pixels
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    /// New height in pixels
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
}

pub(crate) async fn resize_nodes(node: Arc<Node>, args: ResizeNodesArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "resize_nodes", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RenameNodeArgs {
    /// Node ID in colon format e.g. '4029:12345'
    pub node_id: String,
    /// New name for the node. Figma supports slash-separated path notation e.g. 'Icons/Arrow/Left' to organise nodes in component panels.
    pub name: String,
}

pub(crate) async fn rename_node(node: Arc<Node>, args: RenameNodeArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "rename_node", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CloneNodeArgs {
    /// Source node ID in colon format e.g. '4029:12345'
    pub node_id: String,
    /// X position of the clone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Y position of the clone
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    /// Parent node ID for the clone. Defaults to same parent as source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

pub(crate) async fn clone_node(node: Arc<Node>, args: CloneNodeArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "clone_node", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetOpacityArgs {
    /// Node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
    /// Opacity value between 0 and 1
    pub opacity: f64,
}

pub(crate) async fn set_opacity(node: Arc<Node>, args: SetOpacityArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_opacity", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetCornerRadiusArgs {
    /// Node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
    /// Uniform corner radius applied to all corners
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f64>,
    /// Top-left corner radius
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_left_radius: Option<f64>,
    /// Top-right corner radius
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_right_radius: Option<f64>,
    /// Bottom-left corner radius
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom_left_radius: Option<f64>,
    /// Bottom-right corner radius
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom_right_radius: Option<f64>,
}

pub(crate) async fn set_corner_radius(node: Arc<Node>, args: SetCornerRadiusArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_corner_radius", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetAutoLayoutArgs {
    /// Frame node ID in colon format e.g. '4029:12345'
    pub node_id: String,
    /// Auto-layout direction: HORIZONTAL, VERTICAL, or NONE
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_mode: Option<String>,
    /// Top padding
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_top: Option<f64>,
    /// Right padding
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<f64>,
    /// Bottom padding
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_bottom: Option<f64>,
    /// Left padding
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<f64>,
    /// Gap between children
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_spacing: Option<f64>,
    /// Main-axis alignment: MIN, CENTER, MAX, or SPACE_BETWEEN
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_axis_align_items: Option<String>,
    /// Cross-axis alignment: MIN, CENTER, MAX, or BASELINE
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter_axis_align_items: Option<String>,
    /// Main-axis sizing: FIXED or AUTO (hug)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_axis_sizing_mode: Option<String>,
    /// Cross-axis sizing: FIXED or AUTO (hug)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter_axis_sizing_mode: Option<String>,
    /// Wrap behaviour: NO_WRAP or WRAP
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_wrap: Option<String>,
    /// Gap between wrapped rows/columns (only when layoutWrap is WRAP)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter_axis_spacing: Option<f64>,
}

pub(crate) async fn set_auto_layout(node: Arc<Node>, args: SetAutoLayoutArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_auto_layout", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeleteNodesArgs {
    /// Node IDs to delete in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
}

pub(crate) async fn delete_nodes(node: Arc<Node>, args: DeleteNodesArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "delete_nodes", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetVisibleArgs {
    /// Node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
    /// true to show the node, false to hide it
    pub visible: bool,
}

pub(crate) async fn set_visible(node: Arc<Node>, args: SetVisibleArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_visible", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LockNodesArgs {
    /// Node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
}

pub(crate) async fn lock_nodes(node: Arc<Node>, args: LockNodesArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "lock_nodes", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UnlockNodesArgs {
    /// Node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
}

pub(crate) async fn unlock_nodes(node: Arc<Node>, args: UnlockNodesArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "unlock_nodes", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RotateNodesArgs {
    /// Node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
    /// Rotation angle in degrees (positive = counter-clockwise in Figma)
    pub rotation: f64,
}

pub(crate) async fn rotate_nodes(node: Arc<Node>, args: RotateNodesArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "rotate_nodes", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReorderNodesArgs {
    /// Node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
    /// Order operation: bringToFront, sendToBack, bringForward, or sendBackward
    pub order: String,
}

pub(crate) async fn reorder_nodes(node: Arc<Node>, args: ReorderNodesArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "reorder_nodes", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetBlendModeArgs {
    /// Node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
    /// Blend mode: NORMAL, MULTIPLY, SCREEN, OVERLAY, DARKEN, LIGHTEN, COLOR_DODGE, COLOR_BURN, HARD_LIGHT, SOFT_LIGHT, DIFFERENCE, EXCLUSION, HUE, SATURATION, COLOR, LUMINOSITY, PASS_THROUGH
    pub blend_mode: String,
}

pub(crate) async fn set_blend_mode(node: Arc<Node>, args: SetBlendModeArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_blend_mode", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetConstraintsArgs {
    /// Node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
    /// Horizontal constraint: MIN (left), MAX (right), CENTER, STRETCH, or SCALE
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub horizontal: Option<String>,
    /// Vertical constraint: MIN (top), MAX (bottom), CENTER, STRETCH, or SCALE
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vertical: Option<String>,
}

pub(crate) async fn set_constraints(node: Arc<Node>, args: SetConstraintsArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_constraints", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReparentNodesArgs {
    /// Node IDs to move in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
    /// Target parent node ID in colon format e.g. '4029:99'
    pub parent_id: String,
}

pub(crate) async fn reparent_nodes(node: Arc<Node>, args: ReparentNodesArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "reparent_nodes", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchRenameNodesArgs {
    /// Node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
    /// String (or regex pattern when useRegex=true) to search for in the node name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub find: Option<String>,
    /// Replacement string. Required when find is provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replace: Option<String>,
    /// Treat find as a regular expression (default false)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_regex: Option<bool>,
    /// Regex flags e.g. 'gi' (default 'g'). Only used when useRegex=true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regex_flags: Option<String>,
    /// String to prepend to the node name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// String to append to the node name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
}

pub(crate) async fn batch_rename_nodes(node: Arc<Node>, args: BatchRenameNodesArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "batch_rename_nodes", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FindReplaceTextArgs {
    /// Text string (or regex pattern when useRegex=true) to search for
    pub find: String,
    /// Replacement string (use empty string to delete matches)
    pub replace: String,
    /// Root node ID to scope the search. Defaults to the entire current page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    /// Treat find as a regular expression (default false)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_regex: Option<bool>,
    /// Regex flags e.g. 'gi' (default 'g'). Only used when useRegex=true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regex_flags: Option<String>,
}

pub(crate) async fn find_replace_text(node: Arc<Node>, args: FindReplaceTextArgs) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "find_replace_text", &args).await
}
