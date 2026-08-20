use chumsky::error::Rich;
use chumsky::input::{Input, Stream, ValueInput};
use chumsky::prelude::{choice, empty, just, recursive, via_parser};
use chumsky::span::SimpleSpan;
use chumsky::{IterParser, Parser, extra};
use logos::Span;
use miette::{Diagnostic, LabeledSpan, Result, SourceSpan, miette};
use string_interner::DefaultSymbol;
use thiserror::Error;

use crate::ast::nodes::FileTreeRoot;
use crate::ast::nodes::expressions::Expression;
use crate::ast::nodes::statements::Statement;
use crate::interner::INTERNER;
use crate::lexer::Tokens;
use crate::parse::call::{function_call_parser, native_parser};

pub mod call;

// Global warning on the parser: please use .boxed() sometimes, so that the
// compilation time decreases. This is like INTERCAL, you need to say please
// sometimes.

// This functions define abstract parsers that will be instancieted by chumsky.
// This does not actually parse anything.

fn expression_parser<'tok, 'src: 'tok, I>()
-> impl Parser<'tok, I, Expression, extra::Err<Rich<'tok, Tokens>>> + Clone + 'tok
where
    I: ValueInput<'tok, Token = Tokens, Span = SimpleSpan>,
{
    // exp := (exp) | native_call
    // This is over complicated as more rules will be added

    recursive(|exp| {
        // Anything that starts with a special unique token
        // --> has maximal priority and can be in anything
        let priority = choice((exp.clone().delimited_by(
            just(Tokens::LeftParenthesis),
            just(Tokens::RightParenthesis)
                .recover_with(via_parser(empty().to(Tokens::RightParenthesis))),
        ),))
        .boxed();

        choice((
            priority.clone(),
            function_call_parser().map(Expression::FunctionCall),
        ))
    })
    .labelled("expression")
    .as_context()
}

fn statement_parser<'tok, 'src: 'tok, I>()
-> impl Parser<'tok, I, Statement, extra::Err<Rich<'tok, Tokens>>>
where
    I: ValueInput<'tok, Token = Tokens, Span = SimpleSpan>,
{
    choice((
        native_parser().map(Statement::Deprecated),
        expression_parser().boxed().map(Statement::Expression),
    ))
    .boxed()
}

fn root_parser<'tok, 'src: 'tok, I>()
-> impl Parser<'tok, I, FileTreeRoot, extra::Err<Rich<'tok, Tokens>>>
where
    I: ValueInput<'tok, Token = Tokens, Span = SimpleSpan>,
{
    statement_parser()
        .repeated()
        .collect()
        .boxed()
        .map(FileTreeRoot::new)
}

fn error_symbol() -> Result<DefaultSymbol> {
    let mut interner = INTERNER
        .lock()
        .map_err(|e| miette!("Unable to access interner: {}", e))?;
    Ok(interner.get_or_intern_static("?"))
}

#[derive(Error, Debug, Diagnostic)]
#[error("{message}")]
#[diagnostic()]
struct ParsingSingleError {
    message: String,
    #[label(primary, "{span_message}")]
    span: SourceSpan,
    span_message: String,
    #[label(collection)]
    spans: Vec<LabeledSpan>,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Parsing error")]
#[diagnostic(help("Always try to fix the first parsing error as they might be cascades"))]
struct ParsingErrors {
    #[related]
    related: Vec<ParsingSingleError>,
}

fn convert_to_err(errs: Vec<Rich<'_, Tokens>>) -> ParsingErrors {
    // Greatly inspired from
    // https://codeberg.org/zesterer/chumsky/src/branch/main/examples/nano_rust.rs
    ParsingErrors {
        related: errs
            .iter()
            .map(|err| ParsingSingleError {
                message: err.to_string(),
                span: err.span().into_range().into(),
                span_message: err.reason().to_string(),
                spans: err
                    .contexts()
                    .map(|(label, span)| {
                        LabeledSpan::new_with_span(
                            Some(format!("parsing {label}")),
                            span.into_range(),
                        )
                    })
                    .collect(),
            })
            .collect(),
    }
}

pub fn parse<'a>(
    tokens: Vec<(Result<Tokens, ()>, Span)>,
    src_len: usize,
) -> Result<FileTreeRoot> {
    // Greatly inspired by
    // https://codeberg.org/zesterer/chumsky/src/branch/main/examples/logos.rs
    // Converts from a logos format to a chumsky format
    // See the example for full explanations

    let error_symbol = error_symbol()?;

    let iter = tokens.into_iter().map(|(token, span)| match token {
        Ok(tok) => (tok, span.into()),
        Err(()) => (Tokens::Error(error_symbol.clone()), span.into()),
    });

    let token_stream = Stream::from_iter(iter).map((0..src_len).into(), |(t, s): (_, _)| (t, s));

    root_parser().parse(token_stream).into_result().map_err(|errs| convert_to_err(errs).into())
}

#[cfg(test)]
mod test {
    use crate::{lexer::tokenise, parse::parse};
    use insta::assert_compact_debug_snapshot;

    #[test]
    fn parse_skr_app() {
        let src = "skr_app";
        let tokens = tokenise(src).unwrap();
        assert_compact_debug_snapshot!(parse(tokens, src.len()));
    }
}
