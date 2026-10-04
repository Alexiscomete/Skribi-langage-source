use chumsky::span::SimpleSpan;
use string_interner::DefaultSymbol;

use crate::ast::nodes::SymbolWrapper;

#[derive(Debug)]
pub struct VariableUsage {
    pub name: SymbolWrapper,
    pub span: SimpleSpan,
}

impl VariableUsage {
    pub fn new(name: DefaultSymbol, span: SimpleSpan) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}
