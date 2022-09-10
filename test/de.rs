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

#[test]
fn from_config_should_deserialize_simple_struct() {
    let root = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [
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
            ]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
        )
        .build();

    // act
    let result = from_config::<Foo>(root.as_config());

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
        .add_in_memory(
            [
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
            ]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
        )
        .build();

    // act
    let result = from_config::<Parent>(root.as_config());

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
        .add_in_memory(
            [("Bar", "test"), ("Baz", "true")]
                .iter()
                .map(|t| (t.0.to_owned(), t.1.to_owned()))
                .collect(),
        )
        .build();

    // act
    let error = from_config::<Foo>(root.as_config()).err().unwrap();

    // assert
    assert_eq!(error, Error::MissingValue("Doom"));
}

#[test]
fn from_config_should_fail_with_invalid_type() {
    // arrange
    let root = DefaultConfigurationBuilder::new()
        .add_in_memory(
            [
                ("Bar", "test"),
                ("Baz", "notabool"),
                ("Doom:0", "1"),
                ("Doom:1", "2"),
                ("Doom:2", "3"),
            ]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
        )
        .build();

    // act
    let error = from_config::<Foo>(root.as_config()).err().unwrap();

    // assert
    assert_eq!(error, Error::Custom(String::from("provided string was not `true` or `false` while parsing value \'notabool\' provided by Baz")));
}
