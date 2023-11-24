# splirst: A Rust implementation of `split`

The most important thing to know about this implementation is that it's based on `split` circa 2013, which is old but happens to be the version that ships with MacOS.

## Installation

Maybe someday! Honestly I can't recommend it, just use the current version of `split` and `csplit` in the GNU coreutils ([repo](https://github.com/coreutils/coreutils)).

## Usage

```bash
Usage: splirst [OPTIONS] <FILE_PATH> [PREFIX]

Arguments:
  <FILE_PATH>
  [PREFIX]     [default: x]

Options:
  -a, --suffix-length <SUFFIX_LENGTH>  [default: 2]
  -d, --numeric-suffix
  -l, --line-count <LINE_COUNT>        [default: 1000]
  -n, --chunk-count <CHUNK_COUNT>
  -b, --byte-count <BYTE_COUNT>
  -p, --pattern <PATTERN>
  -h, --help                           Print help
```
