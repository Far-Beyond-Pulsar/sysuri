#[cfg(windows)]
use crate::error::{Error, Result};
#[cfg(windows)]
use crate::types::UriScheme;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Register a URI scheme on Windows
///
/// This function writes to HKEY_CURRENT_USER\Software\Classes
/// which doesn't require administrator privileges.
#[cfg(windows)]
pub fn register(scheme: &UriScheme) -> Result<()> {
    if !scheme.is_valid_scheme() {
        return Err(Error::InvalidScheme(scheme.scheme.clone()));
    }

    let executable = scheme.executable.to_str()
        .ok_or_else(|| Error::InvalidExecutable(
            format!("Path contains invalid UTF-8: {:?}", scheme.executable)
        ))?;

    // Verify executable exists
    if !scheme.executable.exists() {
        return Err(Error::InvalidExecutable(
            format!("Executable does not exist: {}", executable)
        ));
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu.open_subkey_with_flags("Software\\Classes", KEY_WRITE)
        .map_err(|e| Error::platform(format!("Failed to open Software\\Classes: {}", e)))?;

    // Create the scheme key
    let (scheme_key, _) = classes.create_subkey(&scheme.scheme)
        .map_err(|e| Error::platform(format!("Failed to create scheme key: {}", e)))?;

    // Set the default value to the description
    scheme_key.set_value("", &scheme.description)
        .map_err(|e| Error::platform(format!("Failed to set description: {}", e)))?;

    // Set URL Protocol value (empty string)
    scheme_key.set_value("URL Protocol", &"")
        .map_err(|e| Error::platform(format!("Failed to set URL Protocol: {}", e)))?;

    // Create DefaultIcon key if icon is provided
    if let Some(icon_path) = &scheme.icon {
        let icon_str = icon_path.to_str()
            .ok_or_else(|| Error::platform("Icon path contains invalid UTF-8"))?;

        let (icon_key, _) = scheme_key.create_subkey("DefaultIcon")
            .map_err(|e| Error::platform(format!("Failed to create DefaultIcon key: {}", e)))?;

        icon_key.set_value("", &icon_str)
            .map_err(|e| Error::platform(format!("Failed to set icon: {}", e)))?;
    }

    // Create shell\open\command key
    let (command_key, _) = scheme_key.create_subkey("shell\\open\\command")
        .map_err(|e| Error::platform(format!("Failed to create command key: {}", e)))?;

    // Set the command with "%1" to pass the URI as an argument
    let command = format!("\"{}\" \"%1\"", executable);
    command_key.set_value("", &command)
        .map_err(|e| Error::platform(format!("Failed to set command: {}", e)))?;

    Ok(())
}

/// Unregister a URI scheme on Windows
#[cfg(windows)]
pub fn unregister(scheme: &str) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu.open_subkey_with_flags("Software\\Classes", KEY_WRITE)
        .map_err(|e| Error::platform(format!("Failed to open Software\\Classes: {}", e)))?;

    classes.delete_subkey_all(scheme)
        .map_err(|e| Error::platform(format!("Failed to delete scheme key: {}", e)))?;

    Ok(())
}

/// Check if a URI scheme is registered on Windows
#[cfg(windows)]
pub fn is_registered(scheme: &str) -> Result<bool> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Check HKEY_CURRENT_USER first
    if let Ok(classes) = hkcu.open_subkey("Software\\Classes") {
        if classes.open_subkey(scheme).is_ok() {
            return Ok(true);
        }
    }

    // Check HKEY_CLASSES_ROOT (system-wide)
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    Ok(hkcr.open_subkey(scheme).is_ok())
}