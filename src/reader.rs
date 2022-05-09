use crate::value::{ValueType, Value};
use crate::virtual_data::VirtualData;
use crate::error::{DcsvResult, DcsvError};
use std::io::BufRead;
use crate::utils::{self, ALPHABET};

pub struct Reader {
    read_header: bool,
    delimiter : Option<char>,
    line_delimiter: Option<char>,
    pub data : VirtualData,
}

impl Reader {
    pub fn new() -> Self {
        Self {
            read_header: true,
            delimiter : None,
            line_delimiter : None,
            data: VirtualData::new(),
        }
    }

    /// Read csv value from buf read stream
    ///
    /// This return read value as virtual data struct
    pub fn read_from_stream(&mut self, csv_stream: impl BufRead) -> DcsvResult<VirtualData> {
        let mut csv_lines = csv_stream.split(self.line_delimiter.unwrap_or('\n') as u8);

        let mut row_count = 1;
        if self.read_header {
            let header = csv_lines.next();
            if let None = header {
                return Err(DcsvError::InvalidRowData(format!(
                            "Given data does not have a header"
                )));
            }
            let header = utils::csv_row_from_split(header.as_ref(), self.delimiter)?.ok_or(DcsvError::InvalidRowData("Given row data is not valid".to_string()))?;
            println!("HEADER : {:?}", header);
            self.add_multiple_columns(&header)?;
        }

        let mut row = csv_lines.next();
        while let Some(_) = &row {
            let split = utils::csv_row_from_split(row.as_ref(), self.delimiter)?.ok_or(DcsvError::InvalidRowData("Given row data is not valid".to_string()))?;

            // No column data, add arbitrary data
            // This has to be inside row read loop 
            // because column should have same length with row
            // and it is impossible to deduce the length of row before reading it.
            if !self.read_header {
                self.add_multiple_columns(&self.make_arbitrary_column(split.len()))?;
            }

            // Given row data has different length with column
            if split.len() != self.data.get_column_count() {
                self.data.drop();
                return Err(DcsvError::InvalidRowData(format!(
                            "Row of line \"{}\" has different length.",
                            row_count + 1
                )));
            }

            self.add_row_fast(&split)?;
            row = csv_lines.next();
            row_count += 1;
        }

        Ok(std::mem::replace(&mut self.data, VirtualData::new()))
    }

    /// Use given delimiter instead of default one : ",".
    pub fn use_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter.replace(delimiter);
        self
    }

    /// Use line delimiter instead of default one : "\n".
    pub fn use_line_delimiter(mut self, delimiter: char) -> Self {
        self.line_delimiter.replace(delimiter);
        self
    }

    // <DRY>
    // DRY Codes

    fn add_row_fast(&mut self, row: &Vec<String>) -> DcsvResult<()> {
        self.data.insert_row(self.data.get_row_count(), Some(&row.iter().map(|val| Value::Text(val.to_string())).collect::<Vec<_>>()))?;
        Ok(())
    }

    fn add_multiple_columns(&mut self, column_names : &Vec<String>) -> DcsvResult<()> {
        for (idx, col) in column_names.iter().enumerate() {
            self.data.insert_column(
                idx,
                col,
                ValueType::Text,
                None, 
                None
            )?;
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
