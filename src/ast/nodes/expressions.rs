use crate::ast::nodes::{
    binop::Binop, calls::functions::FunctionCall, declarations::variable::VariableDeclarationRef,
    numbers::Number,
};

#[derive(PartialEq, Clone, Debug)]
pub enum Expression {
    FunctionCall(FunctionCall),
    Number(Number),
    Binop(Binop),
    VariableDeclaration(VariableDeclarationRef),
}
