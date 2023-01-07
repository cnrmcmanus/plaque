#![allow(dead_code, unstable_name_collisions)]
#![feature(iter_intersperse)]

mod app;
mod engine;
mod flavor;
mod instruction;
mod program;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    crossterm::terminal::enable_raw_mode()?;

    let input_filename = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::Error::msg("missing input filename"))?;

    let program =
        program::Program::load(input_filename, flavor::overflow::INSTRUCTION_SET.to_vec())?;

    app::run(program)
}
