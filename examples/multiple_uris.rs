use sysuri::{UriScheme, register, is_registered, parse_args, extract_scheme};
use std::env;

fn main() {
    // Check if we were launched via a URI
    if let Some(uri) = parse_args() {
        if let Some(scheme) = extract_scheme(&uri) {
            println!("Application launched with {} URI: {}", scheme, uri);

            match scheme {
                "myapp" => {
                    println!("Handling myapp protocol...");
                    // Handle myapp:// URIs
                }
                "myapp-dev" => {
                    println!("Handling myapp-dev protocol (development mode)...");
                    // Handle myapp-dev:// URIs
                }
                "myapp-secure" => {
                    println!("Handling myapp-secure protocol (secure mode)...");
                    // Handle myapp-secure:// URIs
                }
                _ => {
                    println!("Unknown scheme: {}", scheme);
                }
            }
        }
        return;
    }

    println!("Multiple URI Schemes Registration Example");
    println!("==========================================\n");

    let exe = env::current_exe().expect("Failed to get current executable path");

    // Define multiple URI schemes
    let schemes = vec![
        UriScheme::new("myapp", "My Application - Production", exe.clone()),
        UriScheme::new("myapp-dev", "My Application - Development", exe.clone()),
        UriScheme::new("myapp-secure", "My Application - Secure Channel", exe.clone()),
    ];

    println!("Registering {} URI schemes...\n", schemes.len());

    // Register all schemes
    for scheme in &schemes {
        print!("Registering {}:// ... ", scheme.scheme);

        match register(scheme) {
            Ok(_) => {
                println!("✓ Success");
            }
            Err(e) => {
                println!("✗ Failed: {}", e);
            }
        }
    }

    // Verify all registrations
    println!("\nVerifying registrations:");
    for scheme in &schemes {
        match is_registered(&scheme.scheme) {
            Ok(true) => {
                println!("  ✓ {}:// is registered", scheme.scheme);
            }
            Ok(false) => {
                println!("  ✗ {}:// is NOT registered", scheme.scheme);
            }
            Err(e) => {
                println!("  ⚠ {}:// verification failed: {}", scheme.scheme, e);
            }
        }
    }

    println!("\nYou can now test with:");
    for scheme in &schemes {
        println!("  {}://test", scheme.scheme);
    }

    // Uncomment to clean up registrations
    // println!("\nCleaning up registrations...");
    // for scheme in &schemes {
    //     match unregister(&scheme.scheme) {
    //         Ok(_) => println!("  ✓ Unregistered {}://", scheme.scheme),
    //         Err(e) => println!("  ✗ Failed to unregister {}: {}", scheme.scheme, e),
    //     }
    // }
}
