#![cfg(feature = "async")]

use config::prelude::*;

fn assert_send_and_sync(_: impl Send + Sync) {}

#[test]
fn root_should_be_send_and_sync() {
    // arrange
    let root = config::builder().add_in_memory(&[("Key", "Value")]).build();

    // act

    // assert
    assert_send_and_sync(root);
}

#[test]
fn configuration_should_be_send_and_sync() {
    // arrange
    let cfg = config::builder()
        .add_in_memory(&[("Key", "Value")])
        .build()
        .load()
        .unwrap();

    // act

    // assert
    assert_send_and_sync(cfg);
}
