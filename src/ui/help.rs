use std::cmp::max;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    terminal::Frame,
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::program::Mode;

#[derive(Debug)]
struct HelpItem<'a> {
    hotkey: &'a str,
    label: &'a str,
}

impl HelpItem<'_> {
    fn new<'a>(hotkey: &'a str, label: &'a str) -> HelpItem<'a> {
        HelpItem { hotkey, label }
    }

    fn to_cell(&self, hotkey_width: usize, label_width: usize) -> Cell<'_> {
        Cell::from(Span::from(format!(
            " {:hotkey_width$} = {:label_width$}",
            self.hotkey, self.label
        )))
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
            HelpItem::new("→", "Step"),
            HelpItem::new("←", "Undo"),
            HelpItem::new("space", "Play/Pause"),
            HelpItem::new("↓", "Step to Breakpoint"),
            HelpItem::new("↑", "Undo to Breakpoint"),
            HelpItem::new("e", "Editor Mode"),
            HelpItem::new("x", "Reset"),
            HelpItem::new("esc/q", "Quit"),
        ],
        Mode::Editor => vec![
            HelpItem::new("esc", "Done"),
            HelpItem::new("↑↓←→", "Move Cursor"),
            HelpItem::new("ctrl+s", "Save"),
            HelpItem::new("ctrl+c", "Copy"),
            HelpItem::new("ctrl+x", "Cut"),
            HelpItem::new("ctrl+v", "Paste"),
            HelpItem::new("bksp", "Backward Delete"),
            HelpItem::new("del", "Forward Delete"),
            HelpItem::new("tab", "Indent"),
        ],
        Mode::Input => vec![
            HelpItem::new("enter", "Submit"),
            HelpItem::new("shift+enter", "Newline"),
        ],
    };

    let (columns, widths): (Vec<_>, Vec<_>) = help_items
        .as_slice()
        .chunks(height)
        .map(|column| {
            let (hotkey_width, label_width) = column.iter().fold((0, 0), |accum, item| {
                (
                    max(accum.0, item.hotkey.chars().count()),
                    max(accum.1, item.label.chars().count()),
                )
            });
            let cells: Vec<_> = column
                .iter()
                .map(|item| item.to_cell(hotkey_width, label_width))
                .collect();
            let total_width = hotkey_width + label_width + 5;

            (cells, Constraint::Length(total_width as u16))
        })
        .unzip();

    let rows = (0..height).map(|i| {
        Row::new(
            columns
                .iter()
                .flatten()
                .skip(i)
                .step_by(height)
                .cloned()
                .collect::<Vec<_>>(),
        )
    });

    let table = Table::new(rows)
        .block(Block::default().borders(Borders::ALL).title(title))
        .widths(&widths)
        .column_spacing(2);

    frame.render_widget(table, area);
}
