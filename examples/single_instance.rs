/// Single Instance URI Handler Example
///
/// This example demonstrates how to keep a single application instance running
/// and handle multiple URI activations. When a URI is activated:
///
/// 1. If no instance is running, it starts and waits for URIs
/// 2. If an instance is already running, it sends the URI to that instance
///
/// This uses a simple file-based IPC mechanism for cross-platform compatibility.
/// For production use, consider using proper IPC libraries like:
/// - interprocess (cross-platform)
/// - named pipes on Windows
/// - Unix domain sockets on Unix systems

use sysuri::{UriScheme, register, parse_args};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Write as IoWrite, BufReader, BufRead};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::thread;

const LOCK_FILE_NAME: &str = "myapp_single_instance.lock";
const IPC_FILE_NAME: &str = "myapp_uri_queue.txt";

fn main() {
    println!("Single Instance URI Handler Example");
    println!("====================================\n");

    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check for special commands
    if args.len() > 1 {
        match args[1].as_str() {
            "--register" => {
                register_scheme();
                return;
            }
            "--stop" => {
                stop_instance();
                return;
            }
            _ => {}
        }
    }

    // Check if a URI was provided
    if let Some(uri) = parse_args() {
        println!("Received URI activation: {}", uri);

        // Try to send to existing instance
        if try_send_to_existing_instance(&uri) {
            println!("Sent URI to existing instance.");
            return;
        } else {
            println!("No existing instance found, starting new instance...");
            // Start new instance and process this URI
            run_instance(Some(uri));
        }
    } else {
        println!("No URI provided. Starting in server mode...");
        println!("Commands:");
        println!("  --register : Register the URI scheme");
        println!("  --stop     : Stop the running instance");
        println!("\nPress Ctrl+C to stop the server.\n");

        run_instance(None);
    }
}

/// Register the URI scheme with the OS
fn register_scheme() {
    let exe = env::current_exe().expect("Failed to get current executable path");

    let scheme = UriScheme::new(
        "myapp",
        "My Application Protocol",
        exe
    );

    match register(&scheme) {
        Ok(_) => {
            println!("✓ Successfully registered myapp:// scheme");
            println!("\nTest it with:");

            #[cfg(windows)]
            println!("  start myapp://test/data");

            #[cfg(target_os = "macos")]
            println!("  open myapp://test/data");

            #[cfg(target_os = "linux")]
            println!("  xdg-open myapp://test/data");

            println!("\nOr try non-hierarchical URIs:");
            println!("  myapp:action");
            println!("  myapp:open-file");
        }
        Err(e) => {
            eprintln!("✗ Failed to register scheme: {}", e);
        }
    }
}

/// Stop a running instance by deleting the lock file
fn stop_instance() {
    let lock_path = get_lock_path();

    match fs::remove_file(&lock_path) {
        Ok(_) => println!("✓ Stopped instance (removed lock file)"),
        Err(e) => eprintln!("✗ Failed to stop instance: {}", e),
    }
}

/// Run the main instance loop
fn run_instance(initial_uri: Option<String>) {
    // Try to acquire lock
    let lock_path = get_lock_path();

    if lock_path.exists() {
        // Check if the lock is stale (older than 5 minutes)
        if let Ok(metadata) = fs::metadata(&lock_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    if elapsed.as_secs() < 300 {
                        println!("Another instance is already running.");
                        return;
                    } else {
                        println!("Removing stale lock file...");
                        let _ = fs::remove_file(&lock_path);
                    }
                }
            }
        }
    }

    // Create lock file
    match File::create(&lock_path) {
        Ok(mut file) => {
            let _ = file.write_all(format!("{}", std::process::id()).as_bytes());
            println!("✓ Acquired instance lock (PID: {})", std::process::id());
        }
        Err(e) => {
            eprintln!("✗ Failed to create lock file: {}", e);
            return;
        }
    }

    // Process initial URI if provided
    if let Some(uri) = initial_uri {
        handle_uri(&uri);
    }

    println!("\n🔄 Listening for URI activations...");
    println!("   (Press Ctrl+C to stop)\n");

    // Main loop: check for incoming URIs
    let ipc_path = get_ipc_path();
    let mut last_check = SystemTime::now();

    loop {
        thread::sleep(Duration::from_millis(500));

        // Update lock file timestamp
        if let Ok(elapsed) = SystemTime::now().duration_since(last_check) {
            if elapsed.as_secs() >= 5 {
                let _ = File::create(&lock_path);
                last_check = SystemTime::now();
            }
        }

        // Check for incoming URIs
        if ipc_path.exists() {
            if let Ok(uris) = read_and_clear_ipc_file(&ipc_path) {
                for uri in uris {
                    if !uri.trim().is_empty() {
                        println!("📨 Received: {}", uri);
                        handle_uri(&uri);
                    }
                }
            }
        }
    }

    // Cleanup (unreachable in this example due to infinite loop)
    // In production, you'd want to handle signals properly
    // let _ = fs::remove_file(&lock_path);
}

/// Handle a received URI
fn handle_uri(uri: &str) {
    println!("🔧 Processing URI: {}", uri);

    // Parse the URI
    if let Some(colon_pos) = uri.find(':') {
        let scheme = &uri[..colon_pos];
        let rest = &uri[colon_pos + 1..];

        println!("   Scheme: {}", scheme);

        // Handle hierarchical vs non-hierarchical
        if rest.starts_with("//") {
            let path = &rest[2..];
            println!("   Path: {}", path);

            // Parse path components
            let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            if !parts.is_empty() {
                println!("   Action: {}", parts[0]);
                if parts.len() > 1 {
                    println!("   Parameters: {:?}", &parts[1..]);
                }
            }
        } else {
            // Non-hierarchical URI (like tel:, mailto:)
            println!("   Data: {}", rest);
        }
    }

    println!("   ✓ Processing complete\n");

    // Your actual URI handling logic goes here
    // For example:
    // - Open a window
    // - Process a command
    // - Load data
    // - etc.
}

/// Try to send a URI to an existing instance
fn try_send_to_existing_instance(uri: &str) -> bool {
    let lock_path = get_lock_path();

    // Check if lock file exists
    if !lock_path.exists() {
        return false;
    }

    // Check if lock is recent (within last 10 seconds)
    if let Ok(metadata) = fs::metadata(&lock_path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                if elapsed.as_secs() > 10 {
                    // Lock is stale
                    return false;
                }
            }
        }
    }

    // Write URI to IPC file
    let ipc_path = get_ipc_path();

    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&ipc_path)
    {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{}", uri) {
                eprintln!("Failed to write to IPC file: {}", e);
                return false;
            }
            true
        }
        Err(e) => {
            eprintln!("Failed to open IPC file: {}", e);
            false
        }
    }
}

/// Read and clear the IPC file
fn read_and_clear_ipc_file(path: &PathBuf) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

    // Clear the file
    let _ = fs::remove_file(path);

    Ok(lines)
}

/// Get the path for the lock file
fn get_lock_path() -> PathBuf {
    let mut path = env::temp_dir();
    path.push(LOCK_FILE_NAME);
    path
}

/// Get the path for the IPC file
fn get_ipc_path() -> PathBuf {
    let mut path = env::temp_dir();
    path.push(IPC_FILE_NAME);
    path
}
