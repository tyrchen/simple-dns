use anyhow::Result;
use simple_dns::config::Config;
use tokio::net::UdpSocket;
use trust_dns_server::ServerFuture;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config: Config = serde_yaml::from_str(include_str!("../fixtures/config.yaml"))?;

    let addr = config.bind;

    let catalog = config
        .load_catalog()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    let mut server_fut = ServerFuture::new(catalog);
    let udp_socket = UdpSocket::bind(addr).await?;
    server_fut.register_socket(udp_socket);

    server_fut.block_until_done().await?;

    Ok(())
}
