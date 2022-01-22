use dialoguer::Input;
use eyre::{eyre, Context, Result};
use seb::ast::{Biblio, BiblioBuilder, Builder, Entry, FieldQuery, QuotedString};

pub fn user_select<S: ToString>(prompt: &str, items: &[S]) -> Result<usize> {
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(prompt)
        .default(0)
        .items(items)
        .interact_opt()
        .wrap_err_with(|| eyre!("User selection cancelled"))?;

    if let Some(index) = selection {
        Ok(index)
    } else {
        Err(eyre!("No selection made - cancelling operation"))
    }
}

pub fn user_select_entry(mut entries: Vec<Entry>) -> Result<Entry> {
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Confirm entry")
        .default(0)
        .items(&entries_titles(&entries))
        .interact_opt()
        .unwrap();

    if let Some(index) = selection {
        Ok(entries.remove(index))
    } else {
        Err(eyre!("No entry confirmed - cancelling operation"))
    }
}

fn entries_titles(entries: &[Entry]) -> Vec<String> {
    entries.iter().map(|e| e.title().to_string()).collect()
}

pub fn user_input(prompt: String) -> Result<String> {
    Input::new()
        .with_prompt(prompt)
        .interact_text()
        .wrap_err_with(|| eyre!("User input cancelled"))
}

pub fn user_resolve_biblio_builder(mut res: Result<Biblio, BiblioBuilder>) -> eyre::Result<Biblio> {
    while let Err(mut builder) = res {
        for entry_builder in builder.unresolved() {
            user_resolve_entry(entry_builder)?;
        }
        res = builder.build();
    }

    // unwrap is safe because of the termination of the while let Err loop above.
    Ok(res.unwrap())
}

pub fn user_resolve_entry(builder: &mut Builder) -> eyre::Result<()> {
    let title = builder
        .get_field("title")
        .map_or_else(|| "No title".to_owned(), |qs| qs.to_string());
    println!("Missing required fields for entry: {title}");

    let fields: Vec<_> = builder.required_fields().cloned().collect();

    for field in fields {
        let input = user_input(format!("Enter value for the {field} field"))?;
        builder.set_field(&field, QuotedString::new(input));
    }
    Ok(())
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
