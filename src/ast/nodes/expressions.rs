use crate::ast::nodes::{binop::Binop, calls::functions::FunctionCall, numbers::Number};

#[derive(PartialEq, Clone, Debug)]
pub enum Expression {
    FunctionCall(FunctionCall),
    Number(Number),
    Binop(Binop),
}
