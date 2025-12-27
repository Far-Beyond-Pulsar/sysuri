use crate::error::{Error, Result};
use crate::types::UriScheme;

#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::process::Command;

/// Register a URI scheme on Linux
///
/// On Linux, URI schemes are registered through .desktop files in
/// ~/.local/share/applications/ and updated using xdg-mime.
#[cfg(target_os = "linux")]
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

    let desktop_file_path = create_desktop_file(scheme)?;

    // Update the desktop database
    update_desktop_database()?;

    // Set as default handler for the URI scheme
    set_default_handler(&scheme.scheme, &desktop_file_path)?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn create_desktop_file(scheme: &UriScheme) -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|e| Error::platform(format!("Failed to get HOME: {}", e)))?;

    let applications_dir = PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("applications");

    fs::create_dir_all(&applications_dir)
        .map_err(|e| Error::platform(format!("Failed to create applications directory: {}", e)))?;

    let desktop_filename = format!("{}-handler.desktop", scheme.scheme);
    let desktop_file_path = applications_dir.join(&desktop_filename);

    let executable_str = scheme.executable.to_str()
        .ok_or_else(|| Error::platform("Executable path contains invalid UTF-8"))?;

    let icon_str = scheme.icon.as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or("");

    let desktop_content = format!(
        r#"[Desktop Entry]
Version=1.0
Type=Application
Name={name}
Comment={description}
Exec={executable} %u
Icon={icon}
Terminal=false
Categories=Network;
MimeType=x-scheme-handler/{scheme};
"#,
        name = scheme.scheme,
        description = scheme.description,
        executable = executable_str,
        icon = icon_str,
        scheme = scheme.scheme,
    );

    fs::write(&desktop_file_path, desktop_content)
        .map_err(|e| Error::platform(format!("Failed to write desktop file: {}", e)))?;

    // Make the desktop file executable (not strictly necessary but good practice)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&desktop_file_path)
            .map_err(|e| Error::platform(format!("Failed to get file metadata: {}", e)))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&desktop_file_path, perms)
            .map_err(|e| Error::platform(format!("Failed to set permissions: {}", e)))?;
    }

    Ok(desktop_file_path)
}

#[cfg(target_os = "linux")]
fn update_desktop_database() -> Result<()> {
    let home = std::env::var("HOME")
        .map_err(|e| Error::platform(format!("Failed to get HOME: {}", e)))?;

    let applications_dir = PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("applications");

    // Try to update the desktop database, but don't fail if the command doesn't exist
    let _ = Command::new("update-desktop-database")
        .arg(applications_dir)
        .output();

    Ok(())
}

#[cfg(target_os = "linux")]
fn set_default_handler(scheme: &str, desktop_file_path: &PathBuf) -> Result<()> {
    let desktop_filename = desktop_file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::platform("Invalid desktop file path"))?;

    let mime_type = format!("x-scheme-handler/{}", scheme);

    // Use xdg-mime to set the default handler
    let output = Command::new("xdg-mime")
        .arg("default")
        .arg(desktop_filename)
        .arg(&mime_type)
        .output()
        .map_err(|e| Error::platform(format!("Failed to run xdg-mime: {}", e)))?;

    if !output.status.success() {
        return Err(Error::platform(format!(
            "xdg-mime failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Unregister a URI scheme on Linux
#[cfg(target_os = "linux")]
pub fn unregister(scheme: &str) -> Result<()> {
    let home = std::env::var("HOME")
        .map_err(|e| Error::platform(format!("Failed to get HOME: {}", e)))?;

    let desktop_filename = format!("{}-handler.desktop", scheme);
    let desktop_file_path = PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("applications")
        .join(&desktop_filename);

    if desktop_file_path.exists() {
        fs::remove_file(&desktop_file_path)
            .map_err(|e| Error::platform(format!("Failed to remove desktop file: {}", e)))?;
    }

    // Update the desktop database
    update_desktop_database()?;

    Ok(())
}

/// Check if a URI scheme is registered on Linux
#[cfg(target_os = "linux")]
pub fn is_registered(scheme: &str) -> Result<bool> {
    let mime_type = format!("x-scheme-handler/{}", scheme);

    let output = Command::new("xdg-mime")
        .arg("query")
        .arg("default")
        .arg(&mime_type)
        .output()
        .map_err(|e| Error::platform(format!("Failed to run xdg-mime: {}", e)))?;

    if !output.status.success() {
        return Ok(false);
    }

    let handler = String::from_utf8_lossy(&output.stdout);
    Ok(!handler.trim().is_empty())
}

// Stub implementations for non-Linux platforms
#[cfg(not(target_os = "linux"))]
pub fn register(_scheme: &UriScheme) -> Result<()> {
    Err(Error::UnsupportedPlatform)
}

#[cfg(not(target_os = "linux"))]
pub fn unregister(_scheme: &str) -> Result<()> {
    Err(Error::UnsupportedPlatform)
}

#[cfg(not(target_os = "linux"))]
pub fn is_registered(_scheme: &str) -> Result<bool> {
    Err(Error::UnsupportedPlatform)
}
