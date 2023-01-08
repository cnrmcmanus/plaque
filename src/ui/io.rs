use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    terminal::Frame,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::program::{Mode, Program};

const NEWLINE_COLOR: Color = Color::Rgb(80, 80, 80);

pub fn render<B: Backend>(frame: &mut Frame<B>, area: Rect, program: &Program) {
    let input_output_height = area.height - 6;
    let output_height = input_output_height / 2;
    let input_height = input_output_height - output_height;

    let panel = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Length(input_height),
                Constraint::Length(output_height),
                Constraint::Length(6),
            ]
            .as_ref(),
        )
        .split(area);

    render_output(frame, panel[0], program);
    render_input(frame, panel[1], program);
    render_debug(frame, panel[2], program);
}

/// Display Input/Output text
fn io_text(buffer: &[u8]) -> Text {
    let text = std::str::from_utf8(buffer).unwrap();
    let newlines = text.matches('\n').count();
    let lines = text
        .split('\n')
        .enumerate()
        // add line ending marker B6 to each line except the last
        .map(|(i, line)| {
            if i != newlines {
                Spans::from(vec![
                    Span::from(line),
                    Span::styled("\u{B6}", Style::default().fg(NEWLINE_COLOR)),
                ])
            } else {
                Spans::from(vec![Span::from(line)])
            }
        })
        .collect::<Vec<_>>();

    Text::from(lines)
}

pub fn render_input<B: Backend>(frame: &mut Frame<B>, area: Rect, program: &Program) {
    let text = match program.mode {
        Mode::Input => &program.input_buffer,
        _ => &program.engine.input,
    };
    let input = Paragraph::new(io_text(text))
        .block(Block::default().title("Input").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(input, area);
}

pub fn render_output<B: Backend>(frame: &mut Frame<B>, area: Rect, program: &Program) {
    let output = Paragraph::new(io_text(&program.engine.output))
        .block(Block::default().title("Output").borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(output, area);
}

fn render_debug<B: Backend>(frame: &mut Frame<B>, area: Rect, program: &Program) {
    let lines = program
        .debug_messages
        .iter()
        .rev()
        .map(|message| Spans::from(vec![Span::raw(message)]))
        .collect::<Vec<_>>();

    let debug = Paragraph::new(lines).block(Block::default().title("Debug").borders(Borders::ALL));

    frame.render_widget(debug, area);
}
