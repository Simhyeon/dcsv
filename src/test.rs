#[cfg(test)]
mod testos {
    use std::{fs::File, io::BufReader};

    use crate::{DcsvResult, Reader, ReaderOption, Value, VirtualArray};

    #[test]
    fn read_csv() -> DcsvResult<()> {
        let csv_value = "a,a,a
b,2,4
A,,
B,,";

        let mut data: VirtualArray = Reader::new()
            .use_delimiter(';') // Default is comma
            .use_line_delimiter('|') // Default is '\n, \r\n'
            .array_from_stream(BufReader::new(
                File::open("file_name.csv").expect("Failed to read file"),
            ))
            .expect("Failed to retrieve csv value from file");

        let value: &str = data.get_cell(1, 1).expect("Failed to get cell");

        data.set_row(data.get_row_count(), &["abc"])
            .expect("Failed to set data into a row");

        data.set_column(2, "false")
            .expect("Failed to set values in column");

        let (x, y) = (0, 0);
        data.set_cell(x, y, "WOW")
            .expect("Failed to set value to a given cell");

        let (src, target) = (0, data.get_row_count() - 1);
        data.move_row(src, target).expect("Failed to relocate row");

        let (src, target) = (0, data.get_column_count() - 1);
        data.move_column(src, target)
            .expect("Failed to relocate a column");

        let data: crate::VirtualData = Reader::new()
            .use_delimiter(';') // Default is comma
            .use_line_delimiter('|') // Default is '\n, \r\n'
            .data_from_stream(BufReader::new(
                File::open("file_name.csv").expect("Failed to read file"),
            ))
            .expect("Failed to retrieve csv value from file");

        // Refer docs.rs for various VirtualData methods
        let value: Option<&Value> = data.get_cell(1, 1);
        Ok(())
    }
}
