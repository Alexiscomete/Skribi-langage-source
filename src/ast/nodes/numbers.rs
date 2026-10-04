use chumsky::span::SimpleSpan;

use crate::ast::nodes::SymbolWrapper;

#[derive(Debug)]
pub struct Number {
    pub content: SymbolWrapper,
    pub span: SimpleSpan,
}
