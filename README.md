# dcsv

**Dynamic csv reader and lose validater library.**

If you use structured csv data, use [csv crate](https://crates.io/crates/csv)

## Feature

- Read csv which has undecided format
- Optionally validate csv values

## Usage

```toml
[dependencies]
dcsv = "*"
```

```rust
use dcsv::Reader;
use std::fs::File;

let reader = Reader::new()
	.use_separtor(';') // Default is comma
	.use_line_separtor('|') // Default is semi colon
	.read_from(
			File::open("file_name.csv")
			.expect("Failed to read file")
			)
	.expect("Failed to retrieve csv value from file");
```

## Why?

Most of the times, well known csv crate is much more performant and eaiser to
use because csv format doesn't change a lot. However there are cases where
program expects undecided csv formats and contents, which means program has to
ensure every value is compatible with expected standard. In such case, csv crate's
type guarantee is painful to handle with.

See the examples for usages.

- [CSV editor](https://github.com/simhyeon/ced)
- [CSV raw query](https://github.com/simhyeon/cindex)
