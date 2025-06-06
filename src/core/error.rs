use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Hardware not available: {0}")]
    HardwareNotAvailable(String),
    
    #[error("Monitor not initialized")]
    NotInitialized,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    #[error("Channel send error")]
    ChannelError,
    
    #[error("GPU error: {0}")]
    GpuError(String),
    
    #[error("Collection error: {0}")]
    CollectionError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, MonitorError>;