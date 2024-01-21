//! Test multiple csv formats

/// Simple test module
#[cfg(test)]
mod testos {
    use crate::{DcsvResult, Reader, VCont};
    use std::io::BufRead;

    /// Read multiple csv files and check if panics
    #[test]
    fn read_csv() -> DcsvResult<()> {
        use std::io::Write;
        let files =
            std::io::BufReader::new(std::fs::File::open("test_files").expect("Failed wow..."));
        for line in files.lines() {
            let line = line.expect("Wowzer");
            Reader::new().data_from_stream(&*std::fs::read(line).expect("Welp"))?;
        }

        let data = Reader::new()
            .use_space_delimiter(true)
            .data_from_stream(&*std::fs::read("test_src/r4d.csv").expect("Failed"))?;
        writeln!(
            std::io::stdout(),
            "{}",
            data.get_formatted_string("\n", crate::CellAlignType::Center)
        )
        .expect("FAIL");

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
