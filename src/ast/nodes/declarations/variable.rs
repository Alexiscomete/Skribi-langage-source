use std::{cell::RefCell, rc::Rc};

use chumsky::span::SimpleSpan;

use crate::ast::nodes::{SymbolWrapper, expressions::Expression};

#[derive(Debug)]
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

/// Dynamic ownership warning:
/// Why this type? The variable declaration node must be referenced in its
/// usages, and they _may_ change some properties of the declaration. So we
/// need mutability and shared reference.
/// Why a pub type? We may want to change this to an arena later.
#[derive(Debug, Clone)]
pub struct VariableDeclarationRef {
    content: Rc<RefCell<VariableDeclaration>>,
}

impl VariableDeclarationRef {
    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&VariableDeclaration) -> R,
    {
        let guard = self.content.borrow();
        f(&guard)
    }

    pub fn write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut VariableDeclaration) -> R,
    {
        let mut guard = self.content.borrow_mut();
        f(&mut guard)
    }
}

impl From<VariableDeclaration> for VariableDeclarationRef {
    fn from(value: VariableDeclaration) -> Self {
        VariableDeclarationRef {
            content: Rc::new(RefCell::new(value)),
        }
    }
}
