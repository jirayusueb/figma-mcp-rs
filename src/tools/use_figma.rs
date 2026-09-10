//! Escape hatch: run Plugin API JavaScript inside the Figma sandbox.
//! Covers every node type and API the granular tools don't wrap (FigJam
//! tables/shapes/code blocks, Slides, component variants, vector networks).

use std::sync::Arc;

use rmcp::model::CallToolResult;
use super::McpError;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UseFigmaArgs {
    /// JavaScript executed as the body of `async () => { ... }` inside the Figma plugin sandbox. Top-level `await` is allowed. `return` the data you need — console.log is discarded and only the returned value comes back.
    pub code: String,
}

pub(crate) async fn use_figma(
    node: Arc<Node>,
    args: UseFigmaArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "use_figma", &args).await
}
