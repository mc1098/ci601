use std::path::PathBuf;

use seb::{file::FormatFile, format::Format};

#[allow(clippy::module_name_repetitions)]
pub fn open_or_create_format_file<F: Format>(
    file_name: Option<PathBuf>,
) -> Result<FormatFile<F>, Box<dyn std::error::Error>> {
    let file = if let Some(path) = file_name {
        log::trace!("opening {} file as a {} file", path.display(), F::name());
        FormatFile::open(&path).or_else(|_| {
            log::info!(
                "No .{} file found in the current directory - creating the file `{}`",
                F::ext(),
                path.display()
            );
            FormatFile::create(&path)
        })?
    } else {
        log::trace!("Searching current directory for any {} files", F::name());
        FormatFile::find(".")?
    };

    Ok(file)
}
