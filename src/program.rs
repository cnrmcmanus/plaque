use crate::editor::Editor;
use crate::engine::{Engine, Exception, InstructionPointer};
use crate::instruction::Instruction;

use std::collections::HashMap;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Interactive,
    Editor,
    Input,
}

#[derive(Debug)]
pub struct Program {
    pub engine: Engine,
    pub instruction_set: HashMap<char, Instruction>,
    pub editor: Editor,
    pub instruction_positions: Vec<(usize, usize)>,
    pub mode: Mode,
    pub input_buffer: Vec<u8>,
    pub stdin: Option<Vec<u8>>,
    pub debug_messages: Vec<String>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            engine: Engine::new(vec![]),
            instruction_set: HashMap::new(),
            editor: Editor::new(),
            instruction_positions: vec![],
            mode: Mode::Interactive,
            input_buffer: vec![],
            stdin: None,
            debug_messages: vec![],
        }
    }

    pub fn load<S: Into<String>>(
        filename: S,
        instruction_set: Vec<Instruction>,
    ) -> io::Result<Program> {
        let mut program = Program::new();

        program.set_instructions(instruction_set);
        program.editor.filepath = Some(PathBuf::from(filename.into()));
        program.hotload()?;
        program.step();

        Ok(program)
    }

    pub fn blank(instruction_set: Vec<Instruction>) -> Program {
        let mut program = Program::new();

        program.set_instructions(instruction_set);
        program.step();

        program
    }

    pub fn set_instructions(&mut self, instruction_set: Vec<Instruction>) {
        self.instruction_set = HashMap::new();
        for instruction in instruction_set {
            self.instruction_set.insert(instruction.symbol, instruction);
        }
    }

    pub fn read_instruction(&self, character: char) -> Option<Instruction> {
        self.instruction_set.get(&character).copied()
    }

    pub fn hotload(&mut self) -> io::Result<()> {
        let path = self
            .editor
            .filepath
            .as_ref()
            .ok_or(io::ErrorKind::NotFound)?;
        let code_text = std::fs::read_to_string(path)?;
        self.editor.lines = code_text
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<String>>();
        self.index_instructions();

        Ok(())
    }

    pub fn index_instructions(&mut self) {
        self.engine.instructions = vec![];
        self.instruction_positions = vec![];

        for (line_number, line) in self.editor.lines.iter().enumerate() {
            for (column_number, character) in line.chars().enumerate() {
                if let Some(instruction) = self.read_instruction(character) {
                    self.engine.instructions.push(instruction);
                    self.instruction_positions
                        .push((line_number, column_number));
                }
            }
        }

        if self.engine.instructions.is_empty() {
            self.engine.instruction_pointer = InstructionPointer::Start;
        } else if let InstructionPointer::Index(i) = self.engine.instruction_pointer {
            self.engine.instruction_pointer =
                InstructionPointer::Index(std::cmp::min(i, self.instruction_positions.len() - 1));
        }
    }

    pub fn step(&mut self) {
        if let Err(exception) = self.engine.step() {
            match exception {
                Exception::Error(message) => {
                    self.debug_messages.push(message);
                }
                Exception::RequestingInput => {
                    self.enter_input_mode();
                }
            }
        }
    }

    pub fn undo(&mut self) {
        self.engine.undo().ok();
    }

    pub fn reset(&mut self) {
        self.engine.reset();
        if let Some(stdin) = &self.stdin {
            self.engine.input = stdin.clone();
        }
        self.step();
    }

    pub fn is_interactive_mode(&self) -> bool {
        self.mode == Mode::Interactive
    }

    pub fn is_editor_mode(&self) -> bool {
        self.mode == Mode::Editor
    }

    pub fn is_input_mode(&self) -> bool {
        self.mode == Mode::Input
    }

    pub fn read_stdin(&mut self) {
        self.stdin = if atty::isnt(atty::Stream::Stdin) {
            let stdin = io::stdin()
                .lock()
                .bytes()
                .map(|x| x.unwrap_or_default())
                .collect::<Vec<_>>();
            self.engine.input = stdin.clone();
            Some(stdin)
        } else {
            None
        };
    }

    pub fn enter_input_mode(&mut self) {
        self.mode = Mode::Input;
        self.input_buffer = self.engine.input.clone();
    }

    pub fn exit_input_mode(&mut self, commit: bool) {
        self.mode = Mode::Interactive;
        if commit {
            self.engine.input = self.input_buffer.clone();
        }
        self.input_buffer = vec![];
    }

    pub fn add_input(&mut self, c: char) {
        if c.is_ascii() {
            self.input_buffer.push(c as u8);
        }
    }

    pub fn cursor(&self) -> Option<(usize, usize)> {
        match self.engine.instruction_pointer {
            InstructionPointer::Index(i) => Some(self.instruction_positions[i]),
            _ => None,
        }
    }
}
