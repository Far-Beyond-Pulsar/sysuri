#[cfg(target_os = "macos")]
use crate::error::{Error, Result};
#[cfg(target_os = "macos")]
use crate::types::UriScheme;

#[cfg(target_os = "macos")]
use std::fs;
#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::process::Command;

/// Register a URI scheme on macOS
///
/// On macOS, URI schemes are typically registered through the application's Info.plist
/// and then activated using LSSetDefaultHandlerForURLScheme.
///
/// For command-line applications, we need to create a minimal .app bundle.
#[cfg(target_os = "macos")]
pub fn register(scheme: &UriScheme) -> Result<()> {
    if !scheme.is_valid_scheme() {
        return Err(Error::InvalidScheme(scheme.scheme.clone()));
    }

    // Verify executable exists
    if !scheme.executable.exists() {
        return Err(Error::InvalidExecutable(
            format!("Executable does not exist: {:?}", scheme.executable)
        ));
    }

    // Get or create the app bundle
    let app_bundle = create_app_bundle(scheme)?;

    // Use LSSetDefaultHandlerForURLScheme via the lsregister command
    set_default_handler(&scheme.scheme, &app_bundle)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn create_app_bundle(scheme: &UriScheme) -> Result<PathBuf> {
    let app_name = format!("{}.app", scheme.scheme);
    let home = std::env::var("HOME")
        .map_err(|e| Error::platform(format!("Failed to get HOME: {}", e)))?;

    let apps_dir = PathBuf::from(home).join("Applications");
    fs::create_dir_all(&apps_dir)
        .map_err(|e| Error::platform(format!("Failed to create Applications directory: {}", e)))?;

    let app_bundle = apps_dir.join(&app_name);
    let contents_dir = app_bundle.join("Contents");
    let macos_dir = contents_dir.join("MacOS");

    // Create bundle directory structure
    fs::create_dir_all(&macos_dir)
        .map_err(|e| Error::platform(format!("Failed to create app bundle directories: {}", e)))?;

    // Copy or symlink the executable
    let bundle_exec = macos_dir.join(&scheme.scheme);
    if bundle_exec.exists() {
        fs::remove_file(&bundle_exec)
            .map_err(|e| Error::platform(format!("Failed to remove old executable: {}", e)))?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(&scheme.executable, &bundle_exec)
            .map_err(|e| Error::platform(format!("Failed to symlink executable: {}", e)))?;
    }

    #[cfg(not(unix))]
    {
        fs::copy(&scheme.executable, &bundle_exec)
            .map_err(|e| Error::platform(format!("Failed to copy executable: {}", e)))?;
    }

    // Create Info.plist
    let plist_path = contents_dir.join("Info.plist");
    let plist_content = create_info_plist(scheme);
    fs::write(&plist_path, plist_content)
        .map_err(|e| Error::platform(format!("Failed to write Info.plist: {}", e)))?;

    Ok(app_bundle)
}

#[cfg(target_os = "macos")]
fn create_info_plist(scheme: &UriScheme) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>{executable}</string>
    <key>CFBundleIdentifier</key>
    <string>com.custom.{scheme}</string>
    <key>CFBundleName</key>
    <string>{scheme}</string>
    <key>CFBundleDisplayName</key>
    <string>{description}</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>{description}</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>{scheme}</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
"#,
        executable = scheme.scheme,
        scheme = scheme.scheme,
        description = scheme.description,
    )
}

#[cfg(target_os = "macos")]
fn set_default_handler(scheme: &str, app_bundle: &PathBuf) -> Result<()> {
    // Register the app with Launch Services
    let output = Command::new("/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister")
        .arg("-f")
        .arg(app_bundle)
        .output()
        .map_err(|e| Error::platform(format!("Failed to run lsregister: {}", e)))?;

    if !output.status.success() {
        return Err(Error::platform(format!(
            "lsregister failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Unregister a URI scheme on macOS
#[cfg(target_os = "macos")]
pub fn unregister(scheme: &str) -> Result<()> {
    let home = std::env::var("HOME")
        .map_err(|e| Error::platform(format!("Failed to get HOME: {}", e)))?;

    let app_bundle = PathBuf::from(home)
        .join("Applications")
        .join(format!("{}.app", scheme));

    if app_bundle.exists() {
        fs::remove_dir_all(&app_bundle)
            .map_err(|e| Error::platform(format!("Failed to remove app bundle: {}", e)))?;
    }

    Ok(())
}

/// Check if a URI scheme is registered on macOS
#[cfg(target_os = "macos")]
pub fn is_registered(scheme: &str) -> Result<bool> {
    let output = Command::new("/usr/bin/open")
        .arg("-Ra")
        .arg(format!("{}://", scheme))
        .output()
        .map_err(|e| Error::platform(format!("Failed to check registration: {}", e)))?;

    Ok(output.status.success())
}