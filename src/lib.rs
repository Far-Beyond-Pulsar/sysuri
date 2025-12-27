//! # sysuri
//!
//! A cross-platform Rust crate for registering custom URI schemes with the operating system.
//!
//! ## Features
//!
//! - Cross-platform support (Windows, macOS, Linux)
//! - Simple API for registering and unregistering URI schemes
//! - Callback-based URI handling
//! - No unsafe code
//! - Comprehensive error handling
//!
//! ## Quick Start
//!
//! ```no_run
//! use sysuri::{UriScheme, register};
//! use std::path::PathBuf;
//! use std::env;
//!
//! // Get the current executable path
//! let exe = env::current_exe().unwrap();
//!
//! // Create a URI scheme
//! let scheme = UriScheme::new(
//!     "myapp",
//!     "My Application Protocol",
//!     exe
//! );
//!
//! // Register it with the OS
//! register(&scheme).unwrap();
//! ```
//!
//! ## URI Handler
//!
//! When your application is launched via a custom URI, the URI is typically passed
//! as a command-line argument. You can use the `parse_args` function to extract it:
//!
//! ```no_run
//! use sysuri::parse_args;
//!
//! fn main() {
//!     if let Some(uri) = parse_args() {
//!         println!("Opened with URI: {}", uri);
//!         // Handle the URI...
//!     } else {
//!         println!("Normal application startup");
//!         // Run normal application logic...
//!     }
//! }
//! ```

mod error;
mod types;
mod platform;

pub use error::{Error, Result};
pub use types::{UriScheme, UriHandler, FnHandler};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Global URI handler registry
static HANDLERS: once_cell::sync::Lazy<Arc<Mutex<HashMap<String, Arc<dyn UriHandler>>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Register a URI scheme with the operating system
///
/// This function registers the URI scheme so that when a URI with this scheme
/// is opened (e.g., by clicking a link), your application will be launched.
///
/// # Arguments
///
/// * `scheme` - The URI scheme to register
///
/// # Returns
///
/// Returns `Ok(())` if the registration was successful, or an `Error` if it failed.
///
/// # Example
///
/// ```no_run
/// use sysuri::{UriScheme, register};
/// use std::env;
///
/// let exe = env::current_exe().unwrap();
/// let scheme = UriScheme::new("myapp", "My App", exe);
/// register(&scheme).unwrap();
/// ```
pub fn register(scheme: &UriScheme) -> Result<()> {
    platform::register(scheme)
}

/// Unregister a URI scheme from the operating system
///
/// # Arguments
///
/// * `scheme` - The scheme name to unregister
///
/// # Example
///
/// ```no_run
/// use sysuri::unregister;
///
/// unregister("myapp").unwrap();
/// ```
pub fn unregister(scheme: &str) -> Result<()> {
    platform::unregister(scheme)
}

/// Check if a URI scheme is already registered
///
/// # Arguments
///
/// * `scheme` - The scheme name to check
///
/// # Returns
///
/// Returns `Ok(true)` if the scheme is registered, `Ok(false)` if not,
/// or an `Error` if the check failed.
///
/// # Example
///
/// ```no_run
/// use sysuri::is_registered;
///
/// if is_registered("myapp").unwrap() {
///     println!("myapp:// is already registered");
/// }
/// ```
pub fn is_registered(scheme: &str) -> Result<bool> {
    platform::is_registered(scheme)
}

/// Register a URI handler callback
///
/// This function registers a callback that will be invoked when a URI
/// is passed to your application. Note that the actual URI is typically
/// passed as a command-line argument, so you'll need to call `handle_uri`
/// with the argument to trigger the callback.
///
/// # Arguments
///
/// * `scheme` - The scheme name to handle
/// * `handler` - The handler that will process URIs
///
/// # Example
///
/// ```no_run
/// use sysuri::{register_handler, FnHandler};
///
/// let handler = FnHandler::new(|uri| {
///     println!("Received URI: {}", uri);
/// });
///
/// register_handler("myapp", handler);
/// ```
pub fn register_handler<H: UriHandler + 'static>(scheme: &str, handler: H) {
    let mut handlers = HANDLERS.lock().unwrap();
    handlers.insert(scheme.to_string(), Arc::new(handler));
}

/// Handle a URI by calling the registered handler
///
/// This function should be called with the URI that was passed to your
/// application (typically as a command-line argument).
///
/// # Arguments
///
/// * `uri` - The full URI to handle (e.g., "myapp://action/data")
///
/// # Returns
///
/// Returns `Ok(())` if a handler was found and called, or an `Error` if
/// no handler was registered for the scheme.
///
/// # Example
///
/// ```no_run
/// use sysuri::handle_uri;
///
/// handle_uri("myapp://open/file").unwrap();
/// ```
pub fn handle_uri(uri: &str) -> Result<()> {
    let scheme = extract_scheme(uri)
        .ok_or_else(|| Error::InvalidScheme(uri.to_string()))?;

    let handlers = HANDLERS.lock().unwrap();
    if let Some(handler) = handlers.get(scheme) {
        handler.handle_uri(uri);
        Ok(())
    } else {
        Err(Error::platform(format!("No handler registered for scheme: {}", scheme)))
    }
}

/// Parse command-line arguments to find a URI
///
/// This is a convenience function that looks through command-line arguments
/// for something that looks like a custom URI scheme.
///
/// # Returns
///
/// Returns the first argument that contains "://" or `None` if no URI is found.
///
/// # Example
///
/// ```no_run
/// use sysuri::parse_args;
///
/// if let Some(uri) = parse_args() {
///     println!("Opened with URI: {}", uri);
/// }
/// ```
pub fn parse_args() -> Option<String> {
    std::env::args()
        .skip(1)
        .find(|arg| arg.contains("://"))
}

/// Extract the scheme from a URI
///
/// # Arguments
///
/// * `uri` - The full URI (e.g., "myapp://action")
///
/// # Returns
///
/// Returns the scheme part (e.g., "myapp") or `None` if the URI is invalid.
///
/// # Example
///
/// ```
/// use sysuri::extract_scheme;
///
/// assert_eq!(extract_scheme("myapp://test"), Some("myapp"));
/// assert_eq!(extract_scheme("invalid"), None);
/// ```
pub fn extract_scheme(uri: &str) -> Option<&str> {
    if !uri.contains("://") {
        return None;
    }
    uri.split("://").next().filter(|s| !s.is_empty())
}

/// Check if the application should run in URI handler mode
///
/// This is a convenience function that combines `parse_args` and `handle_uri`.
/// If a URI is found in the arguments, it will be handled and `true` is returned.
/// Otherwise, `false` is returned and the application should run normally.
///
/// # Returns
///
/// Returns `Ok(true)` if a URI was found and handled, `Ok(false)` if no URI
/// was found (normal startup), or an `Error` if handling failed.
///
/// # Example
///
/// ```no_run
/// use sysuri::{should_handle_uri, register_handler, FnHandler};
///
/// fn main() {
///     // Register handler first
///     register_handler("myapp", FnHandler::new(|uri| {
///         println!("Got URI: {}", uri);
///     }));
///
///     // Check if we should handle a URI
///     match should_handle_uri() {
///         Ok(true) => {
///             println!("Handled URI, exiting...");
///             return;
///         }
///         Ok(false) => {
///             println!("Normal startup");
///         }
///         Err(e) => {
///             eprintln!("Error handling URI: {}", e);
///         }
///     }
///
///     // Run normal application logic...
/// }
/// ```
pub fn should_handle_uri() -> Result<bool> {
    if let Some(uri) = parse_args() {
        handle_uri(&uri)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_scheme() {
        assert_eq!(extract_scheme("myapp://test"), Some("myapp"));
        assert_eq!(extract_scheme("http://example.com"), Some("http"));
        assert_eq!(extract_scheme("invalid"), None);
        assert_eq!(extract_scheme("://invalid"), None);
    }

    #[test]
    fn test_handler_registration() {
        let handler = FnHandler::new(|uri| {
            println!("Test handler: {}", uri);
        });

        register_handler("test", handler);

        // This should work
        let result = handle_uri("test://something");
        assert!(result.is_ok());

        // This should fail (no handler)
        let result = handle_uri("unknown://something");
        assert!(result.is_err());
    }
}
