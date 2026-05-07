use crate::{context, path, Merge};
use indexmap::{
    map::{self, Entry::*},
    IndexMap,
};
use std::{
    borrow::{
        Borrow,
        Cow::{self, *},
    },
    cmp::Ordering,
    fmt::{Debug, Display, Formatter, Result},
    hash::{Hash, Hasher},
    mem::replace,
    str,
};

#[derive(Clone, Eq)]
struct Key<'a>(Cow<'a, str>);

impl PartialEq for Key<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Ord for Key<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut order = self.0.len().cmp(&other.0.len());

        if order != Ordering::Equal {
            return order;
        }

        for (a, b) in self.0.chars().zip(other.0.chars()) {
            order = a.to_ascii_uppercase().cmp(&b.to_ascii_uppercase());

            if order != Ordering::Equal {
                break;
            }
        }

        order
    }
}

impl PartialOrd for Key<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0
            .as_ref()
            .chars()
            .map(|c| c.to_ascii_uppercase())
            .for_each(|c| c.hash(state))
    }
}

impl AsRef<str> for Key<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<String> for Key<'static> {
    #[inline]
    fn from(value: String) -> Self {
        Key(Owned(value))
    }
}

impl<'a> From<&'a str> for Key<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Key(Borrowed(value))
    }
}

/// An unsized wrapper for case-insensitive key lookups in the settings IndexMap.
/// This type has the same Hash/Eq semantics as Key, allowing it to be
/// used as a query type via the Borrow trait.
#[derive(Eq)]
#[repr(transparent)]
struct KeyRef(str);

impl KeyRef {
    #[inline]
    fn new(s: &str) -> &Self {
        // SAFETY: KeyRef is #[repr(transparent)] over str,
        // so &str and &KeyRef have the same layout.
        unsafe { &*(s as *const str as *const KeyRef) }
    }
}

impl PartialEq for KeyRef {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Hash for KeyRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0
            .chars()
            .map(|c| c.to_ascii_uppercase())
            .for_each(|c| c.hash(state))
    }
}

impl Borrow<KeyRef> for Key<'_> {
    #[inline]
    fn borrow(&self) -> &KeyRef {
        KeyRef::new(self.0.as_ref())
    }
}

impl Debug for Key<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Key<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&self.0, f)
    }
}

/// Represents a case-insensitive collection of configuration settings.
#[derive(Clone, Default)]
pub struct Settings(IndexMap<Key<'static>, (String, u8)>);

impl Settings {
    /// Initializes new [Settings].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the number of settings.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Gets a value indicating whether the collection is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Gets an iterator over the keys in the collection in an arbitrary order.
    #[inline]
    pub fn keys(&self) -> Keys<'_> {
        Keys(self.0.keys())
    }

    pub(crate) fn get_subkey(&self, path: &str, key: &str) -> Option<&str> {
        const CH_256: usize = 256;
        let delimiter = path::delimiter();
        let len = path.len() + 1 + key.len();

        if len <= CH_256 {
            let mut buf = [0u8; CH_256];

            buf[..path.len()].copy_from_slice(path.as_bytes());
            buf[path.len()] = delimiter as u8;
            buf[path.len() + 1..len].copy_from_slice(key.as_bytes());

            // SAFETY: path and key are valid UTF-8 str slices, and delimiter
            // is an ASCII character, so the concatenation is valid UTF-8.

            let combined = unsafe { std::str::from_utf8_unchecked(&buf[..len]) };
            self.0.get(KeyRef::new(combined)).map(|(s, _)| s.as_str())
        } else {
            let combined = format!("{path}{delimiter}{key}");
            self.0.get(KeyRef::new(combined.as_str())).map(|(s, _)| s.as_str())
        }
    }

    pub(crate) fn get_with_id(&self, key: &str) -> Option<(&str, u8)> {
        self.0.get(KeyRef::new(key)).map(|(s, i)| (s.as_str(), *i))
    }

    /// Gets the setting with the specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - The case-insensitive key of the setting to retrieve
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(KeyRef::new(key)).map(|(s, _)| s.as_str())
    }

    /// Adds or updates a setting with the specified key and value.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the setting to add or update
    /// * `value` - The value of the setting to add or update
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        let id = context::id();
        let value = value.into();

        match self.0.entry(Key::from(key.into())) {
            Occupied(mut e) => {
                let entry = e.get_mut();
                let old = replace(&mut entry.0, value);

                entry.1 |= id;

                let entry = e.get();

                context::overridden(entry.1, e.key().as_ref(), &old, &entry.0);
                Some(old)
            }
            Vacant(e) => {
                e.insert((value, id));
                None
            }
        }
    }

    /// Shrinks the capacity of the settings as much as possible. It will drop down as much as possible while
    /// maintaining the internal rules and possibly leaving some space in accordance with the resize policy.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }
}

/// Represents a iterator over the key/value pairs in [Settings].
pub struct Iter<'a>(map::Iter<'a, Key<'static>, (String, u8)>);

/// Represents a consuming iterator over the key/value pairs in [Settings].
pub struct IntoIter(map::IntoIter<Key<'static>, (String, u8)>);

/// Represents a iterator over the keys in [Settings].
pub struct Keys<'a>(map::Keys<'a, Key<'static>, (String, u8)>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, (v, _))| (k.as_ref(), v.as_str()))
    }
}

impl Iterator for IntoIter {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, (v, _))| (k.0.into_owned(), v))
    }
}

impl<'a> IntoIterator for &'a Settings {
    type Item = (&'a str, &'a str);
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter(self.0.iter())
    }
}

impl IntoIterator for Settings {
    type Item = (String, String);
    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter())
    }
}

impl<'a> Iterator for Keys<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(AsRef::as_ref)
    }
}

impl Debug for Settings {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Settings {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut pairs = self.0.iter();

        if let Some((key, (value, _))) = pairs.next() {
            write!(f, "{key}: {value}")?;

            for (key, (value, _)) in pairs {
                f.write_str(", ")?;
                write!(f, "{key}: {value}")?;
            }
        }

        Ok(())
    }
}

impl Merge for Settings {
    fn merge(&mut self, other: &Self) {
        // merging from another collection cannot ensure any existing identifier comes from one of the current providers
        // that can be traced so always replace the current identifier
        let id = context::id();

        for (key, (value, _)) in &other.0 {
            self.0.insert(key.clone(), (value.clone(), id));
        }
    }
}
