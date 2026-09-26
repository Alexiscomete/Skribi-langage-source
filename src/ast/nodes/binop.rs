use std::fmt::{Display, Write};

use crate::{ast::nodes::expressions::Expression, lexer::Tokens};

#[derive(PartialEq, Clone, Debug)]
pub enum BinopEnum {
    Add,
    Substract,
    Multiply,
    Divide,
}

impl From<Tokens> for BinopEnum {
    fn from(value: Tokens) -> Self {
        match value {
            Tokens::Mul => Self::Multiply,
            Tokens::Div => Self::Divide,
            Tokens::Plus => Self::Add,
            Tokens::Minus => Self::Substract,
            _ => unreachable!(),
        }
    }
}

impl Display for BinopEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char(match self {
           Self::Add => '+',
           Self::Substract => '-',
           Self::Multiply => '*',
           Self::Divide => '/'
        })
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Binop {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub binop: BinopEnum,
}
