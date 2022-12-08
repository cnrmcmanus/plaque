#![allow(dead_code)]

mod engine;
mod flavor;
mod instruction;
mod program;

fn main() -> Result<(), String> {
    let input_filename = std::env::args()
        .nth(1)
        .ok_or_else(|| "missing input filename".to_string())?;

    let program =
        program::Program::load(input_filename, flavor::overflow::INSTRUCTION_SET.to_vec())
            .map_err(|e| e.to_string())?;

    println!("{:?}\n", program);

    Ok(())
}
