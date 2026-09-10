mod bridge;
mod election;
mod follower;
mod leader;
mod node;
mod plugin_assets;
mod prompts;
mod schema;
mod tools;
mod types;

use std::sync::Arc;

use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let version = env!("CARGO_PKG_VERSION");

    let mut ip = String::from("127.0.0.1");
    let mut port: u16 = 1994;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ip" => ip = args.next().unwrap_or_else(|| die("missing value for --ip")),
            "--port" => {
                port = args
                    .next()
                    .unwrap_or_else(|| die("missing value for --port"))
                    .parse()
                    .unwrap_or_else(|_| die("invalid port"));
            }
            "--help" | "-h" => {
                eprintln!("Usage: figma-mcp-rs [--ip 127.0.0.1] [--port 1994]");
                return Ok(());
            }
            other => die(&format!("unknown argument: {other}")),
        }
    }

    let parsed: std::net::IpAddr = ip
        .parse()
        .unwrap_or_else(|_| die(&format!("invalid IP address: {ip:?}")));
    if !parsed.is_loopback() {
        eprintln!(
            "WARNING: binding to {ip} — server will be reachable from the network with no authentication"
        );
    }
    // Install plugin assets to ~/.figma-mcp-rs/plugin/
    match plugin_assets::install() {
        Ok(p) => eprintln!("[plugin] assets ready: {}", p.display()),
        Err(e) => eprintln!("[plugin] could not install plugin assets: {e}"),
    }


    let node = Arc::new(node::Node::new(&ip, port, version));
    let election = Arc::new(election::Election::new(Arc::clone(&node), &ip, port));
    election.start().await;

    eprintln!("Starting figma-mcp-rs {version} (role: {})", node.role_name());

    {
        let shutdown_node = Arc::clone(&node);
        let election_stop = Arc::clone(&election);
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            eprintln!("Shutting down...");
            election_stop.stop();
            shutdown_node.stop();
        });
    }

    let server = tools::FigmaServer::new(Arc::clone(&node));
    let running = server.serve(rmcp::transport::stdio()).await?;
    running.waiting().await?;

    election.stop();
    node.stop();
    Ok(())
}


fn die(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(1);
}
