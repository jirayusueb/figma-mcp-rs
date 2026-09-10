// MANIFEST: execute_code | ExecuteCodeArgs | Run arbitrary Figma Plugin API code inside the plugin sandbox and return its value.

use std::sync::Arc;

use rmcp::model::CallToolResult;
use super::McpError;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::node::Node;

/// Arguments for execute_code tool.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExecuteCodeArgs {
    /// The code to execute. It runs as an async function body — use `await` and end with a `return`.
    /// Use this only for operations the dedicated tools do not cover; prefer the specific tool when one exists.
    pub code: String,
    /// Optional timeout in milliseconds (1-25000, default 5000). Clipped to keep it under the bridge's 30s deadline.
    #[serde(default)]
    pub timeout_ms: Option<f64>,
}
pub(crate) async fn execute_code(node: Arc<Node>, args: ExecuteCodeArgs) -> Result<CallToolResult, McpError> {
    super::relay(&node, "execute_code", &args).await
}
