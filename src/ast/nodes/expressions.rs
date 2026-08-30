use crate::ast::nodes::{calls::functions::FunctionCall, numbers::Number};

#[derive(PartialEq, Clone, Debug)]
pub enum Expression {
    FunctionCall(FunctionCall),
    Number(Number),
}
