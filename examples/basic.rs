use sysuri::{UriScheme, register, is_registered, parse_args};
use std::env;

fn main() {
    // Check if we were launched via a URI
    if let Some(uri) = parse_args() {
        println!("Application launched with URI: {}", uri);
        println!("You can parse this URI and handle it accordingly.");
        return;
    }

    println!("Basic URI Registration Example");
    println!("================================\n");

    // Get the current executable path
    let exe = env::current_exe().expect("Failed to get current executable path");
    println!("Executable path: {:?}\n", exe);

    // Create a URI scheme
    let scheme_name = "myapp";
    let scheme = UriScheme::new(
        scheme_name,
        "My Application Custom Protocol",
        exe
    );

    println!("Registering URI scheme: {}://", scheme_name);

    // Check if already registered
    match is_registered(scheme_name) {
        Ok(true) => {
            println!("⚠ Scheme is already registered. Re-registering...");
        }
        Ok(false) => {
            println!("✓ Scheme is not yet registered.");
        }
        Err(e) => {
            eprintln!("✗ Error checking registration: {}", e);
            return;
        }
    }

    // Register the scheme
    match register(&scheme) {
        Ok(_) => {
            println!("✓ Successfully registered {}:// scheme!", scheme_name);
            println!("\nYou can now test it by opening a URL like:");
            println!("  {}://test/path?param=value", scheme_name);
            println!("\nOn different platforms:");
            println!("  Windows: Click a link or use 'start {}://test'", scheme_name);
            println!("  macOS:   Use 'open {}://test'", scheme_name);
            println!("  Linux:   Use 'xdg-open {}://test'", scheme_name);
        }
        Err(e) => {
            eprintln!("✗ Failed to register scheme: {}", e);
            std::process::exit(1);
        }
    }

    // Verify registration
    match is_registered(scheme_name) {
        Ok(true) => {
            println!("\n✓ Verification successful: scheme is registered!");
        }
        Ok(false) => {
            println!("\n⚠ Warning: scheme registration could not be verified");
        }
        Err(e) => {
            eprintln!("\n✗ Error verifying registration: {}", e);
        }
    }
}
