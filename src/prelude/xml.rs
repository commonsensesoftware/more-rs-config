use crate::{xml, Builder, FileSource};

/// Defines `*.xml` file extension methods for a [configuration builder](Builder).
pub trait XmlExt: Sized {
    /// Adds a `*.xml` file as a configuration source.
    ///
    /// # Arguments
    ///
    /// * `file` - The `*.xml` [file source](FileSource) information
    fn add_xml_file<T: Into<FileSource>>(self, file: T) -> Self;
}

impl XmlExt for Builder {
    fn add_xml_file<F: Into<FileSource>>(mut self, file: F) -> Self {
        self.add(xml::Provider::new(file.into()));
        self
    }
}
