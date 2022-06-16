/// # Dynamic csv manipulation library
///
/// Dcsv is a dynamic csv container library which offers reading and writing features.
///
/// # Basic
///
/// Dcsv has two major structs of Reader and VirtualData. Reader reads csv data as byte stream and
/// return virtual data. Changes to virtual data is not linked to original source. User needs to
/// save virtual data to desired destination.
///
/// If you want static form of data, use read_only_ref method to get data as records form.
///
/// ## Usage
///
/// ```rust
/// use dcsv::{Reader, VirtualData, Value};
/// use std::io::BufReader;
/// use std::fs::File;
///
/// let data: VirtualData = Reader::new()
///     .use_delimiter(';')      // Default is comma
///     .use_line_delimiter('|') // Default is '\n, \r\n'
///     .read_from_stream(
///         BufReader::new(
///             File::open("file_name.csv")
///                 .expect("Failed to read file")
///         )
///     )
///     .expect("Failed to retrieve csv value from file");
///
/// // Refer docs.rs for various VirtualData methods
/// let value : Option<&Value> = data.get_cell(1,1).expect("Failed to get cell");
mod error;
mod parser;
mod reader;
mod test;
pub mod utils;
mod value;
mod virtual_array;
mod virtual_data;

pub use error::{DcsvError, DcsvResult};
pub use reader::{Reader, ReaderOption};

pub use value::LIMITER_ATTRIBUTE_LEN;
pub use virtual_data::SCHEMA_HEADER;

pub use value::{Value, ValueLimiter, ValueType};
pub use virtual_array::VirtualArray;
pub use virtual_data::{Column, ReadOnlyData, ReadOnlyDataRef, Row, VirtualData};
