use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Sent from the server to the Figma plugin over WebSocket.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeRequest {
    #[serde(rename = "type")]
    pub r#type: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Received from the Figma plugin over WebSocket.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BridgeResponse {
    #[serde(rename = "type")]
    pub r#type: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl BridgeResponse {
    pub fn error_text(&self) -> &str {
        self.error.as_deref().unwrap_or("")
    }
}

/// Wire format for follower → leader `/rpc` calls.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RpcRequest {
    pub tool: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl Default for RpcRequest {
    fn default() -> Self {
        Self { tool: String::new(), node_ids: None, params: None }
    }
}

/// Returned by the leader `/rpc` endpoint.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Current role of this server process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Role {
    #[default]
    Unknown,
    Leader,
    Follower,
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::Unknown => "UNKNOWN",
            Role::Leader => "LEADER",
            Role::Follower => "FOLLOWER",
        }
    }
}
