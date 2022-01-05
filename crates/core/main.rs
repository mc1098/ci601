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

mod app;
mod file;

use clap::{AppSettings, Parser, Subcommand};
use log::{info, trace};
use seb::format::{BibTex, Reader, Writer};

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
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

    let mut file = file::open_or_create_format_file::<BibTex>(cli.file)?;

    let mut biblio = file.read_ast()?;

    let entry = match &cli.command {
        Commands::Doi { doi } => {
            trace!("doi subcommand called with value of '{}'", doi);
            info!("Checking current bibliography for possible duplicate doi..");
            app::check_entry_field_duplication(&biblio, "doi", doi)?;
            info!("No duplicate found!");
            app::select_entry_by_doi(doi)?
        }
        Commands::Isbn { isbn } => {
            trace!("isbn subcommand called with value of '{}'", isbn);
            info!("Checking current bibliography for possible duplicate isbn..");
            app::check_entry_field_duplication(&biblio, "isbn", isbn)?;
            info!("No duplicate found!");
            app::select_entry_by_isbn(isbn)?
        }
    };

    biblio.insert(entry);

    info!("Adding selected entry into bibliography");
    file.write_ast(biblio)?;
    info!("Done!");
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