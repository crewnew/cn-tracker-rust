use std::str::FromStr;

pub type ConditionalFn = Box<dyn Fn() -> bool>;

pub trait Executable {
    fn execute(&self) -> anyhow::Result<()>;
}

pub struct Instruction(Box<dyn Fn() -> anyhow::Result<()>>);

impl <F: Fn() -> anyhow::Result<()> + 'static> From<F> for Instruction {
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
            }
        }
       
        if let Some(else_if_conditionals) = &self.else_if_conditionals {
            for conditional in else_if_conditionals {
                conditional.execute()?;
            }
        } else if let Some(else_instructions) = &self.else_instructions {
            for instruction in else_instructions {
                instruction.execute()?;
            }
        }

        Ok(())
    }
}
