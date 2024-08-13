use crate::instruction::Instruction::{self, *};

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

    pub fn execute(&mut self, instruction: Instruction) -> EngineResult {
        match instruction {
            IncrementPointer => {
                self.next_cell()?;
                self.next_instruction()
            }
            DecrementPointer => {
                self.prev_cell()?;
                self.next_instruction()
            }
            Increment => {
                self.map_cell(|cell| cell.wrapping_add(1));
                self.next_instruction()
            }
            Decrement => {
                self.map_cell(|cell| cell.wrapping_sub(1));
                self.next_instruction()
            }
            Output => {
                self.output.push(self.cell());
                self.next_instruction()
            }
            Input => match self.pop_input() {
                None => {
                    let cell = self.cell();
                    self.set_cell(0);
                    self.input_cell_history.push(cell);
                    self.next_instruction()
                }
                Some(input) => {
                    let cell = self.cell();
                    self.set_cell(input);
                    self.input_cell_history.push(cell);
                    self.next_instruction()
                }
            },
            JumpForward => match self.cell() {
                0 => self.goto_matching_jump(),
                _ => self.next_instruction(),
            },
            JumpBackward => match self.cell() {
                0 => self.next_instruction(),
                _ => self.goto_matching_jump(),
            },
            Breakpoint => {
                self.next_instruction()?;
                Exception::Breakpoint.result()
            }
        }
    }

    pub fn unexecute(&mut self, instruction: Instruction) -> EngineResult {
        match instruction {
            IncrementPointer => {
                self.prev_cell()?;
                self.prev_instruction()
            }
            DecrementPointer => {
                self.next_cell()?;
                self.prev_instruction()
            }
            Increment => {
                self.map_cell(|cell| cell.wrapping_sub(1));
                self.prev_instruction()
            }
            Decrement => {
                self.map_cell(|cell| cell.wrapping_add(1));
                self.prev_instruction()
            }
            Output => {
                self.output.pop();
                self.prev_instruction()
            }
            Input => match self.input_cell_history.pop() {
                None => Exception::error("no input to undo").result(),
                Some(cell) => {
                    let input = self.cell();
                    self.set_cell(cell);
                    self.push_input(input);
                    self.prev_instruction()
                }
            },
            JumpForward => match self.cell() {
                0 => self.goto_matching_jump(),
                _ => self.prev_instruction(),
            },
            JumpBackward => match self.cell() {
                0 => self.prev_instruction(),
                _ => self.goto_matching_jump(),
            },
            Breakpoint => {
                self.prev_instruction()?;
                Exception::Breakpoint.result()
            }
        }
    }

    pub fn step(&mut self) -> EngineResult {
        match self.current_instruction() {
            Some(instruction) => self
                .execute(instruction)
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
            .cloned()
            .ok_or_else(|| Exception::error("no previous instruction to undo"))?;

        self.unexecute(instruction)
            .tap(|_| {
                self.history.pop();
            })
            .tap_err(|e| {
                if e == &Exception::Breakpoint {
                    self.history.pop();
                }
            })
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

    pub fn goto_matching_jump(&mut self) -> EngineResult {
        let err = || Exception::error("no matching jump");
        let start = match self.instruction_pointer {
            InstructionPointer::Index(i) => Ok(i),
            _ => err().result(),
        }?;
        let i = Instruction::matching_jump(start, &self.instructions).ok_or_else(err)?;
        self.goto(i)
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

    fn ok(result: EngineResult) {
        assert_eq!(result, Ok(()))
    }

    fn engine() -> Engine {
        Engine::new(vec![JumpForward, JumpForward, Increment, JumpBackward])
    }

    #[test]
    fn goto_sets_instruction_pointer() {
        let mut engine = engine();
        ok(engine.goto(2));
        assert_eq!(engine.current_instruction(), Some(Increment));
        assert_eq!(engine.instruction_pointer, InstructionPointer::Index(2));
    }

    #[test]
    fn goto_overrun_fails_gracefully() {
        let mut engine = engine();
        assert!(engine.goto(4).is_err());
        assert_eq!(engine.instruction_pointer, InstructionPointer::Start);
    }

    #[test]
    fn goto_matching_sets_instruction_pointer() {
        let mut engine = engine();
        ok(engine.goto(1));
        ok(engine.goto_matching_jump());
        assert_eq!(engine.current_instruction(), Some(JumpBackward));
        assert_eq!(engine.instruction_pointer, InstructionPointer::Index(3));
        ok(engine.goto_matching_jump());
        assert_eq!(engine.current_instruction(), Some(JumpForward));
        assert_eq!(engine.instruction_pointer, InstructionPointer::Index(1));
    }

    #[test]
    fn goto_matching_fails_gracefully_on_overrun() {
        let mut engine = engine();
        ok(engine.goto(0));
        assert!(engine.goto_matching_jump().is_err());
        assert_eq!(engine.current_instruction(), Some(JumpForward));
        assert_eq!(engine.instruction_pointer, InstructionPointer::Index(0));
    }
}
