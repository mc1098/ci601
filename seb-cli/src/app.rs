use eyre::eyre;
use log::trace;
use seb::ast::{Biblio, BiblioResolver, EntryExt, Resolver as EntryResolver};

use crate::interact::{user_resolve_entry, user_select, user_select_entry};

#[inline]
pub fn take_first_resolvable(
    bib: Result<Biblio, BiblioResolver>,
) -> Result<Box<dyn EntryExt>, EntryResolver> {
    bib.map(|bib| bib.into_entries().remove(0))
        .or_else(|mut b| b.checked_remove(0).expect("BiblioResolver was empty!"))
}

pub fn user_select_resolvable(
    bib: Result<Biblio, BiblioResolver>,
) -> eyre::Result<Result<Box<dyn EntryExt>, EntryResolver>> {
    match bib {
        Ok(bib) => user_select_entry(bib.into_entries()).map(Ok),
        Err(resolver) => select_from_resolver(resolver),
    }
}

fn select_from_resolver(
    mut resolver: BiblioResolver,
) -> eyre::Result<Result<Box<dyn EntryExt>, EntryResolver>> {
    let items = resolver
        .iter()
        .map(|fq| {
            fq.get_field("title")
                .map_or_else(|| "No title".to_owned(), |qs| qs.to_string())
        })
        .collect::<Vec<_>>();

    let selection = user_select("Choose an entry", &items)?;
    resolver.checked_remove(selection).ok_or_else(|| {
        eyre!("Internal error: user selection should be valid and not cause an out of index error")
    })
}

#[inline]
pub fn resolve_entry_resolver(entry_resolver: EntryResolver) -> eyre::Result<Box<dyn EntryExt>> {
    let mut res = Err(entry_resolver);
    loop {
        match res {
            Ok(entry) => return Ok(entry),
            Err(mut entry_resolver) => {
                user_resolve_entry(&mut entry_resolver)?;
                res = entry_resolver.resolve();
            }
        }
    }
}

pub fn check_entry_field_duplication(bib: &Biblio, name: &str, value: &str) -> eyre::Result<()> {
    trace!("Checking current bibliography for possible duplicate {name} of '{value}'");
    if bib.contains_field(name, |f| &**f == value) {
        Err(eyre!(
            "An entry already exists with a {} field with the value of '{}'.",
            name,
            value
        ))
    } else {
        trace!("No duplicate found!");
        Ok(())
    }
}

#[test]
fn field_dup_macro() {
    use seb::ast::{Manual, QuotedString};
    use std::collections::HashMap;

    let mut bib = Biblio::new(vec![]);
    let name = "doi";
    let doi = QuotedString::new("test".to_owned());

    assert!(check_entry_field_duplication(&bib, name, &doi).is_ok());

    let data = Manual {
        kind: seb::ast::kind::Manual.into(),
        cite: String::new(),
        title: QuotedString::new("test".to_owned()),
        optional: HashMap::from([(name.to_owned(), doi.clone())]),
    };

    bib.insert(Box::new(data));

    assert!(check_entry_field_duplication(&bib, name, &doi).is_err());
}
