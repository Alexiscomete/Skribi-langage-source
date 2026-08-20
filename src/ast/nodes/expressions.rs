use crate::ast::nodes::calls::functions::FunctionCall;

#[derive(PartialEq, Clone, Debug)]
pub enum Expression {
    FunctionCall(FunctionCall),
}
