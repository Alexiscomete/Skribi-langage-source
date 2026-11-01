use std::{
    fmt::{Debug, Display, Error},
    marker::PhantomData,
    sync::Arc,
};

use log::error;
use string_interner::DefaultSymbol;

use crate::{
    ast::nodes::statements::Statement,
    file::File,
    interner::{INTERNER, Interner},
};

pub mod binop;
pub mod calls;
pub mod declarations;
pub mod deprecated;
pub mod expressions;
pub mod numbers;
pub mod statements;

pub struct StateSimple;
pub struct StateBinded;

pub struct FileTreeRoot<T = StateSimple> {
    pub content: Vec<Statement>,
    pub file: Option<Arc<File>>,
    state: PhantomData<T>,
}

impl<T> FileTreeRoot<T> {
    pub fn new(content: Vec<Statement>) -> FileTreeRoot<T> {
        FileTreeRoot {
            content,
            file: None,
            state: PhantomData::default(),
        }
    }
}

#[derive(Clone, PartialEq, Copy, Hash, Eq)]
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

pub fn into_str(interner: &Interner, symbol: SymbolWrapper) -> &str {
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

impl Debug for FileTreeRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FileTreeRoot({:?}, {:?})", self.content, self.file)
    }
}
