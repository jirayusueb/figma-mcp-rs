//! Follower: proxies MCP tool calls to the leader via HTTP /rpc.
//! Ported from figma-mcp-go internal/follower.go.

use std::time::Duration;

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use serde_json::Value;

use crate::types::{BridgeResponse, RpcRequest, RpcResponse};

const RPC_TIMEOUT: Duration = Duration::from_secs(35);
const PING_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone)]
pub struct Follower {
    leader_url: String,
    client: Client<HttpConnector, Full<Bytes>>,
}

impl Follower {
    pub fn new(ip: &str, port: u16) -> Self {
        Self {
            leader_url: format!("http://{ip}:{port}"),
            client: Client::builder(TokioExecutor::new()).build_http(),
        }
    }

    /// Proxy a tool call to the leader.
    pub async fn send(
        &self,
        tool: &str,
        node_ids: Vec<String>,
        params: Value,
    ) -> Result<BridgeResponse, String> {
        let rpc_req = RpcRequest {
            tool: tool.to_string(),
            node_ids: (!node_ids.is_empty()).then_some(node_ids),
            params: (!params.is_null()).then_some(params),
        };
        let body = serde_json::to_string(&rpc_req).map_err(|e| format!("marshal: {e}"))?;

        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(format!("{}/rpc", self.leader_url))
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(body)))
            .map_err(|e| format!("new request: {e}"))?;

        let resp = tokio::time::timeout(RPC_TIMEOUT, self.client.request(req))
            .await
            .map_err(|_| "rpc call: timeout".to_string())?
            .map_err(|e| format!("rpc call: {e}"))?;

        let (_, body) = resp.into_parts();
        let bytes = body
            .collect()
            .await
            .map_err(|e| format!("read response: {e}"))?
            .to_bytes();

        let rpc_resp: RpcResponse = serde_json::from_slice(&bytes)
            .map_err(|e| format!("unmarshal: {e}"))?;

        if let Some(err) = rpc_resp.error {
            if !err.is_empty() {
                return Ok(BridgeResponse {
                    r#type: tool.to_string(),
                    error: Some(err),
                    ..Default::default()
                });
            }
        }
        Ok(BridgeResponse {
            r#type: tool.to_string(),
            data: rpc_resp.data,
            ..Default::default()
        })
    }

    /// Check if the leader is alive. Returns true if healthy.
    pub async fn ping(&self) -> bool {
        let req = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(format!("{}/ping", self.leader_url))
            .body(Full::<Bytes>::default());
        let Ok(req) = req else { return false };
        match tokio::time::timeout(PING_TIMEOUT, self.client.request(req)).await {
            Ok(Ok(resp)) => resp.status() == hyper::StatusCode::OK,
            _ => false,
        }
    }
    /// Get the leader's status, including pluginConnected. Returns None on any failure.
    pub async fn ping_status(&self) -> Option<Value> {
        let req = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(format!("{}/ping", self.leader_url))
            .body(Full::<Bytes>::default())
            .ok()?;
        let resp = tokio::time::timeout(PING_TIMEOUT, self.client.request(req))
            .await
            .ok()?
            .ok()?;
        if resp.status() != hyper::StatusCode::OK {
            return None;
        }
        let (_, body) = resp.into_parts();
        let bytes = body.collect().await.ok()?.to_bytes();
        serde_json::from_slice(&bytes).ok()
    }
}
