mod editor;
mod help;
mod io;
mod tape;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};

use crate::program::Program;

pub fn draw(program: &mut Program, frame: &mut Frame) {
    let size = frame.area();

    let window = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Length(1),
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
        .split(window[1]);

    render_filename(frame, window[0], program);
    editor::render(frame, top_panel[0], program);
    io::render(frame, top_panel[1], program);
    tape::render(frame, window[2], program);
    help::render(frame, window[3], program.mode);
}

fn render_filename(frame: &mut Frame, area: Rect, program: &Program) {
    let filename = program
        .editor
        .filepath
        .as_ref()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str());

    let mut title = filename.unwrap_or("[Untitled]").to_string();

    if program.editor.dirty {
        title.push('*');
    }

    let paragraph = Paragraph::new(title).alignment(Alignment::Center).style(
        Style::default()
            .bg(Color::Rgb(200, 200, 200))
            .fg(Color::Rgb(50, 50, 50))
            .add_modifier(Modifier::BOLD),
    );

    frame.render_widget(paragraph, area);
}
