// MANIFEST: set_reactions | SetReactionsArgs | Set prototype reactions on a node. Use mode "replace" (default) to overwrite all reactions, or "append" to add to existing ones.\n\nSupported triggers: ON_CLICK, ON_HOVER, ON_PRESS, ON_DRAG, AFTER_TIMEOUT, MOUSE_ENTER, MOUSE_LEAVE, MOUSE_UP, MOUSE_DOWN\nSupported action types: NODE (navigation), BACK, CLOSE, URL\n  NODE navigation values: NAVIGATE, OVERLAY, SCROLL_TO, SWAP, CHANGE_TO\nTransition types: DISSOLVE, SMART_ANIMATE, MOVE_IN, MOVE_OUT, PUSH, SLIDE_IN, SLIDE_OUT\n  DISSOLVE / SMART_ANIMATE: {"type":"DISSOLVE","duration":0.3,"easing":{"type":"EASE_OUT"}}\n  Directional (PUSH, MOVE_IN, MOVE_OUT, SLIDE_IN, SLIDE_OUT): also require "direction" (LEFT|RIGHT|TOP|BOTTOM) and "matchLayers" (bool):\n    {"type":"PUSH","direction":"LEFT","matchLayers":false,"duration":0.3,"easing":{"type":"EASE_OUT"}}\n\nEach reaction has a "trigger" and an "actions" array (plural). Each action in the array is an Action object.\n\nExample — on-click navigate with dissolve:\n{"nodeId":"1:2","reactions":[{"trigger":{"type":"ON_CLICK"},"actions":[{"type":"NODE","destinationId":"1:3","navigation":"NAVIGATE","transition":{"type":"DISSOLVE","duration":0.3,"easing":{"type":"EASE_OUT"}},"preserveScrollPosition":false}]}]}\n\nExample — on-click navigate with push (directional transition):\n{"nodeId":"1:2","reactions":[{"trigger":{"type":"ON_CLICK"},"actions":[{"type":"NODE","destinationId":"1:3","navigation":"NAVIGATE","transition":{"type":"PUSH","direction":"LEFT","matchLayers":false,"duration":0.3,"easing":{"type":"EASE_OUT"}},"preserveScrollPosition":false}]}]}\n\nExample — open URL on hover:\n{"nodeId":"1:2","reactions":[{"trigger":{"type":"ON_HOVER"},"actions":[{"type":"URL","url":"https://example.com"}]}]}\n\nExample — auto-advance after 3 seconds:\n{"nodeId":"1:2","reactions":[{"trigger":{"type":"AFTER_TIMEOUT","timeout":3000},"actions":[{"type":"NODE","destinationId":"1:4","navigation":"NAVIGATE","transition":{"type":"DISSOLVE","duration":0.3,"easing":{"type":"EASE_OUT"}},"preserveScrollPosition":false}]}]}\n\nExample — go back on click:\n{"nodeId":"1:2","reactions":[{"trigger":{"type":"ON_CLICK"},"actions":[{"type":"BACK"}]}]}
// MANIFEST: remove_reactions | RemoveReactionsArgs | Remove prototype reactions from a node. Omit indices to remove all reactions. Provide a zero-based indices array to remove specific reactions (use get_reactions first to see current indices).

//! Ported from figma-mcp-go internal/tools_write_prototype.go.

use std::sync::Arc;

use super::McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetReactionsArgs {
    /// Node ID in colon format e.g. '4029:12345'
    pub node_id: String,
    /// Array of reaction objects. Each has a 'trigger' and an 'actions' array (plural) of Action objects.
    pub reactions: Vec<serde_json::Value>,
    /// "replace" (default) overwrites all existing reactions; "append" adds to them
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

pub(crate) async fn set_reactions(
    node: Arc<Node>,
    args: SetReactionsArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_reactions", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RemoveReactionsArgs {
    /// Node ID in colon format e.g. '4029:12345'
    pub node_id: String,
    /// Zero-based indices of reactions to remove. Omit or pass [] to remove all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indices: Option<Vec<f64>>,
}

pub(crate) async fn remove_reactions(
    node: Arc<Node>,
    args: RemoveReactionsArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "remove_reactions", &args).await
}
