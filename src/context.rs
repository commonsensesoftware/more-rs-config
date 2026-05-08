use std::cell::RefCell;
use std::cmp::min;
use std::fmt::{Formatter, Result};
use std::mem::{forget, take};
use tracing::warn;

thread_local!(static ID: RefCell<(u8, Vec::<String>)> = const { RefCell::new((0, Vec::new())) });

/// Represents a configuration scope.
pub struct Scope;

impl From<Scope> for Vec<String> {
    #[inline]
    fn from(scope: Scope) -> Self {
        let names = exit();
        forget(scope);
        names
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        let _ = exit();
    }
}

/// Gets the current configuration provider identifier.
pub fn id() -> u8 {
    ID.with(|id| id.borrow().0)
}

/// Advances to the next configuration provider.
pub fn next() {
    ID.with(|id| (id.borrow_mut()).0 += 1);
}

/// Enters a new configuration context.
///
/// # Arguments
///
/// * `names` - The names of the providers in the new context
pub fn enter(names: Vec<String>) -> Scope {
    if names.len() > u8::BITS as usize {
        warn!(
            "{} providers exceeds the limit of {}; some provider diagnostics may not output",
            names.len(),
            u8::BITS
        );
    }

    ID.with(|id| *id.borrow_mut() = (1, names));
    Scope
}

fn exit() -> Vec<String> {
    ID.with(|id| take(&mut *id.borrow_mut()).1)
}

/// Traces a diagnostic message for an overridden value.
///
/// # Arguments
///
/// * `providers` - A bitmap of providers that have contributed to the override
/// * `key` - The configuration key that has been overridden
/// * `old` - The old value of the configuration key
/// * `new` - The new value of the configuration key
pub fn overridden(providers: u8, key: &str, old: &str, new: &str) {
    ID.with(|ctx| {
        let (id, names) = &*ctx.borrow();
        crate::overridden(*id, names, providers, key, old, new)
    });
}

/// Expands the ordered set of providers from the supplied bitmap into the provided formatter.
///
/// # Arguments
///
/// * `providers` - A bitmap of set providers to expand
/// * `names` - The names of the providers
/// * `f` - The formatter to expand the names into
pub fn expand(providers: u8, names: &[String], f: &mut Formatter<'_>) -> Result {
    let mut len = providers.count_ones() as usize;

    if len == 0 {
        Ok(())
    } else {
        let mut skip = 0;

        if let Some(keep) = f.width() {
            if keep == 0 {
                return Ok(());
            }

            let constrained = min(len, keep);

            skip = len - constrained;
            len = constrained;
        }

        let mut count = 0;
        let mut i = 0;

        while count < len && count < names.len() {
            while providers & (1u8 << i) == 0 {
                i += 1;
            }

            if skip > 0 {
                skip -= 1;
                i += 1;
                continue;
            }

            if count > 0 {
                f.write_str(" → ")?;
            }

            f.write_str(&names[i as usize])?;
            count += 1;
            i += 1;
        }

        Ok(())
    }
}
