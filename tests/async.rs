#![cfg(feature = "async")]

use config::prelude::*;

fn assert_send_and_sync(_: impl Send + Sync) {}

#[test]
fn builder_should_be_send_and_sync() {
    // arrange
    let builder = config::builder().add_in_memory(&[("Key", "Value")]);

    // act

    // assert
    assert_send_and_sync(builder);
}

#[test]
fn configuration_should_be_send_and_sync() {
    // arrange
    let cfg = config::builder().add_in_memory(&[("Key", "Value")]).build().unwrap();

    // act

    // assert
    assert_send_and_sync(cfg);
}
