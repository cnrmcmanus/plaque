mod code;
mod io;
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

    let top_panel = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            [
                Constraint::Length(15),
                Constraint::Min(10),
                Constraint::Length(15),
            ]
            .as_ref(),
        )
        .split(window[0]);

    io::render_input(frame, top_panel[0], program);
    code::render(frame, top_panel[1], program);
    io::render_output(frame, top_panel[2], program);
    tape::render(frame, window[1], program);
}
