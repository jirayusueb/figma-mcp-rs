// MANIFEST: navigate_to_page | NavigateToPageArgs | Switch the active Figma page. Provide either pageId or pageName.
// MANIFEST: group_nodes | GroupNodesArgs | Group two or more nodes into a GROUP. All nodes must share the same parent.
// MANIFEST: ungroup_nodes | UngroupNodesArgs | Ungroup one or more GROUP nodes, moving their children to the parent and removing the group.
// MANIFEST: swap_component | SwapComponentArgs | Swap the main component of an existing INSTANCE node, replacing it with a different component while keeping position and size.
// MANIFEST: detach_instance | DetachInstanceArgs | Detach one or more component instances, converting them to plain frames. The link to the main component is broken; all visual properties are preserved.

//! Component/page tools: page navigation, grouping, component swap/detach.

use std::sync::Arc;

use super::McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NavigateToPageArgs {
    /// Page node ID in colon format e.g. '0:1'
    #[serde(default)]
    pub page_id: Option<String>,
    /// Exact page name to navigate to
    #[serde(default)]
    pub page_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GroupNodesArgs {
    /// Node IDs to group (minimum 2), in colon format e.g. ['4029:12345', '4029:12346']
    pub node_ids: Vec<String>,
    /// Optional name for the new group
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UngroupNodesArgs {
    /// GROUP node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SwapComponentArgs {
    /// INSTANCE node ID in colon format e.g. 4029:12345
    pub node_id: String,
    /// Target COMPONENT node ID in colon format (from get_local_components)
    pub component_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DetachInstanceArgs {
    /// INSTANCE node IDs in colon format e.g. ['4029:12345']
    pub node_ids: Vec<String>,
}

pub(crate) async fn navigate_to_page(
    node: Arc<Node>,
    args: NavigateToPageArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "navigate_to_page", &args).await
}

pub(crate) async fn group_nodes(
    node: Arc<Node>,
    args: GroupNodesArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "group_nodes", &args).await
}

pub(crate) async fn ungroup_nodes(
    node: Arc<Node>,
    args: UngroupNodesArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "ungroup_nodes", &args).await
}

pub(crate) async fn swap_component(
    node: Arc<Node>,
    args: SwapComponentArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "swap_component", &args).await
}

pub(crate) async fn detach_instance(
    node: Arc<Node>,
    args: DetachInstanceArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "detach_instance", &args).await
}
