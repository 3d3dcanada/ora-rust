//! OrA - Omni-Recursive Agent

use clap::{Args, Parser, Subcommand};
use ora_rust::{gateway::create_router, AppState, Config};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Parser)]
#[command(
    name = "ora",
    version,
    about = "Run the OrA backend server or MCP stdio server.",
    long_about = "OrA exposes two runtime modes: `serve` starts the HTTP and WebSocket API, and `mcp` starts the MCP JSON-RPC server over stdio for desktop or IDE integrations."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(long, hide = true)]
    mcp_mode: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the HTTP and WebSocket backend.
    Serve(RuntimeArgs),
    /// Run the MCP server over stdio.
    Mcp(RuntimeArgs),
}

#[derive(Debug, Args)]
struct RuntimeArgs {
    /// Load configuration from a specific TOML file.
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
}

enum RunMode {
    Serve,
    Mcp,
}

impl Cli {
    fn into_runtime(self) -> (RunMode, Option<PathBuf>) {
        match self.command {
            Some(Command::Serve(args)) => (RunMode::Serve, args.config),
            Some(Command::Mcp(args)) => (RunMode::Mcp, args.config),
            None if self.mcp_mode => (RunMode::Mcp, None),
            None => (RunMode::Serve, None),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let (mode, config_path) = cli.into_runtime();
    let config = match config_path {
        Some(path) => Config::load_from(&path)?,
        None => Config::load()?,
    };

    let port = config.port;
    let host = config.host.clone();
    println!("OrA v{} - starting", env!("CARGO_PKG_VERSION"));

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

    match mode {
        RunMode::Mcp => {
            println!("Starting MCP stdio mode");
            let mcp_server = ora_rust::gateway::mcp::McpServer::new(state);
            mcp_server.run_stdio().await?;
        }
        RunMode::Serve => {
            let app = create_router(config, state);
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            println!("Listening on http://{}:{}", host, port);

            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
