/// Result type alias for sysuri operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during URI registration and handling
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Platform-specific error occurred
    #[error("Platform error: {0}")]
    Platform(String),

    /// Invalid URI scheme provided
    #[error("Invalid URI scheme: {0}")]
    InvalidScheme(String),

    /// IO error occurred
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Permission denied when trying to register URI
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// URI scheme already registered by another application
    #[error("URI scheme '{0}' is already registered")]
    AlreadyRegistered(String),

    /// Unsupported platform
    #[error("Unsupported platform")]
    UnsupportedPlatform,

    /// Invalid executable path
    #[error("Invalid executable path: {0}")]
    InvalidExecutable(String),
}

impl Error {
    /// Create a new platform-specific error
    pub fn platform<S: Into<String>>(msg: S) -> Self {
        Error::Platform(msg.into())
    }

    /// Create a new permission denied error
    pub fn permission_denied<S: Into<String>>(msg: S) -> Self {
        Error::PermissionDenied(msg.into())
    }
}
