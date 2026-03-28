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
///
/// For Windows 10+, this also registers application capabilities so the
/// application appears in the default apps settings.
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

    // Register application capabilities (Windows 10+)
    // This makes the application appear in "Default Apps" settings
    register_app_capabilities(scheme, executable)?;

    Ok(())
}

/// Register application capabilities for Windows 10+ default apps support
///
/// This creates the necessary registry entries so the application appears
/// in Windows Settings > Default Apps > Choose default apps by protocol
#[cfg(windows)]
fn register_app_capabilities(scheme: &UriScheme, executable: &str) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Generate app name from executable
    let app_name = scheme.executable
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&scheme.scheme);

    let prog_id = format!("{}.{}", app_name, &scheme.scheme);

    // 1. Create ProgID entry
    let classes = hkcu.open_subkey_with_flags("Software\\Classes", KEY_WRITE)
        .map_err(|e| Error::platform(format!("Failed to open Software\\Classes: {}", e)))?;

    let (prog_id_key, _) = classes.create_subkey(&prog_id)
        .map_err(|e| Error::platform(format!("Failed to create ProgID key: {}", e)))?;

    prog_id_key.set_value("", &format!("{} Protocol", scheme.description))
        .map_err(|e| Error::platform(format!("Failed to set ProgID description: {}", e)))?;

    prog_id_key.set_value("URL Protocol", &"")
        .map_err(|e| Error::platform(format!("Failed to set URL Protocol in ProgID: {}", e)))?;

    // Add Application subkey with AppUserModelId
    let (app_key, _) = prog_id_key.create_subkey("Application")
        .map_err(|e| Error::platform(format!("Failed to create Application key: {}", e)))?;

    app_key.set_value("ApplicationName", &scheme.description)
        .map_err(|e| Error::platform(format!("Failed to set ApplicationName: {}", e)))?;

    app_key.set_value("ApplicationDescription", &scheme.description)
        .map_err(|e| Error::platform(format!("Failed to set ApplicationDescription: {}", e)))?;

    // Set icon if provided
    if let Some(icon_path) = &scheme.icon {
        let icon_str = icon_path.to_str()
            .ok_or_else(|| Error::platform("Icon path contains invalid UTF-8"))?;

        let (icon_key, _) = prog_id_key.create_subkey("DefaultIcon")
            .map_err(|e| Error::platform(format!("Failed to create DefaultIcon in ProgID: {}", e)))?;

        icon_key.set_value("", &icon_str)
            .map_err(|e| Error::platform(format!("Failed to set icon in ProgID: {}", e)))?;

        app_key.set_value("ApplicationIcon", &icon_str)
            .map_err(|e| Error::platform(format!("Failed to set ApplicationIcon: {}", e)))?;
    } else {
        // Use executable as icon
        let icon_ref = format!("{},0", executable);
        app_key.set_value("ApplicationIcon", &icon_ref)
            .map_err(|e| Error::platform(format!("Failed to set ApplicationIcon: {}", e)))?;
    }

    // Add command
    let (command_key, _) = prog_id_key.create_subkey("shell\\open\\command")
        .map_err(|e| Error::platform(format!("Failed to create command in ProgID: {}", e)))?;

    let command = format!("\"{}\" \"%1\"", executable);
    command_key.set_value("", &command)
        .map_err(|e| Error::platform(format!("Failed to set command in ProgID: {}", e)))?;

    // 2. Register Capabilities
    let capabilities_path = format!("Software\\{}\\Capabilities", app_name);
    let (capabilities_key, _) = hkcu.create_subkey(&capabilities_path)
        .map_err(|e| Error::platform(format!("Failed to create Capabilities key: {}", e)))?;

    capabilities_key.set_value("ApplicationName", &scheme.description)
        .map_err(|e| Error::platform(format!("Failed to set Capabilities ApplicationName: {}", e)))?;

    capabilities_key.set_value("ApplicationDescription", &format!("Handle {} URIs", &scheme.scheme))
        .map_err(|e| Error::platform(format!("Failed to set Capabilities ApplicationDescription: {}", e)))?;

    // Register URL associations
    let (url_assoc_key, _) = capabilities_key.create_subkey("URLAssociations")
        .map_err(|e| Error::platform(format!("Failed to create URLAssociations key: {}", e)))?;

    url_assoc_key.set_value(&scheme.scheme, &prog_id)
        .map_err(|e| Error::platform(format!("Failed to set URL association: {}", e)))?;

    // 3. Register the application in RegisteredApplications
    let (registered_apps, _) = hkcu.create_subkey("Software\\RegisteredApplications")
        .map_err(|e| Error::platform(format!("Failed to create RegisteredApplications key: {}", e)))?;

    registered_apps.set_value(app_name, &capabilities_path)
        .map_err(|e| Error::platform(format!("Failed to register application: {}", e)))?;

    Ok(())
}

/// Unregister a URI scheme on Windows
///
/// This removes all registry entries including application capabilities
#[cfg(windows)]
pub fn unregister(scheme: &str) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu.open_subkey_with_flags("Software\\Classes", KEY_WRITE)
        .map_err(|e| Error::platform(format!("Failed to open Software\\Classes: {}", e)))?;

    // Delete the scheme key
    classes.delete_subkey_all(scheme)
        .map_err(|e| Error::platform(format!("Failed to delete scheme key: {}", e)))?;

    // Try to find and delete ProgID and app capabilities
    // We need to find the app name by looking through registered applications
    if let Ok(reg_apps) = hkcu.open_subkey("Software\\RegisteredApplications") {
        // Find all registered apps that reference this scheme
        let mut apps_to_remove = Vec::new();

        for app_name in reg_apps.enum_values().filter_map(|r| r.ok()) {
            let app_name_str = app_name.0;
            if let Ok(caps_path) = reg_apps.get_value::<String, _>(&app_name_str) {
                // Check if this app handles our scheme
                if let Ok(caps_key) = hkcu.open_subkey(format!("{}\\URLAssociations", caps_path)) {
                    if caps_key.get_value::<String, _>(scheme).is_ok() {
                        apps_to_remove.push((app_name_str.clone(), caps_path));
                    }
                }
            }
        }

        // Remove the apps and their capabilities
        let reg_apps_write = hkcu.open_subkey_with_flags("Software\\RegisteredApplications", KEY_WRITE)
            .map_err(|e| Error::platform(format!("Failed to open RegisteredApplications for writing: {}", e)))?;

        for (app_name, caps_path) in apps_to_remove {
            // Delete from RegisteredApplications
            let _ = reg_apps_write.delete_value(&app_name);

            // Delete capabilities
            if let Some(parent_path) = caps_path.rsplitn(2, '\\').nth(1) {
                if let Ok(parent_key) = hkcu.open_subkey_with_flags(parent_path, KEY_WRITE) {
                    let _ = parent_key.delete_subkey_all("Capabilities");
                }
            }

            // Delete ProgID
            let prog_id = format!("{}.{}", app_name, scheme);
            let _ = classes.delete_subkey_all(&prog_id);
        }
    }

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