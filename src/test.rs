#[cfg(test)]
mod testos {
    use crate::Reader;

    #[test]
    fn read_csv() {
        let csv = "a,b,c
1,2,3";
        let data_0 = Reader::new()
            .trim(true)
            .custom_header(&["first", "second", "third"])
            .read_from_stream(csv.as_bytes())
            .expect("Failed to read");
        println!("{}", data_0);
    }
}
