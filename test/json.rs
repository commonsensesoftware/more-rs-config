use crate::util::new_temp_path;
use config::{ext::*, *};
use serde_json::json;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[test]
fn add_json_file_should_load_settings_from_file() {
    // arrange
    let json = json!({"service": {
       "enabled": false},
     "feature": {
         "nativeCopy": {
             "disabled": true}}
    });
    let path = new_temp_path("test_settings_1.json");
    let mut file = File::create(&path).unwrap();

    file.write_all(json.to_string().as_bytes()).unwrap();

    let config = DefaultConfigurationBuilder::new()
        .add_json_file(&path)
        .build();
    let section = config.section("Feature").section("NativeCopy");

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
    expected = r"The configuration file 'C:\fake\settings.json' was not found and is not optional."
)]
fn add_json_file_should_panic_if_file_does_not_exist() {
    // arrange
    let path = PathBuf::from(r"C:\fake\settings.json");

    // act
    let _ = DefaultConfigurationBuilder::new()
        .add_json_file(&path)
        .build();

    // assert
    // panics
}

#[test]
fn add_optional_json_file_should_load_settings_from_file() {
    let json = json!({"service": {
       "enabled": false},
     "feature": {
       "nativeCopy": {
           "disabled": true}}
    });
    let path = new_temp_path("test_settings_2.json");
    let mut file = File::create(&path).unwrap();

    file.write_all(json.to_string().as_bytes()).unwrap();

    let config = DefaultConfigurationBuilder::new()
        .add_json_file(FileSource::optional(&path))
        .build();
    let section = config.section("Feature").section("NativeCopy");

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
fn add_json_file_should_not_panic_if_file_does_not_exist() {
    // arrange
    let path = PathBuf::from(r"C:\fake\settings.json");

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_json_file(FileSource::optional(&path))
        .build();

    // assert
    assert_eq!(config.children().len(), 0);
}

#[test]
fn simple_json_array_should_be_converted_to_key_value_pairs() {
    // arrange
    let json = json!({"ip": ["1.2.3.4", "7.8.9.10", "11.12.13.14"]});
    let path = new_temp_path("array_settings_1.json");
    let mut file = File::create(&path).unwrap();

    file.write_all(json.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_json_file(&path)
        .build();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(config.get("ip:0").unwrap(), "1.2.3.4");
    assert_eq!(config.get("ip:1").unwrap(), "7.8.9.10");
    assert_eq!(config.get("ip:2").unwrap(), "11.12.13.14");
}

#[test]
fn complex_json_array_should_be_converted_to_key_value_pairs() {
    // arrange
    let json = json!({"ip": [
        {"address": "1.2.3.4", "hidden": false},
        {"address": "5.6.7.8", "hidden": true}
    ]});
    let path = new_temp_path("array_settings_2.json");
    let mut file = File::create(&path).unwrap();

    file.write_all(json.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_json_file(&path)
        .build();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(config.get("ip:0:address").unwrap(), "1.2.3.4");
    assert_eq!(config.get("ip:0:hidden").unwrap(), "false");
    assert_eq!(config.get("ip:1:address").unwrap(), "5.6.7.8");
    assert_eq!(config.get("ip:1:hidden").unwrap(), "true");
}

#[test]
fn nested_json_array_should_be_converted_to_key_value_pairs() {
    // arrange
    let json = json!({"ip": [
        ["1.2.3.4", "5.6.7.8"],
        ["9.10.11.12", "13.14.15.16"]
    ]});
    let path = new_temp_path("array_settings_3.json");
    let mut file = File::create(&path).unwrap();

    file.write_all(json.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_json_file(&path)
        .build();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }
    assert_eq!(config.get("ip:0:0").unwrap(), "1.2.3.4");
    assert_eq!(config.get("ip:0:1").unwrap(), "5.6.7.8");
    assert_eq!(config.get("ip:1:0").unwrap(), "9.10.11.12");
    assert_eq!(config.get("ip:1:1").unwrap(), "13.14.15.16");
}

#[test]
fn json_array_item_should_be_implicitly_replaced() {
    // arrange
    let json1 = json!({"ip": ["1.2.3.4", "7.8.9.10", "11.12.13.14"]});
    let json2 = json!({"ip": ["15.16.17.18"]});
    let path1 = new_temp_path("array_settings_4.json");
    let path2 = new_temp_path("array_settings_5.json");
    let mut file = File::create(&path1).unwrap();

    file.write_all(json1.to_string().as_bytes()).unwrap();
    file = File::create(&path2).unwrap();
    file.write_all(json2.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_json_file(&path1)
        .add_json_file(&path2)
        .build();

    // assert
    if path1.exists() {
        remove_file(&path1).ok();
    }
    if path2.exists() {
        remove_file(&path2).ok();
    }
    assert_eq!(config.section("ip").children().len(), 3);
    assert_eq!(config.get("ip:0").unwrap(), "15.16.17.18");
    assert_eq!(config.get("ip:1").unwrap(), "7.8.9.10");
    assert_eq!(config.get("ip:2").unwrap(), "11.12.13.14");
}

#[test]
fn json_array_item_should_be_explicitly_replaced() {
    // arrange
    let json1 = json!({"ip": ["1.2.3.4", "7.8.9.10", "11.12.13.14"]});
    let json2 = json!({"ip": {"1": "15.16.17.18"}});
    let path1 = new_temp_path("array_settings_6.json");
    let path2 = new_temp_path("array_settings_7.json");
    let mut file = File::create(&path1).unwrap();

    file.write_all(json1.to_string().as_bytes()).unwrap();
    file = File::create(&path2).unwrap();
    file.write_all(json2.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_json_file(&path1)
        .add_json_file(&path2)
        .build();

    // assert
    if path1.exists() {
        remove_file(&path1).ok();
    }
    if path2.exists() {
        remove_file(&path2).ok();
    }
    assert_eq!(config.section("ip").children().len(), 3);
    assert_eq!(config.get("ip:0").unwrap(), "1.2.3.4");
    assert_eq!(config.get("ip:1").unwrap(), "15.16.17.18");
    assert_eq!(config.get("ip:2").unwrap(), "11.12.13.14");
}

#[test]
fn json_arrays_should_be_merged() {
    // arrange
    let json1 = json!({"ip": ["1.2.3.4", "7.8.9.10", "11.12.13.14"]});
    let json2 = json!({"ip": {"3": "15.16.17.18"}});
    let path1 = new_temp_path("array_settings_8.json");
    let path2 = new_temp_path("array_settings_9.json");
    let mut file = File::create(&path1).unwrap();

    file.write_all(json1.to_string().as_bytes()).unwrap();
    file = File::create(&path2).unwrap();
    file.write_all(json2.to_string().as_bytes()).unwrap();

    // act
    let config = DefaultConfigurationBuilder::new()
        .add_json_file(&path1)
        .add_json_file(&path2)
        .build();

    // assert
    if path1.exists() {
        remove_file(&path1).ok();
    }
    if path2.exists() {
        remove_file(&path2).ok();
    }
    assert_eq!(config.section("ip").children().len(), 4);
    assert_eq!(config.get("ip:0").unwrap(), "1.2.3.4");
    assert_eq!(config.get("ip:1").unwrap(), "7.8.9.10");
    assert_eq!(config.get("ip:2").unwrap(), "11.12.13.14");
    assert_eq!(config.get("ip:3").unwrap(), "15.16.17.18");
}

#[test]
fn json_file_should_reload_when_changed() {
    // arrange
    let path = new_temp_path("reload_settings_1.json");
    let mut json = json!(
    {
        "service": {
            "enabled": false
        },
        "feature": {
            "nativeCopy": {
                "disabled": true
            }
        }
    });

    let mut file = File::create(&path).unwrap();
    file.write_all(json.to_string().as_bytes()).unwrap();
    drop(file);

    let config = DefaultConfigurationBuilder::new()
        .add_json_file(&path.is().reloadable())
        .build();
    let section = config.section("Feature").section("NativeCopy");
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

    json = json!(
    {
        "service": {
            "enabled": false
        },
        "feature": {
            "nativeCopy": {
                "disabled": false
            }
        }
    });

    file = File::create(&path).unwrap();
    file.write_all(json.to_string().as_bytes()).unwrap();
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
    let section = config.section("Feature").section("NativeCopy");
    let current = section.get("Disabled").unwrap_or_default();

    // assert
    if path.exists() {
        remove_file(&path).ok();
    }

    assert_eq!(&initial, "true");
    assert_eq!(&current, "false");
}
