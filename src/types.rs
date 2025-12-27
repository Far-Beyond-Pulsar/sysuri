use std::path::PathBuf;

/// Represents a custom URI scheme registration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriScheme {
    /// The scheme name (e.g., "myapp" for "myapp://")
    pub scheme: String,

    /// Human-readable description of the URI scheme
    pub description: String,

    /// Path to the executable that handles this URI scheme
    pub executable: PathBuf,

    /// Optional icon path for the URI scheme
    pub icon: Option<PathBuf>,
}

impl UriScheme {
    /// Create a new URI scheme registration
    ///
    /// # Arguments
    ///
    /// * `scheme` - The URI scheme name (without "://" suffix)
    /// * `description` - Human-readable description
    /// * `executable` - Path to the executable that handles URIs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sysuri::UriScheme;
    /// use std::path::PathBuf;
    ///
    /// let scheme = UriScheme::new(
    ///     "myapp",
    ///     "My Application Protocol",
    ///     PathBuf::from("/path/to/myapp")
    /// );
    /// ```
    pub fn new<S: Into<String>>(scheme: S, description: S, executable: PathBuf) -> Self {
        Self {
            scheme: scheme.into(),
            description: description.into(),
            executable,
            icon: None,
        }
    }

    /// Set an optional icon for the URI scheme
    ///
    /// # Arguments
    ///
    /// * `icon` - Path to the icon file
    pub fn with_icon(mut self, icon: PathBuf) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Validate the URI scheme name
    ///
    /// Returns `true` if the scheme is valid according to RFC 3986
    pub fn is_valid_scheme(&self) -> bool {
        if self.scheme.is_empty() {
            return false;
        }

        // First character must be alphabetic
        let mut chars = self.scheme.chars();
        if let Some(first) = chars.next() {
            if !first.is_ascii_alphabetic() {
                return false;
            }
        }

        // Subsequent characters must be alphanumeric, +, -, or .
        chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
    }

    /// Get the full URI format (e.g., "myapp://")
    pub fn full_uri(&self) -> String {
        format!("{}://", self.scheme)
    }
}

/// Handler for incoming URI requests
///
/// Implement this trait to handle URIs that are opened by the OS
pub trait UriHandler: Send + Sync {
    /// Called when a URI matching the registered scheme is opened
    ///
    /// # Arguments
    ///
    /// * `uri` - The full URI that was opened (e.g., "myapp://action/data")
    fn handle_uri(&self, uri: &str);
}

/// A simple function-based URI handler
pub struct FnHandler<F>
where
    F: Fn(&str) + Send + Sync,
{
    handler: F,
}

impl<F> FnHandler<F>
where
    F: Fn(&str) + Send + Sync,
{
    /// Create a new function-based handler
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F> UriHandler for FnHandler<F>
where
    F: Fn(&str) + Send + Sync,
{
    fn handle_uri(&self, uri: &str) {
        (self.handler)(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_schemes() {
        let scheme = UriScheme::new("myapp", "Test", PathBuf::from("/test"));
        assert!(scheme.is_valid_scheme());

        let scheme = UriScheme::new("my-app", "Test", PathBuf::from("/test"));
        assert!(scheme.is_valid_scheme());

        let scheme = UriScheme::new("my+app", "Test", PathBuf::from("/test"));
        assert!(scheme.is_valid_scheme());

        let scheme = UriScheme::new("my.app", "Test", PathBuf::from("/test"));
        assert!(scheme.is_valid_scheme());
    }

    #[test]
    fn test_invalid_schemes() {
        let scheme = UriScheme::new("", "Test", PathBuf::from("/test"));
        assert!(!scheme.is_valid_scheme());

        let scheme = UriScheme::new("1myapp", "Test", PathBuf::from("/test"));
        assert!(!scheme.is_valid_scheme());

        let scheme = UriScheme::new("my_app", "Test", PathBuf::from("/test"));
        assert!(!scheme.is_valid_scheme());

        let scheme = UriScheme::new("my app", "Test", PathBuf::from("/test"));
        assert!(!scheme.is_valid_scheme());
    }

    #[test]
    fn test_full_uri() {
        let scheme = UriScheme::new("myapp", "Test", PathBuf::from("/test"));
        assert_eq!(scheme.full_uri(), "myapp://");
    }
}
