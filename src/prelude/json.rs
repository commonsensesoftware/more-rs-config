use crate::{json, Builder, FileSource};

/// Defines `*.json` file extension methods for a [configuration builder](Builder).
pub trait JsonExt: Sized {
    /// Adds a `*.json` file as a configuration source.
    ///
    /// # Arguments
    ///
    /// * `file` - The `*.json` [file source](FileSource) information
    fn add_json_file<T: Into<FileSource>>(self, file: T) -> Self;
}

impl JsonExt for Builder {
    fn add_json_file<F: Into<FileSource>>(mut self, file: F) -> Self {
        self.add(json::Provider::new(file.into()));
        self
    }
}
