#![allow(dead_code)]

mod engine;
mod instruction;

fn main() {
    let engine = engine::Engine::new(vec![]);
    println!("{:?}", engine);
}
