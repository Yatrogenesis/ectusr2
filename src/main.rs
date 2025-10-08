use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

mod config;
mod mcp;
mod api;
mod tools;
mod orchestrator;
mod budget;
mod resources;
mod prompts;
mod errors;
mod util;

#[derive(Parser, Debug)]
#[command(name = "ectusr2", version, about = "Ectus-R MCP server (Rust)")]
struct Cli {
    /// Base URL of ECTUS-R API
    #[arg(long, default_value = "http://localhost:8000")]
    api_url: String,
    /// Optional API key
    #[arg(long, )]
    api_key: Option<String>,
    /// Orchestrator backend
    #[arg(long = "orchestrator", default_value = "kubernetes")]
    orchestrator_backend: String,
    /// Monthly budget limit (USD)
    #[arg(long = "budget-limit", )]
    budget_limit: Option<f32>,
    /// Budget policy (hard|soft)
    #[arg(long = "budget-policy", )]
    budget_policy: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).init();

    let cli = Cli::parse();
    let cfg = config::Config::from_cli(cli)?;

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting ectusr2");
    mcp::server::run(cfg).await
}

