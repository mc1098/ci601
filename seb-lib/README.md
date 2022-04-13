# `seb-lib`

[![CI](https://github.com/mc1098/seb/actions/workflows/ci.yml/badge.svg)](https://github.com/mc1098/seb/actions/workflows/ci.yml)
[![MIT licensed][mit-badge][mit-url]

[mit-url]: ../LICENSE

seb is a library which supports searching for bibliography entries from select APIs
and adding them to an in memory bibliography model. seb supports transforming the in-memory
bibliography to `Format`s such as `BibTex`.

## Why isn't `seb-lib` on crates.io? 

This is an individual university project so external input is not accepted, this being said crates.io
is great for package management but also offers discoverability to new crates that could lead to
external input.

Thus using `seb-lib` as a dependency requires using the repository URL.

## Usage

To use `seb-lib`, first add this to your `Cargo.toml`:

```toml
#[dependencies]
seb = { git = "https://github.com/mc1098/seb/tree/main/seb-lib" }
```

## Features

- Bibliographic formats represented by the `Format` trait implementation.
- `Entry` enum provides a statically enforcable set of entry types.
- Enforced minimum required fields.
- Dynamic resolution of entries without the minimum required fields.
- Ready made API functions to search for entries by:
  - DOI
  - ISBN
  - IETF RFC Number
  - Title
- API function that supports parsing a URL with a supported `Format`.
- Simple bibliography management with the `Biblio` type.

Also see the [Cargo features](#cargo-features).


### Cargo Features

- [`bibtex`]
- [`file`]

The [`bibtex`] feature is the only default feature so if the `BibTeX` `Format` is not required then
you will need to disable default features in your `Cargo.toml` file:

```toml
#[dependencies]
# seb with default features disabled
seb = { git = "https://github.com/mc1098/seb/tree/main/seb-lib", default-feature = false }
```

[`bibtex`]: #bibtex
[`file`]: #file

#### `bibtex`

This crate implements the `Format` trait for the [BibTeX] format but puts it behind a default feature
gate incase a user of `seb-lib` would prefer to implement this type in a different way.

This implementation supports parsing all the features of BibTeX but will expand variables and cross
references into full entries. This `Format` implementation should accept and create valid BibTeX files
but the outputted BibTeX file might not be the most optimal interms of reducing redundant data.

[BibTeX]: http://www.bibtex.org/

#### `file`

The `file` feature contains all the utility functions for finding a `File` in a given path or the current
directory. This module also wraps a normal `File` as a `FormatFile` so that the source of the data is
associated with a `Format` and users don't have to be mindful of which `File` or `String` is in what `Format`
as the type system stops you from mixing two different `Format`s.

## Platforms

- Windows
- macOS
- Linux (Ubuntu)

There are potentially others but they are not tested in the current CI so there is no guarantee that
they will continue to work in the future.

## Supported Rust Versions

`seb-lib` is built against the latest stable release. The current `seb-lib` version is not guaranteed to build on
Rust versions earlier than the latest stable version.

## License 

This project is licensed under the [MIT license].

[MIT license]: https://github.com/mc1098/seb/blob/main/LICENSE

## Contribution

This is an individual university project with specific criteria that is not open to contribution therefore
any contributions in form of issues or PRs will be closed and ignored.
