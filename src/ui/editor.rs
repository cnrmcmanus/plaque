use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

use crate::program::Program;

pub fn render<B: Backend>(frame: &mut Frame<B>, area: Rect, program: &Program) {
    let focused_code_style = Style::default().add_modifier(Modifier::UNDERLINED);
    let code_style = Style::default();
    let comment_style = Style::default().fg(Color::Rgb(150, 150, 150));

    let line_count = program.editor.lines.len();
    let line_count_digits = (line_count.checked_ilog10().unwrap_or(0) + 1) as usize;
    let text = program
        .editor
        .lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let spans = std::iter::once(Span::styled(
                format!("{:0>line_count_digits$} ", i + 1),
                comment_style,
            ))
            .chain(line.chars().enumerate().map(|(j, chr)| {
                if program.cursor() == Some((i, j)) {
                    Span::styled(chr.to_string(), focused_code_style)
                } else if program.instruction_positions.contains(&(i, j)) {
                    Span::styled(chr.to_string(), code_style)
                } else {
                    Span::styled(chr.to_string(), comment_style)
                }
            }))
            .collect::<Vec<_>>();

            Spans::from(spans)
        })
        .collect::<Vec<_>>();

    let program =
        Paragraph::new(text).block(Block::default().title("Editor").borders(Borders::ALL));

    frame.render_widget(program, area);
}
