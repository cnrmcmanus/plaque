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

    let text = program
        .editor
        .lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let spans = line
                .chars()
                .enumerate()
                .map(|(j, chr)| {
                    if program.cursor() == Some((i, j)) {
                        Span::styled(chr.to_string(), focused_code_style)
                    } else if program.instruction_positions.contains(&(i, j)) {
                        Span::styled(chr.to_string(), code_style)
                    } else {
                        Span::styled(chr.to_string(), comment_style)
                    }
                })
                .collect::<Vec<_>>();

            Spans::from(spans)
        })
        .collect::<Vec<_>>();

    let program =
        Paragraph::new(text).block(Block::default().title("Program").borders(Borders::ALL));

    frame.render_widget(program, area);
}
