use unicode_width::UnicodeWidthStr;

use crate::Value;

#[derive(Clone, Debug, Default)]
pub struct Meta {
    pub max_unicode_width: usize,
}

impl Meta {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn update_width(&mut self, target: &Value) {
        let new_width = match target {
            Value::Number(num) => {
                if *num == 0 {
                    0
                } else {
                    (num.ilog10() + 1) as usize
                }
            }
            Value::Text(text) => UnicodeWidthStr::width(text.as_str()),
        };
        self.max_unicode_width = self.max_unicode_width.max(new_width);
    }
}
