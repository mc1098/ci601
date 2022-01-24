use dialoguer::Input;
use eyre::{eyre, Context, Result};
use seb::ast::{Biblio, BiblioBuilder, Builder as EntryBuilder, Entry, FieldQuery, QuotedString};

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
    let items = entries_titles(&entries);
    user_select("Confirm entry", &items).map(|i| entries.remove(i))
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

pub fn user_resolve_biblio_builder(builder: BiblioBuilder) -> eyre::Result<Biblio> {
    let mut res = Err(builder);

    // BiblioBuilder::unresolved only returns unresolved entry builders and when this returns no
    // builders than BiblioBuilder::build is guaranteed to be successful. This loop relies on this
    // behaviour in order to prevent an infinite loop occurring.
    //
    // A single iteration of this loop should resolve the BiblioBuilder and the second iteration
    // will match the `Ok` arm and return the resolved Biblio.
    loop {
        match res {
            Ok(bib) => return Ok(bib),
            Err(mut builder) => {
                for entry_builder in builder.unresolved() {
                    user_resolve_entry(entry_builder)?;
                }
                res = builder.build();
            }
        }
    }
}

pub fn user_resolve_entry(builder: &mut EntryBuilder) -> eyre::Result<()> {
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