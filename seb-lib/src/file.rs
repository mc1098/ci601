//! Format supporting Filesystem operations.
//!
//! This module contains basic methods for opening/creating files into a supported format and also
//! provides types to read and write from those format files.

use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    marker::PhantomData,
    path::Path,
};

use crate::{
    format::{Format, Reader, Writer},
    Error, ErrorKind,
};

use glob::glob;

/// A reference to an open file on the filesystem which should have the textual content that
/// matches the generic [`Format`].
///
/// `FormatFile`s are automatically closed when they go out of scope. Errors detected on closing are
/// ignored by the implementation of `Drop`.
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

    /// Attempts to open a format file in read and write mode.
    ///
    /// # Errors
    /// This function will return an error if `path` does not already exist or the user lacks
    /// permissions to open the file.
    ///
    /// # Examples
    /// ```no_run
    /// use seb::{
    ///     file::FormatFile,
    ///     format::BibTex
    /// };
    ///
    /// fn main() -> Result<(), seb::Error> {
    ///     let mut f = FormatFile::<BibTex>::open("foo.bib")?;
    ///     Ok(())
    /// }
    ///
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        let path_buf = path.with_extension(F::ext());
        open_file_for_read_and_write(path_buf.as_path())
    }

    /// Attempts to find a single format file in the directory.
    ///
    /// This function will use the value from the [`Format::ext`] associated function to
    /// find a file with the same extension.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The path is not a directory
    /// - No file can be found in the directory
    /// - User lacks permissions to open the file
    ///
    /// # Examples
    /// ```no_run
    /// use seb::{
    ///     file::FormatFile,
    ///     format::BibTex,
    /// };
    ///
    /// fn main() -> Result<(), seb::Error> {
    ///     let mut f = FormatFile::<BibTex>::find(".")?;
    ///     Ok(())
    /// }
    ///
    /// ```
    pub fn find<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();

        if !path.is_dir() {
            return Err(Error::new(
                ErrorKind::IO,
                format!("{} is not a directory", path.display()),
            ));
        }

        find_format_file_in_directory(path)
    }

    /// Opens a format file in read and write mode.
    ///
    /// This function will create a file if it does not exist, and will truncate it if it does.
    ///
    /// # Errors
    /// This function will return an error if the user lacks permissions to open or create the
    /// file.
    ///
    /// # Examples
    /// ```no_run
    /// use seb::{
    ///     file::FormatFile,
    ///     format::BibTex
    /// };
    ///
    /// fn main() -> Result<(), seb::Error> {
    ///     let f = FormatFile::<BibTex>::create("foo.bib")?;
    ///     Ok(())
    /// }
    ///
    /// ```
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        let path_buf = path.with_extension(F::ext());
        create_file_for_read_and_write(path_buf.as_path())
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

fn read_file_to_string(file: &mut File) -> Result<String, Error> {
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| Error::wrap_with(ErrorKind::IO, e, "Cannot read contents of file"))
        .map(move |bytes| {
            log::trace!("{bytes} read from the file");
            content
        })
}

impl<F: Format> Writer for FormatFile<F> {
    type Format = F;

    fn write(&mut self, format: F) -> Result<(), Error> {
        fn overrwrite_file_from_start(file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
            // Rewind the cursor back to the start of the file to write over the contents and set
            // the length of the file to be equal to bytes so that existing data is removed
            log::trace!("rewind file cursor to start and write bytes: {bytes:?}");
            file.rewind()?;
            file.set_len(bytes.len() as u64)?;
            file.write_all(bytes)
        }

        let bytes = format.raw().into_bytes();
        overrwrite_file_from_start(&mut self.file, &bytes)
            .map_err(|e| Error::wrap(ErrorKind::IO, e))
    }
}

#[inline]
fn open_file_for_read_and_write<F: Format>(path: &Path) -> Result<FormatFile<F>, Error> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map(FormatFile::<F>::new)
        .map_err(|e| {
            Error::wrap_with(
                ErrorKind::IO,
                e,
                format!(
                    "Failed to open the '{}' file for reading and writing",
                    path.display()
                ),
            )
        })
}

#[inline]
fn create_file_for_read_and_write<F: Format>(path: &Path) -> Result<FormatFile<F>, Error> {
    OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(path)
        .map(FormatFile::<F>::new)
        .map_err(|e| {
            Error::wrap_with(
                ErrorKind::IO,
                e,
                format!(
                    "Failed to create and open the '{}' file for reading and writing",
                    path.display()
                ),
            )
        })
}

fn find_format_file_in_directory<F, P>(dir: P) -> Result<FormatFile<F>, Error>
where
    F: Format,
    P: AsRef<Path>,
{
    let path = dir.as_ref();
    if !path.is_dir() {
        return Err(Error::new(ErrorKind::IO, "Path is not a directory"));
    }

    let pattern = format!("{}/*.{}", path.to_string_lossy(), F::ext());

    let mut iter = glob(&pattern).expect("File pattern should always be valid");

    let path_buf = iter
        .next()
        .ok_or_else(|| {
            Error::new(
                ErrorKind::IO,
                format!(
                    "No .{} file found in the '{}' directory",
                    F::ext(),
                    path.display()
                ),
            )
        })?
        .map_err(|e| {
            Error::wrap_with(
                ErrorKind::IO,
                e,
                "Cannot determine a file path - Do you have the correct permissions?",
            )
        })?;

    if iter.next().is_some() {
        return Err(Error::new(
            ErrorKind::IO,
            format!("More than one .{} file found", F::ext()),
        ));
    }

    open_file_for_read_and_write(path_buf.as_path())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::format::BibTex;

    use assert_fs::{
        fixture::{FileTouch, PathChild},
        NamedTempFile, TempDir,
    };

    #[test]
    #[should_panic(
        expected = "Failed to open the 'file does not exist.bib' file for reading and writing"
    )]
    fn err_when_trying_to_open_bib_file_that_does_not_exist() {
        FormatFile::<BibTex>::open("file does not exist").unwrap();
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
        let res = FormatFile::<BibTex>::open(path);
        file.close().unwrap();

        assert!(res.is_ok());
    }

    #[test]
    fn open_temp_bib_file_by_name_without_ext() {
        let file = create_temp_file("temp.bib");
        // remove ext from temp path
        let path = NamedTempFile::path(&file).with_extension("");
        let res = FormatFile::<BibTex>::open(path);
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
