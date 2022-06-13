use crate::error::{DcsvError, DcsvResult};
use crate::parser::Parser;
use crate::utils::ALPHABET;
use crate::value::{Value, ValueType};
use crate::virtual_data::VirtualData;
use std::io::BufRead;

/// Csv Reader
///
/// User can set various reader option to configure a reading behaviour.
/// Reader's options are not dropped after a read but persists for reader's lifetime.
pub struct Reader {
    option: ReaderOption,
    parser: Parser,
    pub data: VirtualData,
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
            data: VirtualData::new(),
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
    /// This return read value as virtual data struct
    pub fn read_from_stream(&mut self, mut csv_stream: impl BufRead) -> DcsvResult<VirtualData> {
        let mut row_buffer: Vec<u8> = vec![];
        let line_delimiter = self.option.line_delimiter.unwrap_or('\n') as u8;
        self.parser.reset();

        let mut num_bytes = csv_stream
            .read_until(line_delimiter, &mut row_buffer)
            .expect("Failed to read until");
        let mut row_count = 1;
        while num_bytes != 0 {
            // Create column
            // Create row or continue to next line.
            let row = self.parser.feed_chunk(
                std::mem::take(&mut row_buffer),
                self.option.delimiter,
                self.option.consume_dquote,
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
                if self.data.get_column_count() == 0 {
                    if !self.option.custom_header.is_empty() {
                        if self.option.custom_header.len() != row.len() {
                            return Err(DcsvError::InvalidColumn(format!(
                                "Custom value has different length. Given {} but needs {}",
                                self.option.custom_header.len(),
                                row.len()
                            )));
                        }
                        let header = std::mem::take(&mut self.option.custom_header);
                        self.add_multiple_columns(&header)?;
                    } else if self.option.read_header {
                        if self.option.trim {
                            self.add_multiple_columns_ref(
                                &row.iter().map(|s| s.trim()).collect::<Vec<_>>(),
                            )?;
                        } else {
                            self.add_multiple_columns(&row)?;
                        }
                        row_count += 1;
                        num_bytes = csv_stream
                            .read_until(line_delimiter, &mut row_buffer)
                            .expect("Failed to read until");
                        continue;
                    } else {
                        // Create a header
                        self.add_multiple_columns(&self.make_arbitrary_column(row.len()))?;
                    }
                }

                // Given row data has different length with column
                if row.len() != self.data.get_column_count() {
                    self.data.drop_data();
                    return Err(DcsvError::InvalidRowData(format!(
                        "Row of line \"{}\" has different length.",
                        row_count
                    )));
                }

                if self.option.trim {
                    self.add_row_fast_ref(&row.iter().map(|s| s.trim()).collect::<Vec<_>>())?;
                } else {
                    // Add as new row and proceed
                    self.add_row_fast(&row)?;
                }
            }

            // advance row
            row_count += 1;
            num_bytes = csv_stream
                .read_until(line_delimiter, &mut row_buffer)
                .expect("Failed to read until");
        }

        // complete move data as return value to comply borrow rules
        Ok(std::mem::replace(&mut self.data, VirtualData::new()))
    }

    // -----
    // <DRY>
    // DRY Codes
    fn add_row_fast<T: AsRef<str>>(&mut self, row: &[T]) -> DcsvResult<()> {
        self.data.insert_row(
            self.data.get_row_count(),
            Some(
                &row.iter()
                    .map(|val| Value::Text(val.as_ref().to_string()))
                    .collect::<Vec<_>>(),
            ),
        )?;
        Ok(())
    }

    fn add_row_fast_ref<T: AsRef<str>>(&mut self, row: &[T]) -> DcsvResult<()> {
        self.data.insert_row(
            self.data.get_row_count(),
            Some(
                &row.iter()
                    .map(|val| Value::Text(val.as_ref().to_string()))
                    .collect::<Vec<_>>(),
            ),
        )?;
        Ok(())
    }

    fn add_multiple_columns<T: AsRef<str>>(&mut self, column_names: &[T]) -> DcsvResult<()> {
        for (idx, col) in column_names.iter().enumerate() {
            self.data
                .insert_column(idx, col.as_ref(), ValueType::Text, None, None)?;
        }
        Ok(())
    }

    fn add_multiple_columns_ref<T: AsRef<str>>(&mut self, column_names: &[T]) -> DcsvResult<()> {
        for (idx, col) in column_names.iter().enumerate() {
            self.data
                .insert_column(idx, col.as_ref(), ValueType::Text, None, None)?;
        }
        Ok(())
    }

    fn make_arbitrary_column(&self, size: usize) -> Vec<String> {
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
}

/// Reader behaviour related options
pub struct ReaderOption {
    pub trim: bool,
    pub read_header: bool,
    pub consume_dquote: bool,
    pub custom_header: Vec<String>,
    pub delimiter: Option<char>,
    pub line_delimiter: Option<char>,
    pub ignore_empty_row: bool,
}

impl Default for ReaderOption {
    fn default() -> Self {
        Self::new()
    }
}

impl ReaderOption {
    pub fn new() -> Self {
        Self {
            trim: false,
            read_header: true,
            custom_header: vec![],
            consume_dquote: false,
            delimiter: None,
            line_delimiter: None,
            ignore_empty_row: false,
        }
    }
}
