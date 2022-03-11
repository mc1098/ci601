use std::{borrow::Cow, collections::HashMap};

use crate::ast::{FieldQuery, QuotedString};

use super::{Entry, EntryKind};

/// A general `Entry` resolver that allows for retrying resolves of entries multiple times at runtime.
///
/// Each entry type, like `Book`, has an associated `resolver` function in order to create the
/// correct resolver for that type.
///
/// # Examples
///
/// ```
/// use seb::ast::{Entry, EntryKind, Resolver, QuotedString};
///
/// let resolver = Entry::resolver_with_cite(EntryKind::Manual, "cite_key");
///
/// // manual only requires the `title` field to be valid
/// assert_eq!(&["title"][..], resolver.required_fields().collect::<Vec<_>>());
///
/// let mut resolver = resolver.resolve().expect_err("The required title field is not set");
/// resolver.set_field("title", "My manual");
///
/// let entry = resolver.resolve().expect("All required fields have now been set so this is valid");
///
/// assert_eq!("cite_key", entry.cite());
/// assert_eq!("My manual", &**entry.title());
/// ```
///
#[derive(Debug)]
#[cfg_attr(test, derive(Clone, PartialEq))]
pub struct Resolver {
    pub(super) target: EntryKind<'static>,
    pub(super) cite: Option<String>,
    pub(super) req: Vec<&'static str>,
    pub(super) fields: HashMap<String, QuotedString>,
    pub(super) entry_resolve: fn(Self) -> Entry,
}

impl Resolver {
    pub(crate) fn new(
        kind: EntryKind<'static>,
        cite: Option<String>,
        entry_resolve: fn(Self) -> Entry,
    ) -> Self {
        let req = kind
            .required_fields()
            .iter()
            .map(std::ops::Deref::deref)
            .collect();

        Self {
            target: kind,
            cite,
            req,
            fields: HashMap::default(),
            entry_resolve,
        }
    }
    /// Returns the cite key for the entry being built.
    ///
    /// The cite key may either be a known value given to the resolver or will be generated using
    /// the `author` and `year` field if available.
    #[must_use]
    pub fn cite(&self) -> Cow<'_, str> {
        if let Some(cite) = &self.cite {
            Cow::Borrowed(cite.as_str())
        } else {
            let author = self.get_field("author").map_or_else(
                || "Unknown".to_owned(),
                |qs| {
                    let mut s = qs.to_string();
                    s.retain(|c| !c.is_whitespace());
                    s
                },
            );

            let year = self
                .get_field("year")
                .map_or_else(|| "year".to_owned(), |qs| qs.to_string());
            Cow::Owned(format!("{author}{year}"))
        }
    }

    /// Build an entry from the fields added in this resolver.
    ///
    /// # Errors
    /// Returns `Err(Self)` when the required fields have not been set to make a valid [`Entry`],
    /// returning `Self` allows for the user to retry.
    pub fn resolve(self) -> Result<Entry, Self> {
        if self.req.is_empty() {
            Ok((self.entry_resolve)(self))
        } else {
            Err(self)
        }
    }

    /// Returns an iterator of the required fields that need to be set in order to make this
    /// resolver succeed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::borrow::Cow;
    /// use seb::ast::{Entry, EntryKind, QuotedString};
    ///
    /// let mut resolver = Entry::resolver_with_cite(EntryKind::Manual, "cite");
    /// assert_eq!(Some("title"), resolver.required_fields().next());
    ///
    /// // set the `title` field then check if the required_fields is returning an empty iter.
    /// resolver.title(QuotedString::new("My manual".to_owned()));
    /// assert_eq!(None, resolver.required_fields().next());
    /// ```
    pub fn required_fields(&self) -> impl Iterator<Item = &str> {
        self.req.iter().copied()
    }

    fn entry<'a>(&'a mut self, name: &'static str) -> ResolverEntry<'a> {
        ResolverEntry {
            key: Some(name),
            resolver: self,
        }
    }

    /// Gets the next required field entry for in-place manipulation.
    ///
    /// Once this function returns `None` then all required fields have been set and the
    /// [`Resolver::resolve`] method will be successful. Thus, this method can be called in a
    /// while let loop in order to set all required fields.
    ///
    /// The order of required fields returned by this method is not guaranteed and users should not
    /// assume that the order will remain consistent even between minor version changes.
    ///
    /// # Examples
    ///
    /// ```
    /// use seb::ast::{Entry, EntryKind, QuotedString};
    ///
    /// let resolver = Entry::resolver(EntryKind::Manual);
    /// let mut resolver = resolver.resolve().expect_err("Missing title field!");
    ///
    /// // we know that title is the only
    /// let entry = resolver.next_required_entry().unwrap();
    ///
    /// assert_eq!("title", entry.key());
    /// entry.insert(QuotedString::new("My manual".to_owned()));
    ///
    /// assert!(resolver.resolve().is_ok());
    /// ```
    pub fn next_required_entry(&mut self) -> Option<ResolverEntry<'_>> {
        let name = self.req.pop()?;
        Some(self.entry(name))
    }

    /// Sets a field value by field name.
    ///
    /// When the field is set multiple times the last value is used when resolveing the [`Entry`] type.
    /// The `name` of the field is always transformed into the lowercase internally before setting
    /// the field so users of this API don't need to do this.
    ///
    /// The `value` parameter accepts `Into<QuotedString>` types and for `&str` and
    /// `String` this is equivalent to using [`ast::QuotedString::new`] so make sure that
    /// quoting is not required, if it is then use either [`ast::QuotedString::quote`] or
    /// [`ast::QuotedString::from_quoted`]
    #[inline]
    pub fn set_field<I>(&mut self, name: &str, value: I)
    where
        I: Into<QuotedString>,
    {
        // normalize fields to lowercase
        self.set_normalized_field(name.to_lowercase(), value.into());
    }

    /// Set a normalized (lowercase name) field.
    ///
    /// Checks whether this field is a required field and will remove that name from the required
    /// set.
    fn set_normalized_field(&mut self, name: String, value: QuotedString) {
        self.req.retain(|r| *r != name.as_str());
        self.fields.insert(name, value);
    }
}

impl std::fmt::Display for Resolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "error: missing required fields in {} entry\nfound:",
            self.target
        )?;

        for set_field in &self.fields {
            writeln!(f, "    {}: {}", set_field.0, &**set_field.1)?;
        }
        writeln!(f, "missing:")?;
        for req in &self.req {
            writeln!(f, "    {req}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Resolver {}

/// A view into a single required field for a [`Resolver`].
///
/// This entry takes ownership of the required field and if an insert method is not performed on
/// the entry and the field is outstanding then the drop implementation will reinsert the field
/// into the resolvers unset required fields.
///
/// Note: this ownership is safe as this type takes an exclusive mutable reference to the
/// [`Resolver`] so no other methods can be performed on that type until this entry is dropped.
pub struct ResolverEntry<'a> {
    // key is an Option but is always a Some value so unwrapping is always safe apart from the
    // drop implementation which will check whether the value has been taken in order to reinsert
    // it in the required fields.
    key: Option<&'static str>,
    resolver: &'a mut Resolver,
}

impl Drop for ResolverEntry<'_> {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            // key hasn't been taken so we need to reinsert this key back into the required
            // fields Vec.
            self.resolver.req.push(key);
        }
    }
}

impl<'a> ResolverEntry<'a> {
    /// Sets the value of the entry.
    #[allow(clippy::missing_panics_doc)] // see key field comment
    pub fn insert(mut self, default: QuotedString) {
        let key = self.key.take().map(ToOwned::to_owned).unwrap();
        self.resolver.fields.insert(key, default);
    }

    /// Returns a reference to this entry's key.
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // see key field comment
    pub fn key(&self) -> &str {
        self.key.as_ref().unwrap()
    }
}

impl FieldQuery for Resolver {
    fn get_field(&self, name: &str) -> Option<&QuotedString> {
        self.fields.get(name)
    }
}

macro_rules! impl_resolver {
    ($($field:ident),*$(,)?) => {
        impl Resolver {
            $(
                /// Sets the field value with the name of this method.
                ///
                /// This method is equivalent to using the [`Resolver::set_field`] method with the
                /// field name and value.
                ///
                /// The `value` parameter accepts `Into<QuotedString>` types and for `&str` and
                /// `String` this is equivalent to using [`ast::QuotedString::new`] so make sure that
                /// quoting is not required, if it is then use either [`ast::QuotedString::quote`] or
                /// [`ast::QuotedString::from_quoted`]
                #[inline]
                pub fn $field<I>(&mut self, value: I)
                    where I: Into<QuotedString>,
                {
                    self.set_normalized_field(stringify!($field).to_owned(), value.into());
                }
            )*
        }
    };
}

impl_resolver!(
    author,
    book_title,
    chapter,
    institution,
    journal,
    pages,
    publisher,
    school,
    title,
    year,
);

#[cfg(test)]
mod tests {
    use crate::ast::Manual;

    #[test]
    fn resolver_entry_drop_reinserts_required_field() {
        let mut resolver = Manual::resolver();
        // Manual::resolver only requires the `title` field
        // the next_required_entry method pops the `title` value from the `req` Vec and because
        // the result has an exclusive mutable reference we know that the missing field won't cause
        // any issues as it will either be set by the entry or reinserted as part of the drop impl.
        let entry = resolver
            .next_required_entry()
            .expect("Manual resolver requires a title field");
        // drop the entry - this should reinsert the required "title" field
        drop(entry);

        assert!(resolver.required_fields().count() == 1);
    }

    #[test]
    fn resolve_resolver_using_entry() {
        let resolver = Manual::resolver();
        let mut resolver = resolver
            .resolve()
            .expect_err("required Title field not set so should return Resolver");
        let entry = resolver
            .next_required_entry()
            .expect("Manual resolver requires a title field");
        // insert consumes self and the required title field has been removed from the resolver.
        entry.insert("test".into());

        assert!(
            resolver.next_required_entry().is_none(),
            "all required fields should be set so expecting next_required_entry to return None"
        );
        assert!(
            resolver.required_fields().count() == 0,
            "required fields should be set so expecting required_fields to return an empty iterator"
        );

        let entry = resolver
            .resolve()
            .expect("Manual only requires title which should be set!");

        assert_eq!("test", &**entry.title());
    }
}
