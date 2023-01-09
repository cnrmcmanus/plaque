use std::path::PathBuf;

#[derive(Debug)]
pub struct Editor {
    pub filepath: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor: (usize, usize),
    pub pinned_col: usize,
}

#[derive(Debug)]
pub enum CursorMove {
    Up,
    Down,
    Left,
    Right,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            filepath: None,
            lines: vec![],
            cursor: (0, 0),
            pinned_col: 0,
        }
    }

    pub fn line_chars(&self, i: usize) -> usize {
        self.lines[i].chars().count()
    }

    pub fn move_cursor(&mut self, cursor_move: CursorMove) {
        let (row, col) = self.cursor;
        match cursor_move {
            CursorMove::Up => {
                if row > 0 {
                    let clamped_col = std::cmp::min(self.pinned_col, self.line_chars(row - 1));
                    self.cursor = (row - 1, clamped_col);
                }
            }
            CursorMove::Down => {
                if row < self.lines.len() - 1 {
                    let clamped_col = std::cmp::min(self.pinned_col, self.line_chars(row + 1));
                    self.cursor = (row + 1, clamped_col);
                }
            }
            CursorMove::Left => {
                if col > 0 {
                    self.cursor = (row, col - 1);
                } else if row > 0 {
                    self.cursor = (row - 1, self.line_chars(row - 1));
                }
                self.pinned_col = self.cursor.1;
            }
            CursorMove::Right => {
                if col < self.line_chars(row) {
                    self.cursor = (row, col + 1);
                } else if row < self.lines.len() - 1 {
                    self.cursor = (row + 1, 0);
                }
                self.pinned_col = self.cursor.1;
            }
        }
    }
}
