mod lexer;
mod parser;
mod repl;
#[macro_use]
pub mod ast_walk_interpreter;
mod cps_interpreter;
pub mod interpreter;

pub use interpreter::Interpreter;
pub use ast_walk_interpreter::{
    Value,
    Environment,
    RuntimeError,
    evaluate_value,
    evaluate_values
};
