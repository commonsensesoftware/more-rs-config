use std::{env::var, path::PathBuf};

pub(crate) fn new_temp_path(filename: &str) -> PathBuf {
    let temp = var("TEMP")
        .or(var("TMP"))
        .or(var("TMPDIR"))
        .unwrap_or("/tmp".into());
    PathBuf::new().join(temp).join(filename)
}
