/// Reader reads and parses given string into a csv struct
///
/// You can also configure reader with multiple builder methods
use crate::error::{DcsvError, DcsvResult};
use crate::parser::Parser;
use crate::utils::ALPHABET;
use crate::value::Value;
use crate::virtual_data::VirtualData;
use crate::{Column, VCont, VirtualArray};
use std::io::BufRead;

/// Csv Reader
///
/// User can set various reader option to configure a reading behaviour.
/// Reader's options are not dropped after a read but persists for reader's lifetime.
///
/// # Usage
///
/// ```rust
/// use dcsv::Reader;
///
/// let csv_value = "a,b,c
/// 1,2,3";
///
/// let data = Reader::new()
///    .trim(true)
///    .ignore_empty_row(true)
///    .has_header(true)
///    .data_from_stream(csv_value.as_bytes());
/// ```
pub struct Reader {
    option: ReaderOption,
    parser: Parser,
}

impl Default for Reader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader {
    pub fn new() -> Self {
        Self {
            option: ReaderOption::new(),
            parser: Parser::new(),
        }
    }

    /// Build with reader option
    pub fn with_option(mut self, option: ReaderOption) -> Self {
        self.option = option;
        self
    }

    /// Consumes double quote in csv file
    pub fn consume_dquote(mut self, tv: bool) -> Self {
        self.option.consume_dquote = tv;
        self
    }

    /// Ignore empty rows
    ///
    /// This prevents reader from panicking on empty row.
    pub fn ignore_empty_row(mut self, tv: bool) -> Self {
        self.option.ignore_empty_row = tv;
        self
    }

    /// Trim all read values
    pub fn trim(mut self, tv: bool) -> Self {
        self.option.trim = tv;
        self
    }

    /// Allow invalid string while parsing csv values
    pub fn allow_invalid_string(mut self, allow: bool) -> Self {
        self.option.allow_invalid_string = allow;
        self
    }

    /// Whether csv data has header or not
    pub fn has_header(mut self, has_header: bool) -> Self {
        self.option.read_header = has_header;
        self
    }

    /// Set custom header
    ///
    /// This will override "has_header" option and create header from given values.
    pub fn custom_header<T: AsRef<str>>(mut self, headers: &[T]) -> Self {
        self.option.custom_header = headers.iter().map(|s| s.as_ref().to_owned()).collect();
        self
    }

    /// Clear reader option and set to default
    pub fn clear_reader_option(&mut self) {
        self.option = ReaderOption::new();
    }

    /// Use given delimiter instead of default one : ",".
    pub fn use_delimiter(mut self, delimiter: char) -> Self {
        self.option.delimiter.replace(delimiter);
        self
    }

    /// Use given line delimiter instead of default one : "\n, \r\n".
    ///
    /// Only default state will detect both "\n" and "\r\n". If you set "\n" manually, "\r\n" will
    /// be ignored.
    pub fn use_line_delimiter(mut self, delimiter: char) -> Self {
        self.parser.line_delimiter.replace(delimiter);
        self.option.line_delimiter.replace(delimiter);
        self
    }

    /// Read csv value from buf read stream
    ///
    /// This returns read value as virtual data struct
    pub fn data_from_stream(&mut self, mut csv_stream: impl BufRead) -> DcsvResult<VirtualData> {
        let mut row_buffer: Vec<u8> = vec![];
        let line_delimiter = self.option.line_delimiter.unwrap_or('\n') as u8;
        self.parser.reset();

        let mut num_bytes = csv_stream
            .read_until(line_delimiter, &mut row_buffer)
            .expect("Failed to read until");
        let mut data = VirtualData::new();
        let mut row_count = 1;
        while num_bytes != 0 {
            // Create column
            // Create row or continue to next line.
            let row = self.parser.feed_chunk(
                std::mem::take(&mut row_buffer),
                self.option.delimiter,
                self.option.consume_dquote,
                self.option.allow_invalid_string,
            )?;

            // Row has been detected
            if let Some(row) = row {
                // This is a trailing value after new line
                // Simply break
                if row.len() == 1 && row[0].trim().is_empty() {
                    // go to next line
                    if self.option.ignore_empty_row {
                        num_bytes = csv_stream
                            .read_until(line_delimiter, &mut row_buffer)
                            .expect("Failed to read until");
                        row_count += 1;
                        continue;
                    } else {
                        return Err(DcsvError::InvalidRowData(format!(
                                    "Row of line \"{}\" has empty row. Which is unallowed by reader option.",
                                    row_count + 1
                        )));
                    }
                }

                // Add column header if column is empty
                if data.get_column_count() == 0 {
                    if !self.option.custom_header.is_empty() {
                        if self.option.custom_header.len() != row.len() {
                            return Err(DcsvError::InvalidColumn(format!(
                                "Custom value has different length. Given {} but needs {}",
                                self.option.custom_header.len(),
                                row.len()
                            )));
                        }
                        let header = std::mem::take(&mut self.option.custom_header);
                        add_multiple_columns(&mut data, &header)?;
                    } else if self.option.read_header {
                        if self.option.trim {
                            // Trim row
                            add_multiple_columns(
                                &mut data,
                                &row.iter().map(|s| s.trim().to_owned()).collect::<Vec<_>>(),
                            )?;
                        } else {
                            // Don't trim
                            add_multiple_columns(&mut data, &row)?;
                        }
                        row_count += 1;
                        num_bytes = csv_stream
                            .read_until(line_delimiter, &mut row_buffer)
                            .expect("Failed to read until");
                        continue;
                    } else {
                        // Create a header
                        add_multiple_columns(&mut data, &make_arbitrary_column(row.len()))?;
                    }
                }

                // Given row data has different length with column
                if row.len() != data.get_column_count() {
                    data.drop_data();
                    return Err(DcsvError::InvalidRowData(format!(
                        "Row of line \"{}\" has different length.",
                        row_count
                    )));
                }

                if self.option.trim {
                    add_data_row(
                        &mut data,
                        row.iter().map(|s| s.trim().to_string()).collect::<Vec<_>>(),
                    )?;
                } else {
                    // Add as new row and proceed
                    add_data_row(&mut data, row)?;
                }
            }

            // advance row
            row_count += 1;
            num_bytes = csv_stream
                .read_until(line_delimiter, &mut row_buffer)
                .expect("Failed to read until");
        }

        Ok(data)
    }

    /// Read csv value from buf read stream
    ///
    /// This returns read value as virtual array struct
    pub fn array_from_stream(&mut self, mut csv_stream: impl BufRead) -> DcsvResult<VirtualArray> {
        let mut row_buffer: Vec<u8> = vec![];
        let line_delimiter = self.option.line_delimiter.unwrap_or('\n') as u8;
        self.parser.reset();

        let mut num_bytes = csv_stream
            .read_until(line_delimiter, &mut row_buffer)
            .expect("Failed to read until");
        let mut data = VirtualArray::new();
        let mut row_count = 1;
        while num_bytes != 0 {
            // Create column
            // Create row or continue to next line.
            let row = self.parser.feed_chunk(
                std::mem::take(&mut row_buffer),
                self.option.delimiter,
                self.option.consume_dquote,
                self.option.allow_invalid_string,
            )?;

            // Row has been detected
            if let Some(row) = row {
                // This is a trailing value after new line
                // Simply break
                if row.len() == 1 && row[0].trim().is_empty() {
                    // go to next line
                    if self.option.ignore_empty_row {
                        num_bytes = csv_stream
                            .read_until(line_delimiter, &mut row_buffer)
                            .expect("Failed to read until");
                        row_count += 1;
                        continue;
                    } else {
                        return Err(DcsvError::InvalidRowData(format!(
                                    "Row of line \"{}\" has empty row. Which is unallowed by reader option.",
                                    row_count + 1
                        )));
                    }
                }

                // Add column header if column is empty
                if data.get_column_count() == 0 {
                    if !self.option.custom_header.is_empty() {
                        if self.option.custom_header.len() != row.len() {
                            return Err(DcsvError::InvalidColumn(format!(
                                "Custom value has different length. Given {} but needs {}",
                                self.option.custom_header.len(),
                                row.len()
                            )));
                        }
                        let header = std::mem::take(&mut self.option.custom_header);
                        data.columns = header.iter().map(|h| Column::empty(h)).collect::<Vec<_>>();
                    } else if self.option.read_header {
                        if self.option.trim {
                            data.columns = row
                                .iter()
                                .map(|s| Column::empty(s.trim()))
                                .collect::<Vec<_>>();
                        } else {
                            data.columns = row.iter().map(|h| Column::empty(h)).collect::<Vec<_>>();
                        }
                        row_count += 1;
                        num_bytes = csv_stream
                            .read_until(line_delimiter, &mut row_buffer)
                            .expect("Failed to read until");
                        continue;
                    } else {
                        // Create a header
                        data.columns = make_arbitrary_column(row.len())
                            .iter()
                            .map(|h| Column::empty(h))
                            .collect::<Vec<_>>();
                    }
                }

                // Given row data has different length with column
                if row.len() != data.get_column_count() {
                    data.drop_data();
                    return Err(DcsvError::InvalidRowData(format!(
                        "Row of line \"{}\" has different length.",
                        row_count
                    )));
                }

                if self.option.trim {
                    add_array_row(
                        &mut data,
                        row.iter().map(|s| s.trim().to_owned()).collect::<Vec<_>>(),
                    )?;
                } else {
                    // Add as new row and proceed
                    add_array_row(&mut data, row)?;
                }
            }

            // advance row
            row_count += 1;
            num_bytes = csv_stream
                .read_until(line_delimiter, &mut row_buffer)
                .expect("Failed to read until");
        }

        Ok(data)
    }
}

// -----
// <DRY>
// DRY Codes
/// add new data row into a virtual data
fn add_data_row(data: &mut VirtualData, mut row: Vec<String>) -> DcsvResult<()> {
    data.insert_row(
        data.get_row_count(),
        Some(
            &row.iter_mut()
                .map(|val| Value::Text(std::mem::take(val)))
                .collect::<Vec<_>>(),
        ),
    )?;
    Ok(())
}

/// add new data row into a virtual array
fn add_array_row(data: &mut VirtualArray, mut row: Vec<String>) -> DcsvResult<()> {
    data.insert_row(
        data.get_row_count(),
        Some(
            &row.iter_mut()
                .map(|val| Value::Text(std::mem::take(val)))
                .collect::<Vec<_>>(),
        ),
    )?;
    Ok(())
}

/// Add multiple columns with given names
fn add_multiple_columns(data: &mut VirtualData, column_names: &[String]) -> DcsvResult<()> {
    for (idx, col) in column_names.iter().enumerate() {
        data.insert_column(idx, col.as_ref())?;
    }
    Ok(())
}

/// Create arbitrary column names
fn make_arbitrary_column(size: usize) -> Vec<String> {
    let mut column_names: Vec<String> = vec![];
    for index in 0..size {
        let index = index + 1;
        let target = ALPHABET[index % ALPHABET.len() - 1];
        let name = target.repeat(index / ALPHABET.len() + 1);
        column_names.push(name);
    }
    column_names
}
// </DRY>
// -----

/// Reader behaviour related options
pub struct ReaderOption {
    pub trim: bool,
    pub read_header: bool,
    pub consume_dquote: bool,
    pub custom_header: Vec<String>,
    pub delimiter: Option<char>,
    pub line_delimiter: Option<char>,
    pub ignore_empty_row: bool,
    pub allow_invalid_string: bool,
}

impl Default for ReaderOption {
    fn default() -> Self {
        Self::new()
    }
}

impl ReaderOption {
    /// Constructor
    pub fn new() -> Self {
        Self {
            trim: false,
            read_header: true,
            custom_header: vec![],
            consume_dquote: false,
            delimiter: None,
            line_delimiter: None,
            ignore_empty_row: false,
            allow_invalid_string: false,
        }
    }
}
