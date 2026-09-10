// MANIFEST: create_paint_style | CreatePaintStyleArgs | Create a new local paint style with a solid fill color.
// MANIFEST: create_text_style | CreateTextStyleArgs | Create a new local text style (typography preset). Returns the new style's ID. Apply it to nodes with apply_style_to_node. Use get_styles to list existing text styles.
// MANIFEST: create_effect_style | CreateEffectStyleArgs | Create a new local effect style (drop shadow, inner shadow, or blur).
// MANIFEST: create_grid_style | CreateGridStyleArgs | Create a new local layout grid style.
// MANIFEST: update_paint_style | UpdatePaintStyleArgs | Update an existing paint style's name, color, or description. Only paint styles support in-place updates — to modify text, effect, or grid styles, use delete_style and recreate them.
// MANIFEST: delete_style | DeleteStyleArgs | Delete a style (paint, text, effect, or grid) by its ID.
// MANIFEST: apply_style_to_node | ApplyStyleToNodeArgs | Apply an existing local style (paint, text, effect, or grid) to a node, linking the node to that style.
// MANIFEST: set_effects | SetEffectsArgs | Apply one or more effects (drop shadow, inner shadow, layer blur, background blur) directly to a node. Replaces all existing effects. Pass an empty array to clear all effects.
// MANIFEST: bind_variable_to_node | BindVariableToNodeArgs | Bind a local variable to a node property so the property is driven by the variable's value. COLOR variables: use fillColor or strokeColor. BOOLEAN variables: use visible. FLOAT variables: use opacity, rotation, width, height, cornerRadius, topLeftRadius, topRightRadius, bottomLeftRadius, bottomRightRadius, strokeWeight, itemSpacing, paddingTop, paddingRight, paddingBottom, paddingLeft.

//! Style tools: paint/text/effect/grid style creation, updates, and application.

use std::sync::Arc;

use super::McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreatePaintStyleArgs {
    /// Style name e.g. 'Brand/Primary'
    pub name: String,
    /// Fill color as hex e.g. #FF5733
    pub color: String,
    /// Optional style description
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateTextStyleArgs {
    /// Style name — use slash notation to organise into groups e.g. 'Heading/H1', 'Body/Regular'
    pub name: String,
    /// Font size in pixels (default 16)
    #[serde(default)]
    pub font_size: Option<f64>,
    /// Font family name e.g. 'Inter', 'Roboto' (default Inter). Must be installed in Figma.
    #[serde(default)]
    pub font_family: Option<String>,
    /// Font style variant e.g. 'Regular', 'Bold', 'Medium', 'SemiBold' (default Regular)
    #[serde(default)]
    pub font_style: Option<String>,
    /// Text decoration: NONE (default), UNDERLINE, or STRIKETHROUGH
    #[serde(default)]
    pub text_decoration: Option<String>,
    /// Line height value (unit set by lineHeightUnit)
    #[serde(default)]
    pub line_height_value: Option<f64>,
    /// Line height unit: PIXELS (default) or PERCENT
    #[serde(default)]
    pub line_height_unit: Option<String>,
    /// Letter spacing value (unit set by letterSpacingUnit)
    #[serde(default)]
    pub letter_spacing_value: Option<f64>,
    /// Letter spacing unit: PIXELS (default) or PERCENT
    #[serde(default)]
    pub letter_spacing_unit: Option<String>,
    /// Optional human-readable description shown in the Figma style panel
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateEffectStyleArgs {
    /// Style name e.g. 'Shadow/Card'
    pub name: String,
    /// Effect type: DROP_SHADOW (default), INNER_SHADOW, LAYER_BLUR, or BACKGROUND_BLUR
    #[serde(default, rename = "type")]
    pub r#type: Option<String>,
    /// Shadow color as hex e.g. #000000 (default #000000, shadows only)
    #[serde(default)]
    pub color: Option<String>,
    /// Shadow color opacity 0–1 (default 0.25, shadows only)
    #[serde(default)]
    pub opacity: Option<f64>,
    /// Blur radius in pixels (default 8 for shadows, 4 for blurs)
    #[serde(default)]
    pub radius: Option<f64>,
    /// Shadow X offset in pixels (default 0, shadows only)
    #[serde(default)]
    pub offset_x: Option<f64>,
    /// Shadow Y offset in pixels (default 4, shadows only)
    #[serde(default)]
    pub offset_y: Option<f64>,
    /// Shadow spread in pixels (default 0, shadows only)
    #[serde(default)]
    pub spread: Option<f64>,
    /// Optional style description
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateGridStyleArgs {
    /// Style name e.g. 'Grid/Desktop'
    pub name: String,
    /// Grid pattern: GRID (default), COLUMNS, or ROWS
    #[serde(default)]
    pub pattern: Option<String>,
    /// Number of columns or rows (COLUMNS/ROWS only, default 12)
    #[serde(default)]
    pub count: Option<f64>,
    /// Gutter size in pixels (COLUMNS/ROWS only, default 16)
    #[serde(default)]
    pub gutter_size: Option<f64>,
    /// Margin/offset in pixels (COLUMNS/ROWS only, default 0)
    #[serde(default)]
    pub offset: Option<f64>,
    /// Alignment: STRETCH (default), CENTER, MIN, or MAX (COLUMNS/ROWS only)
    #[serde(default)]
    pub alignment: Option<String>,
    /// Grid cell size in pixels (GRID only, default 8)
    #[serde(default)]
    pub section_size: Option<f64>,
    /// Grid line color as hex e.g. #FF0000 (GRID only, default #FF0000)
    #[serde(default)]
    pub color: Option<String>,
    /// Grid line opacity 0–1 (GRID only, default 0.1)
    #[serde(default)]
    pub opacity: Option<f64>,
    /// Optional style description
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdatePaintStyleArgs {
    /// Paint style ID
    pub style_id: String,
    /// New style name
    #[serde(default)]
    pub name: Option<String>,
    /// New fill color as hex e.g. #FF5733
    #[serde(default)]
    pub color: Option<String>,
    /// New style description
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeleteStyleArgs {
    /// Style ID to delete
    pub style_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApplyStyleToNodeArgs {
    /// Target node ID in colon format e.g. 4029:12345
    pub node_id: String,
    /// Style ID to apply (from get_styles)
    pub style_id: String,
    /// For paint styles only — apply to 'fill' (default) or 'stroke'
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetEffectsArgs {
    /// Target node ID in colon format e.g. 4029:12345
    pub node_id: String,
    /// Array of effect objects. Each has: type (DROP_SHADOW | INNER_SHADOW | LAYER_BLUR | BACKGROUND_BLUR), radius, color (hex, shadows only), opacity (0–1, shadows only), offsetX, offsetY (shadows only), spread (shadows only), visible (default true)
    pub effects: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BindVariableToNodeArgs {
    /// Target node ID in colon format e.g. 4029:12345
    pub node_id: String,
    /// Variable ID to bind (from get_variable_defs)
    pub variable_id: String,
    /// Property to bind: fillColor | strokeColor | visible | opacity | rotation | width | height | cornerRadius | topLeftRadius | topRightRadius | bottomLeftRadius | bottomRightRadius | strokeWeight | itemSpacing | paddingTop | paddingRight | paddingBottom | paddingLeft
    pub field: String,
}

pub(crate) async fn create_paint_style(
    node: Arc<Node>,
    args: CreatePaintStyleArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_paint_style", &args).await
}

pub(crate) async fn create_text_style(
    node: Arc<Node>,
    args: CreateTextStyleArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_text_style", &args).await
}

pub(crate) async fn create_effect_style(
    node: Arc<Node>,
    args: CreateEffectStyleArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_effect_style", &args).await
}

pub(crate) async fn create_grid_style(
    node: Arc<Node>,
    args: CreateGridStyleArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_grid_style", &args).await
}

pub(crate) async fn update_paint_style(
    node: Arc<Node>,
    args: UpdatePaintStyleArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "update_paint_style", &args).await
}

pub(crate) async fn delete_style(
    node: Arc<Node>,
    args: DeleteStyleArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "delete_style", &args).await
}

pub(crate) async fn apply_style_to_node(
    node: Arc<Node>,
    args: ApplyStyleToNodeArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "apply_style_to_node", &args).await
}

pub(crate) async fn set_effects(
    node: Arc<Node>,
    args: SetEffectsArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "set_effects", &args).await
}

pub(crate) async fn bind_variable_to_node(
    node: Arc<Node>,
    args: BindVariableToNodeArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "bind_variable_to_node", &args).await
}
