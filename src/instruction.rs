#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction {
    IncrementPointer,
    DecrementPointer,
    Increment,
    Decrement,
    Output,
    Input,
    JumpForward,
    JumpBackward,
    Breakpoint,
}

pub use Instruction::*;

impl Instruction {
    pub fn read(symbol: char) -> Option<Instruction> {
        match symbol {
            '>' => Some(IncrementPointer),
            '<' => Some(DecrementPointer),
            '+' => Some(Increment),
            '-' => Some(Decrement),
            '.' => Some(Output),
            ',' => Some(Input),
            '[' => Some(JumpForward),
            ']' => Some(JumpBackward),
            '$' => Some(Breakpoint),
            _ => None,
        }
    }

    pub fn symbol(&self) -> char {
        match self {
            IncrementPointer => '>',
            DecrementPointer => '<',
            Increment => '+',
            Decrement => '-',
            Output => '.',
            Input => ',',
            JumpForward => '[',
            JumpBackward => ']',
            Breakpoint => '$',
        }
    }
}
