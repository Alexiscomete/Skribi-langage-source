use crate::ast::nodes::{deprecated::Deprecated, expressions::Expression};

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    Deprecated(Deprecated),
}
