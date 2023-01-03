use std::collections::HashMap;

use crate::engine::{Engine, InstructionPointer};
use crate::instruction::Instruction;

#[derive(Debug)]
pub struct Program {
    pub engine: Engine,
    pub instruction_set: HashMap<char, Instruction>,
    pub code_lines: Vec<String>,
    pub instruction_positions: Vec<(usize, usize)>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            engine: Engine::new(vec![]),
            instruction_set: HashMap::new(),
            code_lines: vec![],
            instruction_positions: vec![],
        }
    }

    pub fn load<S: Into<String>>(
        filename: S,
        instruction_set: Vec<Instruction>,
    ) -> Result<Program, std::io::Error> {
        let mut program = Program::new();

        program.set_instructions(instruction_set);
        program.hotload(filename.into())?;
        program.step();

        Ok(program)
    }

    pub fn set_instructions(&mut self, instruction_set: Vec<Instruction>) {
        self.instruction_set = HashMap::new();
        for instruction in instruction_set {
            self.instruction_set.insert(instruction.symbol, instruction);
        }
    }

    pub fn read_char(&self, character: char) -> Option<Instruction> {
        self.instruction_set.get(&character).copied()
    }

    pub fn hotload<S: Into<String>>(&mut self, filename: S) -> Result<(), std::io::Error> {
        let code_text = std::fs::read_to_string(filename.into())?;
        self.code_lines = code_text
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<String>>();

        for (line_number, line) in self.code_lines.iter().enumerate() {
            for (column_number, character) in line.chars().enumerate() {
                if let Some(instruction) = self.read_char(character) {
                    self.engine.instructions.push(instruction);
                    self.instruction_positions
                        .push((line_number, column_number));
                }
            }
        }

        Ok(())
    }

    pub fn step(&mut self) {
        self.engine.step().ok();
    }

    pub fn undo(&mut self) {
        self.engine.undo().ok();
    }

    pub fn cursor(&self) -> Option<(usize, usize)> {
        match self.engine.instruction_pointer {
            InstructionPointer::Index(i) => Some(self.instruction_positions[i]),
            _ => None,
        }
    }
}
