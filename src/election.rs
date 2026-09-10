//! Election: determines the initial role and monitors leader health.
//! Ported from figma-mcp-go internal/election.go.

use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use tokio::task::JoinHandle;

use crate::follower::Follower;
use crate::node::Node;
use crate::types::Role;

pub struct Election {
    node: Arc<Node>,
    follower: Follower,
    monitor: Mutex<Option<JoinHandle<()>>>,
}

impl Election {
    pub fn new(node: Arc<Node>, ip: &str, port: u16) -> Self {
        Self {
            node,
            follower: Follower::new(ip, port),
            monitor: Mutex::new(None),
        }
    }

    /// Determine the initial role and launch the background monitor.
    pub async fn start(&self) {
        self.determine_role().await;
        *self.monitor.lock() = Some(tokio::spawn(monitor_loop(
            Arc::clone(&self.node),
            self.follower.clone(),
        )));
    }

    /// Cancel the background monitor task.
    pub fn stop(&self) {
        if let Some(handle) = self.monitor.lock().take() {
            handle.abort();
        }
    }

    /// Try to become leader; fall back to follower if a healthy leader exists.
    async fn determine_role(&self) {
        if self.node.become_leader().await.is_ok() {
            return;
        }
        // Port taken — check if there is a healthy leader.
        if self.follower.ping().await {
            self.node.become_follower().await;
            return;
        }
        // Port taken but no healthy leader — retry on the next tick.
        eprintln!("[election] port taken but leader not responding — will retry");
    }
}

async fn monitor_loop(node: Arc<Node>, follower: Follower) {
    loop {
        // Jitter: 3–5 seconds.
        let jitter = Duration::from_millis(rand::rng().random_range(3000..5000u64));
        tokio::time::sleep(jitter).await;

        match node.role() {
            Role::Follower => {
                if !follower.ping().await {
                    eprintln!("[election] leader not responding, attempting takeover...");
                    if let Err(e) = node.become_leader().await {
                        eprintln!("[election] takeover failed: {e}");
                    }
                }
            }
            Role::Unknown => {
                if node.become_leader().await.is_err() && follower.ping().await {
                    node.become_follower().await;
                }
            }
            Role::Leader => {}
        }
    }
}
