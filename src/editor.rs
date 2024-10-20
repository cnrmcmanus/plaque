use std::path::PathBuf;

#[derive(Debug)]
pub struct Editor {
    pub filepath: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor: (usize, usize),
    pub selection: Option<(usize, usize)>,
    pub clipboard: Option<Vec<String>>,
    pub window_height: usize,
    pub window_top_line: usize,
    pub pinned_col: usize,
    pub dirty: bool,
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
            selection: None,
            clipboard: None,
            window_height: 1,
            window_top_line: 0,
            pinned_col: 0,
            dirty: false,
        }
    }

    pub fn line_chars(&self, i: usize) -> usize {
        self.lines[i].chars().count()
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let path = self.filepath.as_ref().ok_or(std::io::ErrorKind::NotFound)?;

        let mut contents = self.lines.join("\n");
        contents.push('\n');
        std::fs::write(path, contents)?;

        self.dirty = false;

        Ok(())
    }

    pub fn insert_char(&mut self, c: char) {
        self.delete_selection();

        let (row, col) = self.cursor;

        self.lines[row].insert(col, c);
        self.set_pinned_cursor(row, col + 1);
        self.dirty = true;
    }

    pub fn newline(&mut self) {
        self.delete_selection();

        let (row, col) = self.cursor;
        let line: String = self.lines[row].chars().skip(col).collect();

        self.lines[row].truncate(col);
        self.lines.insert(row + 1, line);
        self.set_pinned_cursor(row + 1, 0);
        self.dirty = true;
        self.scroll_into_view();
    }

    pub fn backward_delete(&mut self) {
        if self.selection.is_some() {
            return self.delete_selection();
        }

        let (row, col) = self.cursor;
        if col > 0 {
            self.lines[row].remove(col - 1);
            self.set_pinned_cursor(row, col - 1);
            self.dirty = true;
        } else if row > 0 {
            let line = self.lines[row].clone();
            let prev_line_len = self.line_chars(row - 1);

            self.lines.remove(row);
            self.lines[row - 1] += &line;
            self.set_pinned_cursor(row - 1, prev_line_len);
            self.dirty = true;
        }
        self.scroll_into_view();
    }

    pub fn forward_delete(&mut self) {
        if self.selection.is_some() {
            return self.delete_selection();
        }

        let (row, col) = self.cursor;
        if col < self.lines[row].len() {
            self.lines[row].remove(col);
            self.dirty = true;
        } else if row + 1 < self.lines.len() {
            let next_line = self.lines.remove(row + 1);
            self.lines[row] += &next_line;
            self.dirty = true;
        }
        self.scroll_into_view();
    }

    pub fn set_pinned_cursor(&mut self, row: usize, col: usize) {
        self.cursor = (row, col);
        self.pinned_col = col;
    }

    pub fn move_cursor(&mut self, cursor_move: CursorMove, selection: bool) {
        let (row, col) = self.cursor;

        self.selection = if selection {
            Some(self.selection.unwrap_or(self.cursor))
        } else {
            None
        };

        match cursor_move {
            CursorMove::Up => {
                if row == 0 {
                    return;
                }
                let row = row - 1;
                let clamped_col = std::cmp::min(self.pinned_col, self.line_chars(row));
                self.cursor = (row, clamped_col);
                self.scroll_into_view();
            }
            CursorMove::Down => {
                if row >= self.lines.len() - 1 {
                    return;
                }
                let row = row + 1;
                let clamped_col = std::cmp::min(self.pinned_col, self.line_chars(row));
                self.cursor = (row, clamped_col);
                self.scroll_into_view();
            }
            CursorMove::Left => {
                if col > 0 {
                    self.set_pinned_cursor(row, col - 1);
                } else if row > 0 {
                    self.set_pinned_cursor(row - 1, self.line_chars(row - 1));
                }
            }
            CursorMove::Right => {
                if col < self.line_chars(row) {
                    self.set_pinned_cursor(row, col + 1);
                } else if row < self.lines.len() - 1 {
                    self.set_pinned_cursor(row + 1, 0);
                }
            }
        }
    }

    pub fn delete_selection(&mut self) {
        let (ci, cj) = self.cursor;
        let Some((si, sj)) = self.selection else {
            return;
        };

        self.selection = None;
        if (si, sj) < (ci, cj) {
            self.cursor = (si, sj);
        }

        let (xi, xj) = std::cmp::min((si, sj), (ci, cj));
        let (yi, yj) = std::cmp::max((si, sj), (ci, cj));

        if xi == yi {
            self.lines[xi].drain(std::cmp::min(xj, yj)..std::cmp::max(xj, yj));
        } else {
            self.lines[xi].truncate(xj);
            self.lines[yi].drain(0..yj);
            self.lines.drain(xi + 1..yi);

            let end = self.lines.remove(xi + 1);
            self.lines[xi] += &end;
        }
        self.scroll_into_view();
    }

    pub fn copy_selection(&mut self) {
        let (ci, cj) = self.cursor;
        let Some((si, sj)) = self.selection else {
            return;
        };

        let (xi, xj) = std::cmp::min((si, sj), (ci, cj));
        let (yi, yj) = std::cmp::max((si, sj), (ci, cj));

        if xi == yi {
            let line = self.lines[xi][xj..yj].to_string();
            self.clipboard = Some(vec![line]);
        } else {
            let mut lines = vec![];
            lines.push(self.lines[xi][xj..].to_string());
            lines.append(self.lines[xi + 1..yi].to_vec().as_mut());
            lines.push(self.lines[yi][..yj].to_string());
            self.clipboard = Some(lines);
        }
    }

    pub fn cut_selection(&mut self) {
        self.copy_selection();
        self.delete_selection();
    }

    pub fn paste(&mut self) {
        let (i, j) = self.cursor;
        let Some(clipboard) = self.clipboard.clone() else {
            return;
        };

        if clipboard.len() == 1 {
            self.lines[i].insert_str(j, &clipboard[0]);
        } else {
            let len = clipboard.len();
            let end = self.lines[i].split_off(j);

            self.lines[i] += &clipboard[0];
            self.lines.splice(i + 1..i + 1, clipboard[1..len].to_vec());
            self.lines[i + len - 1] += &end;
        }
        self.scroll_into_view();
    }

    pub fn indent(&mut self) {
        let (si, sj) = self.selection.unwrap_or(self.cursor);
        let (ci, cj) = self.cursor;
        let xi = std::cmp::min(si, ci);
        let mut yi = std::cmp::max(si, ci);
        let yj = std::cmp::max(sj, cj);

        // don't tab the last line if the range ends at index 0
        if xi != yi && yj == 0 {
            yi -= 1;
        }

        for line in self.lines[xi..yi + 1].iter_mut() {
            line.insert_str(0, "  ");
        }

        // advance the selection and cursor column position by 2
        // unless it ends the range and is at index 0
        if !(si > ci && sj == 0) {
            self.selection = self.selection.map(|_| (si, sj + 2));
        }
        if !(ci > si && cj == 0) {
            self.cursor = (ci, cj + 2);
        }

        self.dirty = true;
    }

    pub fn in_selection(&self, i: usize, j: usize) -> bool {
        if self.lines[i].len() == j {
            return false;
        }

        let Some((si, sj)) = self.selection else {
            return false;
        };
        let (ci, cj) = self.cursor;

        if (ci, cj) < (si, sj) {
            (ci, cj) <= (i, j) && (i, j) < (si, sj)
        } else {
            (si, sj) <= (i, j) && (i, j) < (ci, cj)
        }
    }

    pub fn set_window_height(&mut self, height: usize) {
        self.window_height = height;
    }

    pub fn scroll_into_view(&mut self) {
        let (row, _) = self.cursor;
        let half_window_height = self.window_height / 2;
        if row >= self.window_top_line + self.window_height {
            self.window_top_line = row - half_window_height;
        } else if row < self.window_top_line {
            self.window_top_line = if half_window_height <= row {
                row - half_window_height
            } else {
                0
            }
        }
    }
}
