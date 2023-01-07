mod code;
mod help;
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
        .constraints([Constraint::Min(10), Constraint::Length(30)].as_ref())
        .split(window[0]);

    let io_panel = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(top_panel[1]);

    code::render(frame, top_panel[0], program);
    io::render_output(frame, io_panel[0], program);
    io::render_input(frame, io_panel[1], program);
    tape::render(frame, window[1], program);
    help::render(frame, window[2], program.mode);
}
