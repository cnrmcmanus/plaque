use std::iter;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

use crate::program::Program;

pub fn render<B: Backend>(frame: &mut Frame<B>, area: Rect, program: &mut Program) {
    let cursor_style = Style::default()
        .bg(Color::Rgb(200, 200, 200))
        .fg(Color::Rgb(50, 50, 50));
    let selection_style = Style::default().bg(Color::Rgb(75, 75, 75));
    let focused_code_style = Style::default().add_modifier(Modifier::UNDERLINED);
    let code_style = Style::default();
    let comment_style = Style::default().fg(Color::Rgb(150, 150, 150));

    let height = area.height - 2;
    program.editor.set_window_height(height as usize);
    let end_line = std::cmp::min(
        program.editor.window_top_line + program.editor.window_height,
        program.editor.lines.len(),
    );
    let line_count = program.editor.lines.len();
    let line_count_digits = (line_count.checked_ilog10().unwrap_or(0) + 1) as usize;
    let text = program
        .editor
        .lines
        .iter()
        .skip(program.editor.window_top_line)
        .take(end_line - program.editor.window_top_line)
        .enumerate()
        .map(|(i, line)| {
            let i = program.editor.window_top_line + i;
            let spans = iter::once(Span::styled(
                format!("{:0>line_count_digits$} ", i + 1),
                comment_style,
            ))
            .chain(
                line.chars()
                    .chain(iter::once(' '))
                    .enumerate()
                    .map(|(j, chr)| {
                        let style = if program.is_editor_mode() && program.editor.cursor == (i, j) {
                            cursor_style
                        } else if program.is_editor_mode() && program.editor.in_selection(i, j) {
                            selection_style
                        } else if program.cursor() == Some((i, j)) {
                            focused_code_style
                        } else if program.instruction_positions.contains(&(i, j)) {
                            code_style
                        } else {
                            comment_style
                        };

                        Span::styled(chr.to_string(), style)
                    }),
            )
            .collect::<Vec<_>>();

            Spans::from(spans)
        })
        .collect::<Vec<_>>();

    let program = Paragraph::new(text).block(Block::default().borders(Borders::ALL));

    frame.render_widget(program, area);
}
