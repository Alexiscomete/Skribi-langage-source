use std::{
    fmt::{Debug, Display, Error},
    sync::Arc,
};

use log::error;
use string_interner::DefaultSymbol;

use crate::{
    ast::nodes::statements::Statement,
    file::File,
    interner::{INTERNER, Interner},
};

pub mod calls;
pub mod deprecated;
pub mod expressions;
pub mod statements;

#[derive(Debug)]
pub struct FileTreeRoot {
    pub content: Vec<Statement>,
    pub file: Option<Arc<File>>,
}

impl FileTreeRoot {
    pub fn new(content: Vec<Statement>) -> FileTreeRoot {
        FileTreeRoot {
            content,
            file: None,
        }
    }
}

#[derive(Clone, PartialEq, Copy)]
pub struct SymbolWrapper {
    pub symbol: DefaultSymbol,
}

impl From<DefaultSymbol> for SymbolWrapper {
    fn from(value: DefaultSymbol) -> Self {
        SymbolWrapper { symbol: value }
    }
}

impl From<SymbolWrapper> for DefaultSymbol {
    fn from(val: SymbolWrapper) -> Self {
        val.symbol
    }
}

pub fn into_str<'a>(interner: &'a Interner, symbol: SymbolWrapper) -> &'a str {
    interner.resolve(symbol.into()).unwrap_or("ERROR")
}

impl Debug for SymbolWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let interner = INTERNER.lock().map_err(|_| {
            error!("Failed to get the lock");
            Error
        })?;
        let name = interner.resolve(self.symbol).unwrap_or("ERROR");
        write!(f, "{}", name)
    }
}

impl Display for SymbolWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let interner = INTERNER.lock().map_err(|_| {
            error!("Failed to get the lock");
            Error
        })?;
        let name = interner.resolve(self.symbol).unwrap_or("ERROR");
        write!(f, "{}", name)
    }
}
