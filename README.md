# sysuri

A cross-platform Rust crate for registering custom URI schemes with the operating system.

[![Crates.io](https://img.shields.io/crates/v/sysuri.svg)](https://crates.io/crates/sysuri)
[![Documentation](https://docs.rs/sysuri/badge.svg)](https://docs.rs/sysuri)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- **Cross-platform support**: Windows, macOS, and Linux
- **Simple API**: Register and unregister URI schemes with just a few lines of code
- **Callback-based handling**: Define handlers for incoming URI requests
- **No unsafe code**: Built with safety in mind
- **Well-documented**: Comprehensive API documentation and examples
- **Lightweight**: Minimal dependencies

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sysuri = "0.1"
```

## Quick Start

### Basic Registration

```rust
use sysuri::{UriScheme, register};
use std::env;

fn main() {
    // Get the current executable path
    let exe = env::current_exe().unwrap();

    // Create a URI scheme
    let scheme = UriScheme::new(
        "myapp",
        "My Application Protocol",
        exe
    );

    // Register it with the OS
    register(&scheme).unwrap();

    println!("URI scheme registered! Try: myapp://test");
}
```

### Handling URI Callbacks

When your application is launched via a custom URI, the URI is passed as a command-line argument:

```rust
use sysuri::{register_handler, should_handle_uri, FnHandler};

fn main() {
    // Register a handler for your scheme
    register_handler("myapp", FnHandler::new(|uri| {
        println!("Received URI: {}", uri);
        // Parse and handle the URI...
    }));

    // Check if we should handle a URI
    match should_handle_uri() {
        Ok(true) => {
            println!("Handled URI, exiting...");
            return;
        }
        Ok(false) => {
            println!("Normal application startup");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    // Continue with normal application logic...
}
```

### Alternative: Manual URI Parsing

```rust
use sysuri::parse_args;

fn main() {
    if let Some(uri) = parse_args() {
        println!("Launched with URI: {}", uri);
        // Handle the URI...
        return;
    }

    // Normal startup...
}
```

## URI Format Support

This crate supports both **hierarchical** and **non-hierarchical** URI formats:

### Hierarchical URIs (with `://`)
```
myapp://path/to/resource
http://example.com/page
custom-app://action?param=value
```

### Non-Hierarchical URIs (without `://`)
```
tel:+1-816-555-1212
mailto:user@example.com
sms:+1234567890
myapp:action
```

Both formats are fully supported for registration and handling!

## Platform-Specific Behavior

### Windows

On Windows, sysuri registers URI schemes in `HKEY_CURRENT_USER\Software\Classes`, which doesn't require administrator privileges.

**Windows 10+ Support:**
Starting with version 0.4.0, sysuri automatically registers application capabilities so your application appears in Windows Settings under:
- Settings → Apps → Default Apps → Choose default apps by protocol

This registration includes:
- ProgID creation for proper OS integration
- Application capabilities registration
- URL associations setup
- Integration with the Windows default programs interface

**Important Notes for Windows:**
- The basic registry entries are sufficient for URI handling to work
- For your app to appear in the Windows default apps UI, the enhanced registration (included in v0.4.0+) is required
- For production applications, consider embedding version info and icons using `winres` in a build script
- See the [Windows registration blog post](https://jorgen.tjer.no/post/2021/03/13/registering-a-rust-program-as-a-browser-on-windows/) for additional details on creating a complete Windows application

**Testing:**
```cmd
start myapp://test/path
start tel:+1234567890
```

### macOS

On macOS, sysuri creates a minimal `.app` bundle in `~/Applications` with the appropriate `Info.plist` configuration.

**Testing:**
```bash
open myapp://test/path
```

### Linux

On Linux, sysuri creates a `.desktop` file in `~/.local/share/applications/` and registers it using `xdg-mime`.

**Testing:**
```bash
xdg-open myapp://test/path
```

## Examples

The crate includes several examples demonstrating different usage patterns:

### Basic Registration
```bash
cargo run --example basic
```

Demonstrates simple URI scheme registration and verification.

### Multiple URI Schemes
```bash
cargo run --example multiple_uris
```

Shows how to register multiple related URI schemes (e.g., `myapp://`, `myapp-dev://`, `myapp-secure://`).

### Callback Handlers
```bash
cargo run --example callback_handler
```

Demonstrates the callback-based URI handling system with URI parsing.

### Single Instance Application
```bash
# Register the URI scheme
cargo run --example single_instance -- --register

# Start the server (keeps running)
cargo run --example single_instance

# In another terminal, trigger a URI (it will be sent to the running instance)
# Windows: start myapp://test
# macOS:   open myapp://test
# Linux:   xdg-open myapp://test

# Stop the server
cargo run --example single_instance -- --stop
```

Demonstrates how to implement a single-instance application that:
- Keeps running and listens for URI activations
- Handles multiple URI activations without spawning new processes
- Uses simple file-based IPC for cross-platform compatibility
- Supports both hierarchical (`myapp://path`) and non-hierarchical (`myapp:action`) URIs

This pattern is useful for applications that need to:
- Maintain state between URI activations
- Avoid multiple instances running simultaneously
- Process URIs in a persistent service or daemon

## API Overview

### Core Functions

- `register(scheme: &UriScheme) -> Result<()>` - Register a URI scheme
- `unregister(scheme: &str) -> Result<()>` - Unregister a URI scheme
- `is_registered(scheme: &str) -> Result<bool>` - Check if a scheme is registered

### Handler Functions

- `register_handler(scheme: &str, handler: impl UriHandler)` - Register a URI handler
- `handle_uri(uri: &str) -> Result<()>` - Handle a URI using registered handlers
- `should_handle_uri() -> Result<bool>` - Check and handle URI from command-line args

### Utility Functions

- `parse_args() -> Option<String>` - Extract URI from command-line arguments
- `extract_scheme(uri: &str) -> Option<&str>` - Extract scheme from a URI

### Types

- `UriScheme` - Represents a custom URI scheme registration
- `UriHandler` - Trait for implementing URI handlers
- `FnHandler` - Function-based URI handler implementation

## Advanced Usage

### Custom Handler Implementation

You can implement the `UriHandler` trait for more complex handling:

```rust
use sysuri::UriHandler;

struct MyHandler {
    // Your state...
}

impl UriHandler for MyHandler {
    fn handle_uri(&self, uri: &str) {
        // Custom handling logic...
    }
}
```

### With Icon (Windows/Linux)

```rust
let scheme = UriScheme::new("myapp", "My App", exe)
    .with_icon(PathBuf::from("/path/to/icon.png"));
```

### Conditional Registration

```rust
use sysuri::{is_registered, register};

let scheme_name = "myapp";

if !is_registered(scheme_name)? {
    println!("Registering for the first time...");
    register(&scheme)?;
} else {
    println!("Already registered!");
}
```

## Error Handling

The crate uses a comprehensive error type covering common failure scenarios:

```rust
use sysuri::{register, Error};

match register(&scheme) {
    Ok(_) => println!("Success!"),
    Err(Error::InvalidScheme(s)) => eprintln!("Invalid scheme: {}", s),
    Err(Error::PermissionDenied(msg)) => eprintln!("Permission denied: {}", msg),
    Err(Error::Platform(msg)) => eprintln!("Platform error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## URI Scheme Naming

According to RFC 3986, URI schemes must:
- Start with a letter
- Contain only letters, digits, `+`, `-`, or `.`
- Be case-insensitive (lowercase is recommended)

Valid examples: `myapp`, `my-app`, `app.protocol`, `my+app`

Invalid examples: `my_app`, `123app`, `my app`

The crate validates schemes automatically and returns `Error::InvalidScheme` for invalid names.

## Testing

Run the test suite:

```bash
cargo test
```

Note: Some platform-specific tests may require appropriate permissions and can modify system settings. They use unique scheme names to avoid conflicts.

## Best Practices for Production Applications

### Windows Applications

For production Windows applications, consider these additional steps:

1. **Embed Application Resources** (recommended for professional apps):
   ```toml
   [build-dependencies]
   winres = "0.1"
   ```

   Create a `build.rs`:
   ```rust
   #[cfg(windows)]
   fn main() {
       let mut res = winres::WindowsResource::new();
       res.set_icon("app.ico")
          .set("ProductName", "My Application")
          .set("FileDescription", "My Application Description")
          .set("CompanyName", "Your Company");
       res.compile().unwrap();
   }

   #[cfg(not(windows))]
   fn main() {}
   ```

2. **Use Proper Icons**: Provide `.ico` files for Windows and set them using `UriScheme::with_icon()`

3. **Test in Default Apps UI**: After registration, check Windows Settings → Default Apps to verify your app appears

For more details, see this excellent guide: [Registering a Rust Program as a Browser on Windows](https://jorgen.tjer.no/post/2021/03/13/registering-a-rust-program-as-a-browser-on-windows/)

### Single-Instance Applications

If your application needs to handle multiple URI activations without creating new processes:
- Use the `single_instance` example as a starting point
- Consider using dedicated IPC libraries like `interprocess` for robust communication
- Implement proper signal handling for graceful shutdown
- Use platform-specific single-instance mechanisms (e.g., mutexes on Windows, lock files on Unix)

## Security Considerations

- URI schemes are registered per-user (not system-wide) on all platforms
- The crate validates URI scheme names according to RFC 3986
- Executable paths are verified to exist before registration
- No elevation/admin privileges required
- Always validate and sanitize URI content in your handlers to prevent injection attacks

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the need for a simple, cross-platform URI registration solution
- Built with safety and usability in mind

## Changelog

### Version 0.4.1 (Current)

**New Features:**
- ✨ **Non-hierarchical URI support**: Now supports URIs without `://` such as `tel:+1-816-555-1212`, `mailto:user@example.com`, `sms:+number`
- 🪟 **Windows 10+ default apps integration**: Applications now appear in Windows Settings → Default Apps by protocol
- 🔄 **Single-instance example**: New example showing how to handle multiple URI activations in one running instance
- 📚 **Enhanced documentation**: Comprehensive guide for Windows registration and URI format support

**Improvements:**
- Windows: Added application capabilities registration (ProgID, URL associations, RegisteredApplications)
- Windows: Enhanced unregister to clean up all related registry entries
- URI parsing now correctly handles both hierarchical and non-hierarchical formats
- Added automatic Windows path detection to avoid false positives (e.g., `C:\path`)
- Expanded test coverage for new URI formats

**Bug Fixes:**
- Fixed URI scheme extraction to support non-hierarchical URIs
- Fixed command-line argument parsing to recognize all valid URI formats

### Version 0.3.0

- Fixed incorrect platform cfg attributes in platform modules
- Improved platform-specific imports

### Version 0.2.0

- Removed platform stubs
- Enhanced platform module organization

### Version 0.1.0 (Initial Release)

- Cross-platform URI scheme registration (Windows, macOS, Linux)
- Callback-based URI handling system
- Comprehensive examples and documentation
- Full test coverage
