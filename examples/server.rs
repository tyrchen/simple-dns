use anyhow::Result;
use simple_dns_server::{Config, SimpleDns};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config: Config = serde_yaml::from_str(include_str!("../fixtures/config.yaml"))?;

    let server = SimpleDns::try_load(config).await?;

    server.run().await?;

    Ok(())
}
