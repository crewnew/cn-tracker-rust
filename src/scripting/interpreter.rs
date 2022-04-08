use lazy_static::lazy_static;
use libc::c_void;
use rustc_hash::FxHashMap;
use std::{
    str::FromStr,
    sync::RwLock
};

pub type VariablesMapType = FxHashMap<String, Variable>;
pub type ConditionalFn = Box<dyn Fn() -> bool>;
pub trait Executable {
    fn execute(&self) -> anyhow::Result<()>;
}

lazy_static! {
    pub static ref VARIABLES_MAP: RwLock<VariablesMapType> = Default::default();
}

#[derive(PartialEq)]
pub enum Variable {
    Int(usize),
    Str(String),
    Bool(bool),
    Vector(Vec<Variable>),
    Map(VariablesMapType),
}

impl From<&str> for Variable {
    fn from(string: &str) -> Self {
       Self::Str(string.into())
    }
}

impl From<String> for Variable {
    fn from(string: String) -> Self {
        Self::Str(string)
    }
}

impl From<usize> for Variable {
    fn from(number: usize) -> Self {
        Self::Int(number)
    }
}

impl From<bool> for Variable {
    fn from(boolean: bool) -> Self {
        Self::Bool(boolean)
    }
}

impl From<VariablesMapType> for Variable {
    fn from(variables_map: VariablesMapType) -> Self {
        Self::Map(variables_map)
    }
}

impl PartialEq<String> for Variable {
    fn eq(&self, other: &String) -> bool {
        use Variable::*;
        match self {
            Int(i) => {
                let num: usize = match other.parse() {
                    Ok(num) => num,
                    Err(_) => return false,
                };
                *i == num
            }
            Str(string) => string == other,
            Bool(boolean) => {
                let boolean = *boolean;
                if boolean && other == "true" {
                    true
                } else if !boolean && other == "false" {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

pub struct Instruction(Box<dyn Fn() -> anyhow::Result<()>>);

impl<F: Fn() -> anyhow::Result<()> + 'static> From<F> for Instruction {
    fn from(function: F) -> Self {
        Self(Box::new(function))
    }
}

impl Executable for Instruction {
    fn execute(&self) -> anyhow::Result<()> {
        (self.0)()?;
        Ok(())
    }
}

#[derive(Default)]
pub struct Conditional {
    pub conditions: Vec<Vec<ConditionalFn>>,
    pub instructions: Vec<Instruction>,
    pub else_if_conditionals: Option<Vec<Conditional>>,
    pub else_instructions: Option<Vec<Instruction>>,
}

impl Executable for Conditional {
    fn execute(&self) -> anyhow::Result<()> {
        for conditions in &self.conditions {
            let mut should_execute = false;

            for condition in conditions {
                should_execute = condition();
            }

            if should_execute {
                for instruction in &self.instructions {
                    instruction.execute()?;
                }
                return Ok(())
            }
        }

        if let Some(else_if_conditionals) = &self.else_if_conditionals {
            for conditional in else_if_conditionals{
                if conditional.execute().is_ok() {
                    return Ok(());
                }
            }
        } else if let Some(else_instructions) = &self.else_instructions {
            for instruction in else_instructions {
                instruction.execute()?;
            }
            return Ok(());
        }
        
        anyhow::bail!("Didn't execute anything.");
    }
}
