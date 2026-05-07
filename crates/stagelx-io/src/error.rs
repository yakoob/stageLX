use thiserror::Error;

#[derive(Debug, Error)]
pub enum IoError {
    #[error("network error: {0}")]
    Network(#[from] std::io::Error),
    #[error("invalid packet")]
    InvalidPacket,
    #[error("device not found: {0}")]
    DeviceNotFound(String),
}
