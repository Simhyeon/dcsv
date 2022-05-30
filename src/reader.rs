use crate::error::{DcsvError, DcsvResult};
use crate::parser::Parser;
use crate::utils::{self, ALPHABET};
use crate::value::{Value, ValueType};
use crate::virtual_data::VirtualData;
use std::io::BufRead;

pub struct Reader {
    option: ReaderOption,
    pub data: VirtualData,
}

impl Reader {
    pub fn new() -> Self {
        Self {
            option: ReaderOption::new(),
            data: VirtualData::new(),
        }
    }

    /// Ignore empty row
    ///
    /// This prevents reader from panicking on empty row.
    pub fn ignore_empty_row(mut self, tv: bool) -> Self {
        self.option.ignore_empty_row = tv;
        self
    }

    /// Whether csv data has header or not
    pub fn has_header(mut self, has_header: bool) -> Self {
        self.option.read_header = has_header;
        self
    }

    /// Read csv value from buf read stream
    ///
    /// This return read value as virtual data struct
    pub fn read_from_stream(&mut self, mut csv_stream: impl BufRead) -> DcsvResult<VirtualData> {
        let mut row_buffer: Vec<u8> = vec![];
        let line_delimiter = self.option.line_delimiter.unwrap_or('\n') as u8;
        let mut parser = Parser::new();

        let mut num_bytes = csv_stream
            .read_until(line_delimiter, &mut row_buffer)
            .expect("Failed to read until");
        let mut row_count = 1;
        while num_bytes != 0 {
            // Create column
            // Create row or continue to next line.
            let row = parser.feed_chunk(
                std::mem::replace(&mut row_buffer, vec![]),
                self.option.delimiter,
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
                    if self.option.read_header {
                        self.add_multiple_columns(&row)?;
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
                    self.data.drop();
                    return Err(DcsvError::InvalidRowData(format!(
                        "Row of line \"{}\" has different length.",
                        row_count + 1
                    )));
                }

                // Add as new row and proceed
                self.add_row_fast(&row)?;
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

    /// Use given delimiter instead of default one : ",".
    pub fn use_delimiter(mut self, delimiter: char) -> Self {
        self.option.delimiter.replace(delimiter);
        self
    }

    /// Use line delimiter instead of default one : "\n".
    pub fn use_line_delimiter(mut self, delimiter: char) -> Self {
        self.option.line_delimiter.replace(delimiter);
        self
    }

    // <DRY>
    // DRY Codes

    fn add_row_fast(&mut self, row: &Vec<String>) -> DcsvResult<()> {
        self.data.insert_row(
            self.data.get_row_count(),
            Some(
                &row.iter()
                    .map(|val| Value::Text(val.to_string()))
                    .collect::<Vec<_>>(),
            ),
        )?;
        Ok(())
    }

    fn add_multiple_columns(&mut self, column_names: &Vec<String>) -> DcsvResult<()> {
        for (idx, col) in column_names.iter().enumerate() {
            self.data
                .insert_column(idx, col, ValueType::Text, None, None)?;
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
}

/// Reader behaviour related options
pub(crate) struct ReaderOption {
    pub(crate) read_header: bool,
    pub(crate) delimiter: Option<char>,
    pub(crate) line_delimiter: Option<char>,
    pub(crate) ignore_empty_row: bool,
}

impl ReaderOption {
    pub fn new() -> Self {
        Self {
            read_header: true,
            delimiter: None,
            line_delimiter: None,
            ignore_empty_row: false,
        }
    }
}
