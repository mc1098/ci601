use std::ops::Deref;

/// A string type with extra information about quoted string subsections.
///
/// The term quoted in respect to this type is any substring which normally surrounded by some
/// escape character, this type only supports a quoted substring of a single depth, therefore no
/// quoted of a quoted string.
///
/// This representation can be treated like a normal string when performing operations in memory
/// and the quoted information is more useful when composing this value into a specific format.
///
/// # Examples
///
/// [`QuotedString`] can be used for a normal [`String`] which has no quoted substring.
/// ```no_run
/// use seb::ast::QuotedString;
///
/// let string = QuotedString::new("foo".to_owned());
/// assert_eq!("foo", string.map_quoted(str::to_uppercase));
/// ```
///
/// ```no_run
/// use seb::ast::QuotedString;
///
/// let quoted = QuotedString::quote("foo".to_owned());
/// assert_eq!("FOO", quoted.map_quoted(str::to_uppercase));
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QuotedString {
    markers: Vec<usize>,
    value: String,
}

impl Deref for QuotedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl AsRef<str> for QuotedString {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

impl QuotedString {
    /// Convenient alias for `false` for use in [`Self::from_quoted`].
    pub const NORMAL: bool = false;
    /// Convenient alias for `true` for use in [`Self::from_quoted`].
    pub const ESCAPE: bool = true;

    /// Create a new [`QuotedString`] from a [`String`], this is effectively a newType around
    /// [`String`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use seb::ast::QuotedString;
    ///
    /// let expected = "foo".to_owned();
    /// let string = QuotedString::new(expected.clone());
    ///
    /// // QuotedString impls Deref<Target = str> so we deref and then borrow to match with
    /// // expected &str
    /// assert_eq!(&expected, &*string);
    /// ```
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self {
            markers: Vec::new(),
            value,
        }
    }

    /// Create a new quoted [`String`].
    ///
    /// This represents a value that is being "quoted" entirely.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use seb::ast::QuotedString;
    ///
    /// let quoted = QuotedString::quote("foo".to_owned());
    /// assert_eq!("{foo}", quoted.map_quoted(|s| format!("{{{}}}", s)));
    /// ```
    #[must_use]
    pub fn quote(value: String) -> Self {
        Self {
            markers: vec![0, value.len()],
            value,
        }
    }

    /// Create a new [`QuotedString`] based on escape patterns found in the [`String`].
    ///
    /// The `escape` predicate is used to check each [`char`] in the `quoted` `&str` in order
    /// to create the string with quoted substrings represented by the [`QuotedString`] type.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use seb::ast::QuotedString;
    ///
    /// let string = QuotedString::from_quoted("foo bar $baz$", '$');
    ///
    /// // the deref `&str` will be the string without the identified escape chars
    /// assert_eq!("foo bar baz", &*string);
    /// // we can change the escaped substring to something else
    /// assert_eq!("foo bar BAZ", string.map_quoted(str::to_uppercase));
    /// ```
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_quoted(quoted: &str, pattern: impl EscapePattern) -> Self {
        let mut value = String::with_capacity(quoted.len());
        let mut markers = Vec::new();
        let mut i = 0;

        for c in quoted.chars() {
            if pattern.is_escape(c) {
                markers.push(i);
            } else {
                value.push(c);
                i += 1;
            }
        }

        Self { markers, value }
    }

    /// Create a [`QuotedString`] from a list of tuples, where the bool signifies that the
    /// [`String`] in the tuple is to be quoted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use seb::ast::QuotedString;
    ///
    /// let string = QuotedString::from_parts(vec![
    ///     (false, "foo".to_owned()),
    ///     (true, "bar".to_owned()),
    /// ]);
    ///
    /// assert_eq!("fooBAR", string.map_quoted(str::to_uppercase));
    /// ```
    #[must_use]
    pub fn from_parts(parts: Vec<(bool, String)>) -> Self {
        if parts.is_empty() {
            return Self::default();
        }

        let mut length = 0;
        let mut markers = Vec::new();

        let value = parts
            .into_iter()
            .map(|(b, s)| {
                let new_len = length + s.len();
                if b {
                    markers.push(length);
                    markers.push(new_len);
                }
                length = new_len;
                s
            })
            .collect();

        Self { markers, value }
    }

    /// Replace quoted substrings using the closure provided to this method to create a [`String`]
    /// with those replaced values in place of the substrings.
    ///
    /// The closure takes the quoted substrings and can transform them to any [`String`] and the
    /// resulting [`String`] will contain those transformations in-place of the substrings.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use seb::ast::QuotedString;
    ///
    /// let string = QuotedString::quote("replace".to_owned());
    /// assert_eq!("new", string.map_quoted(|_| "new".to_owned()));
    /// ```
    pub fn map_quoted(&self, f: impl Fn(&str) -> String) -> String {
        let mut res = String::new();

        if self.value.is_empty() {
            return res;
        }

        let mut verbatim = false;
        let mut pos = 0;

        for marker in &self.markers {
            let marker = *marker;
            if verbatim {
                res.push_str(&f(&self.value[pos..marker]));
                verbatim = false;
            } else {
                res.push_str(&self.value[pos..marker]);
                verbatim = true;
            }
            pos = marker;
        }

        if pos < self.value.len() {
            if verbatim {
                res.push_str(&f(&self.value[pos..]));
            } else {
                res.push_str(&self.value[pos..]);
            }
        }
        res
    }
}

/// A char escape pattern.
///
/// A [`EscapePattern`] expresses that the implementing type can be used as a escape pattern for
/// creating quoted subslices in a [`QuotedString`].
///
/// Depending on the type of the pattern, the behaviour of [`Self::is_escape`] can change. The
/// table below describes some of those behaviours.
///
/// | Pattern type                  | Match condition               |
/// |-------------------------------|-------------------------------|
/// | `F: Fn(char) -> bool`         | `F` returns `true` for a char |
/// | `char`                        | is equal to char              |
/// | `&[char]`                     | is contained by slice         |
/// | `const N: usize, [char; N]`   | is contained by array         |
///
/// # Examples
///
/// ```
/// use seb::ast::EscapePattern;
///
/// // Fn(char) -> bool
/// assert!((char::is_uppercase).is_escape('A'));
/// assert_eq!(false, (|c: char| c.is_ascii()).is_escape('ß'));
///
/// // char
/// assert!('$'.is_escape('$'));
/// assert_eq!(false, '$'.is_escape('!'));
///
/// // &[char]
/// assert!((&['{', '}'][..]).is_escape('}'));
/// assert_eq!(false, (&['{', '}'][..]).is_escape(']'));
///
/// // [char; N]
/// assert!(['*'].is_escape('*'));
/// assert_eq!(false, ['{', '}'].is_escape('$'));
/// ```
pub trait EscapePattern {
    /// Checks whether the pattern matches the `char`.
    fn is_escape(&self, c: char) -> bool;
}

impl<F> EscapePattern for F
where
    F: Fn(char) -> bool,
{
    fn is_escape(&self, c: char) -> bool {
        (self)(c)
    }
}

impl EscapePattern for char {
    fn is_escape(&self, c: char) -> bool {
        *self == c
    }
}

impl EscapePattern for &[char] {
    fn is_escape(&self, c: char) -> bool {
        self.contains(&c)
    }
}

impl<const N: usize> EscapePattern for [char; N] {
    fn is_escape(&self, c: char) -> bool {
        self.contains(&c)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn empty_quoted_string_is_equiv_to_empty_string() {
        let string = QuotedString::default();

        assert!(string.is_empty());
        assert_eq!(String::new(), &*string);
    }

    #[test]
    fn quoted_map_to_uppercase() {
        let string = QuotedString::from_quoted("hello, ^world^", '^');
        let res = string.map_quoted(str::to_uppercase);

        assert_eq!("hello, WORLD", res);
    }

    #[test]
    fn quoted_prefix_from_parts_check() {
        let string = QuotedString::from_parts(vec![
            (true, "hello".to_owned()),
            (false, ", world".to_owned()),
        ]);
        let res = string.map_quoted(str::to_uppercase);

        assert_eq!("HELLO, world", res);
    }

    #[test]
    fn quoted_part_in_parts_from_parts_check() {
        let string = QuotedString::from_parts(vec![
            (false, "foo".to_owned()),
            (true, "bar".to_owned()),
            (false, "baz".to_owned()),
        ]);
        let res = string.map_quoted(str::to_uppercase);

        assert_eq!("fooBARbaz", res);
    }

    #[test]
    fn quoted_parts_together_from_parts_check() {
        let string = QuotedString::from_parts(vec![
            (false, "foo".to_owned()),
            (true, "bar".to_owned()),
            (true, "baz".to_owned()),
            (false, "qux".to_owned()),
        ]);
        let res = string.map_quoted(str::to_uppercase);

        assert_eq!("fooBARBAZqux", res);
    }

    #[test]
    fn quoted_postfix_from_parts_check() {
        let string = QuotedString::from_parts(vec![
            (false, "hello, ".to_owned()),
            (true, "world".to_owned()),
        ]);
        let res = string.map_quoted(str::to_uppercase);

        assert_eq!("hello, WORLD", res);
    }

    #[test]
    fn support_bibtex_verbatim() {
        let string = QuotedString::from_quoted(
            "{QuickXsort}: A Fast Sorting Scheme in Theory and Practice",
            ['{', '}'],
        );

        let res = string.map_quoted(|s| format!("{{{}}}", s));
        let expected = "{QuickXsort}: A Fast Sorting Scheme in Theory and Practice";

        assert_eq!(expected, res);
    }
}
