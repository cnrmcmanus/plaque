use crate::engine::{Engine, EngineResult};

pub struct Instruction {
    pub symbol: char,
    pub exec: fn(&mut Engine) -> EngineResult,
    pub unexec: fn(&mut Engine) -> EngineResult,
}

impl Clone for Instruction {
    fn clone(&self) -> Instruction {
        Instruction {
            symbol: self.symbol,
            exec: self.exec,
            unexec: self.unexec,
        }
    }
}

impl Copy for Instruction {}

impl std::cmp::PartialEq for Instruction {
    fn eq(&self, other: &Instruction) -> bool {
        self.symbol == other.symbol
    }
}

impl std::cmp::Eq for Instruction {}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.symbol)
    }
}
