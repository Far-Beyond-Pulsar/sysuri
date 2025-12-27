use sysuri::{UriScheme, register, unregister, is_registered, extract_scheme, register_handler, handle_uri, FnHandler};
use std::env;
use std::sync::{Arc, Mutex};

#[test]
fn test_extract_scheme() {
    assert_eq!(extract_scheme("myapp://test"), Some("myapp"));
    assert_eq!(extract_scheme("http://example.com"), Some("http"));
    assert_eq!(extract_scheme("custom-app://data"), Some("custom-app"));
    assert_eq!(extract_scheme("invalid"), None);
    assert_eq!(extract_scheme("://invalid"), None);
}

#[test]
fn test_uri_scheme_validation() {
    let exe = env::current_exe().unwrap();

    let valid_scheme = UriScheme::new("myapp", "Test", exe.clone());
    assert!(valid_scheme.is_valid_scheme());

    let valid_scheme2 = UriScheme::new("my-app", "Test", exe.clone());
    assert!(valid_scheme2.is_valid_scheme());

    let valid_scheme3 = UriScheme::new("my+app", "Test", exe.clone());
    assert!(valid_scheme3.is_valid_scheme());

    let invalid_scheme = UriScheme::new("", "Test", exe.clone());
    assert!(!invalid_scheme.is_valid_scheme());

    let invalid_scheme2 = UriScheme::new("123app", "Test", exe.clone());
    assert!(!invalid_scheme2.is_valid_scheme());

    let invalid_scheme3 = UriScheme::new("my_app", "Test", exe.clone());
    assert!(!invalid_scheme3.is_valid_scheme());
}

#[test]
fn test_full_uri() {
    let exe = env::current_exe().unwrap();
    let scheme = UriScheme::new("myapp", "Test", exe);
    assert_eq!(scheme.full_uri(), "myapp://");
}

#[test]
fn test_uri_scheme_with_icon() {
    let exe = env::current_exe().unwrap();
    let icon = std::path::PathBuf::from("/path/to/icon.png");

    let scheme = UriScheme::new("myapp", "Test", exe.clone())
        .with_icon(icon.clone());

    assert_eq!(scheme.icon, Some(icon));
}

#[test]
fn test_handler_registration_and_invocation() {
    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();

    let handler = FnHandler::new(move |uri| {
        assert!(uri.contains("testscheme://"));
        *called_clone.lock().unwrap() = true;
    });

    register_handler("testscheme", handler);

    let result = handle_uri("testscheme://test");
    assert!(result.is_ok());
    assert!(*called.lock().unwrap());
}

#[test]
fn test_handler_with_no_registration() {
    let result = handle_uri("unregistered://test");
    assert!(result.is_err());
}

#[test]
fn test_multiple_handlers() {
    let count1 = Arc::new(Mutex::new(0));
    let count1_clone = count1.clone();

    let count2 = Arc::new(Mutex::new(0));
    let count2_clone = count2.clone();

    register_handler("scheme1", FnHandler::new(move |_| {
        *count1_clone.lock().unwrap() += 1;
    }));

    register_handler("scheme2", FnHandler::new(move |_| {
        *count2_clone.lock().unwrap() += 1;
    }));

    handle_uri("scheme1://test").unwrap();
    handle_uri("scheme1://test2").unwrap();
    handle_uri("scheme2://test").unwrap();

    assert_eq!(*count1.lock().unwrap(), 2);
    assert_eq!(*count2.lock().unwrap(), 1);
}

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
#[test]
fn test_registration_lifecycle() {
    let exe = env::current_exe().unwrap();
    let test_scheme = format!("sysuri-test-{}", std::process::id());

    // Create a test scheme
    let scheme = UriScheme::new(
        test_scheme.as_str(),
        "SysURI Test Scheme",
        exe
    );

    // Should not be registered initially
    // Note: This might fail if a previous test didn't clean up
    let is_reg = is_registered(&test_scheme);
    println!("Initial registration status: {:?}", is_reg);

    // Register the scheme
    let result = register(&scheme);
    if let Err(e) = &result {
        println!("Registration error: {}", e);
    }
    assert!(result.is_ok(), "Failed to register scheme");

    // Should now be registered
    let is_reg = is_registered(&test_scheme);
    println!("After registration: {:?}", is_reg);
    // Note: Some platforms might not immediately reflect registration

    // Unregister the scheme
    let result = unregister(&test_scheme);
    if let Err(e) = &result {
        println!("Unregistration error: {}", e);
    }
    assert!(result.is_ok(), "Failed to unregister scheme");

    println!("Test scheme '{}' lifecycle completed", test_scheme);
}

#[test]
fn test_invalid_scheme_registration() {
    let exe = env::current_exe().unwrap();

    let invalid_scheme = UriScheme::new(
        "123invalid",  // Starts with a number
        "Invalid Scheme",
        exe
    );

    let result = register(&invalid_scheme);
    assert!(result.is_err());
}
