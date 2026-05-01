// heavily inspired by hyperium http extensions
// REF: https://github.com/hyperium/http/blob/master/src/extensions.rs

use std::any::{Any, TypeId};
use std::collections::{hash_map::Entry::Vacant, HashMap};
use std::fmt;
use std::hash::{BuildHasherDefault, Hasher};

type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<IdHasher>>;

// With TypeIds as keys, there's no need to hash them. They are already hashes themselves, coming from the compiler.
// The IdHasher just holds the u64 of the TypeId, and then returns it, instead of doing any bit fiddling.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// Represents properties that can be shared across configuration sources.
#[derive(Default)]
pub struct Properties(Option<AnyMap>);

impl Properties {
    /// Initializes new [Properties].
    #[inline]
    pub fn new() -> Properties {
        Properties::default()
    }

    /// Inserts or a replaces a property.
    ///
    /// # Arguments
    ///
    /// * `property` - the property to insert
    pub fn insert<T: Send + Sync + 'static>(&mut self, property: T) {
        self.0
            .get_or_insert_with(AnyMap::default)
            .insert(TypeId::of::<T>(), Box::new(property));
    }

    /// Gets a property.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.0
            .as_ref()
            .and_then(|map| map.get(&TypeId::of::<T>()))
            .and_then(|boxed| (**boxed).downcast_ref())
    }

    /// Gets or inserts a default property.
    ///
    /// # Arguments
    ///
    /// * `property` - the default property to insert
    pub fn get_or_insert<T: Send + Sync + 'static>(&mut self, property: T) -> &T {
        self.get_or_insert_with(|| Result::<_, ()>::Ok(property)).unwrap()
    }

    /// Gets or inserts a default property using the specified factory function.
    ///
    /// # Arguments
    ///
    /// * `factory` - the function used to generate the default property
    pub fn get_or_insert_with<T, E, F>(&mut self, factory: F) -> Result<&T, E>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> Result<T, E>,
    {
        let key = TypeId::of::<T>();
        let map = self.0.get_or_insert_with(AnyMap::default);

        if let Vacant(entry) = map.entry(key) {
            entry.insert(Box::new(factory()?));
        }

        Ok((**map.get(&key).unwrap()).downcast_ref().expect("invalid cast"))
    }
}

impl fmt::Debug for Properties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(Properties)).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct MyType(i32);

    #[test]
    fn properties_should_store_primitive() {
        // arrange
        let mut properties = Properties::new();
        let expected = 5i32;

        properties.insert(expected);

        // act
        let actual = properties.get::<i32>();

        // assert
        assert_eq!(actual, Some(&expected));
    }

    #[test]
    fn properties_should_store_struct() {
        // arrange
        let mut properties = Properties::new();
        let expected = MyType(10);

        properties.insert(MyType(10));

        // act
        let actual = properties.get::<MyType>();

        // assert
        assert_eq!(actual, Some(&expected));
    }

    #[test]
    fn properties_should_return_none_for_missing_item() {
        // arrange
        let mut properties = Properties::new();

        properties.insert(5i32);

        // act
        let actual = properties.get::<bool>();

        // assert
        assert!(actual.is_none());
    }
}
