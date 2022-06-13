use crate::error::DcsvResult;

pub(crate) const ALPHABET: [&str; 26] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z",
];

/// Try getting csv row from split iterator
///
/// This will retur None when fails to get csv row
pub fn csv_row_from_split(
    split: Option<&std::io::Result<Vec<u8>>>,
    delimiter: Option<char>,
) -> DcsvResult<Option<Vec<String>>> {
    let split = split
        .map(|value| {
            if let Ok(value) = value {
                let src = std::str::from_utf8(value);
                match src {
                    Err(_) => None,
                    Ok(src) => Some(csv_row_to_vector(src, delimiter)),
                }
            } else {
                None
            }
        })
        .unwrap_or(None);
    Ok(split)
}

/// Split csv row into a vector of string
pub fn csv_row_to_vector(line: &str, delimiter: Option<char>) -> Vec<String> {
    let mut split = vec![];
    let mut on_quote = false;
    let mut previous = ' ';
    let mut chunk = String::new();
    let mut iter = line.chars().peekable();
    while let Some(ch) = iter.next() {
        match ch {
            '"' => {
                // Add literal double quote if previous was same character
                if previous == '"' {
                    previous = ' '; // Reset previous
                } else {
                    if let Some('"') = iter.peek() {
                    } else {
                        on_quote = !on_quote;
                    }
                    previous = ch;
                }
            }
            // This looks verbose but needs match guard
            // because match pattern doesn't work like what starters think
            _ if ch == delimiter.unwrap_or(',') => {
                if !on_quote {
                    let flushed = std::mem::take(&mut chunk);
                    split.push(flushed);
                    previous = ch;
                    continue;
                }
            }
            _ => previous = ch,
        }
        chunk.push(ch);
    }
    split.push(chunk);
    split
}
