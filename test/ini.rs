use config::{ext::*, *};
use std::env::temp_dir;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[test]
fn add_ini_file_should_load_settings_from_file() {
    // arrange
    let path = temp_dir().join("test_settings_1.ini");
    let mut file = File::create(&path).unwrap();

    file.write_all(b"[Service]\n").unwrap();
    file.write_all(b"Enabled=false\n\n").unwrap();
    file.write_all(b"[Feature.Magic]\n").unwrap();
    file.write_all(b"Disabled=true").unwrap();

    let config = DefaultConfigurationBuilder::new()
        .add_ini_file(&path)
        .build();
    let section = config.section("Feature.Magic");

    // act
    let result = section.get("Disabled");

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    let value = result.unwrap();
    assert_eq!(value, "true");
}

#[test]
#[should_panic(
    expected = r"The configuration file 'C:\fake\settings.ini' was not found and is not optional."
)]
fn add_ini_file_should_panic_if_file_does_not_exist() {
    // arrange
    let path = PathBuf::from(r"C:\fake\settings.ini");

    // act
    let _ = DefaultConfigurationBuilder::new()
        .add_ini_file(&path)
        .build();

    // assert
    // panics
}

#[test]
fn add_optional_ini_file_should_load_settings_from_file() {
    // arrange
    let path = temp_dir().join("test_settings_2.ini");
    let mut file = File::create(&path).unwrap();

    file.write_all(b"[Service]\n").unwrap();
    file.write_all(b"Enabled=false\n\n").unwrap();
    file.write_all(b"[Feature.Magic]\n").unwrap();
    file.write_all(b"Disabled=true").unwrap();

    let config = DefaultConfigurationBuilder::new()
        .add_ini_file(&path.is().optional())
        .build();
    let section = config.section("Feature.Magic");

    // act
    let result = section.get("Disabled");

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    let value = result.unwrap();
    assert_eq!(value, "true");
}

#[test]
fn add_ini_file_should_not_panic_if_file_does_not_exist() {
    // arrange
    let path = PathBuf::from(r"C:\fake\settings.ini");

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_ini_file(&path.is().optional())
        .build();

    // assert
    assert_eq!(config.children().len(), 0);
}

#[test]
fn init_file_should_reload_when_changed() {
    // arrange
    let path = temp_dir().join("test_settings_3.ini");
    let mut file = File::create(&path).unwrap();

    file.write_all(b"[Service]\n").unwrap();
    file.write_all(b"Enabled=false\n\n").unwrap();
    file.write_all(b"[Feature.Magic]\n").unwrap();
    file.write_all(b"Disabled=true").unwrap();
    drop(file);

    let config = DefaultConfigurationBuilder::new()
        .add_ini_file(&path.is().reloadable())
        .build();
    let section = config.section("Feature.Magic");
    let initial = section.get("Disabled").unwrap_or_default();

    drop(section);

    let token = config.reload_token();
    let state = Arc::new((Mutex::new(false), Condvar::new()));
    let other_state = Arc::clone(&state);
    let _unused = token.register(Box::new(move || {
        let (reloaded, event) = &*other_state;
        *reloaded.lock().unwrap() = true;
        event.notify_one();
    }));

    file = File::create(&path).unwrap();
    file.write_all(b"[Service]\n").unwrap();
    file.write_all(b"Enabled=false\n\n").unwrap();
    file.write_all(b"[Feature.Magic]\n").unwrap();
    file.write_all(b"Disabled=false").unwrap();
    drop(file);

    let (mutex, event) = &*state;
    let mut reloaded = mutex.lock().unwrap();

    while !*reloaded {
        reloaded = event
            .wait_timeout(reloaded, Duration::from_secs(1))
            .unwrap()
            .0;
    }

    // act
    let section = config.section("Feature.Magic");
    let current = section.get("Disabled").unwrap_or_default();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }

    assert_eq!(&initial, "true");
    assert_eq!(&current, "false");
}
