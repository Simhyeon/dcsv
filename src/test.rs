/// Test multiple csv formats

#[cfg(test)]
mod testos {
    use crate::{DcsvResult, Reader};
    use std::io::BufRead;

    #[test]
    fn read_csv() -> DcsvResult<()> {
        let files =
            std::io::BufReader::new(std::fs::File::open("test_files").expect("Failed wow..."));
        for line in files.lines() {
            let line = line.expect("Wowzer");
            Reader::new().data_from_stream(&*std::fs::read(line).expect("Welp"))?;
        }

        // Reader specific reads
        // Old MacOS LF Lined ending file
        Reader::new()
            .ignore_empty_row(true)
            .use_line_delimiter('\r')
            .data_from_stream(&*std::fs::read("test_src/lf_line_ending.csv").expect("Welp"))?;

        // Trailing newline file
        Reader::new().ignore_empty_row(true).data_from_stream(
            &*std::fs::read("test_src/trailing_nl_with_dquotes.csv").expect("Welp"),
        )?;

        Ok(())
    }
}
