# dcsv

Dynamic csv reader, editor, and writer library.

If you use structured csv data, use [csv crate](https://crates.io/crates/csv)

If you want higher wrapper around csv editing, use [ced](https://crates.io/crates/ced)

[changes](./docs/change.md)

## Feature

- Read csv which has undecided format
- Optionally validate csv values
- Edit csv values
- Write to file

## Basic usage

Refer [Usage](./docs/usage.md) for more usages. Docs.rs is also useful in general.

Include dcsv in cargo.toml file first.

```toml
[dependencies]
dcsv = "*"
```

```rust
// Trait VCont is required to use most methods
use dcsv::{Reader, VirtualData, Value, VCont};
use std::fs::File;

let data: VirtualData = Reader::new()
    .use_delimiter(';')      // Default is comma
    .use_line_delimiter('|') // Default is '\n, \r\n'
    .data_from_stream(
        BufReader::new(
            File::open("file_name.csv")
                .expect("Failed to read file")
        )
    )
    .expect("Failed to retrieve csv value from file");

// import VCont trait before use
// Refer docs.rs for various VirtualData methods
let value : Option<&Value> = data.get_cell(1,1);
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

## Not yet

- Fully comptaible with csv spec
