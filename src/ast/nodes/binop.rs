use crate::ast::nodes::expressions::Expression;

#[derive(PartialEq, Clone, Debug)]
pub enum BinopEnum {
    Add,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Binop {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub binop: BinopEnum,
}
