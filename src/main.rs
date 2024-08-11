#![allow(dead_code)]
#![feature(iter_intersperse)]

mod app;
mod editor;
mod engine;
mod instruction;
mod program;
mod ui;

use program::Program;

use anyhow::Result;

fn main() -> Result<()> {
    let input_filepath = std::env::args().nth(1);

    let mut program = match input_filepath {
        Some(filepath) => Program::load(filepath)?,
        None => Program::blank(),
    };

    program.read_stdin();

    app::run(program)
}
