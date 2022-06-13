use crate::error::DcsvResult;

/// Csv line parser
pub(crate) struct Parser {
    pub(crate) container: Vec<String>,
    pub(crate) remnant: String,
    pub(crate) on_quote: bool,
    pub(crate) line_delimiter: Option<char>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            container: vec![],
            remnant: String::new(),
            on_quote: false,
            line_delimiter: None,
        }
    }

    pub fn reset(&mut self) {
        self.container.clear();
        self.remnant = String::new();
        self.on_quote = false;
    }

    /// Feed chunk to parser
    ///
    /// This will return Some when chunk composes a fully line.
    /// A complete line is deteced when line_delimiter is met.
    pub fn feed_chunk(
        &mut self,
        chunk: Vec<u8>,
        delim: Option<char>,
        consume_dquote: bool,
    ) -> DcsvResult<Option<Vec<String>>> {
        let line = String::from_utf8(chunk)
            .expect("Failed to convert to string")
            .replace("\r\n", "\n");
        let mut previous = '0';
        let mut value = std::mem::take(&mut self.remnant);
        let mut iter = line.chars().peekable();
        while let Some(ch) = iter.next() {
            match ch {
                _ if ch == delim.unwrap_or(',') => {
                    if !self.on_quote {
                        let flushed = std::mem::take(&mut value);
                        self.container.push(flushed);
                        previous = ch;
                        continue;
                    }
                }
                '"' => {
                    // Add literal double quote if previous was same character
                    if previous == '"' {
                        previous = ' '; // Reset previous
                    } else {
                        if let Some('"') = iter.peek() {
                        } else {
                            self.on_quote = !self.on_quote;
                        }
                        previous = ch;
                        if consume_dquote {
                            continue;
                        }
                    }
                }
                _ => previous = ch,
            }
            value.push(ch);
        }

        // Unterminated quote should not return container
        if self.on_quote {
            self.remnant = value;
            Ok(None)
        } else {
            // If there is yet flushed value, add to container
            if !value.is_empty() {
                // Middle row
                if let Some(stripped) = value.strip_suffix(self.line_delimiter.unwrap_or('\n')) {
                    self.container.push(stripped.to_owned());
                }
                // Last row might not have line separator
                else {
                    self.container.push(value);
                }
            }
            Ok(Some(std::mem::take(&mut self.container)))
        }
    }
}
