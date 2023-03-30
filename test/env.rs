use config::{ext::*, *};
use std::env::var;

#[test]
fn add_env_vars_should_load_environment_variables() {
    // arrange
    let config = DefaultConfigurationBuilder::new().add_env_vars().build();
    let key = if cfg!(windows) { "USERNAME" } else { "USER" };
    let expected = var(key).unwrap();

    // act
    let value = config.get(key).unwrap();

    // assert
    assert_eq!(value, &expected);
}

#[test]
fn add_env_vars_should_load_filtered_environment_variables() {
    // arrange
    let (prefix, key, unexpected) = if cfg!(windows) {
        ("PROCESSOR_", "ARCHITECTURE", "USERNAME")
    } else {
        ("LS_", "COLORS", "USER")
    };
    let config = DefaultConfigurationBuilder::new()
        .add_env_vars_with_prefix(prefix)
        .build();
    let expected = var([prefix, key].join("")).unwrap();

    // act
    let value = config.get(key).unwrap();

    // assert
    assert_eq!(value, &expected);
    assert!(config.get(unexpected).is_none())
}
