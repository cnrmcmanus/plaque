use crate::editor;
use crate::program::{Mode, Program};
use crate::ui;

use anyhow::Result;
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

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
    thread::spawn(move || loop {
        if let CEvent::Key(key) = event::read().unwrap() {
            tx_program.send(key).unwrap();
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
            let control = event.modifiers.contains(KeyModifiers::CONTROL);
            let shift = event.modifiers.contains(KeyModifiers::SHIFT);

            match program.mode {
                Mode::Editor => match event.code {
                    KeyCode::Char('r') if control => {
                        program.reset();
                    }
                    KeyCode::Char('s') if control => {
                        program.editor.save().ok();
                    }
                    KeyCode::Char('c') if control => {
                        program.editor.copy_selection();
                    }
                    KeyCode::Char('x') if control => {
                        program.editor.cut_selection();
                        program.index_instructions();
                    }
                    KeyCode::Char('v') if control => {
                        program.editor.paste();
                        program.index_instructions();
                    }
                    KeyCode::Char(c) => {
                        program.editor.insert_char(c);
                        program.index_instructions();
                    }
                    KeyCode::Enter => {
                        program.editor.newline();
                        program.index_instructions();
                    }
                    KeyCode::Backspace => {
                        program.editor.backward_delete();
                        program.index_instructions();
                    }
                    KeyCode::Delete => {
                        program.editor.forward_delete();
                        program.index_instructions();
                    }
                    KeyCode::Up if control => {
                        program.undo_until_exception();
                    }
                    KeyCode::Up => program.editor.move_cursor(editor::CursorMove::Up, shift),
                    KeyCode::Down if control => {
                        program.step_until_exception();
                    }
                    KeyCode::Down => program.editor.move_cursor(editor::CursorMove::Down, shift),
                    KeyCode::Left if control => {
                        program.undo().ok();
                    }
                    KeyCode::Left => program.editor.move_cursor(editor::CursorMove::Left, shift),
                    KeyCode::Right if control => {
                        program.step().ok();
                    }
                    KeyCode::Right => program.editor.move_cursor(editor::CursorMove::Right, shift),
                    KeyCode::Tab => {
                        program.editor.indent();
                        program.index_instructions();
                    }
                    KeyCode::Esc => {
                        tx_ui.send(()).unwrap();
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
            let mut guard = shared_state.lock().unwrap();
            ui::draw(&mut guard, frame);
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
