use std::{fmt::{Display, Formatter}};

use logos::{Logos, SpannedIter};

// NOTE: logos is smart: like CSS, it calculates a priority score based on the
// specificity of the rule. "token" has the priority over anything else. Then,
// regex, with complicated rules. Sometimes, the priority argument can be used
// to avoid confusions.

#[derive(Logos, Clone, PartialEq, Debug)]
pub enum Tokens {
    /// Names: variables, functions, ...
    /// As we have spans, a solution found on online to avoid lifetime issues
    /// is to use them instead of storing a ref.
    #[regex(r#"[a-zA-Z_][a-zA-Z0-9_]*"#)]
    Identifier,
    /// Deprecated keyword to detect native calls,
    /// still there to test compatibility
    #[token("skr_app")]
    NativeCall,

    #[token("(")]
    LeftParenthesis,
    #[token(")")]
    RightParenthesis,

    /// Note: no need of them in parsing
    #[regex(r"[ \t\n]+", logos::skip)]
    Ignore,

    /// Any character not used by other tokens,
    /// mainly used when parsing bloc title
    #[regex(".", priority = 0)]
    Error,
}

impl Display for Tokens {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            // TODO: find a way to print them correctly
            Self::Identifier => "id",
            Self::LeftParenthesis => "(",
            Self::RightParenthesis => ")",
            Self::Ignore => " ",
            Self::NativeCall => "skr_app",
            Self::Error => "?",
        })
    }
}

/// Split a file content into tokens
/// We must use a ref in this case
pub fn tokenise(arg: &'_ str) -> SpannedIter<'_, Tokens> {
    // Inspired from the logos example
    Tokens::lexer(arg).spanned()
}
