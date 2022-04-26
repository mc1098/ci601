`seb-lib` 0.2.1
===============

Bug Fixes:

* [FIX #146](https://github.com/mc1098/ci601/pull/146):
  Fix incorrect `inbook`/`incollection` resolution.
* [FIX #153](https://github.com/mc1098/ci601/pull/153):
  Fix too many files error message.

Features:

* [FEAT #136](https://github.com/mc1098/ci601/pull/136):
  Add `file` feature.
* [FEAT #155](https://github.com/mc1098/ci601/pull/155):
  Add new `add_required_fields` method to entry resolver.
* [FEAT #156](https://github.com/mc1098/ci601/pull/156):
  Add `get` method to `Biblio` and `set_fields_from_entry` method to entry resolver.

`seb-cli` 0.2.0
===============

Breaking Changes:

* [BREAKING #142](https://github.com/mc1098/ci601/pull/142):
  Default bibliography is no longer crated when one cannot be found.

Features:

* [FEAT #152](https://github.com/mc1098/ci601/pull/152):
  Add `check` command.
* [FEAT #155](https://github.com/mc1098/ci601/pull/155):
  Add `fields` option to `new` command.
* [FEAT #156](https://github.com/mc1098/ci601/pull/156):
  Add `derive` feature to create a new entry based on an existing one.

`seb-lib` 0.2.0
===============

Breaking Changes:

* [BREAKING #124](https://github.com/mc1098/ci601/pull/124):
  `Resolver::required_fields` now returns a `impl Iterator<Item = &str>` instead of a `impl Iterator<Item = Cow<'static, str>>`.
* [BREAKING #125](https://github.com/mc1098/ci601/pull/125):
  Removes the `Entry::find_field` - import the `FieldQuery` trait and use the `FieldQuery::get_field` for the same functionality.

Bug Fixes:

* [FIX #134](https://github.com/mc1098/ci601/pull/134):
  Fix `month` field output to produce the short month name without braces for `BibTex` format.
* [FIX #137](https://github.com/mc1098/ci601/pull/137):
  Fix `date` fields are now normalized into date parts (`year`, `month`, `day`) for `BibTex` format.
* [FIX #138](https://github.com/mc1098/ci601/pull/138):
  Fix `crossref` field support to fully resolve those entries.

Features:

* [FEAT #136](https://github.com/mc1098/ci601/pull/136):
  Add `From<&str>` for `EntryKind<'static>`.
* [FEAT #139](https://github.com/mc1098/ci601/pull/139):
  Add `bibtex` feature gate for the `BibTex` `Format`.

`seb-cli` 0.1.1
===============

Bug Fixes:

* [FIX #123](https://github.com/mc1098/ci601/pull/123):
  Fix `ietf` subcommand duplication check to not just check the `number` field.

Features:

* [FEAT #133](https://github.com/mc1098/ci601/pull/133):
  Group BibTeX entries by entry types in the resulting bibliography file.
* [FEAT #136](https://github.com/mc1098/ci601/pull/136):
  `new` subcommand for creating bibliographic entries manually (enforces required fields).

0.1.0
=====

First release so features only!

Features:

* [FEAT #2](https://github.com/mc1098/ci601/pull/2) & [FEAT #57](https://github.com/mc1098/ci601/pull/57):
  Search and add bibliographic entry by [Digital Object Identifier (doi)](https://en.wikipedia.org/wiki/Digital_object_identifier).
* [FEAT #3](https://github.com/mc1098/ci601/pull/3) & [FEAT #57](https://github.com/mc1098/ci601/pull/57):
  Search and add bibliographic entry by [International Standard Book Number (ISBN)](https://en.wikipedia.org/wiki/International_Standard_Book_Number).
* [FEAT #6](https://github.com/mc1098/ci601/pull/6):
  - `--file <FILE>` or `-f <FILE>` option to choose bibliography file
  - `--verbosity` or `-v` flag to enable more output from `ci601`.
  - `--quiet` or `-q` flag to enable quiet mode (only stdout output is the citation key added)
* [FEAT #23](https://github.com/mc1098/ci601/pull/23) & [FEAT #57](https://github.com/mc1098/ci601/pull/57):
  User can confirm the entry to add from an interactive select list.
* [FEAT #43](https://github.com/mc1098/ci601/pull/43):
  Override citation key for entry with `--cite <CITE>` option.
* [FEAT #52](https://github.com/mc1098/ci601/pull/52) & [FEAT #57](https://github.com/mc1098/ci601/pull/57):
  Search and add bibliographic entry by its [IETF RFC Number](https://www.ietf.org/standards/rfcs/).
* [FEAT #55](https://github.com/mc1098/ci601/pull/55):
  Remove a bibliographic entry by its citation key.
* [FEAT #61](https://github.com/mc1098/ci601/pull/61):
  Resolve entrys with missing fields at runtime.
* [FEAT #66](https://github.com/mc1098/ci601/pull/66):
  Interact mode enabled with `--interact` or `-i` (gates FEAT #23 & 61)
* [FEAT #89](https://github.com/mc1098/ci601/pull/89):
  Search and add bibliographic entry by title.

