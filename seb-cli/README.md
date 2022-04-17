# Search-Edit-Bibliography (seb)

[![CI](https://github.com/mc1098/seb/actions/workflows/ci.yml/badge.svg)](https://github.com/mc1098/seb/actions/workflows/ci.yml)
[![MIT licensed][mit-badge]][mit-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: ../LICENSE

`seb` is a command line tool for searching and editing bibliographic entries to a supported format file.

`seb` is a useful tool for searching known bibliographic entries by `doi`, `ISBN`, `title` etc.
and then adding those entries to the bibliography file.

Currently available subcommands:

- [`seb add`](#add-subcommand)
- [`seb new`](#new-subcommand)
- [`seb rm`](#rm-subcommand)

## Add Subcommand

The `add` subcommand is used to search and add a bibliographic entry to a bibliography file. The `add`
subcommand has many subcommands for searching entries using certain identifiers, such as `doi` or `ISBN`,
or from certain sources, such as the IETF Datatracker to search bibliographic entries for RFCs.

### `seb add doi`

Search a bibliographic entry by its [Digital Object Identifier (doi)](https://en.wikipedia.org/wiki/Digital_object_identifier)
and add it to the current bibliography.

```bash
$ # Search and add bibliographic entry to current bibliography, by default the current bibliography
$ # assumes that a single .bib file exists in the current directory.
$ seb add doi "10.1007/s00453-019-00634-0"
```

```bibtex
@article{Edelkamp_2019,
    author = {Stefan Edelkamp and Armin Wei{\ss} and Sebastian Wild},
    title = {{QuickXsort}: A Fast Sorting Scheme in Theory and Practice},
    journal = {Algorithmica},
    year = {2019},
    pages = {509--588},
    url = {https://doi.org/10.1007%2Fs00453-019-00634-0},
    publisher = {Springer Science and Business Media {LLC}},
    number = {3},
    doi = {10.1007/s00453-019-00634-0},
    month = {oct},
    volume = {82},
}
```

### `seb add ietf`

Search a bibliographic entry by its [IETF RFC Number](https://www.ietf.org/standards/rfcs/)
and add it to the current bibliography.

```bash
$ # Search and add bibliographic entry to current bibliography, by default the current bibliography
$ # assumes that a single .bib file exists in the current directory.
$ seb add ietf 7230
```

_[RFC 7230: Hypertext Transfer Protocol (HTTP/1.1): Message Syntax and Routing](https://datatracker.ietf.org/doc/html/rfc7230)_

```bibtex
@misc{rfc7230,
    title = {{Hypertext Transfer Protocol (HTTP/1.1): Message Syntax and Routing}},
    author = {Roy T. Fielding and Julian Reschke},
    howpublished = {RFC 7230},
    series = {Request for Comments},
    year = {2014},
    url = {https://rfc-editor.org/rfc/rfc7230.txt},
    publisher = {RFC Editor},
    number = {7230},
    month = {June},
    doi = {10.17487/RFC7230},
    pagetotal = {89},
}
```

### `seb add isbn`

Search a bibliographic entry by its [International Standard Book Number (ISBN)](https://en.wikipedia.org/wiki/International_Standard_Book_Number)
and add it to the current bibliography.

```bash
$ # Search and add bibliographic entry to current bibliography, by default the current bibliography
$ # assumes that a single .bib file exists in the current directory.
$ seb add isbn 0735619670
```

```bibtex
@book{SteveMcConnell2004,
    author = {Steve McConnell},
    title = {Code Complete},
    publisher = {DV-Professional},
    year = {2004},
    isbn = {0735619670},
}
```

## New Subcommand

The `new` subcommand is used to interactively<sup>[1]</sup> add a minimal bibliographic entry. The `new` command
expects a single <KIND> argument which can be any string but a set of <KIND> values will have different
required fields:

<sup>[1]</sup>_The `new` subcommand is interactive so ignores the `interact` flag and always sets it to `true`._

```console
$ seb new book
```

- article
- book
- booklet
- "book chapter"
- "book pages"
- "book section"
- "in proceedings"
- manual
- "master thesis"
- "phd thesis"
- proceedings
- "tech report"
- unpublished

For other field values this becomes a custom entry type and only requires a `title` field value.

Additional required fields can be added inline using the `--field` option which accepts multiple
values:

```console
$ seb new book --fields url series
```

This would require field values for the fields `url` and `series` - any duplicate field names that
are already required are ignored.

## Rm Subcommand

Removes a bibliographic entry from the bibliography found in the current directory, or at the path
specified when using the `--file` option, by the cite key.

```bash
$ # Remove bibliographic entry from the current bibliography.
$ seb rm rfc7230
```

_"rfc7230" is the default cite key for the BibTeX format when adding an ietf entry with the RFC
number of 7230_

## Resolution of required fields

`seb` will try and find the current bibliography that matches the file format, default is BibTeX (.bib),
and parse it into a intemediate representation which for each entry type has a required set of fields
in order to be considered resolved.

Running `seb` normally with an incomplete entry will result in an error that
explains the fields found and which were missing, it will also include a hint to run interactive mode:

```bash
$ # Non help command with seb will trigger this file check
error: missing required fields in book entry
found:
    author: Me
    title: My Incomplete book entry
    publisher: Also me
missing:
    year

hint: consider enabling interactive mode (-i / --interact) to add missing fields.
```
_Note: This output demonstrates that a valid book entry requires the following: author, title,
publisher, year._

### Resolving entries in interactive mode

The above error output also comes with a hint - 'consider enabling interactive mode..'. This simply
allows `seb` to provide the user the ability to add the missing field information manually:

```bash
$ # Non help command with seb with the -i flag set
Missing required fields for entry: My Incomplete book entry
Enter value for the year field: <CURSOR>
```

The `<CURSOR>` is where the cursor for input will be so the user can set the field value for `year`,
in this case only one field needs to be added manually, however, `seb` will ask to resolve all required
fields and will do this for every incomplete entry.

**Resolving entries in interactive mode also applies to any entries found using the `add` subcommand!**

## File formats

`seb` is being developed to accomodate multiple file formats for bibliography.

Current supported formats:
- `BibTeX` (default)

## Supported Rust Versions

`seb-lib` is built against the latest stable release. The current `seb-lib` version is not guaranteed to build on
Rust versions earlier than the latest stable version.

## License 

This project is licensed under the [MIT license].

[MIT license]: https://github.com/mc1098/seb/blob/main/LICENSE

## Contribution

This is an individual university project with specific criteria that is not open to contribution therefore
any contributions in form of issues or PRs will be closed and ignored.
