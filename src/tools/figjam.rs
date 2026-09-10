//! FigJam-specific tools. Sticky/connector creation only exists in FigJam.

use std::sync::Arc;

use rmcp::model::CallToolResult;
use super::McpError;
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
