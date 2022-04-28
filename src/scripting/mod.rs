mod interpreter;
/// Parse -> Interpret instructions -> Pass the instructions into an execution thread -> Execute
/// instructions
mod parser;

pub use interpreter::*;
pub use parser::*;
