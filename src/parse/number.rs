use chumsky::{Parser, error::Rich, extra, input::ValueInput, select, span::SimpleSpan};

use crate::{ast::nodes::numbers::Number, lexer::Tokens};

pub(super) fn number_parser<'tok, 'src: 'tok, I>()
-> impl Parser<'tok, I, Number, extra::Err<Rich<'tok, Tokens>>> + Clone + 'tok
where
    I: ValueInput<'tok, Token = Tokens, Span = SimpleSpan>,
{
    select! {
        Tokens::Number(sym) => sym
    }
    .map_with(|sym, extra| Number {
        content: sym.into(),
        span: extra.span(),
    })
}
