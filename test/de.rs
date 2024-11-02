use std::collections::HashMap;

use config::{ext::*, ConfigurationBuilder, DefaultConfigurationBuilder};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Size {
    Small,
    Medium,
    Large,
}

impl Default for Size {
    fn default() -> Size {
        Size::Medium
    }
}

pub fn default_kaboom() -> u16 {
    8080
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct CustomNewType(u32);

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct Foo {
    bar: String,
    baz: bool,
    zoom: Option<u16>,
    doom: Vec<u64>,
    boom: Vec<String>,
    #[serde(default = "default_kaboom")]
    kaboom: u16,
    #[serde(default, alias = "DebugMode")]
    debug_mode: bool,
    #[serde(default)]
    size: Size,
    provided: Option<String>,
    #[serde(alias = "NewType")]
    new_type: CustomNewType,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct Name {
    first: String,
    last: String,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct GrandChild {
    name: Name,
    age: u8,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct Child {
    name: Name,
    #[serde(alias = "MagicNumbers")]
    magic_numbers: Vec<u8>,
    children: Vec<GrandChild>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct Parent {
    name: Name,
    child: Child,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct UserDefined {
    settings: HashMap<String, Vec<String>>,
}

#[test]
fn from_config_should_deserialize_simple_struct() {
    let root = DefaultConfigurationBuilder::new()
        .add_in_memory(&[
            ("Bar", "test"),
            ("Baz", "true"),
            ("Doom:0", "1"),
            ("Doom:1", "2"),
            ("Doom:2", "3"),
            ("Boom", ""),
            ("Size", "small"),
            ("Provided", "test"),
            ("NewType", "42"),
            ("Child:Ignored", "42"),
        ])
        .build()
        .unwrap();

    // act
    let result = from_config::<Foo>(root.deref());

    // assert
    match result {
        Ok(actual) => assert_eq!(
            actual,
            Foo {
                bar: String::from("test"),
                baz: true,
                zoom: None,
                doom: vec![1, 2, 3],
                boom: vec![],
                kaboom: 8080,
                debug_mode: false,
                size: Size::Small,
                provided: Some(String::from("test")),
                new_type: CustomNewType(42)
            }
        ),
        Err(e) => panic!("{:#?}", e),
    }
}

#[test]
fn from_config_should_deserialize_parent_child_struct() {
    let root = DefaultConfigurationBuilder::new()
        .add_in_memory(&[
            ("Name:First", "Jane"),
            ("Name:Last", "Doe"),
            ("Child:Name:First", "John"),
            ("Child:Name:Last", "Doe"),
            ("Child:MagicNumbers:0", "42"),
            ("Child:MagicNumbers:1", "7"),
            ("Child:MagicNumbers:2", "13"),
            ("Child:Children:0:Name:First", "Bob"),
            ("Child:Children:0:Name:Last", "Doe"),
            ("Child:Children:0:Age", "7"),
            ("Child:Children:1:Name:First", "Sally"),
            ("Child:Children:1:Name:Last", "Doe"),
            ("Child:Children:1:Age", "5"),
        ])
        .build()
        .unwrap();

    // act
    let result = from_config::<Parent>(root.deref());

    // assert
    match result {
        Ok(actual) => assert_eq!(
            actual,
            Parent {
                name: Name {
                    first: String::from("Jane"),
                    last: String::from("Doe")
                },
                child: Child {
                    name: Name {
                        first: String::from("John"),
                        last: String::from("Doe")
                    },
                    magic_numbers: vec![42, 7, 13],
                    children: vec![
                        GrandChild {
                            name: Name {
                                first: String::from("Bob"),
                                last: String::from("Doe")
                            },
                            age: 7
                        },
                        GrandChild {
                            name: Name {
                                first: String::from("Sally"),
                                last: String::from("Doe")
                            },
                            age: 5
                        },
                    ]
                }
            }
        ),
        Err(e) => panic!("{:#?}", e),
    }
}

#[test]
fn from_config_should_fail_with_missing_value() {
    // arrange
    let root = DefaultConfigurationBuilder::new()
        .add_in_memory(&[("Bar", "test"), ("Baz", "true")])
        .build()
        .unwrap();

    // act
    let error = from_config::<Foo>(root.deref()).err().unwrap();

    // assert
    assert_eq!(error, Error::MissingValue("Doom"));
}

#[test]
fn from_config_should_fail_with_invalid_type() {
    // arrange
    let root = DefaultConfigurationBuilder::new()
        .add_in_memory(&[
            ("Bar", "test"),
            ("Baz", "notabool"),
            ("Doom:0", "1"),
            ("Doom:1", "2"),
            ("Doom:2", "3"),
        ])
        .build()
        .unwrap();

    // act
    let error = from_config::<Foo>(root.deref()).err().unwrap();

    // assert
    assert_eq!(error, Error::Custom(String::from("provided string was not `true` or `false` while parsing value \'notabool\' provided by Baz")));
}

#[test]
fn from_config_should_deserialize_nested_string_map() {
    // arrange
    let root = DefaultConfigurationBuilder::new()
        .add_in_memory(&[
            ("Dimensions:Foo", "bar"),
            ("Dimensions:Bar", "foo"),
            ("Dimensions:Baz", "other"),
        ])
        .build()
        .unwrap();

    // act
    let result = from_config::<HashMap<String, String>>(root.section("Dimensions").deref());

    // assert
    match result {
        Ok(actual) => assert_eq!(
            actual,
            [("Foo", "bar"), ("Bar", "foo"), ("Baz", "other")]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<HashMap<_, _>>(),
        ),
        Err(e) => panic!("{:#?}", e),
    }
}

#[test]
fn from_config_should_deserialize_nested_typed_map() {
    // arrange
    let root = DefaultConfigurationBuilder::new()
        .add_in_memory(&[
            ("Limits:Foo", "42"),
            ("Limits:Bar", "0"),
            ("Limits:Baz", "420"),
        ])
        .build()
        .unwrap();

    // act
    let result = from_config::<HashMap<String, usize>>(root.section("Limits").deref());

    // assert
    match result {
        Ok(actual) => assert_eq!(
            actual,
            [("Foo", 42), ("Bar", 0), ("Baz", 420)]
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect::<HashMap<_, usize>>(),
        ),
        Err(e) => panic!("{:#?}", e),
    }
}

#[test]
fn from_config_should_deserialize_map_with_nested_vec() {
    // arrange
    let root = DefaultConfigurationBuilder::new()
        .add_in_memory(&[
            ("Settings:Key1:0", "bar"),
            ("Settings:Key2:0", "foo"),
            ("Settings:Key3:0", "a"),
            ("Settings:Key3:1", "b"),
            ("Settings:Key3:2", "c"),
        ])
        .build()
        .unwrap();
    let expected = HashMap::from([
        ("Key1".to_owned(), vec!["bar".to_owned()]),
        ("Key2".to_owned(), vec!["foo".to_owned()]),
        (
            "Key3".to_owned(),
            vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
        ),
    ]);

    // act
    let result = from_config::<HashMap<String, Vec<String>>>(root.section("Settings").deref());

    // assert
    match result {
        Ok(actual) => assert_eq!(actual, expected),
        Err(e) => panic!("{:#?}", e),
    }
}

#[test]
fn reify_should_deserialize_map_with_nested_vec() {
    // arrange
    let root = DefaultConfigurationBuilder::new()
        .add_in_memory(&[
            ("Settings:Key1:0", "bar"),
            ("Settings:Key2:0", "foo"),
            ("Settings:Key3:0", "a"),
            ("Settings:Key3:1", "b"),
            ("Settings:Key3:2", "c"),
        ])
        .build()
        .unwrap();
    let expected = UserDefined {
        settings: HashMap::from([
            ("Key1".to_owned(), vec!["bar".to_owned()]),
            ("Key2".to_owned(), vec!["foo".to_owned()]),
            (
                "Key3".to_owned(),
                vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
            ),
        ]),
    };

    // act
    let actual: UserDefined = root.reify();

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn from_config_should_deserialize_deep_nested_map() {
    // arrange
    let root = DefaultConfigurationBuilder::new()
        .add_in_memory(&[
            ("Key1:0:Subkey1_0:0", "1"),
            ("Key1:0:Subkey1_0:1", "2"),
            ("Key1:0:Subkey1_0:2", "3"),
            ("Key1:0:Subkey2_0:0", "4"),
            ("Key1:0:Subkey2_0:1", "5"),
            ("Key1:0:Subkey2_0:2", "6"),
            ("Key1:1:Subkey3_0:0", "7"),
            ("Key1:1:Subkey3_0:1", "8"),
            ("Key1:1:Subkey3_0:2", "9"),
            ("Key2:0:Subkey4_0:0", "a"),
            ("Key2:0:Subkey4_0:1", "b"),
            ("Key2:0:Subkey4_0:2", "c"),
            ("Key3:0:Subkey5_0:0", "d"),
            ("Key3:0:Subkey5_0:1", "e"),
            ("Key3:0:Subkey5_0:2", "f"),
            ("Key3:0:Subkey6_0:0", "x"),
            ("Key3:0:Subkey6_0:1", "y"),
            ("Key3:0:Subkey6_0:2", "z"),
        ])
        .build()
        .unwrap();
    let expected = HashMap::from([
        (
            "Key1".to_owned(),
            vec![
                HashMap::from([
                    (
                        "Subkey1_0".to_owned(),
                        vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
                    ),
                    (
                        "Subkey2_0".to_owned(),
                        vec!["4".to_owned(), "5".to_owned(), "6".to_owned()],
                    ),
                ]),
                HashMap::from([(
                    "Subkey3_0".to_owned(),
                    vec!["7".to_owned(), "8".to_owned(), "9".to_owned()],
                )]),
            ],
        ),
        (
            "Key2".to_owned(),
            vec![HashMap::from([(
                "Subkey4_0".to_owned(),
                vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
            )])],
        ),
        (
            "Key3".to_owned(),
            vec![HashMap::from([
                (
                    "Subkey5_0".to_owned(),
                    vec!["d".to_owned(), "e".to_owned(), "f".to_owned()],
                ),
                (
                    "Subkey6_0".to_owned(),
                    vec!["x".to_owned(), "y".to_owned(), "z".to_owned()],
                ),
            ])],
        ),
    ]);

    // act
    let result = from_config::<HashMap<String, Vec<HashMap<String, Vec<String>>>>>(root.deref());

    // assert
    match result {
        Ok(actual) => assert_eq!(actual, expected),
        Err(e) => panic!("{:#?}", e),
    }
}
