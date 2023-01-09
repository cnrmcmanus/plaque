use crate::editor;
use crate::program::{Mode, Program};
use crate::ui;

use anyhow::Result;
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
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
    let (tx_program, rx_program) = mpsc::channel::<KeyEvent>();
    let (tx_ui, rx_ui) = mpsc::channel::<()>();

    spawn_input_thread(tx_program);
    spawn_program_thread(shared_state.clone(), rx_program, tx_ui);
    ui_loop(shared_state, rx_ui)
}

pub fn spawn_input_thread(tx_program: Sender<KeyEvent>) {
    let tick_rate = Duration::from_millis(50);
    thread::spawn(move || loop {
        if event::poll(tick_rate).unwrap() {
            if let CEvent::Key(key) = event::read().unwrap() {
                tx_program.send(key).unwrap();
            }
        }
    });
}

pub fn spawn_program_thread(
    shared_state: SharedState,
    rx_program: Receiver<KeyEvent>,
    tx_ui: Sender<()>,
) {
    thread::spawn(move || loop {
        if let Ok(event) = rx_program.recv() {
            let mut guard = shared_state.lock().unwrap();
            let program = &mut guard;

            match program.mode {
                Mode::Interactive => match event.code {
                    KeyCode::Char('e') => {
                        program.mode = Mode::Editor;
                    }
                    KeyCode::Char('x') => {
                        program.reset();
                    }
                    KeyCode::Char('q') => {
                        tx_ui.send(()).unwrap();
                    }
                    KeyCode::Right => {
                        program.step();
                    }
                    KeyCode::Left => {
                        program.undo();
                    }
                    _ => {}
                },
                Mode::Editor => match event.code {
                    KeyCode::Up => program.editor.move_cursor(editor::CursorMove::Up),
                    KeyCode::Down => program.editor.move_cursor(editor::CursorMove::Down),
                    KeyCode::Left => program.editor.move_cursor(editor::CursorMove::Left),
                    KeyCode::Right => program.editor.move_cursor(editor::CursorMove::Right),
                    KeyCode::Esc => {
                        program.mode = Mode::Interactive;
                    }
                    _ => {}
                },
                Mode::Input => match event.code {
                    KeyCode::Char(c) => {
                        program.add_input(c);
                    }
                    KeyCode::Enter => {
                        if event.modifiers.contains(KeyModifiers::SHIFT) {
                            program.add_input('\n');
                        } else {
                            program.exit_input_mode(true);
                        }
                    }
                    _ => {}
                },
            };
        }
    });
}

pub fn ui_loop(shared_state: SharedState, rx_ui: Receiver<()>) -> Result<()> {
    let tick_rate = Duration::from_millis(10);
    let mut stdout = std::io::stdout();

    crossterm::terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|frame| {
            let guard = shared_state.lock().unwrap();
            ui::draw(&guard, frame);
            drop(guard);
        })?;

        if let Ok(()) = rx_ui.try_recv() {
            break;
        }

        thread::sleep(tick_rate);
    }

    crossterm::terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
