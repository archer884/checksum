# checksum

```shell
$ checksum --help
checksum 0.1.0
A simple checksum tool.

In theory, failed assertions return non-zero exit codes. This behavior has not
been tested, and I'm not that good at shell scripting. Good luck!

USAGE:
    checksum.exe <path> [SUBCOMMAND]

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information


ARGS:
    <path>
            A file path


SUBCOMMANDS:
    assert    Test a file against a provided checksum
    eq        Test a file against another file
    help      Prints this message or the help of the given subcommand(s)
```

Checksum provides two modes of operation:

## Assert

Assert that a given file has a given checksum. Although checksum's output is always lower case, the case of the input checksum does not matter.

```shell
$ checksum ./src/main.rs assert e612653753e3e48d779b31c3b92f4b90222b85fcc272031c83d3f226c1fbdd9e
True
```

## Eq

Check a given file for equality with another file. Hashing is done in parallel; on systems with fast drives, this may provide some benefit. On systems with platter drives, I'm sorry.

```shell
$ checksum ./src/main.rs eq ./src/iter.rs
False
```
