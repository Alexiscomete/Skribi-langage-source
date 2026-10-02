use chumsky::span::SimpleSpan;

use crate::ast::nodes::{SymbolWrapper, expressions::Expression};

#[derive(PartialEq, Clone, Debug)]
pub struct VariableDeclaration {
    pub name: SymbolWrapper,
    pub allocated_type: SymbolWrapper,
    pub span: SimpleSpan,
    pub arg: Box<Expression>,
}

impl VariableDeclaration {
    pub fn new(
        name: SymbolWrapper,
        allocated_type: SymbolWrapper,
        span: SimpleSpan,
        arg: Expression,
    ) -> Self {
        VariableDeclaration {
            name,
            allocated_type,
            span,
            arg: Box::new(arg),
        }
    }
}
