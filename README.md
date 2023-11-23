# splirst: A Rust implementation of `split`

## Installation

Coming soon!

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

## Benchmarking performance

|File size|File type|Split method|`splirst`|`split`|
|--------|----------|------------|--------|------|
|15G|SQL|`-n10`|7.9s|10s|
|15G|SQL|`-b1G`|7.3s|6.2s|
|15G|SQL|`-pCREATE`|52.5s|3:34.7m|
|97M|SQL|`-n10`|0.2s|0.05s|
|97M|SQL|`-b50M`|0.3s|0.05s|
|97M|SQL|`-pCREATE`|0.5s|1.4s|
|128K|TXT|`-l500`|0.2s|0.01s|
|128K|TXT|`-b100K`|0.2s|0.004s|
|128K|TXT|`-n10`|0.2s|0.05s|
