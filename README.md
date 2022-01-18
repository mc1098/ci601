# Search-Edit-Bibliography (seb)

`seb` is a command line tool for searching and editing bibliographic entries to a supported format file.

`seb` is most useful as a tool for searching for known bibliographic entries by `doi`, `ISBN`, `title` etc.
and then adding those entries to the bibliography file.

Currently available subcommands:

- [`seb add`](#add-subcommand)
- [`seb rm`](#rm-subcommand)

## File formats

`seb` is being developed to accomodate multiple file formats for bibliography.

Current supported formats:
- `BibTeX` (default)

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

## `seb add ietf`

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

## Rm Subcommand

Removes a bibliographic entry from the bibliography found in the current directory, or at the path
specified when using the `--file` option, by the cite key.

```bash
$ # Remove bibliographic entry from the current bibliography.
$ seb rm rfc7230
```

_"rfc7230" is the default cite key for the BibTeX format when adding an ietf entry with the RFC
number of 7230_
