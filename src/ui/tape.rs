use num_integer::Integer;
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    terminal::Frame,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

use crate::program::Program;

const CELL_COLOR: Color = Color::Rgb(255, 255, 255);
const INDEX_COLOR: Color = Color::Rgb(150, 150, 150);
const EMPTY_COLOR: Color = Color::Rgb(80, 80, 80);

pub fn render<B: Backend>(frame: &mut Frame<B>, area: Rect, program: &Program) {
    let tape_pointer = program.engine.tape_pointer;
    let tape_length = program.engine.tape.len();
    let tape_space = TapeSpace::new(frame.size().width as usize - 2, tape_pointer, tape_length);
    let right_slots = tape_space.used_right_slots + tape_space.unused_right_slots;

    let cell_style = Style::default().fg(CELL_COLOR);
    let index_style = Style::default().fg(INDEX_COLOR);
    let empty_style = Style::default().fg(EMPTY_COLOR);

    let tape_iter = program.engine.tape.iter();
    let mut cells = ["---"]
        .repeat(tape_space.unused_left_slots)
        .into_iter()
        .map(|blob| Span::styled(blob, empty_style))
        .chain(
            tape_iter
                .chain([&0u8].repeat(tape_space.unused_right_slots))
                .skip(tape_pointer - tape_space.used_left_slots)
                .take(tape_space.used_left_slots + 1 + right_slots)
                .map(|cell| Span::styled(format!("{cell:0>3}"), cell_style)),
        )
        .collect::<Vec<Span>>();

    let mut indexes = ["---"]
        .repeat(tape_space.unused_left_slots)
        .into_iter()
        .map(|blob| Span::styled(blob, empty_style))
        .chain(
            (tape_pointer - tape_space.used_left_slots..tape_pointer + 1 + right_slots)
                .map(|i| Span::styled(format!("{:0>3}", i % 1000), index_style)),
        )
        .collect::<Vec<Span>>();

    let text = vec![
        Spans::from("\u{25BC}"),
        join_tape_spans(cells.as_mut(), &tape_space),
        join_tape_spans(indexes.as_mut(), &tape_space),
    ];

    let tape = Paragraph::new(text)
        .block(Block::default().title("Tape").borders(Borders::ALL))
        .alignment(Alignment::Center);

    frame.render_widget(tape, area);
}

struct TapeSpace {
    used_left_slots: usize,
    unused_left_slots: usize,
    left_overflow: usize,

    used_right_slots: usize,
    unused_right_slots: usize,
    right_overflow: usize,
}

impl TapeSpace {
    fn new(width: usize, tape_pointer: usize, tape_length: usize) -> TapeSpace {
        let available = width - 3;
        let (half, remainder) = (available / 2, available % 2);
        let (left, right) = (half + remainder, half);
        let (left_slots, right_slots) = (left.div_ceil(&4), right.div_ceil(&4));

        let used_left_slots = std::cmp::min(left_slots, tape_pointer);
        let unused_left_slots = left_slots - used_left_slots;

        let used_right_slots = std::cmp::min(right_slots, tape_length - tape_pointer - 1);
        let unused_right_slots = right_slots - used_right_slots;

        let (left_overflow, right_overflow) = ((left_slots * 4) - left, (right_slots * 4) - right);

        TapeSpace {
            used_left_slots,
            unused_left_slots,
            left_overflow,

            used_right_slots,
            unused_right_slots,
            right_overflow,
        }
    }
}

fn join_tape_spans<'a>(spans: &mut Vec<Span<'a>>, tape_space: &TapeSpace) -> Spans<'a> {
    let len = spans.len();

    // remove any overflow from the first and last elements
    spans[0].content = spans[0].content[..3 - tape_space.left_overflow]
        .to_string()
        .into();
    spans[len - 1].content = spans[len - 1].content[tape_space.right_overflow..]
        .to_string()
        .into();

    let joined = spans
        .clone()
        .into_iter()
        .intersperse(Span::styled("|", Style::default().fg(EMPTY_COLOR)))
        .collect::<Vec<Span>>();

    Spans::from(joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow() {
        #[rustfmt::skip]
        let overflows = [(0, 0), (0, 1), (1, 1), (1, 2), (2, 2), (2, 3), (3, 3), (3, 0)];
        for i in 0..16 {
            let tape_space = TapeSpace::new(27 - i, 0, 1);
            assert_eq!(tape_space.left_overflow, overflows[i % 8].0);
            assert_eq!(tape_space.right_overflow, overflows[i % 8].1);
        }
    }

    #[test]
    fn overflow_unaffected_by_tape_position() {
        for i in 0..16 {
            let tape_space = TapeSpace::new(24, i, 100);
            assert_eq!(tape_space.left_overflow, 1);
            assert_eq!(tape_space.right_overflow, 2);
        }
    }

    #[test]
    fn slots_by_tape_position() {
        let used_slots = [(0, 3), (1, 3), (2, 3), (3, 3), (3, 2), (3, 1), (3, 0)];
        for (i, used_slot) in used_slots.iter().enumerate() {
            let tape_space = TapeSpace::new(27, i, 7);
            assert_eq!(tape_space.used_left_slots, used_slot.0);
            assert_eq!(tape_space.used_right_slots, used_slot.1);
            assert_eq!(tape_space.unused_left_slots, 3 - used_slot.0);
            assert_eq!(tape_space.unused_right_slots, 3 - used_slot.1);
        }
    }

    #[test]
    fn unused_slots_by_width() {
        #[rustfmt::skip]
        let unused_slots = [(1, 1), (1, 1), (1, 1), (1, 1), (1, 1), (1, 1), (1, 1), (1, 0), (0, 0)];
        for (i, unused_slot) in unused_slots.iter().enumerate() {
            let tape_space = TapeSpace::new(19 - i, 1, 3);
            assert_eq!(tape_space.used_left_slots, 1);
            assert_eq!(tape_space.used_right_slots, 1);
            assert_eq!(tape_space.unused_left_slots, unused_slot.0);
            assert_eq!(tape_space.unused_right_slots, unused_slot.1);
        }
    }
}
