mod add;

use add::AddCommands;

use seb::ast::Biblio;

use clap::{AppSettings, Subcommand};
use log::trace;

#[derive(Subcommand)]
#[non_exhaustive]
pub enum Commands {
    /// Add an entry to the current bibliography file
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Add {
        #[clap(subcommand)]
        command: AddCommands,
    },
    /// Remove an entry from the bibliography file using the cite key
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Rm {
        /// The cite key of the entry to remove
        cite: String,
    },
}

impl Commands {
    pub fn execute(self, biblio: &mut Biblio, interact: bool) -> eyre::Result<String> {
        match self {
            Commands::Add { command } => command.execute(biblio, interact),
            Commands::Rm { cite } => {
                dbg!("rm subcommand called with the value of '{cite}'");
                trace!("Checking current bibliography for entry with this cite key..");
                if biblio.remove(&cite) {
                    Ok("Entry removed from bibliography".to_owned())
                } else {
                    Ok(format!("No entry found with the cite key of '{cite}'"))
                }
            }
        }
    }
}
