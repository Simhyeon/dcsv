# dcsv

Dynamic csv reader, editor, and writer library.

If you use structured csv data, use [csv crate](https://crates.io/crates/csv)

## Feature

- Read csv which has undecided format
- Optionally validate csv values
- Edit csv values
- Write to file

## Usage

```toml
[dependencies]
dcsv = "*"
```

```rust
use dcsv::{Reader, VirtualData, Value};
use std::fs::File;

let data: VirtualData = Reader::new()
	.use_separtor(';') // Default is comma
	.use_line_separtor('|') // Default is semi colon
	.read_from(
			File::open("file_name.csv")
			.expect("Failed to read file")
			)
	.expect("Failed to retrieve csv value from file");

// Refer docs.rs for various VirtualData methods
let value : Option<&Value> = data.get_cell(1,1).expect("Failed to get cell");
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
