use std::io::stderr;
use anyhow::{Context, Result};
use erpnext_mcp::{ERPNextClient, ERPNextServer};
use rmcp::ServiceExt;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<()> {
    // Attempt to load .env file if present
    dotenvy::dotenv().ok();

    // CRITICAL: Configure tracing exclusively to stderr to avoid polluting stdout JSON-RPC channel
    fmt()
        .with_writer(stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Initializing ERPNext MCP Server (Rust)...");

    let base_url = std::env::var("ERPNEXT_URL")
        .context("Missing required environment variable: ERPNEXT_URL. Please set ERPNEXT_URL (e.g. https://erp.example.com)")?;

    let api_key = std::env::var("ERPNEXT_API_KEY").ok();
    let api_secret = std::env::var("ERPNEXT_API_SECRET").ok();

    if api_key.is_some() && api_secret.is_some() {
        info!("Authentication configured with API key and secret.");
    } else {
        info!("No API key/secret provided; running in unauthenticated mode.");
    }

    let client = ERPNextClient::new(&base_url, api_key.as_deref(), api_secret.as_deref())
        .context("Failed to initialize ERPNext API client")?;

    let server = ERPNextServer::new(client);

    info!(
        target: "erpnext_mcp",
        "Connecting to stdio transport and starting MCP message loop..."
    );

    // Serve MCP JSON-RPC protocol over stdio
    let service = server
        .serve(rmcp::transport::stdio())
        .await
        .context("Failed to bind stdio transport for MCP server")?;

    // Wait until the transport is closed
    if let Err(e) = service.waiting().await {
        error!("MCP service encountered an error while running: {e}");
        return Err(e.into());
    }

    info!("ERPNext MCP Server gracefully stopped.");
    Ok(())
}
