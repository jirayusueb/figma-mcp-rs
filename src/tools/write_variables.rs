// MANIFEST: create_variable_collection | CreateVariableCollectionArgs | Create a new local variable collection with an optional initial mode name. NOTE — Figma free plan limits each collection to 1 mode. If you need Light/Dark (or any multi-mode) theming and the user is on the free plan, do NOT try to call add_variable_mode; instead use the name-prefix workaround: create all variables in a single collection and prefix each variable name with its mode, e.g. 'light/color-bg' and 'dark/color-bg'. Inform the user of this limitation.
// MANIFEST: add_variable_mode | AddVariableModeArgs | Add a new mode to an existing variable collection (e.g. Light/Dark, Desktop/Mobile). IMPORTANT — Figma free plan only allows 1 mode per collection; calling this tool on a free-plan account will return the error 'Limited to 1 modes only'. If that error occurs, stop retrying and switch to the name-prefix workaround: keep the single default mode and create variables prefixed by mode, e.g. 'light/color-bg' and 'dark/color-bg' in the same collection. Tell the user that native multi-mode variables require a paid Figma plan (Professional or above).
// MANIFEST: create_variable | CreateVariableArgs | Create a new variable (design token) inside an existing collection. Returns the new variable's ID. Use get_variable_defs to find collection IDs, set_variable_value to set values per mode, and bind_variable_to_node to apply the variable to a node property.
// MANIFEST: set_variable_value | SetVariableValueArgs | Set a variable's value for a specific mode.
// MANIFEST: delete_variable | DeleteVariableArgs | Delete a single variable (provide variableId) or an entire collection and all its variables (provide collectionId). Provide exactly one of the two — not both.

//! Variable (design token) tools: collections, modes, variables, and values.

use std::sync::Arc;

use super::McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateVariableCollectionArgs {
    /// Collection name
    pub name: String,
    /// Name for the initial mode (default 'Mode 1')
    #[serde(default)]
    pub initial_mode_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AddVariableModeArgs {
    /// Variable collection ID
    pub collection_id: String,
    /// Name for the new mode
    pub mode_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateVariableArgs {
    /// Variable name — use slash notation to group e.g. 'Color/Primary', 'Spacing/MD'
    pub name: String,
    /// ID of the variable collection to add this variable to (from get_variable_defs)
    pub collection_id: String,
    /// Variable type: COLOR (hex color), FLOAT (numeric dimension/spacing), STRING (text), or BOOLEAN (true/false toggle)
    #[serde(rename = "type")]
    pub r#type: String,
    /// Initial value for the first mode. COLOR: hex e.g. #FF5733. FLOAT: number e.g. 16. STRING: text. BOOLEAN: true or false.
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetVariableValueArgs {
    /// Variable ID
    pub variable_id: String,
    /// Mode ID within the collection
    pub mode_id: String,
    /// Value to set. COLOR: hex e.g. #FF5733. FLOAT: number e.g. 16. STRING: text. BOOLEAN: true or false.
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeleteVariableArgs {
    /// Variable ID to delete
    #[serde(default)]
    pub variable_id: Option<String>,
    /// Collection ID to delete (removes all variables in the collection)
    #[serde(default)]
    pub collection_id: Option<String>,
}

pub(crate) async fn create_variable_collection(
    node: Arc<Node>,
    args: CreateVariableCollectionArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_variable_collection", &args).await
}

pub(crate) async fn add_variable_mode(
    node: Arc<Node>,
    args: AddVariableModeArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "add_variable_mode", &args).await
}

pub(crate) async fn create_variable(
    node: Arc<Node>,
    args: CreateVariableArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_variable", &args).await
}

pub(crate) async fn set_variable_value(
    node: Arc<Node>,
    args: SetVariableValueArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_variable_value", &args).await
}

pub(crate) async fn delete_variable(
    node: Arc<Node>,
    args: DeleteVariableArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "delete_variable", &args).await
}
