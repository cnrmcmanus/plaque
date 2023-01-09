use num_integer::Integer;
use std::cmp::max;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::program::Mode;

struct HelpItem<'a> {
    hotkey: &'a str,
    label: &'a str,
}

impl HelpItem<'_> {
    fn new<'a>(hotkey: &'a str, label: &'a str) -> HelpItem<'a> {
        HelpItem { hotkey, label }
    }

    fn to_cell(&self, hotkey_width: usize, label_width: usize) -> Cell<'_> {
        Cell::from(Spans::from(vec![
            Span::from(" "),
            Span::from(self.hotkey),
            Span::from(" ".repeat(hotkey_width - self.hotkey.chars().count())),
            Span::from(" = "),
            Span::from(self.label),
            Span::from(" ".repeat(label_width - self.label.chars().count())),
            Span::from(" "),
        ]))
    }
}

pub fn render<B: Backend>(frame: &mut Frame<B>, area: Rect, mode: Mode) {
    let height = 3;

    let title = format!(
        "Help ({})",
        match mode {
            Mode::Interactive => "interactive mode",
            Mode::Editor => "editor mode",
            Mode::Input => "input mode",
        }
    );

    let help_items: Vec<HelpItem> = match mode {
        Mode::Interactive => vec![
            HelpItem::new("right", "Step"),
            HelpItem::new("left", "Undo"),
            HelpItem::new("space", "Play/Pause"),
            HelpItem::new("e", "Editor Mode"),
            HelpItem::new("x", "Reset"),
            HelpItem::new("q", "Quit"),
        ],
        Mode::Editor => vec![HelpItem::new("esc", "Done")],
        Mode::Input => vec![
            HelpItem::new("enter", "Submit"),
            HelpItem::new("shift+enter", "Newline"),
        ],
    };

    let (hotkey_width, label_width) = help_items.iter().fold((0, 0), |accum, item| {
        (
            max(accum.0, item.hotkey.chars().count()),
            max(accum.1, item.label.chars().count()),
        )
    });
    let columns = help_items.len().next_multiple_of(&height);
    let rows = (0..height).map(|i| {
        Row::new(
            help_items
                .iter()
                .skip(i)
                .step_by(height)
                .map(|item| item.to_cell(hotkey_width, label_width))
                .collect::<Vec<Cell>>(),
        )
    });
    let width = (hotkey_width + label_width + 5) as u16;
    let widths = (0..columns)
        .map(|_| Constraint::Length(width))
        .collect::<Vec<Constraint>>();
    let table = Table::new(rows)
        .block(Block::default().borders(Borders::ALL).title(title))
        .widths(widths.as_ref())
        .column_spacing(4);

    frame.render_widget(table, area);
}
