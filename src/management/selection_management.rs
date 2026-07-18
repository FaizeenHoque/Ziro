#[derive(Debug, Clone, Copy)]
pub struct Selection {
    pub anchor: (usize, usize),
}

impl Selection {
    pub fn range(&self, cursor: (usize, usize)) -> ((usize, usize), (usize, usize)) {
        let anchor_key = (self.anchor.1, self.anchor.0);
        let cursor_key = (cursor.1, cursor.0);
        if anchor_key <= cursor_key {
            (self.anchor, cursor)
        } else {
            (cursor, self.anchor)
        }
    }
}
