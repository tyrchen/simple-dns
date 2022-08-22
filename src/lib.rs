use crate::dns::{get_forward_authority, load_zone};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::UdpSocket;
use trust_dns_resolver::Name;
use trust_dns_server::authority::Catalog;
use trust_dns_server::ServerFuture;

mod config;
mod dns;
mod error;

pub use config::Config;
pub use error::SimpleDnsError;

pub struct SimpleDns {
    addr: SocketAddr,
    catalog: Catalog,
}

impl SimpleDns {
    pub async fn try_load(config: Config) -> Result<Self, SimpleDnsError> {
        let mut catalog = Catalog::new();
        for (domain, records) in config.domains {
            let name = Name::from_str(&domain)?;
            let authority = load_zone(&domain, records)?;

            catalog.upsert(name.into(), authority);
        }

        let origin = Name::from_str(".")?;
        let authority = get_forward_authority(origin.clone()).await?;
        catalog.upsert(origin.into(), authority);

        Ok(SimpleDns {
            addr: config.bind,
            catalog,
        })
    }

    pub async fn run(self) -> Result<(), SimpleDnsError> {
        let mut server_fut = ServerFuture::new(self.catalog);
        let udp_socket = UdpSocket::bind(self.addr).await?;
        server_fut.register_socket(udp_socket);

        server_fut.block_until_done().await?;

        Ok(())
    }
}
