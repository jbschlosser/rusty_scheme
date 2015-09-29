#[macro_use]
extern crate mopa;

mod lexer;
mod parser;
mod repl;
#[macro_use]
pub mod interpreter;

pub use interpreter::{
    Interpreter,
    Value,
    Environment,
    RuntimeError,
    Custom,
    evaluate_value,
    evaluate_values
};
