// MANIFEST: add_page | AddPageArgs | Add a new page to the Figma document.
// MANIFEST: delete_page | DeletePageArgs | Delete a page from the Figma document. Cannot delete the only remaining page.
// MANIFEST: rename_page | RenamePageArgs | Rename an existing page in the Figma document.

//! Ported from figma-mcp-go internal/tools_write_page.go.

use std::sync::Arc;

use super::McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AddPageArgs {
    /// Name for the new page (default 'Page')
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Position index to insert the page (0 = first). Defaults to last position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<f64>,
}

pub(crate) async fn add_page(
    node: Arc<Node>,
    args: AddPageArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "add_page", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeletePageArgs {
    /// Page node ID in colon format e.g. '0:2'
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_id: Option<String>,
    /// Exact page name to delete (alternative to pageId)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_name: Option<String>,
}

pub(crate) async fn delete_page(
    node: Arc<Node>,
    args: DeletePageArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "delete_page", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RenamePageArgs {
    /// Page node ID in colon format e.g. '0:2'
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_id: Option<String>,
    /// Current page name to find (alternative to pageId)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_name: Option<String>,
    /// New name for the page
    pub new_name: String,
}

pub(crate) async fn rename_page(
    node: Arc<Node>,
    args: RenamePageArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "rename_page", &args).await
}
