use chumsky::{
    Boxed, Parser,
    error::Rich,
    extra::{self, Full},
    input::ValueInput,
    prelude::{empty, just},
    primitive::choice,
    recovery::via_parser,
    select,
    span::SimpleSpan,
};

use crate::{
    ast::nodes::{calls::functions::FunctionCall, deprecated::Deprecated, expressions::Expression},
    lexer::Tokens,
};

pub fn function_call_parser<'tok, 'src: 'tok, I>(
    exp: Boxed<'tok, '_, I, Expression, Full<Rich<'tok, Tokens>, (), ()>>,
) -> impl Parser<'tok, I, FunctionCall, extra::Err<Rich<'tok, Tokens>>> + Clone
where
    I: ValueInput<'tok, Token = Tokens, Span = SimpleSpan>,
{
    let identifier = select! {
        Tokens::Identifier(id) => id
    };

    // TODO: add a parser for chains
    let base = identifier;
    let margs = choice((exp.map(|x| Some(Box::new(x))), empty().map(|_| None)));
    let call = margs
        .delimited_by(
            just(Tokens::LeftParenthesis),
            just(Tokens::RightParenthesis)
                .recover_with(via_parser(empty().to(Tokens::RightParenthesis))),
        )
        .labelled("function call body");

    base.then(call)
        .map_with(|(base, e), extra| FunctionCall::new(base, extra.span(), e))
        .labelled("function call")
        .as_context()
}

pub fn native_parser<'tok, 'src: 'tok, I>()
-> impl Parser<'tok, I, Deprecated, extra::Err<Rich<'tok, Tokens>>> + Clone
where
    I: ValueInput<'tok, Token = Tokens, Span = SimpleSpan>,
{
    just(Tokens::NativeCall)
        .map_with(|_, extra| Deprecated::new("skr_app should not be used", extra.span()))
}
