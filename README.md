# Ghlink

[![Crates.io](https://img.shields.io/crates/v/ghlink.svg)](https://crates.io/crates/ghlink)
[![Documentation](https://docs.rs/ghlink/badge.svg)](https://docs.rs/ghlink)
[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](LICENSE)

Ghlink creates GitHub permanent links to specified file lines of files hosted
in a GitHub repository.

Usage:

    ghlink [-l1 line1 [-l2 line2] | -s text] file

`ghlink file` prints a link to file.

`ghlink -l1 line1 file` prints a link to line1 in file.

`ghlink -l1 line1 -l2 line2 file` prints a link to lines line1 through line2
in file.

`ghlink -s text file` prints a link to lines matching text in file. If text
is ‘-’, the standard input is used.

## Examples

Print a link to README.md:

```sh
$ ghlink README.md
```

Print a link to line 3 in README.md:

```sh
$ ghlink -l1 3 README.md
```

Print a link to lines 3 through 8 in README.md:

```sh
$ ghlink -l1 3 -l2 8 README.md
```

Print a link to lines matching "Usage:\n\n    ghlink file":

```sh
$ ghlink -s 'Usage:\n\n    ghlink file' README.md
```
