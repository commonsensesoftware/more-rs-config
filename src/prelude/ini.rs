use crate::{ini, Builder, FileSource};

/// Defines `*.ini` file extension methods for a [configuration builder](Builder).
pub trait IniExt: Sized {
    /// Adds an `*.ini` file as a configuration source.
    ///
    /// # Arguments
    ///
    /// * `file` - The `*.ini` [file source](FileSource) information
    fn add_ini_file<T: Into<FileSource>>(self, file: T) -> Self;
}

impl IniExt for Builder {
    fn add_ini_file<F: Into<FileSource>>(mut self, file: F) -> Self {
        self.add(ini::Source::new(file.into()));
        self
    }
}
