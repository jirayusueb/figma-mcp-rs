// MANIFEST: create_frame | CreateFrameArgs | Create a new frame on the current page or inside a parent node.
// MANIFEST: create_rectangle | CreateRectangleArgs | Create a new rectangle on the current page or inside a parent node.
// MANIFEST: create_ellipse | CreateEllipseArgs | Create a new ellipse (circle/oval) on the current page or inside a parent node.
// MANIFEST: create_text | CreateTextArgs | Create a new text node on the current page or inside a parent node. The font is loaded automatically before insertion. Returns the created node ID and bounds. Use set_text to update the content of an existing text node.
// MANIFEST: import_image | ImportImageArgs | Import a base64-encoded image into Figma as a rectangle with an image fill. Use get_screenshot to capture images or provide your own base64 PNG/JPG.
// MANIFEST: create_component | CreateComponentArgs | Convert an existing FRAME node into a reusable COMPONENT. The frame is replaced in place by the new component.
// MANIFEST: create_section | CreateSectionArgs | Create a Figma Section node on the current page. Sections are the modern way to organize frames and groups on a page.

//! Ported from figma-mcp-go internal/tools_write_create.go.

use std::sync::Arc;

use super::McpError;
use rmcp::model::CallToolResult;
use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateFrameArgs {
    /// X position (default 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Y position (default 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    /// Width in pixels (default 100)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    /// Height in pixels (default 100)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    /// Frame name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Fill color as hex e.g. #FFFFFF
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
    /// Auto-layout direction: HORIZONTAL, VERTICAL, or NONE
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_mode: Option<String>,
    /// Auto-layout top padding
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_top: Option<f64>,
    /// Auto-layout right padding
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<f64>,
    /// Auto-layout bottom padding
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_bottom: Option<f64>,
    /// Auto-layout left padding
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<f64>,
    /// Auto-layout gap between children
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_spacing: Option<f64>,
    /// Main-axis alignment: MIN, CENTER, MAX, or SPACE_BETWEEN
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_axis_align_items: Option<String>,
    /// Cross-axis alignment: MIN, CENTER, MAX, or BASELINE
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter_axis_align_items: Option<String>,
    /// Main-axis sizing: FIXED or AUTO (hug)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_axis_sizing_mode: Option<String>,
    /// Cross-axis sizing: FIXED or AUTO (hug)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter_axis_sizing_mode: Option<String>,
    /// Wrap behaviour: NO_WRAP or WRAP
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_wrap: Option<String>,
    /// Gap between wrapped rows/columns (only when layoutWrap is WRAP)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter_axis_spacing: Option<f64>,
    /// Alignment of wrapped tracks: AUTO or SPACE_BETWEEN (only when layoutWrap is WRAP)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter_axis_align_content: Option<String>,
    /// Reverse child z-order so the first child renders on top
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_reverse_z_index: Option<bool>,
    /// Include strokes in layout size calculations
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strokes_included_in_layout: Option<bool>,
    /// Horizontal sizing: FIXED, HUG (auto-layout frames and text only), or FILL (only for a node inside an auto-layout parent)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_sizing_horizontal: Option<String>,
    /// Vertical sizing: FIXED, HUG (auto-layout frames and text only), or FILL (only for a node inside an auto-layout parent)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_sizing_vertical: Option<String>,
    /// AUTO to flow inside the parent's auto layout, ABSOLUTE to position freely inside it
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_positioning: Option<String>,
    /// Parent node ID in colon format. Defaults to current page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

pub(crate) async fn create_frame(
    node: Arc<Node>,
    args: CreateFrameArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_frame", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateRectangleArgs {
    /// X position (default 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Y position (default 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    /// Width in pixels (default 100)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    /// Height in pixels (default 100)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    /// Rectangle name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Fill color as hex e.g. #FF5733
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
    /// Corner radius in pixels
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f64>,
    /// Parent node ID in colon format. Defaults to current page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

pub(crate) async fn create_rectangle(
    node: Arc<Node>,
    args: CreateRectangleArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_rectangle", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateEllipseArgs {
    /// X position (default 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Y position (default 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    /// Width in pixels (default 100)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    /// Height in pixels (default 100)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    /// Ellipse name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Fill color as hex e.g. #3B82F6
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
    /// Parent node ID in colon format. Defaults to current page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

pub(crate) async fn create_ellipse(
    node: Arc<Node>,
    args: CreateEllipseArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_ellipse", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateTextArgs {
    /// Text content to display
    pub text: String,
    /// X position in pixels (default 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Y position in pixels (default 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    /// Font size in pixels (default 14)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    /// Font family name e.g. 'Inter', 'Roboto', 'SF Pro Display' (default Inter). Must be a font installed in Figma.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    /// Font style variant e.g. 'Regular', 'Bold', 'Italic', 'Medium', 'SemiBold' (default Regular). Must match an available style for the chosen fontFamily.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    /// Text color as hex e.g. #000000 (default black)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
    /// Node name shown in the layers panel (defaults to the text content)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Parent node ID in colon format. Defaults to current page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

pub(crate) async fn create_text(
    node: Arc<Node>,
    args: CreateTextArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_text", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportImageArgs {
    /// Base64-encoded image data (PNG or JPG)
    pub image_data: String,
    /// X position (default 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Y position (default 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    /// Width in pixels (default 200)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    /// Height in pixels (default 200)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    /// Node name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Image scale mode: FILL (default), FIT, CROP, or TILE
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_mode: Option<String>,
    /// Parent node ID in colon format. Defaults to current page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

pub(crate) async fn import_image(
    node: Arc<Node>,
    args: ImportImageArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "import_image", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateComponentArgs {
    /// FRAME node ID to convert, in colon format e.g. '4029:12345'
    pub node_id: String,
    /// Optional name for the component. Defaults to the frame's current name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

pub(crate) async fn create_component(
    node: Arc<Node>,
    args: CreateComponentArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_component", &args).await
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateSectionArgs {
    /// Section name (default 'Section')
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// X position (default 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Y position (default 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    /// Width in pixels
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    /// Height in pixels
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
}

pub(crate) async fn create_section(
    node: Arc<Node>,
    args: CreateSectionArgs,
) -> Result<CallToolResult, McpError> {
    crate::tools::relay(&node, "create_section", &args).await
}
