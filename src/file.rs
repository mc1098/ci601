use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::Path,
};

use bibadd::parse::{Format, Parser};
use biblatex::Bibliography;
use eyre::{eyre, Context, Result};
use glob::glob;

#[inline]
fn open_file_for_read_and_append(path: &Path) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .append(true)
        .open(path)
        .wrap_err_with(|| {
            format!(
                "Failed to open the '{}' file for reading and appending.",
                path.display()
            )
        })
}

pub fn open_bib_file_by_name<P: AsRef<Path>>(file_name: P) -> Result<File> {
    let path = file_name.as_ref();
    let path_buf = path.with_extension("bib");
    open_file_for_read_and_append(path_buf.as_path())
}

#[inline]
pub fn find_bib_file_in_current_directory() -> Result<File> {
    find_bib_file_in_directory(Path::new("."))
}

fn find_bib_file_in_directory<P: AsRef<Path>>(dir: P) -> Result<File> {
    let path = dir.as_ref();
    if !path.is_dir() {
        return Err(eyre!("Path entered is not a directory"));
    }

    let pattern = format!("{}/*.bib", path.to_string_lossy());

    let mut iter = glob(&pattern).expect("File pattern should always be valid");

    let path_buf = iter
        .next()
        .ok_or_else(|| eyre!("No .bib file found"))?
        .wrap_err(
            "Cannot determine a .bib file path - bibadd might not have the correct permissions",
        )?;

    if iter.next().is_some() {
        return Err(eyre!(
            "More than one .bib file found - use the '--bib option to select one"
        ));
    }

    open_file_for_read_and_append(path_buf.as_path())
}

fn read_file_to_string(file: &mut File) -> Result<String> {
    let mut content = String::new();
    file.read_to_string(&mut content)
        .wrap_err("Cannot read contents of file")
        .map(move |_| content)
}

pub fn deserialize_file<P, F: Format>(file: &mut File, parser: P) -> Result<Bibliography>
where
    P: Parser<F>,
{
    let contents = read_file_to_string(file)?;
    parser.parse(F::new(contents))
}

//pub fn read_bib_file(file: &mut File) -> Result<Bibliography> {
//    let bib_content = read_file_to_string(file)?;
//
//    Bibliography::parse(&bib_content).ok_or_else(|| eyre!("Cannot parse contents of .bib file"))
//}

#[cfg(test)]
mod tests {

    use super::*;

    use assert_fs::{
        fixture::{FileTouch, PathChild},
        NamedTempFile, TempDir,
    };
    use bibadd::parse::BibTex;

    #[test]
    #[should_panic(
        expected = "Failed to open the 'file does not exist.bib' file for reading and appending"
    )]
    #[should_panic(expected = "Not Found")]
    fn err_when_trying_to_open_bib_file_that_does_not_exist() {
        open_bib_file_by_name("file does not exist").unwrap();
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
        let path = NamedTempFile::path(&file).to_string_lossy();
        let res = open_bib_file_by_name(path.as_ref());
        file.close().unwrap();

        assert!(res.is_ok())
    }

    #[test]
    fn open_temp_bib_file_by_name_without_ext() {
        let file = create_temp_file("temp.bib");
        // remove ext from temp path
        let path = NamedTempFile::path(&file).with_extension("");
        let name = path.to_string_lossy();
        let res = open_bib_file_by_name(name.as_ref());
        file.close().unwrap();

        assert!(res.is_ok());
    }

    #[test]
    #[should_panic(expected = "No .bib file found")]
    fn no_bib_files_in_directory() {
        let dir = TempDir::new().expect("Cannot create temp directory for test");

        find_bib_file_in_directory(TempDir::path(&dir)).unwrap();
    }

    #[test]
    #[should_panic(expected = "not a directory")]
    fn path_is_not_a_directory() {
        find_bib_file_in_directory("not a directory").unwrap();
    }

    #[test]
    #[should_panic(expected = "More than one .bib file")]
    fn multiple_bib_files_in_directory() {
        let dir = TempDir::new().expect("Cannot create temp directory for test");
        dir.child("one.bib").touch().unwrap();
        dir.child("two.bib").touch().unwrap();

        find_bib_file_in_directory(TempDir::path(&dir)).unwrap();
    }

    #[test]
    fn read_bib_file_as_bibliograph() {
        // bibtex1 only contains a single bibtex entry so only check equality for one entry
        let bibtex = include_str!("../tests/data/bibtex1.bib");
        let biblio = Bibliography::parse(bibtex).expect("bibtex1 content is a valid bibtex entry");
        let expected = biblio.iter().next().unwrap();

        let mut file = std::fs::File::open("./tests/data/bibtex1.bib")
            .expect("Cannot open ./tests/data/bibtex1.bib file for test");

        let biblio = deserialize_file(&mut file, BibTex::parse).unwrap();
        let res = biblio.iter().next().unwrap();

        assert_eq!(expected, res);
    }
}
