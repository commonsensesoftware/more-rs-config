use config::*;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};
use tokens::{ChangeToken, SharedChangeToken, SingleChangeToken};

#[derive(Default)]
struct Trigger {
    token: RefCell<SharedChangeToken<SingleChangeToken>>,
}

impl Trigger {
    fn fire(&self) {
        let token = self.token.replace(SharedChangeToken::default());
        token.notify();
    }
}

struct ReloadableConfigProvider {
    counter: u8,
    value: Value,
    trigger: Rc<Trigger>,
}

impl ReloadableConfigProvider {
    fn new(trigger: Rc<Trigger>) -> Self {
        Self {
            counter: 0,
            value: Value::new("0".into()),
            trigger,
        }
    }
}

impl ConfigurationProvider for ReloadableConfigProvider {
    fn get(&self, key: &str) -> Option<Value> {
        if key == "Test" {
            Some(self.value.clone())
        } else {
            None
        }
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        Box::new(self.trigger.token.borrow().clone())
    }

    fn load(&mut self) -> LoadResult {
        self.counter += 1;
        self.value = self.counter.to_string().into();
        Ok(())
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

    let mut root = builder.build().unwrap();

    assert_eq!(root.get("Test").unwrap().as_str(), "1");

    // act
    root.reload().ok();

    // assert
    assert_eq!(root.get("Test").unwrap().as_str(), "2");
}

#[test]
fn reload_token_should_indicate_change_after_reload() {
    // arrange
    let data = Arc::<AtomicU8>::default();
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(ReloadableConfigSource::default()));

    let mut root = builder.build().unwrap();
    let _unused = root.reload_token().register(
        Box::new(|state| {
            state
                .unwrap()
                .downcast_ref::<AtomicU8>()
                .unwrap()
                .store(1, Ordering::SeqCst)
        }),
        Some(data.clone()),
    );

    // act
    root.reload().ok();

    // assert
    assert_eq!(data.load(Ordering::SeqCst), 1);
}

#[test]
fn reload_token_should_indicate_change_after_provider_change() {
    // arrange
    let trigger = Rc::new(Trigger::default());
    let data = Arc::<AtomicU8>::default();
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(ReloadableConfigSource::new(trigger.clone())));

    let root = builder.build().unwrap();
    let _unused = root.reload_token().register(
        Box::new(|state| {
            state
                .unwrap()
                .downcast_ref::<AtomicU8>()
                .unwrap()
                .store(1, Ordering::SeqCst)
        }),
        Some(data.clone()),
    );

    // act
    trigger.fire();

    // assert
    assert_eq!(data.load(Ordering::SeqCst), 1);
}
