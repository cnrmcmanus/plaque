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

    /// Given the index of a '[' or ']' instruction, returns the index of the matching jump
    pub fn matching_jump(index: usize, instructions: &[Instruction]) -> Option<usize> {
        let (mut open_brackets, mut close_brackets) = (0, 0);
        let mut indicies: Box<dyn Iterator<Item = usize>> = match instructions.get(index) {
            Some(JumpForward) => Box::new(index..instructions.len()),
            Some(JumpBackward) => Box::new((0..=index).rev()),
            _ => return None,
        };
        indicies.find(|&i| {
            match instructions[i] {
                JumpForward => open_brackets += 1,
                JumpBackward => close_brackets += 1,
                _ => {}
            }
            open_brackets == close_brackets
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(line: &str) -> Vec<Instruction> {
        line.chars().filter_map(Instruction::read).collect()
    }

    #[test]
    fn jump_matching() {
        let instr = parse("[]++][[]][[++[+]+++]][+++][++[]");

        assert_eq!((instr[21], instr[25]), (JumpForward, JumpBackward));
        assert_eq!(Instruction::matching_jump(21, &instr), Some(25));
        assert_eq!(Instruction::matching_jump(25, &instr), Some(21));

        assert_eq!((instr[9], instr[20]), (JumpForward, JumpBackward));
        assert_eq!(Instruction::matching_jump(9, &instr), Some(20));
        assert_eq!(Instruction::matching_jump(20, &instr), Some(9));

        assert_eq!((instr[26], instr[4]), (JumpForward, JumpBackward));
        assert_eq!(Instruction::matching_jump(26, &instr), None);
        assert_eq!(Instruction::matching_jump(4, &instr), None);

        assert_eq!(instr[2], Increment);
        assert_eq!(Instruction::matching_jump(2, &instr), None);
    }
}
