//! Single plugin WebSocket connection with request/response matching.
//! Ported from figma-mcp-go internal/bridge.go.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep_until;

use crate::types::{BridgeRequest, BridgeResponse};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const GET_DOCUMENT_TIMEOUT: Duration = Duration::from_secs(60);
const PROGRESS_TIMEOUT: Duration = Duration::from_secs(60);
const READ_LIMIT: usize = 100 * 1024 * 1024;

struct Pending {
    tx: Mutex<Option<oneshot::Sender<BridgeResponse>>>,
    deadline: Mutex<Instant>,
}

impl Pending {
    fn resolve(&self, resp: BridgeResponse) {
        if let Some(tx) = self.tx.lock().take() {
            let _ = tx.send(resp);
        }
    }
}

/// Manages the single WebSocket connection from the Figma plugin
/// and matches responses to pending requests via request IDs.
pub struct Bridge {
    writer: Mutex<Option<mpsc::Sender<Message>>>,
    pending: Mutex<HashMap<String, Arc<Pending>>>,
    counter: AtomicI64,
}

impl Default for Bridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Bridge {
    pub fn new() -> Self {
        Self {
            writer: Mutex::new(None),
            pending: Mutex::new(HashMap::new()),
            counter: AtomicI64::new(0),
        }
    }

    /// Handle a plugin WebSocket upgrade. A new connection replaces the old one.
    pub fn handle_upgrade(self: &Arc<Self>, upgrade: WebSocketUpgrade) -> Response {
        let this = Arc::clone(self);
        upgrade
            .max_message_size(READ_LIMIT)
            .max_frame_size(READ_LIMIT)
            .on_upgrade(move |socket| async move { this.on_socket(socket).await })
    }

    async fn on_socket(self: Arc<Self>, socket: WebSocket) {
        let (mut sink, mut stream) = socket.split();
        let (tx, mut rx) = mpsc::channel::<Message>(64);
        let tx_check = tx.clone();

        // Dropping the previous sender ends the old writer task, closing that socket.
        let replaced = {
            let mut w = self.writer.lock();
            let replaced = w.is_some();
            *w = Some(tx);
            replaced
        };
        if replaced {
            eprintln!("[bridge] plugin connected (replaced previous connection)");
        } else {
            eprintln!("[bridge] plugin connected");
        }

        let writer = async move {
            while let Some(msg) = rx.recv().await {
                if sink.send(msg).await.is_err() {
                    break;
                }
            }
        };
        let reader = self.read_loop(&mut stream);
        tokio::join!(writer, reader);

        let mut w = self.writer.lock();
        // Only clear if no newer connection has replaced us.
        if w.as_ref().is_some_and(|w| w.same_channel(&tx_check)) {
            *w = None;
        }
        eprintln!("[bridge] plugin disconnected");
    }

    async fn read_loop(
        self: &Arc<Self>,
        stream: &mut (impl StreamExt<Item = Result<Message, axum::Error>> + Unpin),
    ) {
        while let Some(Ok(msg)) = stream.next().await {
            let Message::Text(text) = msg else { continue };
            let Ok(resp) = serde_json::from_str::<BridgeResponse>(&text) else {
                eprintln!("[bridge] received non-JSON message — ignored");
                continue;
            };
            let progress = resp.progress.unwrap_or(0);
            if progress > 0 && !resp.request_id.is_empty() {
                if let Some(entry) = self.pending.lock().get(&resp.request_id) {
                    *entry.deadline.lock() = Instant::now() + PROGRESS_TIMEOUT;
                    eprintln!(
                        "[bridge] progress {}: {}% {}",
                        resp.request_id,
                        progress,
                        resp.message.as_deref().unwrap_or("")
                    );
                } else {
                    eprintln!("[bridge] progress {} (no pending entry)", resp.request_id);
                }
                continue;
            }
            if resp.request_id.is_empty() {
                eprintln!("[bridge] received message with empty requestID — ignored");
                continue;
            }
            let entry = self.pending.lock().remove(&resp.request_id);
            match entry {
                Some(entry) => {
                    if resp.error_text().is_empty() {
                        eprintln!("[bridge] ← {} ok", resp.request_id);
                    } else {
                        eprintln!(
                            "[bridge] ← {} error: {}",
                            resp.request_id,
                            resp.error_text()
                        );
                    }
                    entry.resolve(resp);
                }
                None => {
                    eprintln!(
                        "[bridge] ← {} received but no pending entry (timed out?)",
                        resp.request_id
                    );
                }
            }
        }
    }

    /// Send a request to the plugin and await the matched response.
    pub async fn send(
        &self,
        request_type: &str,
        node_ids: Vec<String>,
        params: Value,
    ) -> Result<BridgeResponse, String> {
        let writer = self.writer.lock().clone();
        let Some(writer) = writer else {
            return Err("plugin not connected".to_string());
        };

        let request_id = self.next_id();
        let req = BridgeRequest {
            r#type: request_type.to_string(),
            request_id: request_id.clone(),
            node_ids: (!node_ids.is_empty()).then_some(node_ids.clone()),
            params: (!params.is_null()).then_some(params),
        };

        let (tx, mut rx) = oneshot::channel();
        let pending = Arc::new(Pending {
            tx: Mutex::new(Some(tx)),
            deadline: Mutex::new(Instant::now() + initial_timeout(request_type)),
        });

        // Register before sending so a fast response cannot race registration.
        self.pending
            .lock()
            .insert(request_id.clone(), Arc::clone(&pending));

        eprintln!("[bridge] → {request_id} {request_type} nodeIDs={node_ids:?}");

        let frame = serde_json::to_string(&req).map_err(|e| format!("marshal request: {e}"))?;
        if writer.send(Message::Text(frame.into())).await.is_err() {
            self.pending.lock().remove(&request_id);
            eprintln!("[bridge] → {request_id} {request_type} write error");
            return Err("send: write failed".to_string());
        }

        // Wait for the response; progress frames push the deadline out.
        let mut deadline = *pending.deadline.lock();
        loop {
            tokio::select! {
                res = &mut rx => {
                    return match res {
                        Ok(resp) => Ok(resp),
                        Err(_) => Err("request timed out".to_string()),
                    };
                }
                _ = sleep_until(tokio::time::Instant::from_std(deadline)) => {
                    let now = Instant::now();
                    let next = *pending.deadline.lock();
                    if next > now {
                        deadline = next;
                        continue;
                    }
                    self.pending.lock().remove(&request_id);
                    eprintln!("[bridge] → {request_id} {request_type} timed out");
                    return Err("request timed out".to_string());
                }
            }
        }
    }

    /// Shut down the bridge, failing all pending requests.
    pub fn close(&self) {
        *self.writer.lock() = None;
        let mut pending = self.pending.lock();
        for (_, entry) in pending.drain() {
            entry.tx.lock().take(); // drop sender → waiter sees "request timed out"
        }
        eprintln!("[bridge] closed");
    }
    /// Check if a plugin is currently connected.
    pub fn connected(&self) -> bool {
        self.writer.lock().is_some()
    }

    fn next_id(&self) -> String {
        let n = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        format!("req-{}-{n}", hms_utc())
    }
}

fn initial_timeout(request_type: &str) -> Duration {
    if request_type == "get_document" {
        GET_DOCUMENT_TIMEOUT
    } else {
        DEFAULT_TIMEOUT
    }
}

// ponytail: request IDs use UTC HHMMSS (Go used local time); only uniqueness matters.
fn hms_utc() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let s = secs % 86_400;
    format!("{:02}{:02}{:02}", s / 3600, (s % 3600) / 60, s % 60)
}
