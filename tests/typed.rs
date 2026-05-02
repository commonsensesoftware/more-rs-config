use config::{prelude::*, ser, typed, Provider as _, Settings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
struct BoolStruct {
    flag: bool,
}

#[test]
fn serialize_bool_should_write_true() {
    // arrange
    let mut settings = Settings::new();
    let value = BoolStruct { flag: true };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Flag"), Some("true"));
}

#[test]
fn serialize_bool_should_write_false() {
    // arrange
    let mut settings = Settings::new();
    let value = BoolStruct { flag: false };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Flag"), Some("false"));
}

#[derive(Serialize)]
struct I8Struct {
    val: i8,
}

#[derive(Serialize)]
struct I16Struct {
    val: i16,
}

#[derive(Serialize)]
struct I32Struct {
    val: i32,
}

#[derive(Serialize)]
struct I64Struct {
    val: i64,
}

#[derive(Serialize)]
struct U8Struct {
    val: u8,
}

#[derive(Serialize)]
struct U16Struct {
    val: u16,
}

#[derive(Serialize)]
struct U32Struct {
    val: u32,
}

#[derive(Serialize)]
struct U64Struct {
    val: u64,
}

#[test]
fn serialize_should_write_i8() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&I8Struct { val: -42 }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val"), Some("-42"));
}

#[test]
fn serialize_should_write_i16() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&I16Struct { val: -1000 }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val"), Some("-1000"));
}

#[test]
fn serialize_should_write_i32() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&I32Struct { val: 42 }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val"), Some("42"));
}

#[test]
fn serialize_should_write_i64() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&I64Struct { val: 123456789 }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val"), Some("123456789"));
}

#[test]
fn serialize_should_write_u8() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&U8Struct { val: 255 }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val"), Some("255"));
}

#[test]
fn serialize_should_write_u16() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&U16Struct { val: 65535 }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val"), Some("65535"));
}

#[test]
fn serialize_should_write_u32() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&U32Struct { val: 100000 }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val"), Some("100000"));
}

#[test]
fn serialize_should_write_u64() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&U64Struct { val: u64::MAX }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val"), Some("18446744073709551615"));
}

#[derive(Serialize)]
struct F32Struct {
    val: f32,
}

#[derive(Serialize)]
struct F64Struct {
    val: f64,
}

#[test]
fn serialize_should_write_f32() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&F32Struct { val: 3.14 }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val"), Some(3.14f32.to_string().as_str()));
}

#[test]
fn serialize_should_write_f64() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&F64Struct { val: 2.718 }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val"), Some("2.718"));
}

#[derive(Serialize)]
struct StringStruct {
    name: String,
}

#[test]
fn serialize_should_write_string() {
    // arrange
    let mut settings = Settings::new();
    let value = StringStruct {
        name: "Alice".to_owned(),
    };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Name"), Some("Alice"));
}

#[derive(Serialize)]
struct CharStruct {
    sep: char,
}

#[test]
fn serialize_should_write_char() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&CharStruct { sep: ',' }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Sep"), Some(","));
}

#[derive(Serialize)]
struct UnitFieldStruct {
    marker: (),
}

#[derive(Serialize)]
struct Marker;

#[derive(Serialize)]
struct UnitStructWrapper {
    marker: Marker,
}

#[test]
fn serialize_should_write_unit_value() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&UnitFieldStruct { marker: () }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Marker"), Some(""));
}

#[test]
fn serialize_should_write_unit_struct() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&UnitStructWrapper { marker: Marker }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Marker"), Some(""));
}

#[derive(Serialize)]
struct OptionNoneStruct {
    opt: Option<i32>,
}

#[test]
fn serialize_should_write_none() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&OptionNoneStruct { opt: None }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Opt"), Some(""));
}

#[derive(Serialize)]
struct OptionSomeStruct {
    opt: Option<i32>,
}

#[test]
fn serialize_should_write_optional_value() {
    // arrange
    let mut settings = Settings::new();

    // act
    ser::into(&OptionSomeStruct { opt: Some(42) }, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Opt"), Some("42"));
}

#[derive(Serialize)]
struct Level3 {
    c: String,
}

#[derive(Serialize)]
struct Level2 {
    b: Level3,
}

#[derive(Serialize)]
struct Level1 {
    a: Level2,
}

#[test]
fn serialize_should_write_with_three_level_nesting() {
    // arrange
    let mut settings = Settings::new();
    let value = Level1 {
        a: Level2 {
            b: Level3 { c: "val".to_owned() },
        },
    };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("A:B:C"), Some("val"));
}

#[derive(Serialize)]
struct SnakeCaseStruct {
    hello_world: String,
}

#[test]
fn serialize_should_convert_snake_case_to_pascal_case() {
    // arrange
    let mut settings = Settings::new();
    let value = SnakeCaseStruct {
        hello_world: "greeting".to_owned(),
    };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("HelloWorld"), Some("greeting"));
}

#[test]
fn serialize_should_write_map_with_string_keys() {
    #[derive(Serialize)]
    struct MapStruct {
        dims: HashMap<String, String>,
    }

    // arrange
    let mut map = HashMap::new();
    map.insert("width".to_owned(), "100".to_owned());
    map.insert("height".to_owned(), "200".to_owned());
    let mut settings = Settings::new();
    let value = MapStruct { dims: map };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Dims:width"), Some("100"));
    assert_eq!(settings.get("Dims:height"), Some("200"));
}

#[derive(Serialize)]
struct VecStruct {
    items: Vec<String>,
}

#[test]
fn serialize_should_write_vec_zero_based_indices() {
    // arrange
    let mut settings = Settings::new();
    let value = VecStruct {
        items: vec!["first".to_owned(), "second".to_owned()],
    };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Items:0"), Some("first"));
    assert_eq!(settings.get("Items:1"), Some("second"));
}

#[derive(Serialize)]
struct TupleStruct {
    items: (String, String, String),
}

#[test]
fn serialize_should_write_tuple_with_zero_based_indices() {
    // arrange
    let mut settings = Settings::new();
    let value = TupleStruct {
        items: ("a".to_owned(), "b".to_owned(), "c".to_owned()),
    };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Items:0"), Some("a"));
    assert_eq!(settings.get("Items:1"), Some("b"));
    assert_eq!(settings.get("Items:2"), Some("c"));
}

#[derive(Serialize)]
struct NamedItem {
    name: String,
}

#[derive(Serialize)]
struct VecOfStructs {
    items: Vec<NamedItem>,
}

#[test]
fn serialize_should_write_vec_of_structs() {
    // arrange
    let mut settings = Settings::new();
    let value = VecOfStructs {
        items: vec![
            NamedItem {
                name: "Alice".to_owned(),
            },
            NamedItem { name: "Bob".to_owned() },
        ],
    };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Items:0:Name"), Some("Alice"));
    assert_eq!(settings.get("Items:1:Name"), Some("Bob"));
}

#[derive(Serialize)]
enum MyEnum {
    Unit,
    Newtype(i32),
    Tuple(i32, String),
    Struct { id: i32, name: String },
}

#[derive(Serialize)]
struct EnumWrapper {
    val: MyEnum,
}

#[test]
fn serialize_should_write_unit_enum_variant() {
    // arrange
    let mut settings = Settings::new();
    let value = EnumWrapper { val: MyEnum::Unit };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val"), Some("Unit"));
}

#[test]
fn serialize_should_write_new_type_enum_variant() {
    // arrange
    let mut settings = Settings::new();
    let value = EnumWrapper {
        val: MyEnum::Newtype(42),
    };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val:Newtype"), Some("42"));
}

#[test]
fn serialize_should_write_tuple_enum_variant() {
    // arrange
    let mut settings = Settings::new();
    let value = EnumWrapper {
        val: MyEnum::Tuple(1, "hello".to_owned()),
    };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val:Tuple:0"), Some("1"));
    assert_eq!(settings.get("Val:Tuple:1"), Some("hello"));
}

#[test]
fn serialize_should_write_struct_enum_variant() {
    // arrange
    let mut settings = Settings::new();
    let value = EnumWrapper {
        val: MyEnum::Struct {
            id: 42,
            name: "test".to_owned(),
        },
    };

    // act
    ser::into(&value, &mut settings).unwrap();

    // assert
    assert_eq!(settings.get("Val:Struct:Id"), Some("42"));
    assert_eq!(settings.get("Val:Struct:Name"), Some("test"));
}

#[test]
fn serialize_with_non_string_map_key_should_return_error() {
    #[derive(Serialize, Hash, Eq, PartialEq)]
    struct ComplexKey {
        x: i32,
    }

    #[derive(Serialize)]
    struct BadMapStruct {
        data: HashMap<ComplexKey, String>,
    }

    // arrange
    let mut map = HashMap::new();
    map.insert(ComplexKey { x: 1 }, "value".to_owned());
    let mut settings = Settings::new();

    // act
    let result = ser::into(&BadMapStruct { data: map }, &mut settings);

    // assert
    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{}", err);
    assert_eq!(msg, "map keys must be serializable to strings");
}

#[test]
fn provider_load_should_propagate_serialization_error() {
    #[derive(Serialize, Hash, Eq, PartialEq)]
    struct ComplexKey {
        x: i32,
    }

    #[derive(Serialize)]
    struct BadMapStruct {
        data: HashMap<ComplexKey, String>,
    }

    // arrange
    let mut map = HashMap::new();
    map.insert(ComplexKey { x: 1 }, "value".to_owned());
    let provider = typed::Provider::new(BadMapStruct { data: map });
    let mut settings = Settings::new();

    // act
    let result = provider.load(&mut settings);

    // assert
    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{}", err);
    assert_eq!(msg, "map keys must be serializable to strings");
}

#[test]
fn builder_add_typed_should_produce_working_configuration() {
    #[derive(Serialize)]
    struct AppConfig {
        name: String,
        port: u16,
        debug: bool,
        rate: f64,
    }

    // arrange
    let my_struct = AppConfig {
        name: "my-app".to_owned(),
        port: 8080,
        debug: true,
        rate: 3.14,
    };

    // act
    let config = config::builder().add_typed(my_struct).build().unwrap();

    // assert
    assert_eq!(config.get("Name"), Some("my-app"));
    assert_eq!(config.get("Port"), Some("8080"));
    assert_eq!(config.get("Debug"), Some("true"));
    assert_eq!(config.get("Rate"), Some("3.14"));
}

#[test]
fn serde_should_roundtrip_with_struct_and_enum_types() {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct Place {
        name: String,
        longitude: f64,
        latitude: f64,
        favorite: bool,
        reviews: u64,
    }

    // arrange
    let original = Place {
        name: "Eiffel Tower".to_owned(),
        longitude: 2.2945,
        latitude: 48.8584,
        favorite: true,
        reviews: 50000,
    };
    let expected = original.clone();

    // act
    let config = config::builder().add_typed(original).build().unwrap();
    let deserialized: Place = config::de::from(&config).unwrap();

    // assert
    assert_eq!(config.get("Name"), Some("Eiffel Tower"));
    assert_eq!(config.get("Favorite"), Some("true"));
    assert_eq!(config.get("Reviews"), Some("50000"));
    assert_eq!(deserialized, expected);
}

#[test]
fn serde_should_roundtrip_with_nested_vecs_and_options() {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct ServerConfig {
        host: String,
        port: u16,
        tags: Vec<String>,
        max_retries: Option<u32>,
    }

    // arrange
    let original = ServerConfig {
        host: "localhost".to_owned(),
        port: 3000,
        tags: vec!["web".to_owned(), "api".to_owned(), "v2".to_owned()],
        max_retries: Some(5),
    };
    let expected = original.clone();

    // act
    let config = config::builder().add_typed(original).build().unwrap();
    let deserialized: ServerConfig = config::de::from(&config).unwrap();

    // assert
    assert_eq!(config.get("Host"), Some("localhost"));
    assert_eq!(config.get("Port"), Some("3000"));
    assert_eq!(config.get("Tags:0"), Some("web"));
    assert_eq!(config.get("Tags:1"), Some("api"));
    assert_eq!(config.get("Tags:2"), Some("v2"));
    assert_eq!(config.get("MaxRetries"), Some("5"));
    assert_eq!(deserialized, expected);
}

#[test]
fn serde_should_roundtrip_with_enum_variants() {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    enum Enum {
        First,
        Second(String),
        Third { id: usize, kind: String },
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct Place {
        name: String,
        enums: Vec<Enum>,
    }

    // arrange
    let original = Place {
        name: "some name".to_owned(),
        enums: vec![
            Enum::First,
            Enum::Second("test".to_owned()),
            Enum::Third {
                id: 42,
                kind: "Encounter".to_owned(),
            },
        ],
    };
    let expected = original.clone();

    // act
    let config = config::builder().add_typed(original).build().unwrap();
    let deserialized: Place = config::de::from(&config).unwrap();

    // assert
    assert_eq!(config.get("Name"), Some("some name"));
    assert_eq!(config.get("Enums:0"), Some("First"));
    assert_eq!(config.get("Enums:1:Second"), Some("test"));
    assert_eq!(config.get("Enums:2:Third:Id"), Some("42"));
    assert_eq!(config.get("Enums:2:Third:Kind"), Some("Encounter"));
    assert_eq!(deserialized, expected);
}

#[test]
fn serde_should_roundtrip_typed_configuration() {
    #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
    struct Client {
        region: String,
        url: String,
    }

    #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
    struct PerfOptions {
        cores: u8,
    }

    #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
    struct AppOptions {
        text: String,
        perf: PerfOptions,
        clients: Vec<Client>,
    }

    // arrange
    let expected = AppOptions {
        text: "Banana processor".into(),
        perf: PerfOptions { cores: 42 },
        clients: vec![Client {
            region: "us-west".into(),
            url: "http://tempuri.org".into(),
        }],
    };
    let config = config::builder().add_typed(expected.clone()).build().unwrap();

    // act
    let actual: AppOptions = config.reify().unwrap();

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn serde_should_roundtrip_typed_map() {
    // arrange
    let scores: HashMap<String, u8> = HashMap::from([
        ("Mariners".to_owned(), 5u8),
        ("Athletics".to_owned(), 3),
        ("Yankees".to_owned(), 0),
    ]);
    let expected = scores.clone();
    let config = config::builder().add_typed(scores).build().unwrap();

    // act
    let actual: HashMap<String, u8> = config.reify().unwrap();

    // assert
    assert_eq!(actual, expected);
}
