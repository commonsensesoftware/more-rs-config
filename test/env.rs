use config::{ext::*, *};
use std::env::{set_var, var};

#[test]
fn add_env_vars_should_load_environment_variables() {
    // arrange
    let config = DefaultConfigurationBuilder::new()
        .add_env_vars()
        .build()
        .unwrap();
    let expected = var("CARGO_PKG_NAME").unwrap();

    // act
    let value = config.get("CARGO_PKG_NAME").unwrap();

    // assert
    assert_eq!(value.as_str(), expected);
}

#[test]
fn add_env_vars_should_load_filtered_environment_variables() {
    // arrange
    let config = DefaultConfigurationBuilder::new()
        .add_env_vars_with_prefix("CARGO_PKG_")
        .build()
        .unwrap();
    let expected = var("CARGO_PKG_NAME").unwrap();

    // act
    let value = config.get("NAME").unwrap();

    // assert
    assert_eq!(value.as_str(), expected);
    assert!(config.get("PATH").is_none())
}

#[test]
fn add_env_vars_should_translate_double_underscore_to_colon() {
    // arrange
    let expected = "any";

    set_var("Foo__Bar__Baz", expected);

    let config = DefaultConfigurationBuilder::new()
        .add_env_vars()
        .build()
        .unwrap();

    // act
    let value = config.get("Foo:Bar:Baz").unwrap();

    // assert
    assert_eq!(value.as_str(), expected);
}
