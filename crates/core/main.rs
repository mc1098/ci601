#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::perf,
    clippy::style,
    clippy::missing_safety_doc,
    clippy::missing_const_for_fn
)]
#![allow(clippy::as_conversions, clippy::mod_module_files)]

use std::{error, process};

mod file;
use file::{find_format_file_in_current_directory, open_file_by_name};

use seb as lib;

use lib::format::{BibTex, Reader};

use clap::{AppSettings, Parser, Subcommand};
use log::{error, trace};

fn main() {
    if let Err(err) = try_main() {
        error!("{:#}", err);
        process::exit(2);
    }
}

fn try_main() -> Result<(), Box<dyn error::Error>> {
    let cli = Cli::parse();

    // if quiet then ignore verbosity but still show errors
    let verbosity = if cli.quiet {
        1
    } else {
        cli.verbosity as usize + 1
    };

    stderrlog::new().verbosity(verbosity).init()?;

    let mut file = if let Some(file_name) = cli.file {
        trace!("'file' option used with value of '{}'", file_name);
        open_file_by_name::<BibTex, _>(file_name)?
    } else {
        trace!("'file' option not used - try and find any .bib files in current directory");
        find_format_file_in_current_directory::<BibTex>()?
    };

    let biblio = file.read_ast()?;

    match &cli.command {
        Commands::Doi { doi } => lib::add_by_doi(doi, &mut file, biblio)?,
        Commands::Isbn { isbn } => lib::add_by_isbn(isbn, &mut file, biblio)?,
    }

    Ok(())
}

#[derive(Parser)]
#[clap(name = "seb")]
#[clap(about = "Seek and edit bibliographic entries to a supported format file in the terminal")]
#[clap(version, author)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// The name of the file
    #[clap(short, long)]
    file: Option<String>,

    /// How chatty the program is when performing commands
    ///
    /// The number of times this flag is used will increase how chatty
    /// the program is.
    #[clap(short, long, parse(from_occurrences))]
    verbosity: u8,

    /// Prevents the program from writing to stdout, errors will still be printed to stderr.
    #[clap(short, long)]
    quiet: bool,
}

#[derive(Subcommand)]
#[non_exhaustive]
enum Commands {
    /// Search for reference by doi
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Doi {
        /// The doi to search for
        doi: String,
    },
    /// Search for reference by ISBN
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Isbn {
        /// The ISBN to search for
        isbn: String,
    },
}
