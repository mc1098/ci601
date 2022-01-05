# Search-Edit-Bibliography (seb)

`seb` is a command line tool for searching and editing bibliographic entries to a supported format file.

The main use of `seb` is to search for bibliographic entries by `doi`, `ISBN`, `title` etc. and then
add one of those entries to an existing bibliographic file.

Currently available subcommands:

- seb doi
- seb isbn

## File formats

`seb` is being developed to accomodate multiple file formats for bibliography.

Current supported formats:
- `BibTeX` (default)

## Available Subcommands

### `seb doi`

Search a bibliographic entry by its [Digital Object Identifier (doi)](https://en.wikipedia.org/wiki/Digital_object_identifier)
and add it to the current bibliography.

```bash
$ # Search and add bibliographic entry to current bibliography, by default the current bibliography
$ # assumes that a single .bib file exists in the current directory.
$ seb doi "10.1007/s00453-019-00634-0"
```

### `seb isbn`

Search a bibliographic entry by its [International Standard Book Number (ISBN)](https://en.wikipedia.org/wiki/International_Standard_Book_Number)
and add it to the current bibliography.

```bash
$ # Search and add bibliographic entry to current bibliography, by default the current bibliography
$ # assumes that a single .bib file exists in the current directory.
$ seb isbn "0735619670"
```
