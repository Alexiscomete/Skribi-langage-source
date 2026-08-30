use chumsky::span::SimpleSpan;

use crate::ast::nodes::SymbolWrapper;

#[derive(PartialEq, Clone, Debug)]
pub struct Number {
    pub content: SymbolWrapper,
    pub span: SimpleSpan,
}
