use eyre::eyre;
use seb::ast::{Biblio, BiblioBuilder, Builder as EntryBuilder, Entry};

use crate::interact::{user_resolve_entry, user_select, user_select_entry};

#[inline]
pub fn take_first_resolvable(bib: Result<Biblio, BiblioBuilder>) -> Result<Entry, EntryBuilder> {
    bib.map(|bib| bib.into_entries().remove(0))
        .or_else(|mut b| b.checked_remove(0).expect("BiblioBuilder was empty!"))
}

pub fn user_select_resolvable(
    bib: Result<Biblio, BiblioBuilder>,
) -> eyre::Result<Result<Entry, EntryBuilder>> {
    match bib {
        Ok(bib) => user_select_entry(bib.into_entries()).map(Ok),
        Err(builder) => select_from_builder(builder),
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

#[inline]
pub fn resolve_entry_builder(entry_builder: EntryBuilder) -> eyre::Result<Entry> {
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
