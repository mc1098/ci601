use std::{error, process};

mod file;
use crate::file::*;
use bibadd::format::{BibTex, FormatReader};

use clap::{crate_version, App, Arg};
use log::{error, trace};

fn main() {
    stderrlog::new().verbosity(3).init().unwrap();
    if let Err(err) = try_main() {
        error!("{}", err);
        process::exit(2);
    }
}

fn try_main() -> Result<(), Box<dyn error::Error>> {
    let matches = App::new("bibadd")
        .author("mc1098")
        .about("Search and add references easily to .bib files easily in the terminal")
        .version(crate_version!())
        .arg(
            Arg::with_name("search")
                .help("Value used for searching")
                .required(true),
        )
        .arg(
            Arg::with_name("bib")
                .help("The name of the .bib file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("doi")
                .help("Sets the search term as a doi")
                .long("doi"),
        )
        .get_matches();

    // open .bib
    let mut bib_file = if let Some(file_name) = matches.value_of("bib") {
        trace!("'bib' option used with value of '{}'", file_name);
        open_file_by_name::<BibTex, _>(file_name)?
    } else {
        // find the .bib on our own
        trace!("'bib' option not used - try and find any .bib files in current directory");
        find_format_file_in_current_directory::<BibTex>()?
    };

    let biblio = bib_file.read_ast()?;

    let search = matches.value_of("search").unwrap();

    if matches.is_present("doi") {
        bibadd::add_by_doi(search, &mut bib_file, biblio)?;
    } else if matches.is_present("isbn") {
        bibadd::add_by_isbn(search, &mut bib_file, biblio)?;
    } else {
        unimplemented!();
    }

    Ok(())
}
