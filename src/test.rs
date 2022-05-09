#[cfg(test)]
mod test {
    use crate::Reader;
    #[test]
    fn read_csv() {
        let csv_1 = r#"a,b,c,d
1,2,3,4"#;
        let csv_2 = r#"a|b|c|d
1|2|3|4"#;
        let csv_3 = r#"a|b|c|d;1|2|3|4"#;

        let data = Reader::new().read_from_stream(csv_1.as_bytes()).expect("Failed to read");
        println!("{}", data);
        let data = Reader::new().use_delimiter('|').read_from_stream(csv_2.as_bytes()).expect("Failed to read");
        println!("{}", data);
        let data = Reader::new().use_delimiter('|').use_line_delimiter(';').read_from_stream(csv_3.as_bytes()).expect("Failed to read");
        println!("{}", data);
    }
}
