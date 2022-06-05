#[cfg(test)]
mod test {
    use crate::Reader;

    //#[test]
    //fn split() {
    //let test = "1,2,3,4,5\r\n6,7,8,9,10";
    //let arr: Vec<_> = test.split("\n").collect();
    //println!("{arr:#?}");
    //}

    #[test]
    fn read_csv() {
        let csv = "";
        let csv_1 = r#" a , b , c ,"d,
1"
  1  ,  2  ,  3  ,  4  "#;
        let csv_2 = "a,b,c,d\r\n1,2,3,4\r\n";
        let csv_3 = r#"a|b|c|d;1|2|3|4"#;
        let data_0 = Reader::new()
            .trim(true)
            .read_from_stream(csv.as_bytes())
            .expect("Failed to read");
        println!("00 : {}",data_0);
        println!("--{:?}--", data_0.read_only_ref());
        let data = Reader::new()
            .trim(true)
            .read_from_stream(csv_1.as_bytes())
            .expect("Failed to read");
        println!("{data}");
        let data = Reader::new()
            .ignore_empty_row(true)
            .read_from_stream(csv_2.as_bytes())
            .expect("Failed to read");
        println!("222\n{}", data);
        //let data = Reader::new().use_delimiter('|').use_line_delimiter(';').read_from_stream(csv_3.as_bytes()).expect("Failed to read");
        //println!("{}", data);
    }
}
