use chumsky::span::SimpleSpan;
use string_interner::DefaultSymbol;

use crate::{
    ast::nodes::{SymbolWrapper, expressions::Expression},
    bindings::DeclarationLink,
};

/// Represent a call to a function.
/// TODO: add arguments.
/// TODO: replace name with a full path.
#[derive(Debug)]
pub struct FunctionCall {
    pub name: SymbolWrapper,
    pub span: SimpleSpan,
    pub arg: Option<Box<Expression>>,
    pub declaration: Option<DeclarationLink>,
}

impl FunctionCall {
    pub fn new(
        name: DefaultSymbol,
        span: SimpleSpan,
        arg: Option<Box<Expression>>,
    ) -> FunctionCall {
        FunctionCall {
            name: name.into(),
            span,
            arg,
            declaration: None,
        }
    }
}
