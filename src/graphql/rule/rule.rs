#![cfg(feature = "graphql")]

use super::*;

#[derive(Serialize, Deserialize)]
pub struct Rule {
    conditions: Condition,
    statements: Statement
}

#[derive(Serialize, Deserialize)]
pub struct Condition {
    condition: IdValue
}

#[derive(Serialize, Deserialize)]
pub struct Statement {
    statement: IdValue
}
