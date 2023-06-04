use config::{ext::*, *};
use std::env::{set_var, var};

#[test]
fn add_env_vars_should_load_environment_variables() {
    // arrange
    let config = DefaultConfigurationBuilder::new().add_env_vars().build();
    let expected = var("CARGO_PKG_NAME").unwrap();

    // act
    let value = config.get("CARGO_PKG_NAME").unwrap();

    // assert
    assert_eq!(value, &expected);
}

#[test]
fn add_env_vars_should_load_filtered_environment_variables() {
    // arrange
    let config = DefaultConfigurationBuilder::new()
        .add_env_vars_with_prefix("CARGO_PKG_")
        .build();
    let expected = var("CARGO_PKG_NAME").unwrap();

    // act
    let value = config.get("NAME").unwrap();

    // assert
    assert_eq!(value, &expected);
    assert!(config.get("PATH").is_none())
}

#[test]
fn double_underscores_are_translated() {
    // arrange
    let expected = "myvalue";
    set_var("Foo__Bar__Baz", expected);
    let config = DefaultConfigurationBuilder::new().add_env_vars().build();

    // act
    let value = config.get("Foo:Bar:Baz").unwrap();

    // assert
    assert_eq!(value, expected);
}
