use std::sync::{LazyLock, Mutex, MutexGuard};

use la_arena::Arena;
use log::error;
use miette::{Result, miette};

use crate::ast::nodes::declarations::variable::VariableDeclaration;

pub type VariableInterner = LazyLock<Arena<VariableDeclaration>>;
pub type VariableLocked = MutexGuard<'static, VariableInterner>;

/// Will leak data.
/// How to avoid this?
/// Acceptable?
pub static VARIABLE_INTERNER: Mutex<VariableInterner> = Mutex::new(LazyLock::new(Arena::default));

pub fn get_variable_interner() -> Result<MutexGuard<'static, VariableInterner>> {
    VARIABLE_INTERNER
        .lock()
        .map_err(|e| miette!("Unable to access interner: {}", e))
}

pub fn get_variable_interner_typed<T>() -> Result<MutexGuard<'static, VariableInterner>, T>
where
    T: Default,
{
    VARIABLE_INTERNER.lock().map_err(|_| {
        error!("Failed to get the lock");
        T::default()
    })
}
