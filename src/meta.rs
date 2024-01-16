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

    pub fn set_width(&mut self, width: usize) {
        self.max_unicode_width = width;
    }

    pub fn update_width(&mut self, target: &Value) {
        let new_width = target.get_width();
        self.max_unicode_width = self.max_unicode_width.max(new_width);
    }
}
