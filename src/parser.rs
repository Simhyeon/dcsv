//! CSV parser

use crate::error::DcsvResult;

/// CSV line parser
pub(crate) struct Parser {
    pub(crate) container: Vec<String>,
    pub(crate) remnant: String,
    pub(crate) on_quote: bool,
    pub(crate) line_delimiter: Option<char>,
}

impl Parser {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            container: vec![],
            remnant: String::new(),
            on_quote: false,
            line_delimiter: None,
        }
    }

    /// Reset parser states
    pub fn reset(&mut self) {
        self.container.clear();
        self.remnant = String::new();
        self.on_quote = false;
    }

    /// Feed chunk to parser
    ///
    /// This will return Some when chunk composes a fully line.
    /// A complete line is deteced when line_delimiter is met.
    ///
    /// Keep in mind that csv value might have a line delimiter other than a
    /// newline
    pub fn feed_chunk(
        &mut self,
        chunk: Vec<u8>,
        delim: Option<char>,
        space_dlimited: bool,
        consume_dquote: bool,
        allow_invalid_string: bool,
    ) -> DcsvResult<Option<Vec<String>>> {
        let line = if allow_invalid_string {
            String::from_utf8_lossy(&chunk).replace("\r\n", "\n")
        } else {
            String::from_utf8(chunk)
                .expect("Failed to convert to string")
                .replace("\r\n", "\n")
        };

        // Simply cut whitespaces
        if space_dlimited {
            return Ok(Some(
                line.split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            ));
        }

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
            if !value.is_empty() || previous == ',' {
                // Empty but previous was comma
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
