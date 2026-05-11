use config_derive::Deserialize;

// Single type parameter
#[derive(Debug, PartialEq, Deserialize)]
struct Config<T> {
    value: T,
    name: String,
}

// Two type parameters
#[derive(Debug, PartialEq, Deserialize)]
struct Pair<A, B> {
    first: A,
    second: B,
}

// Three type parameters
#[derive(Debug, PartialEq, Deserialize)]
struct Triple<X, Y, Z> {
    x: X,
    y: Y,
    z: Z,
}

// Generic with defaults
#[derive(Debug, PartialEq, Deserialize)]
#[serde(default)]
struct WithDefault<T: Default> {
    value: T,
    label: String,
}

impl<T: Default> Default for WithDefault<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            label: String::new(),
        }
    }
}

// Struct where one type parameter is only used in a skipped field.
// This verifies that bounds are only applied to type parameters used in
// deserializable fields (T gets Deserialize bound, S does not need it).
#[derive(Debug, PartialEq, Deserialize)]
struct BoundsCheck<T, S: Default> {
    active: T,
    #[serde(skip)]
    skipped: S,
}

#[test]
fn deserialize_should_handle_single_type_parameter_with_u32() {
    // arrange
    let json = r#"{"value": 42, "name": "test"}"#;

    // act
    let result: Config<u32> = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        Config {
            value: 42,
            name: "test".to_string()
        }
    );
}

#[test]
fn deserialize_should_handle_single_type_parameter_with_string() {
    // arrange
    let json = r#"{"value": "hello", "name": "test"}"#;

    // act
    let result: Config<String> = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        Config {
            value: "hello".to_string(),
            name: "test".to_string()
        }
    );
}

#[test]
fn deserialize_should_handle_two_type_parameters() {
    // arrange
    let json = r#"{"first": 1, "second": "two"}"#;

    // act
    let result: Pair<u32, String> = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        Pair {
            first: 1,
            second: "two".to_string()
        }
    );
}

#[test]
fn deserialize_should_handle_three_type_parameters() {
    // arrange
    let json = r#"{"x": true, "y": 3.14, "z": "hello"}"#;

    // act
    let result: Triple<bool, f64, String> = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        Triple {
            x: true,
            y: 3.14,
            z: "hello".to_string()
        }
    );
}

#[test]
fn deserialize_should_apply_default_for_absent_generic_field() {
    // arrange
    let json = r#"{"value": 99}"#;

    // act
    let result: WithDefault<u32> = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(
        result,
        WithDefault {
            value: 99,
            label: String::new()
        }
    );
}

#[test]
fn deserialize_in_place_should_update_only_present_fields_for_single_generic() {
    // arrange
    let mut instance = Config {
        value: 10u32,
        name: "original".to_string(),
    };
    let json = r#"{"value": 42}"#;

    // act
    let mut deserializer = serde_json::Deserializer::from_str(json);
    serde::Deserialize::deserialize_in_place(&mut deserializer, &mut instance).unwrap();

    // assert
    assert_eq!(instance.value, 42);
    assert_eq!(instance.name, "original");
}

#[test]
fn deserialize_in_place_should_update_only_present_fields_for_pair() {
    // arrange
    let mut instance = Pair {
        first: 1u32,
        second: "hello".to_string(),
    };
    let json = r#"{"second": "world"}"#;

    // act
    let mut deserializer = serde_json::Deserializer::from_str(json);
    serde::Deserialize::deserialize_in_place(&mut deserializer, &mut instance).unwrap();

    // assert
    assert_eq!(instance.first, 1);
    assert_eq!(instance.second, "world");
}

#[test]
fn deserialize_in_place_should_update_only_present_fields_for_triple() {
    // arrange
    let mut instance = Triple {
        x: true,
        y: 1.0f64,
        z: "original".to_string(),
    };
    let json = r#"{"y": 9.99}"#;

    // act
    let mut deserializer = serde_json::Deserializer::from_str(json);
    serde::Deserialize::deserialize_in_place(&mut deserializer, &mut instance).unwrap();

    // assert
    assert_eq!(instance.x, true);
    assert_eq!(instance.y, 9.99);
    assert_eq!(instance.z, "original");
}

#[test]
fn deserialize_should_not_require_bounds_on_skipped_type_parameter() {
    // arrange
    #[derive(Debug, PartialEq, Default)]
    struct NotDeserializable {
        _data: u8,
    }

    let json = r#"{"active": 42}"#;

    // act
    let result: BoundsCheck<u32, NotDeserializable> = serde_json::from_str(json).unwrap();

    // assert
    assert_eq!(result.active, 42);
    assert_eq!(result.skipped, NotDeserializable::default());
}

#[test]
fn deserialize_in_place_should_not_require_bounds_on_skipped_type_parameter() {
    // arrange
    #[derive(Debug, PartialEq, Default)]
    struct NotDeserializable {
        _data: u8,
    }

    let mut instance = BoundsCheck {
        active: 10u32,
        skipped: NotDeserializable { _data: 5 },
    };
    let json = r#"{"active": 99}"#;

    // act
    let mut deserializer = serde_json::Deserializer::from_str(json);
    serde::Deserialize::deserialize_in_place(&mut deserializer, &mut instance).unwrap();

    // assert
    assert_eq!(instance.active, 99);
    assert_eq!(instance.skipped._data, 5);
}
