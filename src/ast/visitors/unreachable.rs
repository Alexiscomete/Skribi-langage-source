//! Allows to check for unreachable statements

use chumsky::span::{SimpleSpan, Span};
use log::trace;
use miette::{Diagnostic, LabeledSpan, Result};
use thiserror::Error;

use crate::{
    ast::{
        nodes::{FileTreeRoot, into_str},
        visitors::{DefaultCause, MutAstMutVisitor},
    },
    interner::get_interner,
};

#[derive(Error, Debug, Diagnostic)]
#[error("Found unreachable statements")]
#[diagnostic(severity(Warning))]
pub struct UnreachableWarning {
    #[label(collection)]
    spans: Vec<LabeledSpan>,
}

#[derive(Default)]
pub struct UnreachableVisitor {
    spans: Vec<LabeledSpan>,
}

impl UnreachableVisitor {
    pub fn find(file_tree_root: &mut FileTreeRoot) -> Result<Option<UnreachableWarning>> {
        let mut visitor = UnreachableVisitor::default();
        visitor.visit_file_tree_root(file_tree_root)?;
        if !visitor.spans.is_empty() {
            trace!("Found {} unreachable statements", visitor.spans.len());
            Ok(Some(UnreachableWarning {
                spans: visitor.spans,
            }))
        } else {
            Ok(None)
        }
    }
}

enum PruneState {
    Default,
    PruneFollowing,
}

impl MutAstMutVisitor<'_, PruneState> for UnreachableVisitor {
    fn default_t(_: super::DefaultCause) -> miette::Result<PruneState, miette::Error> {
        Ok(PruneState::Default)
    }

    fn visit_function_call(
        &mut self,
        function_call: &mut crate::ast::nodes::calls::functions::FunctionCall,
    ) -> miette::Result<PruneState, miette::Error> {
        let interner = get_interner()?;
        let name = into_str(&interner, function_call.name);

        match name {
            "exit" => Ok(PruneState::PruneFollowing),
            _ => Ok(PruneState::Default),
        }
    }

    fn visit_file_tree_root(
        &mut self,
        file_tree_root: &mut crate::ast::nodes::FileTreeRoot,
    ) -> miette::Result<PruneState, miette::Error> {
        let mut res = None;
        let mut start_index = None;
        trace!("There is {} statements", file_tree_root.content.len());
        for (index, statement) in &mut file_tree_root.content.iter_mut().enumerate() {
            res = Self::aggregate_t(res, self.visit_statement(statement)?);

            if let Some(PruneState::PruneFollowing) = res {
                start_index = Some(index);
                break;
            }
        };
        if let Some(start_index) = start_index && start_index + 1 != file_tree_root.content.len() {
            // Step 1: emit the warning
            let span_generator: Result<SimpleSpan> = (&file_tree_root.content[start_index]).into();
            let span_first: Result<SimpleSpan> = (&file_tree_root.content[start_index + 1]).into();
            let span_last: Result<SimpleSpan> = (&file_tree_root.content[file_tree_root.content.len() - 1]).into();

            self.spans.push(LabeledSpan::new_primary_with_span(
                    Some("This statement makes the following unreachable".to_owned()),
                    span_generator?.into_range(),
            ));

            self.spans.push(LabeledSpan::new_primary_with_span(
                    Some("This code is unreachable, it has been pruned".to_owned()),
                    span_first?.union(span_last?).into_range(),
            ));
            // Step 2: drain everything after
            file_tree_root.content.drain(start_index..);
        }
        trace!("There is {} statements", file_tree_root.content.len());
        if let Some(t) = res {
            Ok(t)
        } else {
            Self::default_t(DefaultCause::ZeroElements)
        }
    }
}
