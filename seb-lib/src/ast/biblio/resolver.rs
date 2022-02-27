use crate::ast::{Biblio, EntryExt, Resolver};

/// A [`Biblio`] resolver used for managing a set of entry resolvers until they all succeed in order
/// to make a [`Biblio`] with valid entries in.
#[derive(Debug)]
pub struct BiblioResolver {
    pub(super) failed: bool,
    pub(super) resolvers: Vec<Resolver>,
    pub(super) entries: Vec<Box<dyn EntryExt>>,
}

impl BiblioResolver {
    /// Attempts to resolve all of the entry resolvers and returns the [`Biblio`] if all of them
    /// succeed.
    ///
    /// # Errors
    ///
    /// Returns [`Err(Self)`] if one of the entry resolvers fail, this allows for resolving the
    /// entry resolvers that failed and then retrying the resolve.
    pub fn resolve(mut self) -> Result<Biblio, Self> {
        let (built, resolvers): (Vec<_>, Vec<_>) =
            try_partition(self.resolvers.into_iter().map(Resolver::resolve));

        self.entries.extend(built);

        if resolvers.is_empty() {
            Ok(Biblio {
                dirty: self.failed,
                entries: self
                    .entries
                    .into_iter()
                    .map(|e| (e.cite().into_owned(), e))
                    .collect(),
            })
        } else {
            self.resolvers = resolvers;
            self.failed = true;
            Err(self)
        }
    }

    /// Returns the resolvers that failed to resolve so that missing fields can be set before trying
    /// to call [`BiblioResolver::resolve`] again.
    pub fn unresolved(&mut self) -> impl Iterator<Item = &mut Resolver> {
        self.resolvers.iter_mut()
    }

    /// Removes either the entry or resolver based on the index.
    ///
    /// The [`BiblioResolver`] can contain both resolvd entries or resolvers and does so in this
    /// order, therefore the index can be used to retrieve either.
    ///
    /// The index should be found using the [`BiblioResolver::map_iter_all`] iterator as this
    /// iterator is in the same order.
    pub fn checked_remove(&mut self, index: usize) -> Option<Result<Box<dyn EntryExt>, Resolver>> {
        if index < self.entries.len() {
            Some(Ok(self.entries.remove(index)))
        } else if index - self.entries.len() < self.resolvers.len() {
            Some(Err(self.resolvers.remove(index - self.entries.len())))
        } else {
            None
        }
    }

    /// Returns an iterator of both resolved and unresolved entries which impl [`FieldQuery`].
    ///
    /// This allows for querying what a possibly unresolved Biblio contains without having to fully
    /// resolve it first.
    pub fn iter(&self) -> impl Iterator<Item = &dyn EntryExt> {
        self.entries
            .iter()
            .map(AsRef::as_ref)
            .chain(self.resolvers.iter().map(|r| r as &dyn EntryExt))
    }
}

fn try_partition<T, E, B, R>(iter: impl Iterator<Item = Result<T, E>>) -> (B, R)
where
    B: Default + Extend<T>,
    R: Default + Extend<E>,
{
    let mut left = B::default();
    let mut right = R::default();

    iter.fold((), |_, res| match res {
        Err(r) => right.extend([r]),
        l => left.extend(l),
    });

    (left, right)
}

impl std::fmt::Display for BiblioResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for unresolved in &self.resolvers {
            writeln!(f, "{}", unresolved)?;
        }

        write!(
            f,
            "hint: consider enabling interactive mode (-i / --interact) to add missing fields."
        )?;
        Ok(())
    }
}
impl std::error::Error for BiblioResolver {}

#[cfg(test)]
mod tests {
    use crate::ast::{self, Manual};

    use super::*;

    #[test]
    fn none_on_checked_remove_from_empty_resolver() {
        let mut resolver = BiblioResolver {
            failed: false,
            resolvers: vec![],
            entries: vec![],
        };

        assert_eq!(None, resolver.checked_remove(0).map(|_| ()));
    }

    fn manual_entry() -> Box<dyn EntryExt> {
        let mut resolver = Manual::resolver_with_cite("cite");
        resolver.set_field("title", "Title");
        resolver.resolve().unwrap()
    }

    #[test]
    fn some_entry_on_checked_removed_with_single_resolved_entry() {
        let entry = manual_entry();

        let mut resolver = BiblioResolver {
            failed: false,
            resolvers: vec![],
            entries: vec![entry],
        };

        let removed = resolver
            .checked_remove(0)
            .expect("Single item in resolver so checked_remove of 0 should return a Some(_)")
            .expect("Single item is a resolved entry so should contain an Ok(Entry)");

        assert_eq!("cite", removed.cite());
        assert_eq!(Some(&"Title".into()), removed.get_field("title"));
    }

    #[test]
    fn some_resolver_on_checked_remove_with_single_resolver() {
        let resolver = ast::Article::resolver();

        let mut biblio_resolver = BiblioResolver {
            failed: false,
            resolvers: vec![resolver.clone()],
            entries: vec![],
        };

        let removed = biblio_resolver
            .checked_remove(0)
            .expect("Single item in resolver so checked_remove of 0 should return a Some(_)")
            .expect_err("Single item is a resolver so should contain an Err(Resolver)");

        assert_eq!(resolver, removed);
    }

    #[test]
    fn checked_remove_indexes_resolved_before_unresolved() {
        let resolver = ast::Article::resolver();

        // use closure so we can create new BiblioResolver after altering internal state
        let create_biblio_resolver_with_both = || BiblioResolver {
            failed: false,
            resolvers: vec![resolver.clone()],
            entries: vec![manual_entry()],
        };

        let mut biblio_resolver = create_biblio_resolver_with_both();
        let removed_first = biblio_resolver
            .checked_remove(0)
            .expect("At least one item in BiblioResolver")
            .expect("Resolved entries should be indexed first so this should be an Ok(Entry)");

        // avoid move using reference because create_biblio_resolver_with_both closure is borrowing
        assert_eq!("cite", removed_first.cite());
        assert_eq!(Some(&"Title".into()), removed_first.get_field("title"));

        let removed_last = biblio_resolver
            .checked_remove(0)
            .expect("Atleast one item in BiblioResolver")
            .expect_err("Resolved entry has been removed so next 0 index should be the resolver so this should be an Err(Resolver)");

        assert_eq!(&resolver, &removed_last);

        // This demonstrates that checked_remove can work somewhat like a pop_front when calling
        // it with '0' and will remove both resolved and unresolved in that order (but no order in
        // those subsets exist).

        // Next is to prove that at the start if we removed '1' we would have found the resolver
        // instead!

        let mut biblio_resolver = create_biblio_resolver_with_both();

        let removed = biblio_resolver
            .checked_remove(1)
            .expect("Two items in BiblioResolver so 1 index is valid")
            .expect_err(
                "Unresolved entries are indexed after resolved so this should be an Err(Resolver)",
            );

        assert_eq!(resolver, removed);

        let removed = biblio_resolver
            .checked_remove(0)
            .expect("Single item in BiblioResolver")
            .expect("Resolved entry should still be at 0 index so this should be an Ok(Entry)");

        assert_eq!("cite", removed.cite());
        assert_eq!(Some(&"Title".into()), removed.get_field("title"));
    }

    #[test]
    fn iter_to_query_fields() {
        let entry = manual_entry();
        let resolver = ast::Article::resolver();

        let biblio_resolver = BiblioResolver {
            failed: false,
            resolvers: vec![resolver],
            entries: vec![entry],
        };

        let mut iter = biblio_resolver.iter();

        let first = iter.next().expect("First of two iter items");
        assert_eq!(Some("Title"), first.get_field("title").map(|qs| &**qs));

        let second = iter.next().expect("Second of two iter items");
        assert_eq!(None, second.get_field("title"));
    }

    #[test]
    fn display_of_resolver_is_correctly_formatted() {
        let resolver_one = ast::Article::resolver();
        let resolver_two = ast::Article::resolver();

        let biblio_resolver = BiblioResolver {
            failed: false,
            resolvers: vec![resolver_one.clone(), resolver_two.clone()],
            entries: vec![],
        };

        let display = biblio_resolver.to_string();
        let expected = format!(
            "{resolver_one}\n{resolver_two}\nhint: consider enabling interactive \
                               mode (-i / --interact) to add missing fields."
        );

        assert_eq!(expected, display);
    }
}
