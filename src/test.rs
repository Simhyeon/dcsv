#[cfg(test)]
mod testos {
    use std::{fs::File, io::BufReader};

    use crate::{Reader, ReaderOption};

    #[test]
    fn read_csv() {
        let csv =
            BufReader::new(File::open("/home/simon/misc/csv_samples/biostats.csv").expect(""));
        let data_0 = Reader::new()
            .trim(true)
            .ignore_empty_row(true)
            .consume_dquote(true)
            .read_from_stream(csv)
            .expect("Failed to read");
        println!("{}", data_0);
    }
}
