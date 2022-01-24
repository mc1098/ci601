use eyre::eyre;
use log::info;
use seb::ast::{Biblio, BiblioBuilder, Builder as EntryBuilder, Entry};

use crate::interact::{user_resolve_entry, user_select, user_select_entry};

pub fn select_entry(
    bib: Result<Biblio, BiblioBuilder>,
    cite: Option<String>,
    confirm: bool,
) -> eyre::Result<Option<Entry>> {
    let mut entry = match (confirm, bib) {
        (_, Ok(bib)) => {
            if let Some(entry) = select_from_resolved(bib, confirm)? {
                entry
            } else {
                return Ok(None);
            }
        },
        (true, Err(_)) => return Err(eyre!("Some entries found do not have the required fields and with the --confirm flag set cannot be resolved by the user")),
        (_, Err(builder)) => select_and_resolve_builder(builder)?,
    };

    if let Some(cite) = cite {
        info!("Overriding cite key value with '{cite}'");
        entry.set_cite(cite);
    }

    Ok(Some(entry))
}

fn select_from_resolved(bib: Biblio, confirm: bool) -> eyre::Result<Option<Entry>> {
    let mut entries = bib.into_entries();
    if confirm {
        if entries.is_empty() {
            Ok(None)
        } else {
            info!("--confirm used - picking the first entry found..");
            Ok(Some(entries.remove(0)))
        }
    } else {
        user_select_entry(entries).map(Some)
    }
}

fn select_from_builder(mut builder: BiblioBuilder) -> eyre::Result<Result<Entry, EntryBuilder>> {
    let items = builder
        .map_iter_all(|fq| {
            fq.get_field("title")
                .map_or_else(|| "No title".to_owned(), |qs| qs.to_string())
        })
        .collect::<Vec<_>>();

    let selection = user_select("Choose an entry", &items)?;
    builder.checked_remove(selection).ok_or_else(|| {
        eyre!("Internal error: user selection should be valid and not cause an out of index error")
    })
}

fn select_and_resolve_builder(builder: BiblioBuilder) -> eyre::Result<Entry> {
    #[inline]
    fn resolve_entry_builder(entry_builder: EntryBuilder) -> eyre::Result<Entry> {
        let mut res = Err(entry_builder);
        loop {
            match res {
                Ok(entry) => return Ok(entry),
                Err(mut entry_builder) => {
                    user_resolve_entry(&mut entry_builder)?;
                    res = entry_builder.build();
                }
            }
        }
    }

    select_from_builder(builder)?.or_else(resolve_entry_builder)
}

pub fn check_entry_field_duplication(bib: &Biblio, name: &str, value: &str) -> eyre::Result<()> {
    if bib.contains_field(name, |f| &**f == value) {
        Err(eyre!(
            "An entry already exists with a {} field with the value of '{}'.",
            name,
            value
        ))
    } else {
        Ok(())
    }
}

#[test]
fn field_dup_macro() {
    use seb::ast::Entry;

    use seb::ast::{Manual, QuotedString};
    use std::collections::HashMap;

    let mut bib = Biblio::new(vec![]);
    let name = "doi";
    let doi = QuotedString::new("test".to_owned());

    assert!(check_entry_field_duplication(&bib, name, &doi).is_ok());

    let data = Manual {
        cite: String::new(),
        title: QuotedString::new("test".to_owned()),
        optional: HashMap::from([(name.to_owned(), doi.clone())]),
    };

    bib.insert(Entry::Manual(data));

    assert!(check_entry_field_duplication(&bib, name, &doi).is_err());
}
