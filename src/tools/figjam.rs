//! FigJam-specific tools. Sticky/connector creation only exists in FigJam.

use std::sync::Arc;

use super::McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::node::Node;

/// Text content for the sticky note (required)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateStickyArgs {
    /// Text content for the sticky note (required)
    pub text: String,
    /// X position on the canvas
    #[serde(default)]
    pub x: Option<f64>,
    /// Y position on the canvas
    #[serde(default)]
    pub y: Option<f64>,
    /// Optional name for the node
    #[serde(default)]
    pub name: Option<String>,
    /// Fill color as hex string e.g. '#FFD54F'
    #[serde(default)]
    pub fill_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateConnectorArgs {
    /// Optional text label for the connector
    #[serde(default)]
    pub text: Option<String>,
    /// Optional node ID the connector starts from, colon format e.g. '4029:12345'
    #[serde(default)]
    pub start_node_id: Option<String>,
    /// Optional node ID the connector ends at, colon format e.g. '4029:67890'
    #[serde(default)]
    pub end_node_id: Option<String>,
}

pub(crate) async fn create_sticky(
    node: Arc<Node>,
    args: CreateStickyArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_sticky", &args).await
}

pub(crate) async fn create_connector(
    node: Arc<Node>,
    args: CreateConnectorArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_connector", &args).await
}
// ── FigJam additions ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateStickiesArgs {
    /// Array of stickies to create (required, max 200)
    pub items: Vec<StickyItem>,
    /// Starting X position for grid (default 0)
    #[serde(default)]
    pub start_x: Option<f64>,
    /// Starting Y position for grid (default 0)
    #[serde(default)]
    pub start_y: Option<f64>,
    /// Number of columns in grid layout (default 5)
    #[serde(default)]
    pub columns: Option<f64>,
    /// Spacing between stickies (default 40)
    #[serde(default)]
    pub spacing: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StickyItem {
    /// Text content (required)
    pub text: String,
    /// X position (optional; if absent, uses grid layout)
    #[serde(default)]
    pub x: Option<f64>,
    /// Y position (optional; if absent, uses grid layout)
    #[serde(default)]
    pub y: Option<f64>,
    /// Optional sticky name
    #[serde(default)]
    pub name: Option<String>,
    /// Fill color as hex
    #[serde(default)]
    pub fill_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateShapeWithTextArgs {
    /// Text content (required)
    pub text: String,
    /// Shape type (default "ROUNDED_RECTANGLE")
    #[serde(default)]
    pub shape_type: Option<String>,
    /// X position
    #[serde(default)]
    pub x: Option<f64>,
    /// Y position
    #[serde(default)]
    pub y: Option<f64>,
    /// Width
    #[serde(default)]
    pub width: Option<f64>,
    /// Height
    #[serde(default)]
    pub height: Option<f64>,
    /// Fill color as hex
    #[serde(default)]
    pub fill_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateTableArgs {
    /// Number of rows (required, 1-50)
    pub rows: f64,
    /// Number of columns (required, 1-50)
    pub columns: f64,
    /// Cell contents (optional, ragged arrays OK)
    #[serde(default)]
    pub cells: Option<Vec<Vec<String>>>,
    /// X position
    #[serde(default)]
    pub x: Option<f64>,
    /// Y position
    #[serde(default)]
    pub y: Option<f64>,
    /// Table name
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateCodeBlockArgs {
    /// Code content (required)
    pub code: String,
    /// Programming language (default "PLAINTEXT")
    #[serde(default)]
    pub language: Option<String>,
    /// X position
    #[serde(default)]
    pub x: Option<f64>,
    /// Y position
    #[serde(default)]
    pub y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AutoArrangeArgs {
    /// Node IDs to arrange (optional; if absent, arranges page children)
    #[serde(default)]
    pub node_ids: Option<Vec<String>>,
    /// Layout mode: "grid", "row", or "column" (default "grid")
    #[serde(default)]
    pub layout: Option<String>,
    /// Spacing between items (default 40)
    #[serde(default)]
    pub spacing: Option<f64>,
    /// Grid columns (default auto based on sqrt of count)
    #[serde(default)]
    pub columns: Option<f64>,
    /// Starting X position
    #[serde(default)]
    pub start_x: Option<f64>,
    /// Starting Y position
    #[serde(default)]
    pub start_y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetBoardContentsArgs {
    /// Include connector information (default true)
    #[serde(default)]
    pub include_connections: Option<bool>,
}

pub(crate) async fn create_stickies(
    node: Arc<Node>,
    args: CreateStickiesArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_stickies", &args).await
}

pub(crate) async fn create_shape_with_text(
    node: Arc<Node>,
    args: CreateShapeWithTextArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_shape_with_text", &args).await
}

pub(crate) async fn create_table(
    node: Arc<Node>,
    args: CreateTableArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_table", &args).await
}

pub(crate) async fn create_code_block(
    node: Arc<Node>,
    args: CreateCodeBlockArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_code_block", &args).await
}

pub(crate) async fn auto_arrange(
    node: Arc<Node>,
    args: AutoArrangeArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "auto_arrange", &args).await
}

pub(crate) async fn get_board_contents(
    node: Arc<Node>,
    args: GetBoardContentsArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "get_board_contents", &args).await
}
