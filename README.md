# checksum

```shell
$ checksum --help
a checksum command

Usage: checksum [OPTIONS] <TARGET>
       checksum [OPTIONS] [TARGET] <COMMAND>

Commands:
  file
  help  Print this message or the help of the given subcommand(s)

Arguments:
  <TARGET>
          a file or directory

Options:
  -c, --compare <COMPARE>
          a file or directory

          Provide this argument to assert that the target and comparison
          targets are equal. Non-equal files will be printed to the screen. Any
          non-equal files will cause the command to return an error code to the
          shell.

  -a, --assert <ASSERT>
          a hash value

          Provide this argument to assert that the target and hash are equal.

  -m, --mode <MODE>
          the hashing algorithm to be used

          For output, the default is sha256, but the default algorithm may be
          overridden by setting an environment variable called
          CHECKSUM_DEFAULT_ALG.

          For internal comparisons, checksum uses Blake3.

          [env: CHECKSUM_DEFAULT_ALG=]

  -f, --force-full-compare
          force full comparison

          Comparisons between directory trees are partial comparisons by
          default. Pass this flag to trigger a full comparison. A full
          comparison is MUCH SLOWER.

  -v, --verbose
          print names of matching files during directory comparisons

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Operation

`checksum` can print a checksum, assert a checksum, compare files, or compare directory trees. As of version 0.8.1, checksum can also create and validate sum files.

### Print

If provided with just a file path, checksum will print a checksum. The default algorithm is sha256, but this can be changed with an environment variable. See below.

```shell
❯ checksum ./src/main.rs
53f44dc9cba08ef467d1d2d26a27260b266e37350beac6c1efc6bd6ebe437516
```

To use a different algorithm pass its flag after the file path.

```shell
❯ checksum ./src/main.rs -m blake3
24f7dc5700cabbed6e1c91436e95081a791338f0798eb58594d23aabd91ec926
```

### Assert

To assert that a file should have a given checksum, pass the file path along with the algorithm and checksum.

```shell
❯ checksum ./src/main.rs \
    -m blake3 \
    -a 24f7dc5700cabbed6e1c91436e95081a791338f0798eb58594d23aabd91ec926
True
```
### Compare

To compare a file against another file, pass in both filenames. As you can see, checksum is a little careless about making sure they're not just the same file.

```shell
❯ checksum ./src/main.rs -c ./src/main.rs
True
```

### Compare trees

To compare one directory tree against another, pass in both directory paths. (Comparing a directory against a file or vice versa is impossible.) Checksum does *not* perform a full comparison of each file in this case. Instead, only the start and end of each file is compared, along with the length of each file.

```shell
❯ checksum ./src/ ./target/
Missing: ./src/cli.rs
Missing: ./src/error.rs
Missing: ./src/fmt.rs
Missing: ./src/iter.rs
Missing: ./src/main.rs
```

It is possible to force a full comparison of files in two directories by passing the `--force` flag. This is, of course, a whale of a lot slower.

## Default algorithm

The default algorithm has changed as of version 0.8. By default, sha256 sums are printed when checksum is asked to print a checksum. This default can be overridden by setting an environment variable called `CHECKSUM_DEFAULT_ALG`. The value of this variable may be any of checksum's normal algorithms.
