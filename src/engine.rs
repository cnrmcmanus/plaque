use crate::instruction::Instruction;

use tap::prelude::*;

#[derive(Debug, Eq, PartialEq)]
pub enum Exception {
    Error(String),
    RequestingInput,
    Breakpoint,
}

impl Exception {
    pub fn error<S: Into<String>>(message: S) -> Exception {
        Exception::Error(message.into())
    }

    pub fn result<T>(self) -> Result<T, Exception> {
        Err(self)
    }
}

pub type EngineResult = Result<(), Exception>;

#[derive(Debug, Eq, PartialEq)]
pub enum InstructionPointer {
    Start,
    End,
    Index(usize),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Engine {
    pub tape: Vec<u8>,
    pub tape_pointer: usize,
    pub instructions: Vec<Instruction>,
    pub instruction_pointer: InstructionPointer,
    pub history: Vec<Instruction>,
    pub output: Vec<u8>,
    pub input: Vec<u8>,
    pub input_cell_history: Vec<u8>,
}

impl Engine {
    pub fn new(instructions: Vec<Instruction>) -> Engine {
        Engine {
            tape: vec![0],
            tape_pointer: 0,
            instructions,
            instruction_pointer: InstructionPointer::Start,
            history: vec![],
            output: vec![],
            input: vec![],
            input_cell_history: vec![],
        }
    }

    pub fn load_instructions(&mut self, instructions: Vec<Instruction>) {
        self.instructions = instructions;
    }

    pub fn goto(&mut self, instruction_index: usize) -> EngineResult {
        if instruction_index < self.instructions.len() {
            self.instruction_pointer = InstructionPointer::Index(instruction_index);
            Ok(())
        } else {
            Exception::error(format!(
                "no instruction at position {} (max {})",
                instruction_index,
                self.instructions.len() - 1
            ))
            .result()
        }
    }

    pub fn step(&mut self) -> EngineResult {
        match self.current_instruction() {
            Some(instruction) => (instruction.exec)(self)
                .tap(|_| self.history.push(instruction))
                .tap_err(|e| {
                    if e == &Exception::Breakpoint {
                        self.history.push(instruction)
                    }
                }),
            None => self.next_instruction(),
        }
    }

    pub fn undo(&mut self) -> EngineResult {
        let instruction = self
            .history
            .last()
            .ok_or_else(|| Exception::error("no previous instruction to undo"))?;

        (instruction.unexec)(self)
            .tap(|_| {
                self.history.pop();
            })
            .tap_err(|e| {
                if e == &Exception::Breakpoint {
                    self.history.pop();
                }
            })
    }

    pub fn reset(&mut self) {
        self.tape = vec![0];
        self.tape_pointer = 0;
        self.instruction_pointer = InstructionPointer::Start;
        self.history = vec![];
        self.output = vec![];
        self.input = vec![];
        self.input_cell_history = vec![];
    }

    pub fn current_instruction(&self) -> Option<Instruction> {
        match self.instruction_pointer {
            InstructionPointer::Start => None,
            InstructionPointer::End => None,
            InstructionPointer::Index(i) => Some(self.instructions[i]),
        }
    }

    pub fn next_instruction(&mut self) -> EngineResult {
        match self.instruction_pointer {
            InstructionPointer::End => {
                Exception::error("already at the end of the instruction list").result()
            }
            InstructionPointer::Start => {
                if self.instructions.is_empty() {
                    Exception::error("no instructions").result()
                } else {
                    self.instruction_pointer = InstructionPointer::Index(0);
                    Ok(())
                }
            }
            InstructionPointer::Index(i) if i + 1 == self.instructions.len() => {
                self.instruction_pointer = InstructionPointer::End;
                Ok(())
            }
            InstructionPointer::Index(i) => {
                self.instruction_pointer = InstructionPointer::Index(i + 1);
                Ok(())
            }
        }
    }

    pub fn prev_instruction(&mut self) -> EngineResult {
        match self.instruction_pointer {
            InstructionPointer::Start => {
                Exception::error("already at the start of the instruction list").result()
            }
            InstructionPointer::End => {
                self.instruction_pointer = InstructionPointer::Index(self.instructions.len() - 1);
                Ok(())
            }
            InstructionPointer::Index(i) if i == 0 => {
                self.instruction_pointer = InstructionPointer::Start;
                Ok(())
            }
            InstructionPointer::Index(i) => {
                self.instruction_pointer = InstructionPointer::Index(i - 1);
                Ok(())
            }
        }
    }

    pub fn goto_next(&mut self, goto: Instruction, matching: Instruction) -> EngineResult {
        let start = match self.instruction_pointer {
            InstructionPointer::End => {
                Exception::error("already at the end of the instruction list").result()
            }
            InstructionPointer::Start => Ok(0),
            InstructionPointer::Index(i) => Ok(i + 1),
        }?;

        let rest = self.instructions.iter().skip(start);
        let mut skip = 0;
        for (i, instruction) in rest.enumerate() {
            if instruction == &goto {
                if skip == 0 {
                    self.instruction_pointer = InstructionPointer::Index(start + i);
                    return Ok(());
                } else {
                    skip -= 1;
                }
            } else if instruction == &matching {
                skip += 1;
            }
        }

        Exception::error(format!("no next {} instruction found", goto.symbol)).result()
    }

    pub fn goto_prev(&mut self, goto: Instruction, matching: Instruction) -> EngineResult {
        let end = match self.instruction_pointer {
            InstructionPointer::Start => {
                Exception::error("already at the start of the instruction list").result()
            }
            InstructionPointer::End => Ok(self.instructions.len() - 1),
            InstructionPointer::Index(i) => Ok(i),
        }?;

        let rest = self.instructions.iter().take(end);
        let mut skip = 0;
        for (i, instruction) in rest.rev().enumerate() {
            if instruction == &goto {
                if skip == 0 {
                    self.instruction_pointer = InstructionPointer::Index(end - i - 1);
                    return Ok(());
                } else {
                    skip -= 1;
                }
            } else if instruction == &matching {
                skip += 1;
            }
        }

        Exception::error(format!("no previous {} instruction found", goto.symbol)).result()
    }

    pub fn next_cell(&mut self) -> EngineResult {
        self.tape_pointer += 1;
        // expand the tape if the cell is new
        if self.tape_pointer == self.tape.len() {
            self.tape.push(0);
        }

        Ok(())
    }

    pub fn prev_cell(&mut self) -> EngineResult {
        if self.tape_pointer != 0 {
            self.tape_pointer -= 1;
            Ok(())
        } else {
            Exception::error("can't decrement intruction pointer: already at first instruction")
                .result()
        }
    }

    pub fn cell(&self) -> u8 {
        self.tape[self.tape_pointer]
    }

    pub fn set_cell(&mut self, value: u8) {
        self.tape[self.tape_pointer] = value;
    }

    pub fn map_cell(&mut self, f: fn(u8) -> u8) {
        let value = self.cell();
        self.set_cell(f(value));
    }

    pub fn pop_input(&mut self) -> Option<u8> {
        let head = self.input.first().cloned();
        if head.is_some() {
            self.input.remove(0);
        }
        head
    }

    pub fn push_input(&mut self, head: u8) {
        self.input.insert(0, head);
    }

    pub fn input(&mut self, buffered: &mut Vec<u8>) {
        let mut input = vec![];
        input.append(buffered);
        input.append(&mut self.input);
        self.input = input;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOOP_A: Instruction = Instruction {
        symbol: 'a',
        exec: |_| Ok(()),
        unexec: |_| Ok(()),
    };
    const NOOP_B: Instruction = Instruction {
        symbol: 'b',
        exec: |_| Ok(()),
        unexec: |_| Ok(()),
    };
    const NOOP_C: Instruction = Instruction {
        symbol: 'c',
        exec: |_| Ok(()),
        unexec: |_| Ok(()),
    };

    fn ok(result: EngineResult) {
        assert_eq!(result, Ok(()))
    }

    #[test]
    fn new_builds_blank_program() {
        let program = Engine::new(vec![NOOP_A, NOOP_B, NOOP_C]);

        assert_eq!(
            program,
            Engine {
                tape: vec![0],
                tape_pointer: 0,
                instructions: vec![NOOP_A, NOOP_B, NOOP_C],
                instruction_pointer: InstructionPointer::Start,
                history: vec![],
                output: vec![],
                input: vec![],
                input_cell_history: vec![],
            }
        );
    }

    #[test]
    fn goto_sets_instruction_pointer() {
        let mut program = Engine::new(vec![NOOP_A, NOOP_B, NOOP_C]);

        ok(program.goto(1));

        assert_eq!(program.current_instruction(), Some(NOOP_B));
        assert_eq!(program.instruction_pointer, InstructionPointer::Index(1));
    }

    #[test]
    fn goto_overrun_fails_gracefully() {
        let mut program = Engine::new(vec![NOOP_A, NOOP_B, NOOP_C]);

        assert!(program.goto(3).is_err());
        assert_eq!(program.instruction_pointer, InstructionPointer::Start);
    }

    #[test]
    fn goto_next_moves_to_next_instruction() {
        let mut program = Engine::new(vec![NOOP_A, NOOP_B, NOOP_C, NOOP_B, NOOP_A, NOOP_C]);

        ok(program.goto(0));
        ok(program.goto_next(NOOP_C, NOOP_A));

        assert_eq!(program.current_instruction(), Some(NOOP_C));
        assert_eq!(program.instruction_pointer, InstructionPointer::Index(2));
    }

    #[test]
    fn goto_next_matches_nesting() {
        let mut program = Engine::new(vec![NOOP_A, NOOP_B, NOOP_A, NOOP_C, NOOP_B, NOOP_C]);

        ok(program.goto(0));
        ok(program.goto_next(NOOP_C, NOOP_A));

        assert_eq!(program.current_instruction(), Some(NOOP_C));
        assert_eq!(program.instruction_pointer, InstructionPointer::Index(5));
    }

    #[test]
    fn goto_next_fails_gracefully_on_overrun() {
        let mut program = Engine::new(vec![NOOP_A, NOOP_B, NOOP_C, NOOP_A]);

        ok(program.goto(0));
        ok(program.goto_next(NOOP_C, NOOP_A));

        assert!(program.goto_next(NOOP_C, NOOP_A).is_err());
        assert_eq!(program.current_instruction(), Some(NOOP_C));
        assert_eq!(program.instruction_pointer, InstructionPointer::Index(2));
    }

    #[test]
    fn goto_prev_moves_to_prev_instruction() {
        let mut program = Engine::new(vec![NOOP_A, NOOP_B, NOOP_C, NOOP_B, NOOP_A, NOOP_C]);

        ok(program.goto(5));
        ok(program.goto_prev(NOOP_A, NOOP_C));

        assert_eq!(program.current_instruction(), Some(NOOP_A));
        assert_eq!(program.instruction_pointer, InstructionPointer::Index(4));
    }

    #[test]
    fn goto_prev_nmatches_nesting() {
        let mut program = Engine::new(vec![NOOP_A, NOOP_B, NOOP_A, NOOP_C, NOOP_B, NOOP_C]);

        ok(program.goto(5));
        ok(program.goto_prev(NOOP_A, NOOP_C));

        assert_eq!(program.current_instruction(), Some(NOOP_A));
        assert_eq!(program.instruction_pointer, InstructionPointer::Index(0));
    }

    #[test]
    fn goto_prev_fails_gracefully_on_underrun() {
        let mut program = Engine::new(vec![NOOP_C, NOOP_A, NOOP_B, NOOP_C]);

        ok(program.goto(3));
        ok(program.goto_prev(NOOP_A, NOOP_C));

        assert!(program.goto_prev(NOOP_A, NOOP_C).is_err());
        assert_eq!(program.current_instruction(), Some(NOOP_A));
        assert_eq!(program.instruction_pointer, InstructionPointer::Index(1));
    }
}
