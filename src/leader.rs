//! Leader: owns the plugin WebSocket bridge and serves /ping + /rpc.
//! Ported from figma-mcp-go internal/leader.go.

use std::sync::Arc;

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use parking_lot::Mutex;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::bridge::Bridge;
use crate::schema::validate_rpc;
use crate::types::RpcResponse;

pub struct Leader {
    pub bridge: Arc<Bridge>,
    pub version: String,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
}

impl Leader {
    /// Bind the port and begin serving. Returns an error immediately if the
    /// port is already in use (caller detects another leader).
    pub async fn start(ip: &str, port: u16, version: &str) -> Result<Arc<Leader>, String> {
        let listener = TcpListener::bind((ip, port))
            .await
            .map_err(|e| e.to_string())?;

        let (tx, rx) = oneshot::channel::<()>();
        let leader = Arc::new(Leader {
            bridge: Arc::new(Bridge::new()),
            version: version.to_string(),
            shutdown: Mutex::new(Some(tx)),
        });

        let app = Router::new()
            .route("/ping", get(handle_ping))
            .route("/rpc", post(handle_rpc))
            .route("/ws", get(handle_ws))
            .with_state(Arc::clone(&leader));

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = rx.await;
                })
                .await
            {
                eprintln!("[leader] serve error: {e}");
            }
        });

        eprintln!("[leader] listening on {ip}:{port}");
        Ok(leader)
    }

    /// Shut down the HTTP server and close the bridge.
    pub fn stop(&self) {
        if let Some(tx) = self.shutdown.lock().take() {
            let _ = tx.send(());
        }
        self.bridge.close();
        eprintln!("[leader] stopped");
    }
}

async fn handle_ping(State(leader): State<Arc<Leader>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": leader.version,
        "pluginConnected": leader.bridge.connected(),
    }))
}

async fn handle_ws(
    State(leader): State<Arc<Leader>>,
    upgrade: WebSocketUpgrade,
) -> Response {
    leader.bridge.handle_upgrade(upgrade)
}

async fn handle_rpc(
    State(leader): State<Arc<Leader>>,
    Json(req): Json<crate::types::RpcRequest>,
) -> Response {
    let tool = req.tool.clone();
    let node_ids = req.node_ids.clone().unwrap_or_default();
    let params = req.params.clone().unwrap_or(serde_json::Value::Null);

    eprintln!("[leader] rpc {tool} nodeIDs={node_ids:?}");

    if let Some(validation_err) = validate_rpc(&tool, &node_ids, &params) {
        eprintln!("[leader] rpc {tool} validation error: {validation_err}");
        return rpc_error_response(StatusCode::BAD_REQUEST, validation_err);
    }

    match leader.bridge.send(&tool, node_ids, params).await {
        Ok(resp) => {
            if !resp.error_text().is_empty() {
                eprintln!("[leader] rpc {tool} plugin error: {}", resp.error_text());
                Json(RpcResponse {
                    data: None,
                    error: resp.error.clone(),
                })
                .into_response()
            } else {
                Json(RpcResponse {
                    data: resp.data,
                    error: None,
                })
                .into_response()
            }
        }
        Err(err) => {
            eprintln!("[leader] rpc {tool} bridge error: {err}");
            Json(RpcResponse {
                data: None,
                error: Some(err),
            })
            .into_response()
        }
    }
}

fn rpc_error_response(status: StatusCode, error: String) -> Response {
    (status, Json(RpcResponse { data: None, error: Some(error) })).into_response()
}

