use crate::{mem, Builder};

/// Defines in-memory extension methods for a [configuration builder](Builder).
pub trait MemoryExt: Sized {
    /// Adds an in-memory configuration source using the specified data.
    ///
    /// # Arguments
    ///
    /// * `data` - The in-memory data to add
    fn add_in_memory<S: AsRef<str>>(self, data: &[(S, S)]) -> Self;
}

impl MemoryExt for Builder {
    fn add_in_memory<S: AsRef<str>>(mut self, data: &[(S, S)]) -> Self {
        self.add(mem::Source::new(data));
        self
    }
}
