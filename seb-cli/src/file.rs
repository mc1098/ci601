use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
};

use seb::{
    format::{Format, Reader, Writer},
    Error, ErrorKind,
};

use eyre::{eyre, Context};
use glob::glob;
use log::{info, trace};

#[allow(clippy::module_name_repetitions)]
pub struct FormatFile<F: Format> {
    file: File,
    _format: PhantomData<F>,
}

impl<F: Format> FormatFile<F> {
    fn new(file: File) -> Self {
        Self {
            file,
            _format: PhantomData,
        }
    }
}

impl<F: Format> Reader for FormatFile<F> {
    type Format = F;

    fn read(&mut self) -> Result<Self::Format, Error> {
        read_file_to_string(&mut self.file)
            .map(F::new)
            .map_err(|e| Error::wrap(ErrorKind::IO, e))
    }
}

impl<F: Format> Writer for FormatFile<F> {
    type Format = F;

    fn write(&mut self, format: F) -> Result<(), Error> {
        fn overrwrite_file_from_start(file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
            // Rewind the cursor back to the start of the file to write over the contents and set
            // the length of the file to be equal to bytes so that existing data is removed
            file.rewind()?;
            file.set_len(bytes.len() as u64)?;
            file.write_all(bytes)
        }

        let bytes = format.raw().into_bytes();
        overrwrite_file_from_start(&mut self.file, &bytes)
            .map_err(|e| Error::wrap(ErrorKind::IO, e))
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn open_or_create_format_file<F: Format>(
    file_name: Option<PathBuf>,
) -> eyre::Result<FormatFile<F>> {
    if let Some(path) = file_name {
        trace!("opening {} file as a {} file", path.display(), F::name());
        open_file_by_name(&path)
    } else {
        trace!("Searching current directory for any {} files", F::name());
        if let Ok(file) = find_format_file_in_current_directory() {
            Ok(file)
        } else {
            let path = PathBuf::from("bibliography").with_extension(F::ext());
            info!(
                "No .{} file found in current directory - creating the new file `{}`",
                F::ext(),
                path.display()
            );
            create_file_by_name(&path)
        }
    }
}

#[inline]
fn open_file_for_read_and_write<F: Format>(path: &Path) -> eyre::Result<FormatFile<F>> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map(FormatFile::<F>::new)
        .wrap_err_with(|| {
            format!(
                "Failed to open the '{}' file for reading and appending.",
                path.display()
            )
        })
}

#[inline]
fn create_file_for_read_and_write<F: Format>(path: &Path) -> eyre::Result<FormatFile<F>> {
    OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(path)
        .map(FormatFile::<F>::new)
        .wrap_err_with(|| {
            format!(
                "Failed to open the '{}' file for reading and appending.",
                path.display()
            )
        })
}

fn open_file_by_name<F>(path: &Path) -> eyre::Result<FormatFile<F>>
where
    F: Format,
{
    let path_buf = path.with_extension(F::ext());
    open_file_for_read_and_write(path_buf.as_path())
}

fn create_file_by_name<F>(path: &Path) -> eyre::Result<FormatFile<F>>
where
    F: Format,
{
    let path_buf = path.with_extension(F::ext());
    create_file_for_read_and_write(path_buf.as_path())
}

#[inline]
fn find_format_file_in_current_directory<F: Format>() -> eyre::Result<FormatFile<F>> {
    let path = Path::new(".").with_extension(F::ext());
    find_format_file_in_directory(path)
}

fn find_format_file_in_directory<F, P>(dir: P) -> eyre::Result<FormatFile<F>>
where
    F: Format,
    P: AsRef<Path>,
{
    let path = dir.as_ref();
    if !path.is_dir() {
        return Err(eyre!("Path entered is not a directory"));
    }

    let pattern = format!("{}/*.{}", path.to_string_lossy(), F::ext());

    let mut iter = glob(&pattern).expect("File pattern should always be valid");

    let path_buf = iter
        .next()
        .ok_or_else(|| {
            eyre!(
                "No .{} file found in the '{}' directory",
                F::ext(),
                path.display()
            )
        })?
        .wrap_err("Cannot determine a file path - Do you have the correct permissions?")?;

    if iter.next().is_some() {
        return Err(eyre!(
            "More than one .{} file found - use the --file option to select one",
            F::ext()
        ));
    }

    open_file_for_read_and_write(path_buf.as_path())
}

fn read_file_to_string(file: &mut File) -> eyre::Result<String> {
    let mut content = String::new();
    file.read_to_string(&mut content)
        .wrap_err("Cannot read contents of file")
        .map(move |_| content)
}

#[cfg(test)]
mod tests {

    use super::*;
    use seb::format::BibTex;

    use assert_fs::{
        fixture::{FileTouch, PathChild},
        NamedTempFile, TempDir,
    };

    #[test]
    #[should_panic(
        expected = "Failed to open the 'file does not exist.bib' file for reading and appending"
    )]
    fn err_when_trying_to_open_bib_file_that_does_not_exist() {
        open_file_by_name::<BibTex>(&PathBuf::from("file does not exist")).unwrap();
    }

    fn create_temp_file(name: &str) -> NamedTempFile {
        // create temp file locally
        let file = NamedTempFile::new(name).expect("Cannot create temp file for test");
        // touch the temp file so it can be discovered by code
        file.touch().expect("Failure on touch of new temp file");
        file
    }

    #[test]
    fn open_temp_bib_file_with_ext() {
        let file = create_temp_file("temp.bib");
        let path = NamedTempFile::path(&file);
        let res = open_file_by_name::<BibTex>(path);
        file.close().unwrap();

        assert!(res.is_ok());
    }

    #[test]
    fn open_temp_bib_file_by_name_without_ext() {
        let file = create_temp_file("temp.bib");
        // remove ext from temp path
        let path = NamedTempFile::path(&file).with_extension("");
        let res = open_file_by_name::<BibTex>(&path);
        file.close().unwrap();

        assert!(res.is_ok());
    }

    #[test]
    #[should_panic(expected = "No .bib file found")]
    fn no_files_in_directory() {
        let dir = TempDir::new().expect("Cannot create temp directory for test");

        find_format_file_in_directory::<BibTex, _>(TempDir::path(&dir)).unwrap();
    }

    #[test]
    #[should_panic(expected = "not a directory")]
    fn path_is_not_a_directory() {
        find_format_file_in_directory::<BibTex, _>("not a directory").unwrap();
    }

    #[test]
    #[should_panic(expected = "More than one .bib file")]
    fn multiple_bib_files_in_directory() {
        let dir = TempDir::new().expect("Cannot create temp directory for test");
        dir.child("one.bib").touch().unwrap();
        dir.child("two.bib").touch().unwrap();

        find_format_file_in_directory::<BibTex, _>(TempDir::path(&dir)).unwrap();
    }

    #[test]
    fn read_bib_file_as_bibliograph() {
        // bibtex1 only contains a single bibtex entry so only check equality for one entry
        let bibtex = include_str!("../../seb-lib/tests/data/bibtex1.bib");
        let expected = BibTex::new(bibtex.to_owned())
            .parse()
            .unwrap()
            .expect("bibtex1 content is a valid bibtex entry")
            .into_entries()
            .pop()
            .unwrap();

        dbg!(&bibtex);

        let file = std::fs::File::open("../seb-lib/tests/data/bibtex1.bib")
            .expect("Cannot open ../seb-lib/tests/data/bibtex1.bib file for test");

        let mut file: FormatFile<BibTex> = FormatFile::new(file);

        let biblio = file.read_ast().unwrap().unwrap();
        let res = biblio.entries().next().unwrap();

        assert_eq!(&expected, res);
    }
}
