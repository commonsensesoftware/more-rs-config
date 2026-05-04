use crate::{Configuration, OwnedSection, Ref, Section};
use tokens::{ChangeToken, NeverChangeToken};

/// Defines the behavior of a reloadable configuration.
pub trait Reloadable: Sized {
    /// Gets a value indicating whether the configuration can be reloaded.
    fn can_reload(&self) -> bool;

    /// Gets a [change token](ChangeToken) that will be notified when the configuration is reloaded.
    fn reload_token(&self) -> impl ChangeToken + 'static;
}

macro_rules! unreloadable {
    ($type:ty) => {
        impl Reloadable for $type {
            #[inline]
            fn can_reload(&self) -> bool {
                false
            }

            #[inline]
            fn reload_token(&self) -> impl ChangeToken + 'static {
                NeverChangeToken
            }
        }
    };
}

unreloadable!(Configuration);
unreloadable!(Section<'_>);
unreloadable!(OwnedSection);

impl Reloadable for Ref<Configuration> {
    #[inline]
    fn can_reload(&self) -> bool {
        (&**self).can_reload()
    }

    #[inline]
    fn reload_token(&self) -> impl ChangeToken + 'static {
        (&**self).reload_token()
    }
}
