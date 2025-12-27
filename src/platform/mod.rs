pub mod windows;
pub mod macos;
pub mod linux;

use crate::error::Result;
use crate::types::UriScheme;

/// Register a URI scheme on the current platform
pub fn register(scheme: &UriScheme) -> Result<()> {
    #[cfg(windows)]
    return windows::register(scheme);

    #[cfg(target_os = "macos")]
    return macos::register(scheme);

    #[cfg(target_os = "linux")]
    return linux::register(scheme);

    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    return Err(crate::error::Error::UnsupportedPlatform);
}

/// Unregister a URI scheme on the current platform
pub fn unregister(scheme: &str) -> Result<()> {
    #[cfg(windows)]
    return windows::unregister(scheme);

    #[cfg(target_os = "macos")]
    return macos::unregister(scheme);

    #[cfg(target_os = "linux")]
    return linux::unregister(scheme);

    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    return Err(crate::error::Error::UnsupportedPlatform);
}

/// Check if a URI scheme is registered on the current platform
pub fn is_registered(scheme: &str) -> Result<bool> {
    #[cfg(windows)]
    return windows::is_registered(scheme);

    #[cfg(target_os = "macos")]
    return macos::is_registered(scheme);

    #[cfg(target_os = "linux")]
    return linux::is_registered(scheme);

    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    return Err(crate::error::Error::UnsupportedPlatform);
}
