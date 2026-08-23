use chumsky::span::SimpleSpan;
use string_interner::DefaultSymbol;

use crate::ast::nodes::SymbolWrapper;

/// Represent a call to a function.
/// TODO: add arguments.
/// TODO: replace name with a full path.
#[derive(PartialEq, Clone, Debug)]
pub struct FunctionCall {
    pub name: SymbolWrapper,
    pub span: SimpleSpan,
}

impl FunctionCall {
    pub fn new(name: DefaultSymbol, span: SimpleSpan) -> FunctionCall {
        FunctionCall {
            name: name.into(),
            span,
        }
    }
}
