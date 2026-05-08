use config::prelude::*;
use serde::{Deserialize, Serialize};

#[test]
fn display_section_should_show_key_and_value() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Section1:Key1", "Value1")])
        .build()
        .unwrap();
    let section = config.section("Section1").section("Key1");

    // act
    let text = format!("{section}");

    // assert
    assert_eq!(text, "Key1 = Value1");
}

#[test]
fn display_section_should_show_key_and_value_with_provider() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Section1:Key1", "Value1")])
        .build()
        .unwrap();
    let section = config.section("Section1").section("Key1");

    // act
    let text = format!("{section:#}");

    // assert
    assert_eq!(text, "Key1 = Value1 (Memory)");
}

#[test]
fn display_section_without_value_should_show_key_with_colon() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Section1:Key1", "Value1")])
        .build()
        .unwrap();
    let section = config.section("Section1");

    // act
    let text = format!("{section}");

    // assert
    assert_eq!(text, "Section1:");
}

#[test]
fn display_section_with_value_should_show_key_equals_value() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Section1", "Hello")])
        .build()
        .unwrap();
    let section = config.section("Section1");

    // act
    let text = format!("{section}");

    // assert
    assert_eq!(text, "Section1 = Hello");
}

#[test]
fn display_section_with_value_should_show_key_equals_value_with_provider() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Section1", "Hello")])
        .build()
        .unwrap();
    let section = config.section("Section1");

    // act
    let text = format!("{section:#}");

    // assert
    assert_eq!(text, "Section1 = Hello (Memory)");
}

#[test]
fn display_owned_section_should_show_key_and_value() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Section1:Key1:Sub1", "Value1")])
        .build()
        .unwrap();
    let owned = config.section("Section1").to_owned();

    // act
    let text = format!("{owned}");

    // assert
    assert_eq!(text, "Section1:");
}

#[test]
fn display_owned_section_should_show_key_and_value_with_provider() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Section1:Key1:Sub1", "Value1")])
        .build()
        .unwrap();
    let owned = config.section("Section1").to_owned();

    // act
    let text = format!("{owned:#}");

    // assert
    assert_eq!(text, "Section1:");
}

#[test]
fn display_owned_section_without_value_should_show_key_with_colon() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Section1:Key1", "Value1")])
        .build()
        .unwrap();
    let owned = config.section("Section1").to_owned();

    // act
    let text = format!("{owned}");

    // assert
    assert_eq!(text, "Section1:");
}

#[test]
fn display_configuration_should_show_sections() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Section1:Key1", "Value1"), ("Section2:Key2", "Value2")])
        .build()
        .unwrap();
    let expected = concat!("Section1:\n", "  Key1 = Value1\n", "Section2:\n", "  Key2 = Value2",);

    // act
    let actual = format!("{config}");

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn display_configuration_should_show_sections_with_providers() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("Section1:Key1", "Value1"), ("Section2:Key2", "Value2")])
        .build()
        .unwrap();
    let expected = concat!(
        "Section1:\n",
        "  Key1 = Value1 (Memory)\n",
        "Section2:\n",
        "  Key2 = Value2 (Memory)",
    );

    // act
    let actual = format!("{config:#}");

    // assert
    assert_eq!(actual, expected);
}

#[test]
fn display_configuration_with_multiple_providers_should_show_override_chain() {
    // arrange
    #[derive(Deserialize, Serialize)]
    struct Settings {
        key: String,
    }

    impl Default for Settings {
        fn default() -> Self {
            Self { key: "First".into() }
        }
    }

    let config = config::builder()
        .add_typed(Settings::default())
        .add_in_memory(&[("Key", "Second")])
        .build()
        .unwrap();

    // act
    let text = format!("{config:#}");

    // assert
    assert_eq!(text, "Key = Second (Typed → Memory)");
}

#[test]
fn display_configuration_with_multiple_providers_should_show_effective_provider() {
    // arrange
    #[derive(Deserialize, Serialize)]
    struct Settings {
        key: String,
    }

    impl Default for Settings {
        fn default() -> Self {
            Self { key: "First".into() }
        }
    }

    let config = config::builder()
        .add_typed(Settings::default())
        .add_in_memory(&[("Key", "Second")])
        .build()
        .unwrap();

    // act
    let text = format!("{config:#1}");

    // assert
    assert_eq!(text, "Key = Second (Memory)");
}

#[test]
fn display_configuration_should_indent_nested_sections() {
    // arrange
    let config = config::builder()
        .add_in_memory(&[("A:B:C:Key", "Val")])
        .build()
        .unwrap();
    let expected = "A:\n  B:\n    C:\n      Key = Val";

    // act
    let actual = format!("{config}");

    // assert
    assert_eq!(actual, expected);
}
