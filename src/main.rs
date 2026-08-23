use anyhow::Result;

// Link the library modules
mod mcp;
mod model;
mod server;
mod storage;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    server::run().await
}
