//! Allows to convert anything into a span

use chumsky::span::{SimpleSpan, Span};
use miette::{Result, miette};

use crate::ast::{nodes::statements::Statement, visitors::AstVisitor};

struct IntoSpanVisitor {}

impl AstVisitor<'_, SimpleSpan> for IntoSpanVisitor {
    fn default_t(_: super::DefaultCause) -> miette::Result<SimpleSpan, miette::Error> {
        Err(miette!("Cannot find a valid span for this element"))
    }

    fn aggregate_t(mut current: Option<SimpleSpan>, new: SimpleSpan) -> Option<SimpleSpan> {
        let res = if let Some(current) = current {
            current.union(new)
        } else {
            new
        };
        current.replace(res);
        current
    }

    fn visit_deprecated(
        &self,
        deprecated: &crate::ast::nodes::deprecated::Deprecated,
    ) -> miette::Result<SimpleSpan, miette::Error> {
        Ok(deprecated.span)
    }

    fn visit_function_call(
        &self,
        function_call: &crate::ast::nodes::calls::functions::FunctionCall,
    ) -> miette::Result<SimpleSpan, miette::Error> {
        Ok(function_call.span)
    }
}

// Complete this section by adding new From impl

impl From<&Statement> for Result<SimpleSpan> {
    fn from(value: &Statement) -> Self {
        let visitor = IntoSpanVisitor {};
        visitor.visit_statement(&value)
    }
}
