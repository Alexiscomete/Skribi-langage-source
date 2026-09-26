//! Allows to calculate the depth of an expression.
//! Used while pretty printing
//! Note that a cache should be implemented if used in more critical parts

use miette::Result;

use crate::ast::{nodes::expressions::Expression, visitors::AstVisitor};

struct IntoSpanVisitor {}

impl AstVisitor<'_, usize> for IntoSpanVisitor {
    fn default_t(_: super::DefaultCause) -> miette::Result<usize, miette::Error> {
        Ok(0)
    }

    fn aggregate_t(mut current: Option<usize>, new: usize) -> Option<usize> {
        current.replace(current.map_or(new, |x| if x < new { new } else { x }));
        current
    }

    fn visit_expression(
        &self,
        expression: &crate::ast::nodes::expressions::Expression,
    ) -> Result<usize, miette::Error> {
        Ok(self.default_expression(expression)? + 1)
    }
}

pub fn expression_max_depth(value: &Expression) -> Result<usize> {
    let visitor = IntoSpanVisitor {};
    visitor.visit_expression(&value)
}
