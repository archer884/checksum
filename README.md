# checksum

```shell
$ checksum --help
checksum 0.1.1
J/A <archer884@gmail.com>
A simple checksum tool.

In theory, failed assertions return non-zero exit codes. This behavior has not been tested,
and I'm not that good at shell scripting. Good luck!

USAGE:
    checksum <path>
    checksum <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <path>    A file path.

SUBCOMMANDS:
    assert           Assert that a file matches a given checksum.
    compare          Compare two files.
    compare-trees    Compare two directory trees.
    help             Prints this message or the help of the given subcommand(s)
```

Checksum provides three modes of operation:

## Assert

Assert that a given file has a given checksum. Although checksum's output is always lower case, the case of the input checksum does not matter.

```shell
$ checksum assert `
    ./src/main.rs `
    e612653753e3e48d779b31c3b92f4b90222b85fcc272031c83d3f226c1fbdd9e
True
```

## Check

Check a given file for equality with another file. Hashing is done in parallel; on systems with fast drives, this may provide some benefit. On systems with platter drives, I'm sorry.

```shell
$ checksum compare ./src/main.rs ./src/iter.rs
False
```

Of course, you can also just pass it the path of a file to print the file's checksum.

## Compare trees

Intended for use when performing bulk copy of files from one directory to another, this mode compares file length, relative filenames, and the hash of the first and last portions of the file in order to determine sameness.
