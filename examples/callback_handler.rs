use sysuri::{UriScheme, register, register_handler, should_handle_uri, FnHandler};
use std::env;

fn main() {
    println!("URI Callback Handler Example");
    println!("=============================\n");

    // Register handlers for different URI schemes
    register_handler("myapp", FnHandler::new(|uri| {
        println!("📱 Main app handler received: {}", uri);
        parse_and_handle_uri(uri);
    }));

    register_handler("myapp-open", FnHandler::new(|uri| {
        println!("📂 Open handler received: {}", uri);
        if let Some(path) = uri.strip_prefix("myapp-open://") {
            println!("   Would open file/resource: {}", path);
        }
    }));

    register_handler("myapp-action", FnHandler::new(|uri| {
        println!("⚡ Action handler received: {}", uri);
        if let Some(action) = uri.strip_prefix("myapp-action://") {
            println!("   Would execute action: {}", action);
        }
    }));

    // Check if we should handle a URI (from command-line args)
    match should_handle_uri() {
        Ok(true) => {
            println!("\n✓ URI handled successfully. Exiting...");
            return;
        }
        Ok(false) => {
            println!("No URI in arguments, running in registration mode...\n");
        }
        Err(e) => {
            eprintln!("✗ Error handling URI: {}\n", e);
        }
    }

    // If no URI was provided, register the schemes
    let exe = env::current_exe().expect("Failed to get current executable path");

    let schemes = vec![
        UriScheme::new("myapp", "My Application Main Protocol", exe.clone()),
        UriScheme::new("myapp-open", "My Application Open Protocol", exe.clone()),
        UriScheme::new("myapp-action", "My Application Action Protocol", exe.clone()),
    ];

    println!("Registering URI schemes with handlers...\n");

    for scheme in &schemes {
        match register(scheme) {
            Ok(_) => {
                println!("✓ Registered {}:// with handler", scheme.scheme);
            }
            Err(e) => {
                eprintln!("✗ Failed to register {}: {}", scheme.scheme, e);
            }
        }
    }

    println!("\n🎉 Setup complete! Try these URIs:");
    println!("  myapp://welcome");
    println!("  myapp://user/profile");
    println!("  myapp-open://document.txt");
    println!("  myapp-action://sync-data");

    println!("\nTest manually with:");
    #[cfg(windows)]
    println!("  start myapp://test");
    #[cfg(target_os = "macos")]
    println!("  open myapp://test");
    #[cfg(target_os = "linux")]
    println!("  xdg-open myapp://test");
}

fn parse_and_handle_uri(uri: &str) {
    // Remove the scheme prefix
    if let Some(path) = uri.strip_prefix("myapp://") {
        let parts: Vec<&str> = path.split('/').collect();

        match parts.get(0) {
            Some(&"welcome") => {
                println!("   🎉 Welcome action triggered!");
            }
            Some(&"user") => {
                if let Some(user_id) = parts.get(1) {
                    println!("   👤 User profile requested: {}", user_id);
                } else {
                    println!("   👤 User list requested");
                }
            }
            Some(&"open") => {
                if let Some(resource) = parts.get(1) {
                    println!("   📂 Opening resource: {}", resource);
                }
            }
            Some(other) => {
                println!("   ❓ Unknown action: {}", other);
            }
            None => {
                println!("   🏠 Home page requested");
            }
        }

        // Parse query parameters if present
        if let Some(query_start) = path.find('?') {
            let query = &path[query_start + 1..];
            println!("   🔍 Query parameters: {}", query);

            for param in query.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    println!("      {} = {}", key, value);
                }
            }
        }
    }
}
