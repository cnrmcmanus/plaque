use tui::{
    backend::Backend,
    terminal::Frame,
};

use crate::program::Program;

pub fn draw<B: Backend>(_program: &Program, _frame: &mut Frame<B>) {}
