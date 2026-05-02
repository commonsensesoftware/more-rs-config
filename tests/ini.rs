use config::{prelude::*, Error};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tempfile::{tempdir, NamedTempFile};
use tokens::ChangeToken;

#[test]
fn add_ini_file_should_load_settings_from_file() {
    // arrange
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(b"[Service]\n").unwrap();
    file.write_all(b"Enabled=false\n\n").unwrap();
    file.write_all(b"[Feature.Magic]\n").unwrap();
    file.write_all(b"Disabled=true").unwrap();

    let config = config::builder().add_ini_file(file.path()).build().unwrap();
    let section = config.section("Feature.Magic");

    // act
    let actual = section.get("Disabled");

    // assert
    assert_eq!(actual, Some("true"));
}

#[test]
fn add_ini_file_should_fail_if_file_does_not_exist() {
    // arrange
    let path = PathBuf::from(r"C:\fake\settings.ini");

    // act
    let result = config::builder().add_ini_file(&path).build();

    // assert
    if let Err(error) = result {
        if matches!(error, Error::MissingFile(_)) {
            assert_eq!(
                &error.to_string(),
                r"The configuration file 'C:\fake\settings.ini' was not found, but is required."
            )
        } else {
            panic!("{:?}", error)
        }
    } else {
        panic!("No error occurred.")
    }
}

#[test]
fn add_optional_ini_file_should_load_settings_from_file() {
    // arrange
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(b"[Service]\n").unwrap();
    file.write_all(b"Enabled=false\n\n").unwrap();
    file.write_all(b"[Feature.Magic]\n").unwrap();
    file.write_all(b"Disabled=true").unwrap();

    let config = config::builder()
        .add_ini_file(file.path().is().optional())
        .build()
        .unwrap();
    let section = config.section("Feature.Magic");

    // act
    let actual = section.get("Disabled");

    // assert
    assert_eq!(actual, Some("true"));
}

#[test]
fn add_ini_file_should_succeed_if_optional_file_does_not_exist() {
    // arrange
    let path = PathBuf::from(r"C:\fake\settings.ini");

    // act
    let config = config::builder().add_ini_file(&path.is().optional()).build().unwrap();

    // assert
    assert_eq!(config.sections().len(), 0);
}

#[test]
fn init_file_should_reload_when_changed() {
    // arrange
    let dir = tempdir().unwrap();
    let path = dir.path().join("settings.ini");
    let mut file = File::create(&path).unwrap();

    file.write_all(b"[Service]\n").unwrap();
    file.write_all(b"Enabled=false\n\n").unwrap();
    file.write_all(b"[Feature.Magic]\n").unwrap();
    file.write_all(b"Disabled=true").unwrap();
    drop(file);

    let builder = config::builder().add_ini_file(&path.is().reloadable());
    let mut config = builder.build().unwrap();
    let section = config.section("Feature.Magic");
    let initial = section.get("Disabled").unwrap_or_default().to_owned();

    drop(section);

    let token = config.reload_token();
    let state = Arc::new((Mutex::new(false), Condvar::new()));
    let _unused = token.register(
        Box::new(|s| {
            let data = s.unwrap();
            let (reloaded, event) = &*(data.downcast_ref::<(Mutex<bool>, Condvar)>().unwrap());
            *reloaded.lock().unwrap() = true;
            event.notify_one();
        }),
        Some(state.clone()),
    );

    file = File::create(&path).unwrap();
    file.write_all(b"[Service]\n").unwrap();
    file.write_all(b"Enabled=false\n\n").unwrap();
    file.write_all(b"[Feature.Magic]\n").unwrap();
    file.write_all(b"Disabled=false").unwrap();
    drop(file);

    let (mutex, event) = &*state;
    let mut reloaded = mutex.lock().unwrap();

    while !*reloaded {
        reloaded = event.wait_timeout(reloaded, Duration::from_secs(1)).unwrap().0;
    }

    config = builder.build().unwrap();

    // act
    let section = config.section("Feature.Magic");
    let current = section.get("Disabled").unwrap_or_default();

    // assert
    assert_eq!(initial, "true");
    assert_eq!(current, "false");
}
