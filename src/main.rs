#![allow(dead_code)]

mod engine;
mod flavor;
mod instruction;

fn main() {
    let engine = engine::Engine::new(vec![]);
    println!("{:?}", engine);
}
