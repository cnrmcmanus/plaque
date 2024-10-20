mod editor;
mod help;
mod io;
mod tape;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    widgets::Paragraph,
};

use crate::program::Program;

pub fn draw<B: Backend>(program: &mut Program, frame: &mut Frame<B>) {
    let size = frame.size();

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

fn render_filename<B: Backend>(frame: &mut Frame<B>, area: Rect, program: &Program) {
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

    let paragraph = Paragraph::new(title)
        .alignment(tui::layout::Alignment::Center)
        .style(
            Style::default()
                .bg(Color::Rgb(200, 200, 200))
                .fg(Color::Rgb(50, 50, 50))
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(paragraph, area);
}
