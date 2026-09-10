//! Node: routes MCP tool calls to the leader bridge or the follower proxy.
//! Ported from figma-mcp-go internal/node.go.

use std::sync::Arc;

use parking_lot::RwLock;
use serde_json::Value;

use crate::follower::Follower;
use crate::leader::Leader;
use crate::schema::normalize_node_id;
use crate::types::{BridgeResponse, Role};

pub struct Node {
    state: RwLock<NodeState>,
    ip: String,
    port: u16,
    version: String,
    follower: Follower,
}

struct NodeState {
    role: Role,
    leader: Option<Arc<Leader>>,
}

impl Node {
    pub fn new(ip: &str, port: u16, version: &str) -> Self {
        Self {
            state: RwLock::new(NodeState { role: Role::Unknown, leader: None }),
            ip: ip.to_string(),
            port,
            version: version.to_string(),
            follower: Follower::new(ip, port),
        }
    }

    pub fn role(&self) -> Role {
        self.state.read().role
    }

    pub fn role_name(&self) -> &'static str {
        self.role().name()
    }

    /// Route a request to the appropriate backend.
    pub async fn send(
        &self,
        tool: &str,
        node_ids: Vec<String>,
        params: Value,
    ) -> Result<BridgeResponse, String> {
        // Normalize hyphen-format node IDs that LLMs sometimes produce.
        let node_ids: Vec<String> = node_ids.iter().map(|id| normalize_node_id(id)).collect();
        let mut params = params;
        // Normalize common param keys that contain node IDs.
        for key in ["nodeId", "parentId"] {
            if let Some(s) = params.get_mut(key).and_then(|v| v.as_str()) {
                let normalized = normalize_node_id(s);
                params[key] = Value::String(normalized);
            }
        }

        let (role, leader) = {
            let s = self.state.read();
            (s.role, s.leader.clone())
        };
        eprintln!("[node] tool={tool} role={} nodeIDs={node_ids:?}", role.name());

        if role == Role::Leader {
            if let Some(leader) = leader {
                return leader.bridge.send(tool, node_ids, params).await;
            }
        }
        self.follower.send(tool, node_ids, params).await
    }

    /// Attempt to bind the port and become Leader. Errors if the port is in use.
    pub async fn become_leader(&self) -> Result<(), String> {
        {
            let s = self.state.read();
            if s.role == Role::Leader {
                return Ok(());
            }
        }
        let leader = Leader::start(&self.ip, self.port, &self.version).await?;
        {
            let mut s = self.state.write();
            s.leader = Some(leader);
            s.role = Role::Leader;
        }
        eprintln!("[node] became LEADER");
        Ok(())
    }

    /// Become Follower, stopping the leader if running.
    pub async fn become_follower(&self) {
        {
            let mut s = self.state.write();
            if s.role == Role::Follower {
                return;
            }
            if let Some(leader) = s.leader.take() {
                leader.stop();
            }
            s.role = Role::Follower;
        }
        eprintln!("[node] became FOLLOWER");
    }

    /// Shut down the node regardless of role.
    pub fn stop(&self) {
        let mut s = self.state.write();
        if let Some(leader) = s.leader.take() {
            leader.stop();
        }
        s.role = Role::Unknown;
    }
    /// Get the current status including role, connectivity, and address.
    pub async fn status(&self) -> serde_json::Value {
        let (role, leader) = {
            let s = self.state.read();
            (s.role, s.leader.clone())
        };

        if role == Role::Leader {
            if let Some(leader) = leader {
                return serde_json::json!({
                    "role": "LEADER",
                    "address": format!("{}:{}", self.ip, self.port),
                    "version": self.version,
                    "pluginConnected": leader.bridge.connected(),
                });
            }
        }

        // Follower: try to get status from leader.
        let ping_status = self.follower.ping_status().await;
        if let Some(status) = ping_status {
            let plugin_connected = status.get("pluginConnected");
            return serde_json::json!({
                "role": "FOLLOWER",
                "address": format!("{}:{}", self.ip, self.port),
                "version": self.version,
                "leaderReachable": true,
                "pluginConnected": plugin_connected,
            });
        }

        // Leader unreachable.
        serde_json::json!({
            "role": "FOLLOWER",
            "address": format!("{}:{}", self.ip, self.port),
            "version": self.version,
            "leaderReachable": false,
            "pluginConnected": serde_json::Value::Null,
        })
    }
}
