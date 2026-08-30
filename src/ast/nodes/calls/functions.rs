use chumsky::span::SimpleSpan;
use string_interner::DefaultSymbol;

use crate::ast::nodes::{SymbolWrapper, expressions::Expression};

/// Represent a call to a function.
/// TODO: add arguments.
/// TODO: replace name with a full path.
#[derive(PartialEq, Clone, Debug)]
pub struct FunctionCall {
    pub name: SymbolWrapper,
    pub span: SimpleSpan,
    pub arg: Option<Box<Expression>>,
}

impl FunctionCall {
    pub fn new(name: DefaultSymbol, span: SimpleSpan, arg: Option<Box<Expression>>) -> FunctionCall {
        FunctionCall {
            name: name.into(),
            span,
            arg,
        }
    }
}
