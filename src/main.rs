use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    resonique_server::server::run().await
}
