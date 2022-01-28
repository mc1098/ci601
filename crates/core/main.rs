#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::perf,
    clippy::style,
    clippy::missing_safety_doc,
    clippy::missing_const_for_fn
)]
#![allow(clippy::as_conversions, clippy::mod_module_files)]

use std::{error, path::PathBuf, process};

mod app;
mod commands;
mod file;
mod interact;

use commands::Commands;
use interact::user_resolve_biblio_resolver;

use seb::format::{BibTex, Reader, Writer};

use clap::{Args, Parser};
use log::trace;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        process::exit(2);
    }
}

fn try_main() -> Result<(), Box<dyn error::Error>> {
    let Cli {
        command,
        global_opts:
            GlobalOpts {
                file,
                interact,
                verbosity,
                quiet,
            },
    } = Cli::parse();

    setup_errlog(verbosity as usize, quiet)?;

    // `quiet` and `interact` cannot be set at the same time
    let interact = quiet ^ interact;

    if interact {
        trace!("Interact mode enabled");
    }

    let mut file = file::open_or_create_format_file::<BibTex>(file)?;
    let biblio = file.read_ast()?;

    let mut biblio = match biblio {
        Err(resolver) if interact => user_resolve_biblio_resolver(resolver)?,
        res => res?,
    };

    let command_res = command.execute(&mut biblio, interact);

    if biblio.dirty() {
        trace!("Updating the bibliography file..");
        file.write_ast(biblio)?;
        trace!("Done!");
    }

    let message = command_res?;
    println!("{message}");
    Ok(())
}

fn setup_errlog(verbosity: usize, quiet: bool) -> Result<(), Box<dyn error::Error>> {
    // if quiet then ignore verbosity but still show errors
    let verbosity = if quiet {
        dbg!("quiet flag used but dbg! and error will still be shown");
        1
    } else {
        verbosity + 2
    };

    stderrlog::new().verbosity(verbosity).init()?;
    Ok(())
}

#[derive(Parser)]
#[clap(name = "seb")]
#[clap(about = "Search and edit bibliographic entries to a supported format file in the terminal")]
#[clap(version, author)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[clap(flatten)]
    global_opts: GlobalOpts,
}

#[derive(Debug, Args)]
struct GlobalOpts {
    /// The name of the file
    #[clap(short, long, parse(from_os_str), global = true)]
    file: Option<PathBuf>,

    /// Enables interactive mode, which allows for dynamically resolving invalid entries.
    #[clap(short, long, global = true)]
    interact: bool,

    /// How chatty the program is when performing commands
    ///
    /// The number of times this flag is used will increase how chatty
    /// the program is.
    #[clap(short, long, parse(from_occurrences), global = true)]
    verbosity: u8,

    /// Prevents the program from writing to stdout, errors will still be printed to stderr.
    #[clap(short, long, global = true)]
    quiet: bool,
}
