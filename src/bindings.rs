pub mod stdlib_bindings;

#[derive(Debug, Clone)]
pub enum Bindings {
    Println,
    Exit,
}

#[derive(Debug, Clone)]
pub enum DeclarationLink {
    Bindings(Bindings),
}
