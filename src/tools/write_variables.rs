// MANIFEST: create_variable_collection | CreateVariableCollectionArgs | Create a new local variable collection with an optional initial mode name. NOTE — Figma free plan limits each collection to 1 mode. If you need Light/Dark (or any multi-mode) theming and the user is on the free plan, do NOT try to call add_variable_mode; instead use the name-prefix workaround: create all variables in a single collection and prefix each variable name with its mode, e.g. 'light/color-bg' and 'dark/color-bg'. Inform the user of this limitation.
// MANIFEST: add_variable_mode | AddVariableModeArgs | Add a new mode to an existing variable collection (e.g. Light/Dark, Desktop/Mobile). IMPORTANT — Figma free plan only allows 1 mode per collection; calling this tool on a free-plan account will return the error 'Limited to 1 modes only'. If that error occurs, stop retrying and switch to the name-prefix workaround: keep the single default mode and create variables prefixed by mode, e.g. 'light/color-bg' and 'dark/color-bg' in the same collection. Tell the user that native multi-mode variables require a paid Figma plan (Professional or above).
// MANIFEST: create_variable | CreateVariableArgs | Create a new variable (design token) inside an existing collection. Returns the new variable's ID. Use get_variable_defs to find collection IDs, set_variable_value to set values per mode, and bind_variable_to_node to apply the variable to a node property.
// MANIFEST: set_variable_value | SetVariableValueArgs | Set a variable's value for a specific mode. Pass value for a literal (hex, rgb(), hsl(), or oklch() for COLOR), or aliasVariableId to point this variable at another variable (semantic token -> primitive).
// MANIFEST: update_variable | UpdateVariableArgs | Update an existing variable's name, description, scopes, code syntax, or publish visibility (pass variableId), or rename a collection / rename one of its modes (pass collectionId, plus modeId + modeName to rename a mode). Provide exactly one of variableId or collectionId.
// MANIFEST: set_variable_mode | SetVariableModeArgs | Switch which mode of a variable collection applies to a node (or the whole current page when nodeId is omitted) — this is how you preview Light/Dark or Desktop/Mobile theming. Omit modeId to clear the override and inherit from the parent.
// MANIFEST: delete_variable | DeleteVariableArgs | Delete a single variable (provide variableId) or an entire collection and all its variables (provide collectionId). Pass collectionId + modeId to remove a single mode instead. Provide exactly one of the two — not both.

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
    /// Variable type: COLOR (hex, rgb(), hsl(), or oklch()), FLOAT (numeric dimension/spacing), STRING (text), or BOOLEAN (true/false toggle)
    #[serde(rename = "type")]
    pub r#type: String,
    /// Initial value for the first mode. COLOR: hex, rgb(), hsl(), or oklch() e.g. #FF5733, hsl(210 90% 55%), oklch(0.72 0.13 250). FLOAT: number e.g. 16. STRING: text. BOOLEAN: true or false.
    #[serde(default)]
    pub value: Option<String>,
    /// Reference another variable instead of a literal value — the target variable's ID (alias). Takes precedence over value.
    #[serde(default)]
    pub alias_variable_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetVariableValueArgs {
    /// Variable ID
    pub variable_id: String,
    /// Mode ID within the collection
    pub mode_id: String,
    /// Value to set. COLOR: hex, rgb(), hsl(), or oklch() e.g. #FF5733, hsl(210 90% 55%), oklch(0.72 0.13 250). FLOAT: number e.g. 16. STRING: text. BOOLEAN: true or false. Omit when using aliasVariableId.
    #[serde(default)]
    pub value: Option<String>,
    /// Point this variable at another variable (alias) for this mode — the target variable's ID. Use for semantic tokens that reference primitives.
    #[serde(default)]
    pub alias_variable_id: Option<String>,
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
    /// Mode ID to remove from collectionId (removes just that mode, not the collection)
    #[serde(default)]
    pub mode_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateVariableArgs {
    /// Variable ID to update (from get_variable_defs)
    #[serde(default)]
    pub variable_id: Option<String>,
    /// Variable collection ID to update instead — renames the collection or one of its modes
    #[serde(default)]
    pub collection_id: Option<String>,
    /// New name for the variable or collection
    #[serde(default)]
    pub name: Option<String>,
    /// New description (variables only)
    #[serde(default)]
    pub description: Option<String>,
    /// Where this variable may be used in the Figma UI, e.g. ["ALL_FILLS"], ["CORNER_RADIUS"], ["GAP"], ["FONT_SIZE"]. Pass ["ALL_SCOPES"] to allow everywhere.
    #[serde(default)]
    pub scopes: Option<Vec<String>>,
    /// Dev Mode code syntax for web e.g. '--color-primary'
    #[serde(default)]
    pub code_syntax_web: Option<String>,
    /// Dev Mode code syntax for Android
    #[serde(default)]
    pub code_syntax_android: Option<String>,
    /// Dev Mode code syntax for iOS
    #[serde(default)]
    pub code_syntax_ios: Option<String>,
    /// Hide from library publishing
    #[serde(default)]
    pub hidden_from_publishing: Option<bool>,
    /// With collectionId — the mode to rename (requires modeName)
    #[serde(default)]
    pub mode_id: Option<String>,
    /// With collectionId + modeId — the mode's new name
    #[serde(default)]
    pub mode_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetVariableModeArgs {
    /// Variable collection whose mode is being overridden (from get_variable_defs)
    pub collection_id: String,
    /// Mode ID to apply; omit to clear the override and inherit from the parent
    #[serde(default)]
    pub mode_id: Option<String>,
    /// Node to scope the mode to, colon format e.g. 4029:12345; omit to set it on the whole current page
    #[serde(default)]
    pub node_id: Option<String>,
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

pub(crate) async fn update_variable(
    node: Arc<Node>,
    args: UpdateVariableArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "update_variable", &args).await
}

pub(crate) async fn set_variable_mode(
    node: Arc<Node>,
    args: SetVariableModeArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_variable_mode", &args).await
}
