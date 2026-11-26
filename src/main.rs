mod mcp;
mod quake;

use std::{env, process::ExitCode};

use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(message) = run().await {
        eprintln!("quake-mcp: {message}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

async fn run() -> Result<(), &'static str> {
    let token = env::var("QUAKE_TOKEN").map_err(|_| "Set QUAKE_TOKEN to your Quake API token")?;
    let service = mcp::QuakeMcp::new(quake::Quake::new(&token)?)
        .serve(stdio())
        .await
        .map_err(|_| "Could not initialize the MCP stdio session")?;
    service
        .waiting()
        .await
        .map_err(|_| "The MCP stdio session ended unexpectedly")?;
    Ok(())
}
