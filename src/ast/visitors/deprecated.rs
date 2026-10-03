use miette::{Diagnostic, LabeledSpan, Result};
use thiserror::Error;

use crate::ast::{nodes::{FileTreeRoot, declarations::variable_interner::get_variable_interner}, visitors::AstMutVisitor};

#[derive(Default)]
pub struct DeprecatedNodesVisitor {
    spans: Vec<LabeledSpan>,
}

impl AstMutVisitor<'_, ()> for DeprecatedNodesVisitor {
    fn default_t(_: super::DefaultCause) -> miette::Result<(), miette::Error> {
        Ok(())
    }

    fn get_variable_interner() -> Result<crate::ast::nodes::declarations::variable_interner::VariableLocked, miette::Error> {
        get_variable_interner()
    }

    fn visit_deprecated(
        &mut self,
        deprecated: &crate::ast::nodes::deprecated::Deprecated,
    ) -> miette::Result<(), miette::Error> {
        self.spans.push(LabeledSpan::new_with_span(
            Some(deprecated.message.to_owned()),
            deprecated.span.into_range(),
        ));
        self.default_deprecated(deprecated)
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Found deprecated parsing features")]
#[diagnostic(severity(Warning))]
pub struct DeprecatedWarning {
    #[label(collection)]
    spans: Vec<LabeledSpan>,
}

impl DeprecatedNodesVisitor {
    pub fn find(file_tree_root: &FileTreeRoot) -> Result<Option<DeprecatedWarning>> {
        let mut visitor = DeprecatedNodesVisitor::default();
        visitor.visit_file_tree_root(file_tree_root)?;
        if !visitor.spans.is_empty() {
            Ok(Some(DeprecatedWarning {
                spans: visitor.spans,
            }))
        } else {
            Ok(None)
        }
    }
}
