//! OrA - Omni-Recursive Agent

use ora_rust::{gateway::create_router, AppState, Config};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let port = config.port;
    let host = config.host.clone();
    println!("OrA v{} - Starting...", env!("CARGO_PKG_VERSION"));

    let vault = ora_rust::security::vault::Vault::new(config.vault_path.clone());
    let security_gates = ora_rust::security::gates::AstParser::new(true);
    let audit_logger = ora_rust::audit::logger::AuditLogger::new(config.audit_path.clone())?;

    let vault2 = vault.clone();
    let kernel = Arc::new(RwLock::new(ora_rust::kernel::Kernel::new(
        config.workspace_root.clone(),
        Arc::new(vault2),
    )?));

    let state = AppState::new(
        config.clone(),
        kernel,
        vault,
        Arc::new(security_gates),
        Arc::new(RwLock::new(audit_logger)),
    );

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--mcp-mode") {
        println!("🚀 OrA starting in MCP Stdio Mode...");
        let mcp_server = ora_rust::gateway::mcp::McpServer::new(state);
        mcp_server.run_stdio().await?;
    } else {
        let app = create_router(config, state);
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        println!("\n🚀 OrA running at http://{}:{}", host, port);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
    }

    Ok(())
}
