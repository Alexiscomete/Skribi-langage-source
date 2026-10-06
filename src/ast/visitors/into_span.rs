//! Allows to convert anything into a span

use chumsky::span::{SimpleSpan, Span};
use miette::{Result, miette};

use crate::ast::{
    nodes::{
        calls::variable::VariableUsage, declarations::variable::VariableDeclarationRef,
        statements::Statement,
    },
    visitors::AstVisitor,
};

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

    fn visit_number(
        &self,
        number: &crate::ast::nodes::numbers::Number,
    ) -> Result<SimpleSpan, miette::Error> {
        Ok(number.span)
    }

    fn visit_variable_declaration(
        &self,
        variable_declaration: &crate::ast::nodes::declarations::variable::VariableDeclarationRef,
    ) -> Result<SimpleSpan, miette::Error> {
        Ok(variable_declaration.read(|v| v.span))
    }

    fn visit_variable_usage(
        &self,
        variable_usage: &crate::ast::nodes::calls::variable::VariableUsage,
    ) -> Result<SimpleSpan, miette::Error> {
        Ok(variable_usage.span)
    }
}

impl From<&Statement> for Result<SimpleSpan> {
    fn from(value: &Statement) -> Self {
        let visitor = IntoSpanVisitor {};
        visitor.visit_statement(value)
    }
}

impl TryFrom<VariableDeclarationRef> for SimpleSpan {
    type Error = miette::ErrReport;

    fn try_from(value: VariableDeclarationRef) -> Result<Self> {
        let visitor = IntoSpanVisitor {};
        visitor.visit_variable_declaration(&value)
    }
}

impl TryFrom<&mut VariableUsage> for SimpleSpan {
    type Error = miette::ErrReport;

    fn try_from(value: &mut VariableUsage) -> Result<Self> {
        let visitor = IntoSpanVisitor {};
        visitor.visit_variable_usage(value)
    }
}
