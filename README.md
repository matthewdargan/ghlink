# Ghlink

Ghlink creates GitHub permanent links to specified file lines of files hosted
in a GitHub repository.

Usage:

    ghlink file [line1] [line2]

`ghlink file` prints a link to file.

`ghlink file line1` prints a link to line1 in file.

`ghlink file line1 line2` prints a link to lines line1 through line2 in file.

The `git` program must be on the system's PATH environment variable.

## Examples

Print a link to README.md:

```sh
$ ghlink README.md
```

Print a link to line 3 in README.md:

```sh
$ ghlink README.md 3
```

Print a link to lines 3 through 8 in README.md:

```sh
$ ghlink README.md 3 8
```
