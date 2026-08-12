use chumsky::span::SimpleSpan;

/// Represent a call to a function.
/// TODO: add arguments.
/// TODO: replace name with a full path.
#[derive(PartialEq, Clone)]
pub struct FunctionCall {
    pub name: SimpleSpan,
    pub span: SimpleSpan,
}

impl FunctionCall {
    pub fn new(name: SimpleSpan, span: SimpleSpan) -> FunctionCall {
        FunctionCall { name, span }
    }
}
