use miette::Result;

use crate::{
    ast::nodes::{SymbolWrapper, into_str},
    bindings::{Bindings, DeclarationLink},
    interner::get_interner,
};

impl DeclarationLink {
    pub fn get_stdlib_binding(symbol: SymbolWrapper) -> Result<Option<DeclarationLink>> {
        let interner = get_interner()?;
        let name = into_str(&interner, symbol);

        Ok(match name {
            "println" => Some(DeclarationLink::Bindings(Bindings::Println)),
            "exit" => Some(DeclarationLink::Bindings(Bindings::Exit)),
            _ => None,
        })
    }
}
