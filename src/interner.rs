use std::sync::{LazyLock, Mutex, MutexGuard};

use log::error;
use miette::{Result, miette};
use string_interner::DefaultStringInterner;

pub type Interner = LazyLock<DefaultStringInterner>;

/// WARNING: avoid passing as arguments, release the lock as soon as possible
pub static INTERNER: Mutex<Interner> = Mutex::new(LazyLock::new(DefaultStringInterner::default));

pub fn get_interner() -> Result<MutexGuard<'static, Interner>> {
    INTERNER
        .lock()
        .map_err(|e| miette!("Unable to access interner: {}", e))
}

pub fn get_interner_typed<T>() -> Result<MutexGuard<'static, Interner>, T>
where
    T: Default,
{
    INTERNER.lock().map_err(|_| {
        error!("Failed to get the lock");
        T::default()
    })
}
