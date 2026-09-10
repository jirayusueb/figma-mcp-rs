// MANIFEST: get_status | @server | Check whether the Figma plugin bridge is connected and get setup instructions when it is not.

use std::sync::Arc;

use rmcp::model::{CallToolResult, ContentBlock};

use crate::node::Node;
use crate::plugin_assets;

pub(crate) async fn get_status(node: Arc<Node>) -> Result<CallToolResult, super::McpError> {
    let status = node.status().await;

    let ok = status
        .get("pluginConnected")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let plugin_manifest_path = plugin_assets::install_dir().and_then(|p| {
        let manifest_path = p.join("manifest.json");
        if manifest_path.exists() {
            Some(manifest_path.to_string_lossy().to_string())
        } else {
            None
        }
    });

    let mut output = serde_json::json!({
        "ok": ok,
        "role": status.get("role").cloned().unwrap_or_else(|| serde_json::Value::String("UNKNOWN".to_string())),
        "address": status.get("address").cloned().unwrap_or_else(|| serde_json::Value::String("unknown".to_string())),
        "version": status.get("version").cloned().unwrap_or_else(|| serde_json::Value::String("unknown".to_string())),
        "pluginConnected": status.get("pluginConnected").cloned().unwrap_or(serde_json::Value::Null),
        "pluginManifestPath": plugin_manifest_path.as_ref().map(|p| serde_json::Value::String(p.clone())).unwrap_or(serde_json::Value::Null),
    });

    if ok {
        output["message"] = serde_json::Value::String(
            "Figma plugin connected — all 84 tools are available.".to_string(),
        );
    } else {
        let fallback_msg = if plugin_manifest_path.is_some() {
            "…select plugin/manifest.json and run the plugin."
        } else {
            "…select plugin/manifest.json from this repo (plugin assets are not installed)."
        };

        let manifest_text = plugin_manifest_path
            .as_ref()
            .map(|p| {
                format!("Plugins → Development → Import plugin from manifest… and select {p}.")
            })
            .unwrap_or_else(|| {
                format!("Plugins → Development → Import plugin from manifest… {fallback_msg}")
            });

        let setup_instructions = [
            "Open Figma Desktop (not the browser) and open the file you want to work in.",
            &manifest_text,
            "Run the plugin (Plugins → Development → Figma MCP RS) and leave its window open.",
            "The plugin connects to ws://{address}/ws; change host/port in the plugin UI if you started the server with --ip/--port.",
        ];
        output["setupInstructions"] = serde_json::Value::Array(
            setup_instructions
                .iter()
                .map(|s| serde_json::Value::String(s.to_string()))
                .collect(),
        );
    }

    Ok(CallToolResult::success(vec![ContentBlock::text(
        output.to_string(),
    )]))
}
