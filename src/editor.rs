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
            lines: vec!["".to_string()],
            cursor: (0, 0),
            pinned_col: 0,
        }
    }

    pub fn line_chars(&self, i: usize) -> usize {
        self.lines[i].chars().count()
    }

    pub fn insert_char(&mut self, c: char) {
        let (row, col) = self.cursor;
        self.lines[row].insert(col, c);
        self.cursor = (row, col + 1);
    }

    pub fn newline(&mut self) {
        let (row, col) = self.cursor;
        let line: String = self.lines[row].chars().skip(col).collect();
        self.lines[row].truncate(col);
        self.lines.insert(row + 1, line);
        self.cursor = (row + 1, 0);
    }

    pub fn backward_delete(&mut self) {
        let (row, col) = self.cursor;
        if col > 0 {
            self.lines[row].remove(col - 1);
            self.cursor = (row, col - 1);
        } else if row > 0 {
            let line = self.lines[row].clone();
            let prev_line_len = self.line_chars(row - 1);

            self.lines.remove(row);
            self.lines[row - 1] += &line;
            self.cursor = (row - 1, prev_line_len);
        }
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
