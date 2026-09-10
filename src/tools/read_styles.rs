// MANIFEST: get_styles | - | Get all local styles in the document (paint, text, effect, and grid). Returns each style's ID, name, type, and properties. Use the style ID with apply_style_to_node or update_paint_style. For design tokens (variables), use get_variable_defs instead.
// MANIFEST: get_variable_defs | - | Get all local variable definitions: collections, modes, and values. Variables are Figma's design token system.
// MANIFEST: get_local_components | - | Get all components defined in the current Figma file.
// MANIFEST: get_annotations | GetAnnotationsArgs | Get dev-mode annotations in the current document or scoped to a specific node. Returns annotation objects with label text, measurement type, and the ID of the annotated node. Omit nodeId to retrieve all annotations on the current page.
// MANIFEST: export_tokens | ExportTokensArgs | Export all design tokens (variables and paint styles) as JSON or CSS custom properties. Ideal for bridging Figma variables into your codebase.

//! Read-style tools. Ported from figma-mcp-go internal/tools_read_styles.go.

use std::sync::Arc;

use super::McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::node::Node;
use crate::tools::{relay, relay_params};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetAnnotationsArgs {
    /// Optional — scope results to annotations on this node and its descendants, colon format e.g. '4029:12345'
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportTokensArgs {
    /// Output format: json (default) or css
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

pub(crate) async fn get_styles(node: Arc<Node>) -> Result<CallToolResult, McpError> {
    relay_params(&node, "get_styles", serde_json::Value::Null).await
}

pub(crate) async fn get_variable_defs(node: Arc<Node>) -> Result<CallToolResult, McpError> {
    relay_params(&node, "get_variable_defs", serde_json::Value::Null).await
}

pub(crate) async fn get_local_components(node: Arc<Node>) -> Result<CallToolResult, McpError> {
    relay_params(&node, "get_local_components", serde_json::Value::Null).await
}

pub(crate) async fn get_annotations(
    node: Arc<Node>,
    args: GetAnnotationsArgs,
) -> Result<CallToolResult, McpError> {
    relay(&node, "get_annotations", &args).await
}

pub(crate) async fn export_tokens(
    node: Arc<Node>,
    args: ExportTokensArgs,
) -> Result<CallToolResult, McpError> {
    relay(&node, "export_tokens", &args).await
}
