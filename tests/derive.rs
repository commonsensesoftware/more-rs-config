use config::prelude::*;

#[derive(Debug, Default, PartialEq, config::Deserialize)]
#[serde(default)]
struct AppConfig {
    host: String,
    port: u16,
    debug: bool,
}

#[derive(Debug, Default, PartialEq, config::Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct CamelConfig {
    server_host: String,
    server_port: u16,
    max_retries: u32,
}

#[derive(Debug, Default, PartialEq, config::Deserialize)]
#[serde(default)]
struct SkipConfig {
    name: String,
    version: u32,
    #[serde(skip)]
    internal_state: String,
}

#[derive(Debug, Default, PartialEq, config::Deserialize)]
#[serde(default)]
struct AliasConfig {
    #[serde(alias = "user_name")]
    name: String,
    age: u32,
}

#[test]
fn bind_should_update_only_present_fields() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("host", "localhost")])
        .build()
        .unwrap();
    let expected = AppConfig {
        host: "localhost".to_string(),
        port: 9090,
        debug: true,
    };
    let mut actual = AppConfig {
        host: "original.com".to_string(),
        port: 9090,
        debug: true,
    };

    // act
    config.bind(&mut actual).unwrap();

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn bind_should_update_multiple_present_fields_but_leaves_absent_unchanged() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("host", "example.com"), ("debug", "false")])
        .build()
        .unwrap();
    let expected = AppConfig {
        host: "example.com".to_string(),
        port: 3000,
        debug: false,
    };
    let mut actual = AppConfig {
        host: "old.com".to_string(),
        port: 3000,
        debug: true,
    };

    // act
    config.bind(&mut actual).unwrap();

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn bind_should_update_all_fields() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("host", "new-host"), ("port", "8080"), ("debug", "true")])
        .build()
        .unwrap();
    let expected = AppConfig {
        host: "new-host".to_string(),
        port: 8080,
        debug: true,
    };
    let mut actual = AppConfig::default();

    // act
    config.bind(&mut actual).unwrap();

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn bind_with_no_matching_fields_leaves_struct_unchanged() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("unknown_key", "value")])
        .build()
        .unwrap();
    let expected = AppConfig {
        host: "keep-me".to_string(),
        port: 1234,
        debug: true,
    };
    let mut actual = AppConfig {
        host: "keep-me".to_string(),
        port: 1234,
        debug: true,
    };

    // act
    config.bind(&mut actual).unwrap();

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn bind_should_retain_last_writer() {
    // arrange
    let config1 = config::builder()
        .add_in_memory(&[("host", "first-host"), ("port", "1000")])
        .build()
        .unwrap();
    let config2 = config::builder()
        .add_in_memory(&[("host", "second-host"), ("debug", "true")])
        .build()
        .unwrap();
    let expected = AppConfig {
        host: "second-host".to_string(),
        port: 1000,
        debug: true,
    };
    let mut actual = AppConfig::default();

    // act
    config1.bind(&mut actual).unwrap();
    config2.bind(&mut actual).unwrap();

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn bind_should_accumulate_updates_across_sources() {
    // arrange
    let config_host = config::builder().add_in_memory(&[("host", "my-host")]).build().unwrap();
    let config_port = config::builder().add_in_memory(&[("port", "5000")]).build().unwrap();
    let config_debug = config::builder().add_in_memory(&[("debug", "true")]).build().unwrap();
    let expected = AppConfig {
        host: "my-host".to_string(),
        port: 5000,
        debug: true,
    };
    let mut actual = AppConfig::default();

    // act
    config_host.bind(&mut actual).unwrap();
    config_port.bind(&mut actual).unwrap();
    config_debug.bind(&mut actual).unwrap();

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn sequential_bind_later_source_overrides_earlier() {
    // arrange
    let config1 = config::builder().add_in_memory(&[("port", "3000")]).build().unwrap();
    let config2 = config::builder().add_in_memory(&[("port", "9000")]).build().unwrap();
    let expected = AppConfig {
        port: 9000,
        host: "unchanged".to_string(),
        debug: false,
    };
    let mut actual = AppConfig {
        host: "unchanged".to_string(),
        port: 0,
        debug: false,
    };

    // act
    config1.bind(&mut actual).unwrap();

    assert_eq!(actual.port, 3000);

    config2.bind(&mut actual).unwrap();

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn bind_should_never_update_skipped_fields() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[
            ("name", "test-app"),
            ("version", "42"),
            ("internal_state", "should-not-be-set"),
        ])
        .build()
        .unwrap();
    let expected = SkipConfig {
        name: "test-app".to_string(),
        version: 42,
        internal_state: "preserved".to_string(),
    };
    let mut actual = SkipConfig {
        name: String::new(),
        version: 0,
        internal_state: "preserved".to_string(),
    };

    // act
    config.bind(&mut actual).unwrap();

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn bind_should_match_primary_name_case_insensitively() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Name", "Alice"), ("Age", "30")])
        .build()
        .unwrap();
    let expected = AliasConfig {
        name: "Alice".to_string(),
        age: 30,
    };
    let mut actual = AliasConfig::default();

    // act
    config.bind(&mut actual).unwrap();

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn bind_should_use_camel_case_rename() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[
            ("serverHost", "api.example.com"),
            ("serverPort", "443"),
            ("maxRetries", "5"),
        ])
        .build()
        .unwrap();
    let expected = CamelConfig {
        server_host: "api.example.com".to_string(),
        server_port: 443,
        max_retries: 5,
    };
    let mut actual = CamelConfig::default();

    // act
    config.bind(&mut actual).unwrap();

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn bind_should_perform_partial_update_with_camel_case() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("serverPort", "8080")])
        .build()
        .unwrap();
    let expected = CamelConfig {
        server_host: "existing.com".to_string(),
        server_port: 8080,
        max_retries: 3,
    };
    let mut actual = CamelConfig {
        server_host: "existing.com".to_string(),
        server_port: 80,
        max_retries: 3,
    };

    // act
    config.bind(&mut actual).unwrap();

    // assert
    assert_eq!(actual, expected);
}
