use std::net::AddrParseError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimpleDnsError {
    #[error("{0}")]
    Internal(String),

    #[error("Config error: {0}")]
    ConfigError(String),
    #[error("Proto error: {0}")]
    ProtoError(#[from] trust_dns_proto::error::ProtoError),
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    AddrParseError(#[from] AddrParseError),
}
