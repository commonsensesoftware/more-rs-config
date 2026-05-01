use cfg_if::cfg_if;
use config::*;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};
use tokens::{ChangeToken, SharedChangeToken, SingleChangeToken};

cfg_if! {
    if #[cfg(feature = "async")] {
        type Rc<T> = std::sync::Arc<T>;
        type Mut<T> = std::sync::Mutex<T>;
    } else {
        type Rc<T> = std::rc::Rc<T>;
        type Mut<T> = std::cell::RefCell<T>;
    }
}

#[derive(Default)]
struct Trigger {
    token: Mut<SharedChangeToken<SingleChangeToken>>,
}

impl Trigger {
    fn fire(&self) {
        cfg_if! {
            if #[cfg(feature = "async")] {
                let token = std::mem::take(&mut *self.token.lock().unwrap());
            } else {
                let token = self.token.replace(SharedChangeToken::default());
            }
        }

        token.notify();
    }
}

struct ReloadableProvider {
    counter: AtomicU8,
    trigger: Rc<Trigger>,
}

impl ReloadableProvider {
    fn new(trigger: Rc<Trigger>) -> Self {
        Self {
            counter: AtomicU8::new(1),
            trigger,
        }
    }
}

impl Provider for ReloadableProvider {
    fn name(&self) -> &str {
        "Test"
    }

    fn load(&self, settings: &mut Settings) -> Result {
        settings.insert("Test", self.counter.fetch_add(1, Ordering::Relaxed).to_string());
        Ok(())
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        cfg_if! {
            if #[cfg(feature = "async")] {
                Box::new((*self.trigger.token.lock().unwrap()).clone())
            } else {
                Box::new(self.trigger.token.borrow().clone())
            }
        }
    }
}

#[derive(Default)]
struct ReloadableSource {
    trigger: Rc<Trigger>,
}

impl ReloadableSource {
    fn new(trigger: Rc<Trigger>) -> Self {
        Self { trigger }
    }
}

impl Source for ReloadableSource {
    fn build(&mut self, _properties: &mut Properties) -> Box<dyn Provider> {
        Box::new(ReloadableProvider::new(self.trigger.clone()))
    }
}

#[test]
fn load_should_reload_providers() {
    // arrange
    let mut builder = config::builder();

    builder.add(ReloadableSource::default());

    let root = builder.build();
    let mut config = root.load().unwrap();

    assert_eq!(config.get("Test"), Some("1"));

    // act
    config = root.load().unwrap();

    // assert
    assert_eq!(config.get("Test"), Some("2"));
}

#[test]
fn reload_token_should_indicate_change_after_provider_change() {
    // arrange
    let trigger = Rc::new(Trigger::default());
    let data = Arc::<AtomicU8>::default();
    let mut builder = config::builder();

    builder.add(ReloadableSource::new(trigger.clone()));

    let root = builder.build();
    let config = root.load().unwrap();
    let _unused = config.reload_token().register(
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
