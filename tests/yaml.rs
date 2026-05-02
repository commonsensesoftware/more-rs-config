use config::{prelude::*, Error, FileSource};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tempfile::{tempdir, NamedTempFile};
use tokens::ChangeToken;

#[test]
fn add_yaml_file_should_load_settings_from_file() {
    // arrange
    let yaml = concat!(
        "service:\n",
        "  enabled: false\n",
        "feature:\n",
        "  nativeCopy:\n",
        "    disabled: true",
    );
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(yaml.as_bytes()).unwrap();

    let config = config::builder().add_yaml_file(file.path()).build().unwrap();
    let section = config.section("Feature").section("NativeCopy");

    // act
    let actual = section.get("Disabled");

    // assert
    assert_eq!(actual, Some("true"));
}

#[test]
fn add_yaml_file_should_fail_if_file_does_not_exist() {
    // arrange
    let path = std::path::PathBuf::from("/fake/nonexistent/settings.yaml");

    // act
    let result = config::builder().add_yaml_file(&path).build();

    // assert
    if let Err(error) = result {
        assert!(
            matches!(error, Error::MissingFile(_)),
            "Expected MissingFile error, got: {:?}",
            error
        );
    } else {
        panic!("No error occurred.")
    }
}

#[test]
fn add_optional_yaml_file_should_load_settings_from_file() {
    // arrange
    let yaml = concat!(
        "service:\n",
        "  enabled: false\n",
        "feature:\n",
        "  nativeCopy:\n",
        "    disabled: true"
    );
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(yaml.as_bytes()).unwrap();

    let config = config::builder()
        .add_yaml_file(FileSource::optional(file.path()))
        .build()
        .unwrap();
    let section = config.section("Feature").section("NativeCopy");

    // act
    let actual = section.get("Disabled");

    // assert
    assert_eq!(actual, Some("true"));
}

#[test]
fn add_yaml_file_should_succeed_if_optional_file_does_not_exist() {
    // arrange
    let path = PathBuf::from("/fake/nonexistent/settings.yaml");

    // act
    let config = config::builder()
        .add_yaml_file(FileSource::optional(&path))
        .build()
        .unwrap();

    // assert
    assert_eq!(config.sections().len(), 0);
}

#[test]
fn simple_yaml_array_should_be_converted_to_key_value_pairs() {
    // arrange
    #[rustfmt::skip]
    let yaml = concat!(
        "ip:\n",
        "  - a\n",
        "  - b\n",
        "  - c",
    );
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();

    // act
    let config = config::builder().add_yaml_file(file.path()).build().unwrap();

    // assert
    assert_eq!(config.get("ip:0"), Some("a"));
    assert_eq!(config.get("ip:1"), Some("b"));
    assert_eq!(config.get("ip:2"), Some("c"));
}

#[test]
fn complex_yaml_array_should_be_converted_to_key_value_pairs() {
    // arrange
    #[rustfmt::skip]
    let yaml = concat!(
        "ip:\n",
        "  - address: 1.2.3.4\n",
        "    hidden: false\n",
        "  - address: 5.6.7.8\n",
        "    hidden: true\n"
    );
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(yaml.as_bytes()).unwrap();

    // act
    let config = config::builder().add_yaml_file(file.path()).build().unwrap();

    // assert
    assert_eq!(config.get("ip:0:Address"), Some("1.2.3.4"));
    assert_eq!(config.get("ip:0:Hidden"), Some("false"));
    assert_eq!(config.get("ip:1:Address"), Some("5.6.7.8"));
    assert_eq!(config.get("ip:1:Hidden"), Some("true"));
}

#[test]
fn nested_yaml_array_should_be_converted_to_key_value_pairs() {
    // arrange
    #[rustfmt::skip]
    let yaml = concat!(
        "ip:\n",
        "  - - 1.2.3.4\n",
        "    - 5.6.7.8\n",
        "  - - 9.10.11.12\n",
        "    - 13.14.15.16",
    );
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(yaml.as_bytes()).unwrap();

    // act
    let config = config::builder().add_yaml_file(file.path()).build().unwrap();

    // assert
    assert_eq!(config.get("ip:0:0"), Some("1.2.3.4"));
    assert_eq!(config.get("ip:0:1"), Some("5.6.7.8"));
    assert_eq!(config.get("ip:1:0"), Some("9.10.11.12"));
    assert_eq!(config.get("ip:1:1"), Some("13.14.15.16"));
}

#[test]
fn multiple_yaml_documents_should_use_first_only() {
    // arrange
    #[rustfmt::skip]
    let yaml = concat!(
        "service:\n",
        "  enabled: true\n",
        "---\n",
        "service:\n",
        "  enabled: false"
    );
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(yaml.as_bytes()).unwrap();

    // act
    let config = config::builder().add_yaml_file(file.path()).build().unwrap();

    // assert
    assert_eq!(config.get("Service:Enabled"), Some("true"));
}

#[test]
fn non_mapping_top_level_yaml_should_return_invalid_file_error() {
    // arrange
    #[rustfmt::skip]
    let yaml = concat!(
        "- item1\n",
        "- item2\n",
        "- item3"
    );
    let mut file = NamedTempFile::new().unwrap();

    file.write_all(yaml.as_bytes()).unwrap();

    // act
    let result = config::builder().add_yaml_file(file.path()).build();

    // assert
    if let Err(error) = result {
        if let Error::InvalidFile { message, .. } = &error {
            assert!(
                message.contains("array"),
                "Expected error message to contain 'array', got: {}",
                message
            );
        } else {
            panic!("Expected InvalidFile error, got: {:?}", error);
        }
    } else {
        panic!("No error occurred.");
    }
}

#[test]
fn yaml_array_item_should_be_implicitly_replaced() {
    // arrange
    #[rustfmt::skip]
    let yaml1 = concat!(
        "ip:\n",
        "  - 1.2.3.4\n",
        "  - 7.8.9.10\n",
        "  - 11.12.13.14"
    );

    #[rustfmt::skip]
    let yaml2 = concat!(
        "ip:\n",
        "  - 15.16.17.18"
    );

    let dir = tempdir().unwrap();
    let path1 = dir.path().join("settings.1.yaml");
    let path2 = dir.path().join("settings.2.yaml");
    let mut file = File::create(&path1).unwrap();

    file.write_all(yaml1.as_bytes()).unwrap();
    file = File::create(&path2).unwrap();
    file.write_all(yaml2.as_bytes()).unwrap();

    // act
    let config = config::builder()
        .add_yaml_file(&path1)
        .add_yaml_file(&path2)
        .build()
        .unwrap();

    // assert
    assert_eq!(config.section("ip").sections().len(), 3);
    assert_eq!(config.get("ip:0"), Some("15.16.17.18"));
    assert_eq!(config.get("ip:1"), Some("7.8.9.10"));
    assert_eq!(config.get("ip:2"), Some("11.12.13.14"));
}

#[test]
fn yaml_array_item_should_be_explicitly_replaced() {
    // arrange
    #[rustfmt::skip]
    let yaml1 = concat!(
        "ip:\n",
        "  - 1.2.3.4\n",
        "  - 7.8.9.10\n",
        "  - 11.12.13.14"
    );

    #[rustfmt::skip]
    let yaml2 = concat!(
        "ip:\n",
        "  \"1\": \"15.16.17.18\"",
    );

    let dir = tempdir().unwrap();
    let path1 = dir.path().join("settings.1.yaml");
    let path2 = dir.path().join("settings.2.yaml");
    let mut file = File::create(&path1).unwrap();

    file.write_all(yaml1.as_bytes()).unwrap();
    file = File::create(&path2).unwrap();
    file.write_all(yaml2.as_bytes()).unwrap();

    // act
    let config = config::builder()
        .add_yaml_file(&path1)
        .add_yaml_file(&path2)
        .build()
        .unwrap();

    // assert
    assert_eq!(config.section("ip").sections().len(), 3);
    assert_eq!(config.get("ip:0"), Some("1.2.3.4"));
    assert_eq!(
        config.get("ip:1"),
        Some("15.16.17.18"),
        "Expected 7.8.9.10 to be replaced with 15.16.17.18"
    );
    assert_eq!(config.get("ip:2"), Some("11.12.13.14"));
}

#[test]
fn yaml_arrays_should_be_merged() {
    // arrange
    #[rustfmt::skip]
    let yaml1 = concat!(
        "ip:\n",
        "  - 1.2.3.4\n",
        "  - 7.8.9.10\n",
        "  - 11.12.13.14"
    );

    #[rustfmt::skip]
    let yaml2 = concat!(
        "ip:\n",
        "  \"3\": \"15.16.17.18\"",
    );

    let dir = tempdir().unwrap();
    let path1 = dir.path().join("settings.1.yaml");
    let path2 = dir.path().join("settings.2.yaml");
    let mut file = File::create(&path1).unwrap();

    file.write_all(yaml1.as_bytes()).unwrap();
    file = File::create(&path2).unwrap();
    file.write_all(yaml2.as_bytes()).unwrap();

    // act
    let config = config::builder()
        .add_yaml_file(&path1)
        .add_yaml_file(&path2)
        .build()
        .unwrap();

    // assert
    assert_eq!(config.section("ip").sections().len(), 4);
    assert_eq!(config.get("ip:0"), Some("1.2.3.4"));
    assert_eq!(config.get("ip:1"), Some("7.8.9.10"));
    assert_eq!(config.get("ip:2"), Some("11.12.13.14"));
    assert_eq!(config.get("ip:3"), Some("15.16.17.18"));
}

#[test]
fn yaml_file_should_reload_when_changed() {
    // arrange
    #[rustfmt::skip]
    let initial_yaml = concat!(
        "service:\n",
        "  enabled: false\n",
        "feature:\n",
        "  nativeCopy:\n",
        "    disabled: true"
    );
    let dir = tempdir().unwrap();
    let path = dir.path().join("settings.yaml");
    let mut file = File::create(&path).unwrap();

    file.write_all(initial_yaml.as_bytes()).unwrap();
    drop(file);

    let builder = config::builder().add_yaml_file(&path.is().reloadable());
    let mut config = builder.build().unwrap();
    let section = config.section("Feature").section("NativeCopy");
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

    #[rustfmt::skip]
    let updated_yaml = concat!(
        "service:\n",
        "  enabled: false\n",
        "feature:\n",
        "  nativeCopy:\n",
        "    disabled: false"
    );

    file = File::create(&path).unwrap();
    file.write_all(updated_yaml.as_bytes()).unwrap();
    drop(file);

    let (mutex, event) = &*state;
    let mut reloaded = mutex.lock().unwrap();

    while !*reloaded {
        reloaded = event.wait_timeout(reloaded, Duration::from_secs(1)).unwrap().0;
    }

    // act
    config = builder.build().unwrap();

    let section = config.section("Feature").section("NativeCopy");
    let current = section.get("Disabled").unwrap_or_default();

    // assert
    assert_eq!(initial, "true");
    assert_eq!(current, "false");
}
