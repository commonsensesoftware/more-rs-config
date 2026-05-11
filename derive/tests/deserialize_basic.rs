use config_derive::Deserialize;

/// Basic struct with no special attributes.
#[derive(Debug, PartialEq, Deserialize)]
struct Simple {
    name: String,
    age: u32,
}

/// Struct with field-level defaults.
#[derive(Debug, PartialEq, Deserialize)]
struct WithDefaults {
    name: String,
    #[serde(default)]
    enabled: bool,
}

/// Struct with container-level default.
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
struct ContainerDefault {
    host: String,
    port: u16,
}

/// Struct with renamed fields.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Renamed {
    server_port: u16,
    host_name: String,
}

/// Struct with aliases.
#[derive(Debug, PartialEq, Deserialize)]
struct WithAlias {
    #[serde(alias = "user_name")]
    name: String,
}

/// Struct with skipped fields.
#[derive(Debug, PartialEq, Deserialize)]
struct WithSkip {
    name: String,
    #[serde(skip)]
    internal: u32,
}

/// Struct with skip_deserializing.
#[derive(Debug, PartialEq, Deserialize)]
struct WithSkipDeserializing {
    name: String,
    #[serde(skip_deserializing)]
    computed: String,
}

#[test]
fn deserialize_should_construct_struct_from_complete_json() {
    // arrange
    let json = r#"{"name": "Alice", "age": 30}"#;

    // act
    let result: Simple = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        Simple {
            name: "Alice".to_string(),
            age: 30
        }
    );
}

#[test]
fn deserialize_should_error_when_required_field_is_missing() {
    // arrange
    let json = r#"{"name": "Alice"}"#;

    // act
    let result = serde_json::from_str::<Simple>(json);

    // assert
    assert!(result.is_err());
}

#[test]
fn deserialize_should_apply_field_default_for_absent_field() {
    // arrange
    let json = r#"{"name": "Bob"}"#;

    // act
    let result: WithDefaults = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        WithDefaults {
            name: "Bob".to_string(),
            enabled: false
        }
    );
}

#[test]
fn deserialize_should_apply_container_default_for_absent_fields() {
    // arrange
    let json = r#"{"host": "localhost"}"#;

    // act
    let result: ContainerDefault = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        ContainerDefault {
            host: "localhost".to_string(),
            port: 0
        }
    );
}

#[test]
fn deserialize_should_apply_container_default_for_all_fields_when_empty() {
    // arrange
    let json = r#"{}"#;

    // act
    let result: ContainerDefault = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        ContainerDefault {
            host: String::new(),
            port: 0
        }
    );
}

#[test]
fn deserialize_should_match_camel_case_renamed_keys() {
    // arrange
    let json = r#"{"serverPort": 8080, "hostName": "example.com"}"#;

    // act
    let result: Renamed = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        Renamed {
            server_port: 8080,
            host_name: "example.com".to_string()
        }
    );
}

#[test]
fn deserialize_should_accept_primary_name_and_alias() {
    // arrange / act / assert — primary name
    let json = r#"{"name": "Alice"}"#;
    let result: WithAlias = serde_json::from_str(json).unwrap();
    assert_eq!(
        result,
        WithAlias {
            name: "Alice".to_string()
        }
    );

    // arrange / act / assert — alias
    let json = r#"{"user_name": "Bob"}"#;
    let result: WithAlias = serde_json::from_str(json).unwrap();
    assert_eq!(
        result,
        WithAlias {
            name: "Bob".to_string()
        }
    );
}

#[test]
fn deserialize_should_use_default_for_skipped_field_regardless_of_input() {
    // arrange
    let json = r#"{"name": "Alice", "internal": 999}"#;

    // act
    let result: WithSkip = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        WithSkip {
            name: "Alice".to_string(),
            internal: 0
        }
    );
}

#[test]
fn deserialize_should_use_default_for_skip_deserializing_field() {
    // arrange
    let json = r#"{"name": "Alice", "computed": "should_be_ignored"}"#;

    // act
    let result: WithSkipDeserializing = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        WithSkipDeserializing {
            name: "Alice".to_string(),
            computed: String::new()
        }
    );
}

#[test]
fn deserialize_should_ignore_unknown_keys() {
    // arrange
    let json = r#"{"name": "Alice", "age": 30, "unknown_field": "ignored"}"#;

    // act
    let result: Simple = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        Simple {
            name: "Alice".to_string(),
            age: 30
        }
    );
}
