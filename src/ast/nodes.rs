use crate::ast::nodes::statements::Statement;

pub mod calls;
pub mod deprecated;
pub mod expressions;
pub mod statements;

pub struct FileTreeRoot {
    pub content: Vec<Statement>,
}

impl FileTreeRoot {
    pub fn new<'tok>(content: Vec<Statement>) -> FileTreeRoot {
        FileTreeRoot { content }
    }
}
