mod tape;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    terminal::Frame,
};

use crate::program::Program;

pub fn draw<B: Backend>(program: &Program, frame: &mut Frame<B>) {
    let size = frame.size();

    let window = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Min(6),
                Constraint::Length(5),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(size);

    tape::render(frame, window[1], program);
}
