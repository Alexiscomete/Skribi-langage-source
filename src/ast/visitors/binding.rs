//! Associate each variable with its declaration.
//! This first version is not using the best data structure for scopes, may be
//! enhanced later.

use std::collections::HashMap;

use chumsky::span::SimpleSpan;
use log::trace;
use miette::{Diagnostic, Result, SourceSpan};
use thiserror::Error;

use crate::ast::{
    nodes::{FileTreeRoot, SymbolWrapper, declarations::variable::VariableDeclarationRef},
    visitors::MutAstMutVisitor,
};

/// Allocation fast, free fast, however recursive search can cost
#[derive(Default)]
pub struct Scope {
    parent: Option<Box<Scope>>,
    map: HashMap<SymbolWrapper, VariableDeclarationRef>,
}

impl Scope {
    fn push(scope: Option<Box<Self>>) -> Box<Self> {
        Box::new(Self {
            parent: scope,
            map: HashMap::default(),
        })
    }

    fn pop(scope: Option<Box<Self>>) -> Option<Box<Self>> {
        if let Some(inner) = scope {
            inner.parent
        } else {
            None
        }
    }

    fn find(scope: &Option<Box<Self>>, symbol: SymbolWrapper) -> Option<VariableDeclarationRef> {
        if let Some(content) = scope {
            if let Some(dec) = content.map.get(&symbol) {
                Some(dec.clone())
            } else {
                Self::find(&content.parent, symbol)
            }
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct Binder {
    root: Option<Box<Scope>>,
}

impl Binder {
    pub fn bind(root: &mut FileTreeRoot) -> Result<()> {
        let mut binder = Self::default();
        binder.root = Some(Scope::push(binder.root));

        binder.visit_file_tree_root(root)?;

        binder.root = Scope::pop(binder.root);
        Ok(())
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Already declared variable `{name}`")]
pub struct AlreadyDeclaredError {
    name: SymbolWrapper,
    #[label("`{name}` previously declared there")]
    span_from: SourceSpan,
    #[label(primary, "`{name}` was redeclared here")]
    span_to: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Undeclared variable `{name}` detected")]
pub struct UndeclaredError {
    name: SymbolWrapper,
    #[label("`{name}` was never declared")]
    span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Out of scope declaration")]
pub struct ScopeError {
    #[label("Invalid declaration position")]
    span: SourceSpan,
}

impl MutAstMutVisitor<'_, ()> for Binder {
    fn default_t(_: super::DefaultCause) -> miette::Result<(), miette::Error> {
        Ok(())
    }

    fn visit_variable_declaration(
        &mut self,
        variable_declaration: &mut crate::ast::nodes::declarations::variable::VariableDeclarationRef,
    ) -> miette::Result<(), miette::Error> {
        let name = variable_declaration.read(|x| x.name);

        self.default_variable_declaration(variable_declaration)?;

        if let Some(declaration) = Scope::find(&self.root, name) {
            let span_from: SimpleSpan = declaration.try_into()?;
            let span_to: SimpleSpan = variable_declaration.clone().try_into()?;

            Err(AlreadyDeclaredError {
                name: name,
                span_from: span_from.into_range().into(),
                span_to: span_to.into_range().into(),
            }
            .into())
        } else if let Some(scope) = &mut self.root {
            trace!("Declaring {name}");
            scope.map.insert(name, variable_declaration.clone());
            Ok(())
        } else {
            let span_to: SimpleSpan = variable_declaration.clone().try_into()?;

            Err(ScopeError {
                span: span_to.into_range().into(),
            }
            .into())
        }
    }

    fn visit_variable_usage(
        &mut self,
        variable_usage: &mut crate::ast::nodes::calls::variable::VariableUsage,
    ) -> Result<(), miette::Error> {
        let name = variable_usage.name;

        if let Some(declaration) = Scope::find(&self.root, name) {
            variable_usage.declaration = Some(declaration);
            Ok(())
        } else {
            let span: SimpleSpan = variable_usage.try_into()?;

            Err(UndeclaredError {
                name: name,
                span: span.into_range().into(),
            }
            .into())
        }
    }
}
