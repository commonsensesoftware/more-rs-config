use config::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use tokens::{ChangeToken, SharedChangeToken};

#[derive(Default)]
struct Trigger {
    token: RefCell<SharedChangeToken>,
}

impl Trigger {
    fn fire(&self) {
        let token = self.token.replace(SharedChangeToken::default());
        let callback = token.trigger().upgrade().unwrap();
        (callback)()
    }
}

struct ReloadableConfigProvider {
    counter: u8,
    value: String,
    trigger: Rc<Trigger>,
}

impl ReloadableConfigProvider {
    fn new(trigger: Rc<Trigger>) -> Self {
        Self {
            counter: 0,
            value: "0".into(),
            trigger,
        }
    }
}

impl ConfigurationProvider for ReloadableConfigProvider {
    fn get(&self, key: &str) -> Option<&str> {
        if key == "Test" {
            Some(&self.value)
        } else {
            None
        }
    }

    fn reload_token(&self) -> Option<Box<dyn ChangeToken>> {
        Some(Box::new(self.trigger.token.borrow().clone()))
    }

    fn load(&mut self) {
        self.counter += 1;
        self.value = self.counter.to_string();
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, _parent_path: Option<&str>) {
        earlier_keys.push("Test".into());
    }
}

#[derive(Default)]
struct ReloadableConfigSource {
    trigger: Rc<Trigger>,
}

impl ReloadableConfigSource {
    fn new(trigger: Rc<Trigger>) -> Self {
        Self { trigger }
    }
}

impl ConfigurationSource for ReloadableConfigSource {
    fn build(&self, _builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider> {
        Box::new(ReloadableConfigProvider::new(self.trigger.clone()))
    }
}

#[test]
fn reload_should_load_providers() {
    // arrange
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(ReloadableConfigSource::default()));

    let mut root = builder.build();

    assert_eq!(root.get("Test").unwrap(), "1");

    // act
    root.reload();

    // assert
    assert_eq!(root.get("Test").unwrap(), "2");
}

#[test]
fn reload_token_should_indicate_change_after_reload() {
    // arrange
    let data = Rc::<Cell<u8>>::default();
    let other = data.clone();
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(ReloadableConfigSource::default()));

    let mut root = builder.build();

    root.reload_token()
        .register(Box::new(move || other.clone().set(1)));

    // act
    root.reload();

    // assert
    assert_eq!(data.get(), 1);
}

#[test]
fn reload_token_should_indicate_change_after_provider_change() {
    // arrange
    let trigger = Rc::new(Trigger::default());
    let data = Rc::<Cell<u8>>::default();
    let other = data.clone();
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(ReloadableConfigSource::new(trigger.clone())));

    let root = builder.build();

    root.reload_token()
        .register(Box::new(move || other.clone().set(1)));

    // act
    trigger.fire();

    // assert
    assert_eq!(data.get(), 1);
}
