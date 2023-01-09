use std::path::PathBuf;

#[derive(Debug)]
pub struct Editor {
    pub filepath: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor: (usize, usize),
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            filepath: None,
            lines: vec![],
            cursor: (0, 0),
        }
    }
}
