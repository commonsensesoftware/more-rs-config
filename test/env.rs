use config::{ext::*, *};
use std::env::var;

#[test]
fn add_env_vars_should_load_environment_variables() {
    // arrange
    let config = DefaultConfigurationBuilder::new()
        .add_env_vars()
        .build()
        .to_config();
    let expected = var("USERNAME").unwrap();

    // act
    let value = config.get("USERNAME").unwrap();

    // assert
    assert_eq!(value, &expected);
}

#[test]
fn add_env_vars_should_load_filtered_environment_variables() {
    // arrange
    let config = DefaultConfigurationBuilder::new()
        .add_env_vars_with_prefix("PROCESSOR_")
        .build()
        .to_config();
    let expected = var("PROCESSOR_ARCHITECTURE").unwrap();

    // act
    let value = config.get("ARCHITECTURE").unwrap();

    // assert
    assert_eq!(value, &expected);
    assert!(config.get("SYSTEMROOT").is_none())
}