# checksum

```shell
$ checksum --help
check file hashes

Basic operation prints a file hash for a file. Alternatively, if the file path provided refers to a hash file (e.g. "foo.md5"), the program will attempt to validate all files listed in the has file. If two paths are given (both files or both directories), the two will be compared. A comparison between directories makes use of the Imprint type for greater efficiency.

If only a left-hand operand is provided, checksum will print the hash of the operand (assuing said operand is a file; it is an error to provide only a directory). The algorithm used for this purpose may be set as an environment variable called CHECKSUM_DEF_ALG. Allowable names include: blake3, md5, sha1, sha256, sha512. These are not case sensitive. This variable may be set at compile time.

A further note on directory comparisons: directory comparisons are asymmetrical. Checksum will ensure that all files from the left hand directory exist in the right hand directory but not vice versa. This is for the common use case that files from the left have been copied to some archive location on the right.

Usage: checksum [OPTIONS] <LEFT> [RIGHT] [COMMAND]

Commands:
  blake3  blake3 mode
  md5     md5 mode
  sha1    sha1 mode
  sha256  sha256 mode
  sha512  sha512 mode
  help    Print this message or the help of the given subcommand(s)

Arguments:
  <LEFT>
          left hand resource

  [RIGHT]
          right hand resource
          
          This resource, whether file or directory, is compared against the left. Both resources must be of matching type: e.g., if the left hand resource is a file, this must also be a file; if the left hand resource is a directory, this must also be a directory. This
          argument is ignored by all subcommands.

Options:
  -f, --full-comparison
          force full comparison
          
          Forces a full comparison between directories. This has no effect except when comparing directories. Warning: this is MUCH slower.

  -v, --verbose
          verbose
          
          Print names of matching files

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version


USAGE:
    checksum <LEFT> [RIGHT] [SUBCOMMAND]

ARGS:
    <LEFT>
            left hand resource

    <RIGHT>
            right hand resource
            
            This resource, whether file or directory, is compared against the left. Both resources
            must be of matching type: e.g., if the left hand resource is a file, this must also be a
            file; if the left hand resource is a directory, this must also be a directory. This
            argument is ignored by all subcommands.

OPTIONS:
    -h, --help
            Print help information

    -V, --version
            Print version information

SUBCOMMANDS:
    blake3
            blake3 mode
    help
            Print this message or the help of the given subcommand(s)
    md5
            md5 mode
    sha1
            sha1 mode
    sha256
            sha256 mode
    sha512
            sha512 mode
```

## Operation

`checksum` can print a checksum, assert a checksum, compare files, or compare trees. As of version 0.7.0, checksum can also validate a checksum file.

### Print

If provided with just a file path, checksum will print a checksum. The default algorithm is md5, but this can be changed with an environment variable.

```shell
❯ checksum .\src\main.rs
0d78c34c81b2fd2c2fe0b0a8c0f74f77
```

To use a different algorithm pass its flag after the file path.

```shell
❯ checksum .\src\main.rs blake3
32f2d7f42f517924e1fb609ea5107ac8ec42d02408a77c0a69294747c5ef8921
```

### Assert

To assert that a file should have a given checksum, pass the file path along with the algorithm and checksum.

```shell
❯ checksum .\src\main.rs `
    blake3 32f2d7f42f517924e1fb609ea5107ac8ec42d02408a77c0a69294747c5ef8921
True
```
### Compare

To compare a file against another file, pass in both filenames. As you can see, checksum is a little careless about making sure they're not just the same file.

```shell
❯ checksum .\src\main.rs .\src\main.rs
True
```

### Compare trees

To compare one directory tree against another, pass in both directory paths. (Comparing a directory against a file or vice versa is impossible.) Checksum does *not* perform a full comparison of each file in this case. Instead, only the start and end of each file is compared, along with the length of each file.

```shell
❯ checksum .\src\ .\target\ --force
Missing: .\src\cli.rs
Missing: .\src\error.rs
Missing: .\src\fmt.rs
Missing: .\src\iter.rs
Missing: .\src\main.rs
```
