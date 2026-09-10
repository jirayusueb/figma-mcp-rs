// MANIFEST: get_document | - | Get the full node tree of the current page (not the whole file — only the active page). Returns all nodes recursively and can be very large. Prefer get_design_context for exploration or when token efficiency matters.
// MANIFEST: get_pages | - | List all pages in the document with their IDs and names. Lightweight alternative to get_document.
// MANIFEST: get_metadata | - | Get metadata about the current Figma document: file name, pages, current page
// MANIFEST: get_selection | - | Get the nodes currently selected in Figma. Returns an empty array if nothing is selected. Use get_design_context or get_node to retrieve deeper detail about a specific node by ID.
// MANIFEST: get_node | GetNodeArgs | Get a single node by ID with full detail. Use get_nodes_info to fetch multiple nodes in one round-trip instead of calling this repeatedly. Node ID must be colon format e.g. '4029:12345', never hyphens.
// MANIFEST: get_nodes_info | GetNodesInfoArgs | Get full details for multiple nodes by ID in one round-trip. Prefer this over calling get_node repeatedly when you need several nodes.
// MANIFEST: get_design_context | GetDesignContextArgs | Get a depth-limited, token-efficient tree of the current selection or page. Use this instead of get_document when exploring large files. Supports detail levels (minimal/compact/full) and dedupe_components for pages heavy with repeated component instances.
// MANIFEST: search_nodes | SearchNodesArgs | Search for nodes by name substring and/or type within a subtree. Use this when you know (part of) the node name. Use scan_nodes_by_types when you want all nodes of a type regardless of name.
// MANIFEST: scan_text_nodes | ScanTextNodesArgs | Scan all TEXT nodes in a subtree and return their content. Shorthand for scan_nodes_by_types with ['TEXT'] — use when you only need text copy from a component or frame.
// MANIFEST: scan_nodes_by_types | ScanNodesByTypesArgs | Find all nodes of specific types in a subtree, regardless of name. Use search_nodes instead when you need to filter by name.
// MANIFEST: get_reactions | GetReactionsArgs | Get the prototype reactions defined on a node. Returns an array of reaction objects — each has a trigger (e.g. ON_CLICK, ON_HOVER, AFTER_TIMEOUT) and an actions array (navigate to node, open URL, go back, etc.). Use set_reactions to add or replace reactions, remove_reactions to delete them.
// MANIFEST: get_viewport | - | Get the current Figma viewport: scroll center, zoom level, and visible bounds.
// MANIFEST: get_fonts | - | List all fonts used in the current page, sorted by usage frequency. Useful for understanding typography without scanning all text nodes.

//! Read-document tools. Ported from figma-mcp-go internal/tools_read_document.go.

use std::sync::Arc;

use super::McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::node::Node;
use crate::tools::{relay, relay_params};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetNodeArgs {
    /// Node ID in colon format e.g. '4029:12345'
    pub node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetNodesInfoArgs {
    /// List of node IDs in colon format e.g. ['4029:12345', '4029:67890']
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetDesignContextArgs {
    /// How many levels deep to traverse (default 2)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<f64>,
    /// Property verbosity: minimal (id/name/type/bounds only), compact (+fills/strokes/opacity), full (everything, default)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// When true, INSTANCE nodes are serialized compactly (mainComponentId + componentProperties + overrides array of differing text/nested content) and unique component definitions are collected once in a top-level componentDefs map. Highly token-efficient for screens with many repeated component instances.
    #[serde(
        default,
        rename = "dedupe_components",
        skip_serializing_if = "Option::is_none"
    )]
    pub dedupe_components: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchNodesArgs {
    /// Name substring to match (case-insensitive)
    pub query: String,
    /// Scope search to this subtree (default: current page), colon format e.g. '4029:12345'
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    /// Filter by Figma node type e.g. ['TEXT', 'FRAME', 'COMPONENT']
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,
    /// Maximum results to return (default: 50)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScanTextNodesArgs {
    /// Root node ID to scan from, colon format e.g. '4029:12345'
    pub node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScanNodesByTypesArgs {
    /// Root node ID to scan from, colon format e.g. '4029:12345'
    pub node_id: String,
    /// Node types to find e.g. ['FRAME', 'COMPONENT', 'INSTANCE']
    pub types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetReactionsArgs {
    /// Node ID in colon format e.g. '4029:12345'
    pub node_id: String,
}

pub(crate) async fn get_document(node: Arc<Node>) -> Result<CallToolResult, McpError> {
    relay_params(&node, "get_document", serde_json::Value::Null).await
}

pub(crate) async fn get_pages(node: Arc<Node>) -> Result<CallToolResult, McpError> {
    relay_params(&node, "get_pages", serde_json::Value::Null).await
}

pub(crate) async fn get_metadata(node: Arc<Node>) -> Result<CallToolResult, McpError> {
    relay_params(&node, "get_metadata", serde_json::Value::Null).await
}

pub(crate) async fn get_selection(node: Arc<Node>) -> Result<CallToolResult, McpError> {
    relay_params(&node, "get_selection", serde_json::Value::Null).await
}

pub(crate) async fn get_node(
    node: Arc<Node>,
    args: GetNodeArgs,
) -> Result<CallToolResult, McpError> {
    relay(&node, "get_node", &args).await
}

pub(crate) async fn get_nodes_info(
    node: Arc<Node>,
    args: GetNodesInfoArgs,
) -> Result<CallToolResult, McpError> {
    relay(&node, "get_nodes_info", &args).await
}

/// Custom mapping (matches Go): `dedupe_components` is the wire-visible schema
/// property name, but the plugin-facing param key is camelCase `dedupeComponents`.
/// `depth`/`detail`/`dedupeComponents` are only included when Go's own truthy
/// conditions (`depth > 0`, non-empty `detail`, `dedupe_components == true`) hold.
pub(crate) async fn get_design_context(
    node: Arc<Node>,
    args: GetDesignContextArgs,
) -> Result<CallToolResult, McpError> {
    let mut params = serde_json::Map::new();
    if let Some(depth) = args.depth {
        if depth > 0.0 {
            params.insert("depth".to_string(), serde_json::json!(depth));
        }
    }
    if let Some(detail) = args.detail {
        if !detail.is_empty() {
            params.insert("detail".to_string(), serde_json::json!(detail));
        }
    }
    if args.dedupe_components == Some(true) {
        params.insert("dedupeComponents".to_string(), serde_json::json!(true));
    }
    relay_params(
        &node,
        "get_design_context",
        serde_json::Value::Object(params),
    )
    .await
}

pub(crate) async fn search_nodes(
    node: Arc<Node>,
    args: SearchNodesArgs,
) -> Result<CallToolResult, McpError> {
    relay(&node, "search_nodes", &args).await
}

pub(crate) async fn scan_text_nodes(
    node: Arc<Node>,
    args: ScanTextNodesArgs,
) -> Result<CallToolResult, McpError> {
    relay(&node, "scan_text_nodes", &args).await
}

pub(crate) async fn scan_nodes_by_types(
    node: Arc<Node>,
    args: ScanNodesByTypesArgs,
) -> Result<CallToolResult, McpError> {
    relay(&node, "scan_nodes_by_types", &args).await
}

pub(crate) async fn get_reactions(
    node: Arc<Node>,
    args: GetReactionsArgs,
) -> Result<CallToolResult, McpError> {
    relay(&node, "get_reactions", &args).await
}

pub(crate) async fn get_viewport(node: Arc<Node>) -> Result<CallToolResult, McpError> {
    relay_params(&node, "get_viewport", serde_json::Value::Null).await
}

pub(crate) async fn get_fonts(node: Arc<Node>) -> Result<CallToolResult, McpError> {
    relay_params(&node, "get_fonts", serde_json::Value::Null).await
}
