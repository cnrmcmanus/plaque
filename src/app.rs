use crate::program::Program;
use crate::ui;

use anyhow::Result;
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent};
use std::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;
use tui::{backend::CrosstermBackend, Terminal};

type SharedState = Arc<Mutex<Program>>;

pub fn run(program: Program) -> Result<()> {
    let shared_state = Arc::new(Mutex::new(program));
    let (tx_ui, rx_ui) = mpsc::channel::<KeyEvent>();
    let (tx_program, rx_program) = mpsc::channel::<KeyEvent>();

    spawn_input_thread(tx_ui, tx_program);
    spawn_program_thread(shared_state.clone(), rx_program);
    ui_loop(shared_state, rx_ui)
}

pub fn spawn_input_thread(tx_ui: Sender<KeyEvent>, tx_program: Sender<KeyEvent>) {
    let tick_rate = Duration::from_millis(50);
    thread::spawn(move || loop {
        if event::poll(tick_rate).unwrap() {
            if let CEvent::Key(key) = event::read().unwrap() {
                tx_ui.send(key).unwrap();
                tx_program.send(key).unwrap();
            }
        }
    });
}

pub fn spawn_program_thread(shared_state: SharedState, rx_program: Receiver<KeyEvent>) {
    thread::spawn(move || loop {
        if let Ok(event) = rx_program.recv() {
            let mut guard = shared_state.lock().unwrap();
            let program = &mut guard;
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
    });
}

pub fn ui_loop(shared_state: SharedState, rx_ui: Receiver<KeyEvent>) -> Result<()> {
    let tick_rate = Duration::from_millis(10);
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        terminal.draw(|frame| {
            let guard = shared_state.lock().unwrap();
            ui::draw(&guard, frame);
            drop(guard);
        })?;

        if let Ok(event) = rx_ui.try_recv() {
            if let KeyCode::Char('q') = event.code {
                break;
            }
        }

        thread::sleep(tick_rate);
    }

    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    terminal.clear()?;

    Ok(())
}
