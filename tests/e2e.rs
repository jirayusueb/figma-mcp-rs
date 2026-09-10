//! End-to-end tests: spawn the binary, attach a fake plugin over WebSocket,
//! drive MCP over stdio, and exercise leader/follower election.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

/// Bind 127.0.0.1:0, return the port, drop the listener.
fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

struct Server {
    child: Child,
    stdout: BufReader<std::process::ChildStdout>,
}

impl Server {
    fn spawn(port: u16) -> Self {
        Self::spawn_in(port, std::env::current_dir().unwrap())
    }

    fn spawn_in(port: u16, dir: std::path::PathBuf) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_figma-mcp-rs"))
            .args(["--port", &port.to_string()])
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn server");
        let stdout = child.stdout.take().unwrap();
        Server {
            child,
            stdout: BufReader::new(stdout),
        }
    }

    fn rpc(&mut self, method: &str, params: Value, id: u64) -> Value {
        let req = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        writeln!(self.child.stdin.as_mut().unwrap(), "{req}").unwrap();
        self.child.stdin.as_mut().unwrap().flush().unwrap();
        let mut line = String::new();
        self.stdout.read_line(&mut line).unwrap();
        serde_json::from_str(&line).expect("JSON-RPC response line")
    }

    fn notify(&mut self, method: &str) {
        let req = json!({"jsonrpc": "2.0", "method": method});
        writeln!(self.child.stdin.as_mut().unwrap(), "{req}").unwrap();
    }

    fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}


/// Minimal blocking HTTP GET returning the status line's code.
fn http_get_status(url_host: &str, path: &str) -> Option<u16> {
    let mut stream =
        TcpStream::connect(url_host).ok()?;
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {url_host}\r\nConnection: close\r\n\r\n"
    )
    .ok()?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf).ok()?;
    buf.split_whitespace().nth(1)?.parse().ok()
}

fn wait_ping(port: u16, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    loop {
        if http_get_status(&format!("127.0.0.1:{port}"), "/ping") == Some(200) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "server did not become healthy in time"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

async fn connect_fake_plugin(port: u16) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let url = format!("ws://127.0.0.1:{port}/ws")
        .into_client_request()
        .unwrap();
    let (ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    ws
}

fn parse_bridge_frame(msg: &Message) -> Option<Value> {
    let text = match msg {
        Message::Text(t) => t.as_str(),
        _ => return None,
    };
    serde_json::from_str(text).ok()
}

#[tokio::test(flavor = "multi_thread")]
async fn e2e_full_stack() {
    let port = free_port();
    let mut server = Server::spawn(port);
    wait_ping(port, Duration::from_secs(15));

    // A malformed line on stdin must not tear the session down (rmcp skips bad frames).
    writeln!(server.child.stdin.as_mut().unwrap(), "{{not json").unwrap();
    server.child.stdin.as_mut().unwrap().flush().unwrap();

    // MCP handshake.
    let init = server.rpc(
        "initialize",
        json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "e2e", "version": "0"}
        }),
        1,
    );
    assert!(
        init.get("result").is_some(),
        "initialize failed: {init}"
    );
    assert_eq!(init["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(init["result"]["serverInfo"]["name"], "figma-mcp-rs");
    assert_eq!(init["result"]["serverInfo"]["version"], env!("CARGO_PKG_VERSION"));
    server.notify("notifications/initialized");

    // tools/list: expect full parity (73 reference + 2 FigJam + use_figma).
    let tools = server.rpc("tools/list", json!({}), 2);
    let names: Vec<String> = tools["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();
    assert!(
        names.len() >= 76,
        "expected >= 76 tools, got {}: {names:?}",
        names.len()
    );
    for expected in ["get_metadata", "create_sticky", "create_connector", "save_screenshots", "get_document", "use_figma"] {
        assert!(names.iter().any(|n| n == expected), "missing tool {expected}");
    }


    // Drive the call from this task: send JSON-RPC line, then pump plugin frames.
    let req = json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call",
        "params": {"name": "get_metadata", "arguments": {}}});
    let mut plugin = connect_fake_plugin(port).await;
    writeln!(server.child.stdin.as_mut().unwrap(), "{req}").unwrap();

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut answered = false;
    while Instant::now() < deadline {
        let msg = tokio::time::timeout(Duration::from_secs(5), futures_util::StreamExt::next(&mut plugin))
            .await
            .expect("plugin frame within 5s")
            .expect("plugin stream open")
            .expect("ws message");
        let frame = parse_bridge_frame(&msg).expect("bridge JSON frame");
        let request_id = frame["requestId"].as_str().unwrap().to_string();
        if frame["progress"].as_i64().unwrap_or(0) > 0 {
            continue; // progress frame must NOT resolve the request
        }
        assert_eq!(frame["type"], "get_metadata");
        let resp = json!({
            "type": "get_metadata",
            "requestId": request_id,
            "data": {"fileName": "E2E-OK", "pages": []}
        });
        futures_util::SinkExt::send(
            &mut plugin,
            Message::Text(resp.to_string().into()),
        )
        .await
        .unwrap();
        answered = true;
        break;
    }
    assert!(answered, "no plugin request arrived");

    // Read the tools/call response from the server's stdout.
    let mut line = String::new();
    server.stdout.read_line(&mut line).unwrap();
    assert!(line.contains("E2E-OK"), "tools/call result missing E2E-OK: {line}");

    futures_util::SinkExt::close(&mut plugin).await.ok();
    server.kill();
}


#[tokio::test(flavor = "multi_thread")]
async fn e2e_election_takeover() {
    let port = free_port();
    let leader = Server::spawn(port);
    wait_ping(port, Duration::from_secs(15));

    // Second instance on the same port becomes follower.
    let mut follower = Command::new(env!("CARGO_BIN_EXE_figma-mcp-rs"))
        .args(["--port", &port.to_string()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn follower");
    let deadline = Instant::now() + Duration::from_secs(20);

    loop {
        if http_get_status(&format!("127.0.0.1:{port}"), "/ping") == Some(200) {
            break;
        }
        assert!(Instant::now() < deadline, "follower did not take over");
        std::thread::sleep(Duration::from_millis(250));
    }

    let mut follower = follower;
    let stderr = {
        // Give the takeover log a moment, then collect stderr.
        std::thread::sleep(Duration::from_millis(500));
        let _ = follower.kill();
        let _ = follower.wait();
        let mut out = String::new();
        if let Some(mut e) = follower.stderr.take() {
            let _ = e.read_to_string(&mut out);
        }
        out
    };
    assert!(
        stderr.contains("FOLLOWER"),
        "follower stderr missing FOLLOWER marker: {stderr}"
    );
}

/// save_screenshots must have several plugin requests in flight at once:
/// the fake plugin refuses to answer until all three arrive, so a serial
/// implementation deadlocks and this test times out.
#[tokio::test(flavor = "multi_thread")]
async fn e2e_save_screenshots_is_concurrent() {
    let dir = tempfile::tempdir().unwrap();
    let port = free_port();
    let mut server = Server::spawn_in(port, dir.path().to_path_buf());
    wait_ping(port, Duration::from_secs(15));

    server.rpc(
        "initialize",
        json!({"protocolVersion": "2025-11-25", "capabilities": {},
               "clientInfo": {"name": "e2e", "version": "0"}}),
        1,
    );
    server.notify("notifications/initialized");

    let mut plugin = connect_fake_plugin(port).await;
    let req = json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
        "name": "save_screenshots",
        "arguments": {"items": [
            {"nodeId": "1:1", "outputPath": "a.png"},
            {"nodeId": "1:2", "outputPath": "b.png"},
            {"nodeId": "1:3", "outputPath": "c.png"}
        ]}}});
    writeln!(server.child.stdin.as_mut().unwrap(), "{req}").unwrap();
    server.child.stdin.as_mut().unwrap().flush().unwrap();

    // Collect all three requests before answering any of them.
    let mut ids = Vec::new();
    while ids.len() < 3 {
        let msg = tokio::time::timeout(Duration::from_secs(10), futures_util::StreamExt::next(&mut plugin))
            .await
            .expect("3 concurrent get_screenshot requests within 10s")
            .expect("plugin stream open")
            .expect("ws message");
        let frame = parse_bridge_frame(&msg).expect("bridge JSON frame");
        if frame["progress"].as_i64().unwrap_or(0) > 0 {
            continue;
        }
        assert_eq!(frame["type"], "get_screenshot");
        ids.push(frame["requestId"].as_str().unwrap().to_string());
    }

    // 1x1 transparent PNG.
    let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    for (i, id) in ids.iter().enumerate() {
        let resp = json!({"type": "get_screenshot", "requestId": id, "data": {"exports": [
            {"nodeId": format!("1:{}", i + 1), "nodeName": format!("Node{i}"),
             "base64": png, "width": 1.0, "height": 1.0}]}});
        futures_util::SinkExt::send(&mut plugin, Message::Text(resp.to_string().into()))
            .await
            .unwrap();
    }

    let mut line = String::new();
    server.stdout.read_line(&mut line).unwrap();
    let out: Value = serde_json::from_str(&line).unwrap();
    let payload: Value =
        serde_json::from_str(out["result"]["content"][0]["text"].as_str().expect("text content"))
            .unwrap();
    assert_eq!(payload["succeeded"], 3, "save_screenshots failed: {payload}");
    assert_eq!(payload["failed"], 0);
    let results = payload["results"].as_array().unwrap();
    for (i, r) in results.iter().enumerate() {
        assert_eq!(r["index"], i, "results must stay in input order: {payload}");
    }
    for name in ["a.png", "b.png", "c.png"] {
        assert!(dir.path().join(name).is_file(), "missing {name}");
    }

    futures_util::SinkExt::close(&mut plugin).await.ok();
    server.kill();
}

/// The `use_figma` escape hatch must hand the script to the plugin as
/// `params.code` (what plugin/src/use-figma.ts reads) and surface the
/// plugin's `data.result` back to the MCP client unchanged.
#[tokio::test(flavor = "multi_thread")]
async fn e2e_use_figma_round_trip() {
    let port = free_port();
    let mut server = Server::spawn(port);
    wait_ping(port, Duration::from_secs(15));

    server.rpc(
        "initialize",
        json!({"protocolVersion": "2025-11-25", "capabilities": {},
               "clientInfo": {"name": "e2e", "version": "0"}}),
        1,
    );
    server.notify("notifications/initialized");

    let mut plugin = connect_fake_plugin(port).await;
    let req = json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
        "name": "use_figma",
        "arguments": {"code": "return { createdNodeIds: [figma.createFrame().id] }"}}});
    writeln!(server.child.stdin.as_mut().unwrap(), "{req}").unwrap();
    server.child.stdin.as_mut().unwrap().flush().unwrap();

    loop {
        let msg = tokio::time::timeout(Duration::from_secs(10), futures_util::StreamExt::next(&mut plugin))
            .await
            .expect("use_figma request within 10s")
            .expect("plugin stream open")
            .expect("ws message");
        let frame = parse_bridge_frame(&msg).expect("bridge JSON frame");
        if frame["progress"].as_i64().unwrap_or(0) > 0 {
            continue;
        }
        assert_eq!(frame["type"], "use_figma");
        assert_eq!(
            frame["params"]["code"],
            "return { createdNodeIds: [figma.createFrame().id] }",
            "plugin reads request.params.code: {frame}"
        );
        let resp = json!({"type": "use_figma", "requestId": frame["requestId"],
                          "data": {"result": {"createdNodeIds": ["1:7"]}}});
        futures_util::SinkExt::send(&mut plugin, Message::Text(resp.to_string().into()))
            .await
            .unwrap();
        break;
    }

    let mut line = String::new();
    server.stdout.read_line(&mut line).unwrap();
    let out: Value = serde_json::from_str(&line).unwrap();
    let payload: Value =
        serde_json::from_str(out["result"]["content"][0]["text"].as_str().expect("text content"))
            .unwrap();
    assert_eq!(payload["result"]["createdNodeIds"][0], "1:7", "unexpected result: {payload}");

    futures_util::SinkExt::close(&mut plugin).await.ok();
    server.kill();
}

