use config::*;
use std::mem::take;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc, Mutex,
};
use tokens::{ChangeToken, SharedChangeToken, SingleChangeToken};

#[derive(Default)]
struct Trigger {
    token: Mutex<SharedChangeToken<SingleChangeToken>>,
}

impl Trigger {
    fn fire(&self) {
        let token = take(&mut *self.token.lock().unwrap());
        token.notify();
    }
}

struct ReloadableProvider {
    counter: AtomicU8,
    trigger: Arc<Trigger>,
}

impl ReloadableProvider {
    fn new(trigger: Arc<Trigger>) -> Self {
        Self {
            counter: AtomicU8::new(1),
            trigger,
        }
    }
}

impl Default for ReloadableProvider {
    fn default() -> Self {
        Self {
            counter: AtomicU8::new(1),
            trigger: Default::default(),
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
        Box::new((*self.trigger.token.lock().unwrap()).clone())
    }
}

#[test]
fn load_should_reload_providers() {
    // arrange
    let mut builder = config::builder();

    builder.add(ReloadableProvider::default());

    let mut config = builder.build().unwrap();

    assert_eq!(config.get("Test"), Some("1"));

    // act
    config = builder.build().unwrap();

    // assert
    assert_eq!(config.get("Test"), Some("2"));
}

#[test]
fn reload_token_should_indicate_change_after_provider_change() {
    // arrange
    let trigger = Arc::new(Trigger::default());
    let data = Arc::<AtomicU8>::default();
    let mut builder = config::builder();

    builder.add(ReloadableProvider::new(trigger.clone()));

    let config = builder.build().unwrap();
    let _unused = config.change_token().register(
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
