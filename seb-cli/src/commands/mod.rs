mod add;

use crate::interact;
use add::AddCommands;

use seb::ast::Biblio;

use clap::Subcommand;

#[derive(Subcommand)]
#[non_exhaustive]
pub enum Commands {
    /// Add an entry to the current bibliography file
    #[clap(arg_required_else_help = true)]
    Add {
        #[clap(subcommand)]
        command: AddCommands,
    },

    /// Check the local bibliography file that all the required fields are present for each entry
    /// type.
    ///
    /// This type of check is done before the `add`, `new`, `rm` commands but can be done
    /// explicitly using this command.
    ///
    /// Entries with missing fields can be resolved interactively only when the `interact` flag is
    /// set using `-i` or `--interact`.
    Check,

    /// Derive a new entry from an existing one.
    ///
    /// This can most commonly be used to make a new entry like a "book chapter" entry based on
    /// an existing "book" entry.
    Derive {
        /// The citation key of the entry to derive from.
        entry: String,

        /// The kind of entry to add.
        ///
        /// The following are known entry types:
        ///
        /// - article
        /// - book
        /// - booklet
        /// - book chapter
        /// - book pages
        /// - book section
        /// - in proceedings
        /// - manual
        /// - master thesis
        /// - phd thesis
        /// - proceedings
        /// - tech report
        /// - unpublished
        ///
        /// Known entry types will require certain fields to be valid
        /// and if another kind entry is found then this will be a custom
        /// entry that only requires a title.
        #[clap(parse(from_str), verbatim_doc_comment)]
        kind: seb::ast::EntryKind<'static>,

        /// Cite to use for new entry
        cite: String,

        /// Require addition fields ontop of what is already required by the kind of entry.
        ///
        /// Any fields already required by the kind are ignored so can be added if essential
        /// when unsure if the kind already requires that field.
        #[clap(long, multiple_values(true))]
        fields: Option<Vec<String>>,
    },

    /// Add a new entry manually
    ///
    /// This subcommand will assume interact flag is set even if not explicitly used.
    #[clap(arg_required_else_help = true)]
    New {
        /// The kind of entry to add.
        ///
        /// The following are known entry types:
        ///
        /// - article
        /// - book
        /// - booklet
        /// - book chapter
        /// - book pages
        /// - book section
        /// - in proceedings
        /// - manual
        /// - master thesis
        /// - phd thesis
        /// - proceedings
        /// - tech report
        /// - unpublished
        ///
        /// Known entry types will require certain fields to be valid
        /// and if another kind entry is found then this will be a custom
        /// entry that only requires a title.
        #[clap(parse(from_str), verbatim_doc_comment)]
        kind: seb::ast::EntryKind<'static>,

        /// Cite to use for new entry
        #[clap(long)]
        cite: Option<String>,

        /// Require addition fields ontop of what is already required by the kind of entry.
        ///
        /// Any fields already required by the kind are ignored so can be added if essential
        /// when unsure if the kind already requires that field.
        #[clap(long, multiple_values(true))]
        fields: Option<Vec<String>>,
    },
    /// Remove an entry from the bibliography file using the cite key
    #[clap(arg_required_else_help = true)]
    Rm {
        /// The cite key of the entry to remove
        cite: String,
    },
}

impl Commands {
    pub fn execute(
        self,
        biblio: &mut Biblio,
        interact: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Commands::Add { command } => command.execute(biblio, interact),
            // trivially if the biblio is already resolved at this point then it was either
            // resolved interactively or was valid so a success message can be returned.
            Commands::Check => Ok("All entries contain the required fields!".to_owned()),
            Commands::Derive {
                entry,
                kind,
                cite,
                fields,
            } => {
                let mut resolver = seb::ast::Entry::resolver_with_cite(kind, cite);

                if let Some(entry) = biblio.get(&entry) {
                    resolver.set_fields_from_entry(entry);
                } else {
                    return Err(eyre::eyre!("No entry found with the cite key of '{entry}'").into());
                }

                if let Some(fields) = fields {
                    resolver.add_required_fields(fields);
                }

                interact::user_resolve_entry(&mut resolver)?;
                let derived_entry = resolver.resolve()?;
                let cite = derived_entry.cite().to_owned();
                biblio.insert(derived_entry);

                log::info!(
                    "Entry with cite '{cite}' derived from '{entry}' and added to bibliography"
                );
                Ok(cite)
            }
            Commands::New { kind, cite, fields } => {
                let mut resolver = if let Some(cite) = cite {
                    seb::ast::Entry::resolver_with_cite(kind, cite)
                } else {
                    seb::ast::Entry::resolver(kind)
                };

                if let Some(fields) = fields {
                    resolver.add_required_fields(fields);
                }

                interact::user_resolve_entry(&mut resolver)?;
                let entry = resolver.resolve()?;
                let cite = entry.cite().to_owned();
                biblio.insert(entry);

                log::info!("Entry with cite '{cite}' added to bibliography");
                Ok(cite)
            }
            Commands::Rm { cite } => {
                log::trace!("Checking current bibliography for entry with this cite key..");
                if biblio.remove(&cite) {
                    Ok("Entry removed from bibliography".to_owned())
                } else {
                    Ok(format!("No entry found with the cite key of '{cite}'"))
                }
            }
        }
    }
}
