use std::path::{Path, PathBuf};

/// Represents a file configuration source.
#[derive(Clone, Default)]
pub struct FileSource {
    /// Gets or sets the source file path.
    pub path: PathBuf,

    /// Gets or sets a value indicating whether the file is optional.
    ///
    /// # Remarks
    ///
    /// The default value is false.
    pub optional: bool,

    /// Gets or sets a value indicating whether the file will be loaded if the underlying file changes.
    ///
    /// # Remarks
    ///
    /// The default value is false.
    pub reload_on_change: bool,
}

impl FileSource {
    /// Initializes a new [FileSource].
    ///
    /// # Arguments
    ///
    /// * `path` - The source file path
    /// * `optional` - Indicates whether the source file must exist
    /// * `reload_on_change` - Indicates if a reload should occur if the source file changes
    #[inline]
    pub fn new(path: PathBuf, optional: bool, reload_on_change: bool) -> Self {
        Self {
            path,
            optional,
            reload_on_change,
        }
    }

    /// Initializes a new, optional file configuration source.
    ///
    /// # Arguments
    ///
    /// * `path` - The source file path
    #[inline]
    pub fn optional<P: AsRef<Path>>(path: P) -> Self {
        Self::new(path.as_ref().to_path_buf(), true, false)
    }
}

impl From<PathBuf> for FileSource {
    #[inline]
    fn from(value: PathBuf) -> Self {
        Self::new(value, false, false)
    }
}

impl From<&PathBuf> for FileSource {
    #[inline]
    fn from(value: &PathBuf) -> Self {
        Self::from(value.clone())
    }
}

impl From<&Path> for FileSource {
    #[inline]
    fn from(value: &Path) -> Self {
        Self::from(value.to_path_buf())
    }
}

impl From<&str> for FileSource {
    #[inline]
    fn from(value: &str) -> Self {
        Self::from(PathBuf::from(value))
    }
}

impl From<String> for FileSource {
    #[inline]
    fn from(value: String) -> Self {
        Self::from(PathBuf::from(value))
    }
}

impl From<&String> for FileSource {
    #[inline]
    fn from(value: &String) -> Self {
        Self::from(PathBuf::from(value))
    }
}

/// Represents a builder for a [file source](FileSource).
pub struct FileSourceBuilder {
    path: PathBuf,
    optional: bool,
    reload_on_change: bool,
}

impl FileSourceBuilder {
    /// Initializes a new file source builder.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to build a file source for
    #[inline]
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            optional: false,
            reload_on_change: false,
        }
    }

    /// Indicates the file source is optional.
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Indicates the file source can be reloaded.
    pub fn reloadable(mut self) -> Self {
        self.reload_on_change = true;
        self
    }

    /// Creates and returns a new [file source](FileSource).
    #[inline]
    pub fn build(&self) -> FileSource {
        FileSource::new(
            self.path.clone(),
            self.optional,
            self.reload_on_change,
        )
    }
}

impl From<FileSourceBuilder> for FileSource {
    #[inline]
    fn from(value: FileSourceBuilder) -> Self {
        value.build()
    }
}

impl From<&FileSourceBuilder> for FileSource {
    #[inline]
    fn from(value: &FileSourceBuilder) -> Self {
        value.build()
    }
}
