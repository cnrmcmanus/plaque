#![allow(dead_code, unstable_name_collisions)]
#![feature(iter_intersperse)]

mod engine;
mod flavor;
mod instruction;
mod program;
mod ui;

use anyhow::Result;
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use tui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<()> {
    crossterm::terminal::enable_raw_mode()?;

    let input_filename = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::Error::msg("missing input filename"))?;

    let program =
        program::Program::load(input_filename, flavor::overflow::INSTRUCTION_SET.to_vec())?;

    let shared_state = Arc::new(Mutex::new(program));
    let (tx_ui, rx_ui) = mpsc::channel::<KeyEvent>();
    let (tx_program, rx_program) = mpsc::channel::<KeyEvent>();
    let tick_rate = Duration::from_millis(50);

    thread::spawn(move || loop {
        if event::poll(tick_rate).unwrap() {
            if let CEvent::Key(key) = event::read().unwrap() {
                tx_ui.send(key).unwrap();
                tx_program.send(key).unwrap();
            }
        }
    });

    let program_state_ref = Arc::clone(&shared_state);
    thread::spawn(move || loop {
        let mut guard = program_state_ref.lock().unwrap();
        let program = &mut guard;

        if let Ok(event) = rx_program.try_recv() {
            match event.code {
                KeyCode::Right => {
                    program.step();
                }
                KeyCode::Left => {
                    program.undo();
                }
                _ => {}
            }
        }

        drop(guard);
        thread::sleep(Duration::from_millis(10));
    });

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let ui_state_ref = Arc::clone(&shared_state);
    loop {
        terminal.draw(|frame| {
            let guard = ui_state_ref.lock().unwrap();
            ui::draw(&guard, frame);
            drop(guard);
        })?;

        if let Ok(event) = rx_ui.recv() {
            if let KeyCode::Char('q') = event.code {
                break;
            }
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    terminal.clear()?;

    Ok(())
}
