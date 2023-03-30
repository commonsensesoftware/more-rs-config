use config::{ext::*, *};
use std::env::var;

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
