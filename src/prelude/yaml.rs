use crate::{yaml, Builder, FileSource};

/// Defines `*.yaml` and `*.yml` file extension methods for a [configuration builder](Builder).
pub trait YamlExt: Sized {
    /// Adds a `*.yaml` file as a configuration source.
    ///
    /// # Arguments
    ///
    /// * `file` - The `*.yaml` [file source](FileSource) information
    fn add_yaml_file<T: Into<FileSource>>(self, file: T) -> Self;
}

impl YamlExt for Builder {
    fn add_yaml_file<F: Into<FileSource>>(mut self, file: F) -> Self {
        self.add(yaml::Provider::new(file.into()));
        self
    }
}
