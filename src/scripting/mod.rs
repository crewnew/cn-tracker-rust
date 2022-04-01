/// Parse -> Interpret instructions -> Pass the instructions into an execution thread -> Execute
/// instructions

mod parser;
mod interpreter;

pub use interpreter::*;
pub use parser::*;
