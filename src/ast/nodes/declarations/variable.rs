use chumsky::span::SimpleSpan;
use la_arena::Idx;
use miette::Result;

use crate::ast::nodes::{
    SymbolWrapper,
    declarations::variable_interner::{get_variable_interner},
    expressions::Expression,
};

/// Dynamic ownership warning:
/// Why this type? The variable declaration node must be referenced in its
/// usages, and they _may_ change some properties of the declaration. So we
/// need mutability and shared reference.
/// Why a pub type? We may want to change this to an arena later.
pub type VariableDeclarationRef = Idx<VariableDeclaration>;

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

impl TryFrom<VariableDeclaration> for VariableDeclarationRef {
    type Error = miette::Error;

    fn try_from(value: VariableDeclaration) -> Result<Self, Self::Error> {
        let mut interner = get_variable_interner()?;
        Ok(interner.alloc(value))
    }
}
