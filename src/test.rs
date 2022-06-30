#[cfg(test)]
mod testos {
    use std::{fs::File, io::BufReader};

    use crate::{DcsvResult, Reader, ReaderOption, VCont, Value, VirtualArray, VirtualData};

    #[test]
    fn read_csv() -> DcsvResult<()> {
        let csv_value = "a,b,c
b,2,4
A,,
B,,";

        let mut data: VirtualData = Reader::new()
            .data_from_stream(csv_value.as_bytes())
            .expect("Failed to retrieve csv value from file");

        data.apply_all(|value| {
            if let Value::Text(v) = value {
                v.push('~');
            }
        });

        println!("{}", data);

        // Refer docs.rs for various VirtualData methods
        let value: Option<&Value> = data.get_cell(1, 1);

        for (idx, value) in data.get_iterator().enumerate() {
            println!("{} - {}", idx, value);
        }

        for (idx, value) in data.get_row_iterator(2)?.enumerate() {
            println!("{} - {}", idx, value);
        }
        Ok(())
    }
}
