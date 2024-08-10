#![allow(dead_code)]
#![feature(iter_intersperse)]

mod app;
mod editor;
mod engine;
mod flavor;
mod instruction;
mod program;
mod ui;

use program::Program;

use anyhow::Result;

fn main() -> Result<()> {
    let input_filepath = std::env::args().nth(1);
    let flavor = flavor::overflow::INSTRUCTION_SET.to_vec();

    let mut program = match input_filepath {
        Some(filepath) => Program::load(filepath, flavor)?,
        None => Program::blank(flavor),
    };

    program.read_stdin();

    app::run(program)
}
