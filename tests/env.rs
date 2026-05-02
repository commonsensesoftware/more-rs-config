use config::prelude::*;
use std::env::{set_var, var};

#[test]
fn add_env_vars_should_load_environment_variables() {
    // arrange
    let config = config::builder().add_env_vars().build().unwrap();
    let expected = var("CARGO_PKG_NAME").unwrap();

    // act
    let actual = config.get("CargoPkgName");

    // assert
    assert_eq!(actual, Some(&*expected));
}

#[test]
fn add_env_vars_should_load_filtered_environment_variables() {
    // arrange
    let config = config::builder()
        .add_env_vars_with_prefix("CARGO_PKG_")
        .build()
        .unwrap();
    let expected = var("CARGO_PKG_NAME").unwrap();

    // act
    let actual = config.get("NAME");

    // assert
    assert_eq!(actual, Some(&*expected));
    assert_eq!(config.get("PATH"), None);
}

#[test]
fn add_env_vars_should_translate_double_underscore_to_colon() {
    // arrange
    let expected = "any";

    set_var("Foo__Bar__Baz", expected);

    let config = config::builder().add_env_vars().build().unwrap();

    // act
    let actual = config.get("Foo:Bar:Baz");

    // assert
    assert_eq!(actual, Some(expected));
}
