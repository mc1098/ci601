use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
};

use seb::format::{Format, Reader, Writer};

use eyre::{eyre, Context, Result};
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

    fn read(&mut self) -> Result<Self::Format> {
        read_file_to_string(&mut self.file).map(F::new)
    }
}

impl<F: Format> Writer for FormatFile<F> {
    type Format = F;

    fn write(&mut self, format: F) -> Result<()> {
        let s = format.raw();
        self.file.rewind()?;
        self.file
            .write_all(s.as_bytes())
            .wrap_err_with(|| eyre!("Cannot write format to file"))
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn open_or_create_format_file<F: Format>(file_name: Option<PathBuf>) -> Result<FormatFile<F>> {
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
fn open_file_for_read_and_write<F: Format>(path: &Path) -> Result<FormatFile<F>> {
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
fn create_file_for_read_and_write<F: Format>(path: &Path) -> Result<FormatFile<F>> {
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

fn open_file_by_name<F>(path: &Path) -> Result<FormatFile<F>>
where
    F: Format,
{
    let path_buf = path.with_extension(F::ext());
    open_file_for_read_and_write(path_buf.as_path())
}

fn create_file_by_name<F>(path: &Path) -> Result<FormatFile<F>>
where
    F: Format,
{
    let path_buf = path.with_extension(F::ext());
    create_file_for_read_and_write(path_buf.as_path())
}

#[inline]
fn find_format_file_in_current_directory<F: Format>() -> Result<FormatFile<F>> {
    let path = Path::new(".").with_extension(F::ext());
    find_format_file_in_directory(path)
}

fn find_format_file_in_directory<F, P>(dir: P) -> Result<FormatFile<F>>
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

fn read_file_to_string(file: &mut File) -> Result<String> {
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
    #[should_panic(expected = "Not Found")]
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
        let bibtex = include_str!("../../tests/data/bibtex1.bib");
        let expected = BibTex::new(bibtex.to_owned())
            .parse()
            .expect("bibtex1 content is a valid bibtex entry")
            .into_iter()
            .next()
            .unwrap();

        let file = std::fs::File::open("./tests/data/bibtex1.bib")
            .expect("Cannot open ./tests/data/bibtex1.bib file for test");
        let mut file: FormatFile<BibTex> = FormatFile::new(file);

        let biblio = file.read_ast().unwrap();
        let res = biblio.into_iter().next().unwrap();

        assert_eq!(expected, res);
    }
}
