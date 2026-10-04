use crate::ast::nodes::{
    binop::Binop,
    calls::{functions::FunctionCall, variable::VariableUsage},
    declarations::variable::VariableDeclarationRef,
    numbers::Number,
};

#[derive(Debug)]
pub enum Expression {
    FunctionCall(FunctionCall),
    Number(Number),
    Binop(Binop),
    VariableDeclaration(VariableDeclarationRef),
    VariableUsage(VariableUsage),
}
