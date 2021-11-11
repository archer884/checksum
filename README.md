# checksum

```shell
$ checksum --help
checksum 0.5.0

USAGE:
    checksum.exe [OPTIONS] <PATH> [COMPARE]

ARGS:
    <PATH>
            a file to be hashed

    <COMPARE>
            a file to compare against

OPTIONS:
    -b, --blake3 <BLAKE3>
            set blake3 mode and supply an (optional) checksum for comparison

    -d, --sha1 <SHA1>
            set sha1 mode and supply an (optional) checksum for comparison

    -f, --force
            when comparing trees, force a full comparison and list exceptions

    -h, --hidden
            when comparing trees, include hidden files

        --help
            Print help information

    -m, --md5 <MD5>
            set md5 mode and supply an (optional) checksum for comparison

    -s, --sha256 <SHA256>
            set sha256 mode and supply an (optional) checksum for comparison

    -V, --version
            Print version information
```

## Operation

`checksum` can print a checksum, assert a checksum, compare files, or compare trees.

### Print

If provided with just a file path, checksum will print a checksum. The default algorithm is sha1.

```shell
❯ checksum .\src\main.rs
1aa86ee54f8f67d506ece60b2a191a75748acc19
```

To use a different algorithm pass its flag after the file path.

```shell
❯ checksum .\src\main.rs --blake3
78d259dc346d9560d48e8885ea43db96b9247f6b73815f463f2e333ad0778fe2
```

### Assert

To assert that a file should have a given checksum, pass the file path along with the algorithm and checksum.

```shell
❯ checksum .\src\main.rs `
    --blake3 78d259dc346d9560d48e8885ea43db96b9247f6b73815f463f2e333ad0778fe2
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

Normally, checksum will abort after finding the first mismatched file. This behavior can be avoided by passing the `--force` flag to force a full directory comparison and list all exceptions.

```shell
❯ checksum .\src\ .\target\ --force
Missing: .\src\cli.rs
Missing: .\src\error.rs
Missing: .\src\fmt.rs
Missing: .\src\iter.rs
Missing: .\src\main.rs
```
