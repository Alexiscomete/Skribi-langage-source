use crate::ast::nodes::{deprecated::Deprecated, expressions::Expression};

#[derive(PartialEq, Clone, Debug)]
pub enum Statement {
    Expression(Expression),
    Deprecated(Deprecated),
}
