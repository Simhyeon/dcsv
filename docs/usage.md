# VirtualData

Virtualdata is a wrapper around hashmap records. You can set limiter to set
optional qualification logics. VirtualData is also faster in removing and
inserting data compared to virtual array.

Duplicate column is not allowed in virtual data, use virtual array instead.

```rust
use dcsv::{Reader, VirtualData, Value};
use std::fs::File;

let mut data: VirtualData = Reader::new()
    .use_delimiter(';')      // Default is comma
    .use_line_delimiter('|') // Default is '\n, \r\n'
    .data_from_stream(
        BufReader::new(
            File::open("file_name.csv")
                .expect("Failed to read file")
        )
    )
    .expect("Failed to retrieve csv value from file");

let value : Option<&str> = data.get_cell(1,1);

data.set_row(data.get_row_count(), vec![Value::Text("abc".to_string())])
	.expect("Failed to set data into a row");

data.set_column("dead", Value::Text(String::from("false")))
	.expect("Failed to set values in column");

let (x,y) = (0,0);
data.set_cell(x, y, Value::Text(String::from("WOW")))
	.expect("Failed to set value to a given cell");

let (src, target) = (0, data.get_row_count() - 1);
data.move_row(src, target)
	.expect("Failed to relocate row");

let (src, target) = (0, data.get_column_count() - 1);
data.move_column(src, target)
	.expect("Failed to relocate a column");
```

# VirtualArray

VirtualArray is a simple wrapper around vectors of string arrays. ( Row:
Vec<Vec<String>> ). VirtualArray is faster in indexing and allows
duplicate column names.

You cannot access column or row values with column name but always with index.

```rust

use dcsv::{Reader, VirtualArray, Value};
use std::fs::File;

let mut data: VirtualArray = Reader::new()
    .use_delimiter(';')      // Default is comma
    .use_line_delimiter('|') // Default is '\n, \r\n'
    .array_from_stream(
        BufReader::new(
            File::open("file_name.csv")
                .expect("Failed to read file")
        )
    )
    .expect("Failed to retrieve csv value from file");

let value : Option<&Value> = data.get_cell(1,1);

data.set_row(data.get_row_count(), vec![Value::Text("abc".to_string())])
	.expect("Failed to set data into a row");

data.set_column(2, Value::Text(String::from("false")))
	.expect("Failed to set values in column");

let (x,y) = (0,0);
data.set_cell(x, y, Value::Text(String::from("WOW")))
	.expect("Failed to set value to a given cell");

let (src, target) = (0, data.get_row_count() - 1);
data.move_row(src, target)
	.expect("Failed to relocate row");

let (src, target) = (0, data.get_column_count() - 1);
data.move_column(src, target)
	.expect("Failed to relocate a column");
```
