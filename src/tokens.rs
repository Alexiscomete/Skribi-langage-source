use crate::skr_errors::CustomError;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum ModifierKeyword {
    Global,
    Constant,
    Private,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum SpaceTypes {
    Space,
    NewLine,
    Tab,
}

#[allow(dead_code)] // TODO : define symbols to remove this
#[derive(Debug, PartialEq)]
pub enum Token {
    Bool(bool),
    Int(u32),
    Float(f32),
    String(String),
    NatCall,
    Add,
    Sub,
    Not,
    Div,
    Mul,
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
    Inside,
    Identifier(String),
    Space(SpaceTypes),
    KeywordModifier(ModifierKeyword),
    KeywordIf,
    KeywordElse,
    KeywordClass,
    KeywordFunction,
    KeywordReturn,
    /// = biuli
    KeywordBubbleScope,
    /// = kodi
    KeywordSimpleScope,
    /// = spoki
    KeywordUnusedScope,
    Invalid(String), // Any character not used by other tokens, only used when parsing bloc title
    // TODO : Pow
    // TODO : and, or, xor, not
    // TODO : comparison operators
    Equal,    // not tokenized for now : missing symbol
    NotEqual, // not tokenized for now : missing symbol
    And,      // not tokenized for now : missing symbol
    Or,       // not tokenized for now : missing symbol
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq)]
pub struct TokenContainer {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

#[cfg(test)]
impl TokenContainer {
    pub fn new(token: Token, line: usize, column: usize) -> Self {
        Self {
            token,
            line,
            column,
        }
    }
}

#[cfg(test)]
impl Into<TokenContainer> for Token {
    fn into(self) -> TokenContainer {
        TokenContainer::new(self, 0, 0)
    }
}

fn tokenize_string(file: &mut Chars, line: usize) -> Result<Token, CustomError> {
    let mut current_ch = file.next();
    let mut string_escape = false;
    let mut res = String::new();

    while let Some(ch) = current_ch {
        if string_escape {
            res.push(match ch {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '0' => '\0',
                _ => ch,
            });
            string_escape = false;
        } else if ch == '\\' {
            string_escape = true;
        } else if ch == '"' {
            return Ok(Token::String(res));
        } else {
            res.push(ch);
        }

        current_ch = file.next();
    }

    Err(CustomError::InvalidString(
        "String not closed".to_string(),
        line,
    ))
}

fn tokenize_number(
    file: &mut Chars,
    line: usize,
    first_char: char,
) -> Result<(Token, Option<char>), CustomError> {
    let mut current_ch = file.next();
    let mut res = String::new();
    res.push(first_char);
    let mut is_float = false;

    while let Some(ch) = current_ch {
        if ch == '.' {
            if is_float {
                return Err(CustomError::InvalidFloat(
                    "A float can have only one . !".to_string(),
                    line,
                ));
            } else {
                is_float = true;
                res.push(ch);
            }
        } else if ch.is_numeric() {
            res.push(ch);
        } else {
            return Ok((
                if is_float {
                    Token::Float(res.parse().unwrap())
                } else {
                    Token::Int(res.parse().unwrap())
                },
                Some(ch),
            ));
        }
        current_ch = file.next();
    }

    Ok((
        if is_float {
            Token::Float(res.parse().unwrap())
        } else {
            Token::Int(res.parse().unwrap())
        },
        None,
    ))
}

fn tokenize_word(file: &mut Chars, first_char: char) -> Result<(Token, Option<char>), CustomError> {
    let mut current_ch = file.next();
    let mut res = String::new();
    res.push(first_char);

    while let Some(ch) = current_ch {
        if ch.is_alphanumeric() || ch == '_' {
            res.push(ch);
        } else {
            return Ok((word_to_token(res), Some(ch)));
        }
        current_ch = file.next();
    }

    Ok((word_to_token(res), None))
}

fn word_to_token(res: String) -> Token {
    match res.as_str() {
        "fu" => Token::KeywordModifier(ModifierKeyword::Global),
        "ju" => Token::KeywordModifier(ModifierKeyword::Constant),
        "pu" => Token::KeywordModifier(ModifierKeyword::Private),
        "ij" => Token::KeywordIf,
        "sula" => Token::KeywordElse,
        "skr_app" => Token::NatCall,
        "io" => Token::Bool(true),
        "no" => Token::Bool(false),
        "ums" => Token::KeywordFunction,
        "kat" => Token::KeywordClass,
        "ei" => Token::KeywordReturn,
        "biuli" => Token::KeywordBubbleScope,
        "kodi" => Token::KeywordSimpleScope,
        "spoki" => Token::KeywordUnusedScope,
        _ => Token::Identifier(res),
    }
}

fn tokenize_comment_classic(file: &mut Chars) {
    let mut current_ch = file.next();
    while let Some(ch) = current_ch {
        if ch == '\n' {
            return;
        }
        current_ch = file.next();
    }
}

macro_rules! add_token {
    ($tokens:expr, $line:expr, $column:expr, $token:expr) => {
        $tokens.push_back(TokenContainer {
            token: $token,
            line: $line,
            column: $column,
        });
    };
}

pub(crate) fn tokenize(file: String) -> Result<VecDeque<TokenContainer>, CustomError> {
    let mut tokens: VecDeque<TokenContainer> = VecDeque::new();
    let mut line = 1;
    let column = 0;

    let mut file_ch = file.chars();
    let mut current_ch = file_ch.next();
    // let mut operator2 = false;

    while let Some(ch) = current_ch {
        if ch == '/' {
            if let Some(next_ch) = file_ch.next() {
                if next_ch == '/' {
                    tokenize_comment_classic(&mut file_ch);
                    add_token!(tokens, line, column, Token::Space(SpaceTypes::NewLine));
                    current_ch = file_ch.next();
                } else {
                    add_token!(tokens, line, column, Token::Div);
                    current_ch = Some(next_ch);
                }
            } else {
                add_token!(tokens, line, column, Token::Div);
            }
        } else if ch.is_alphabetic() || ch == '_' {
            let token = tokenize_word(&mut file_ch, ch)?;
            add_token!(tokens, line, column, token.0);
            current_ch = token.1;
        } else if ch.is_numeric() {
            let token = tokenize_number(&mut file_ch, line, ch)?;
            add_token!(tokens, line, column, token.0);
            current_ch = token.1;
        } else {
            if ch == ' ' {
                // unused - tokens.push(Token::Space(Space::Space));
            } else {
                add_token!(
                    tokens,
                    line,
                    column,
                    match ch {
                        '+' => Token::Add,
                        '-' => Token::Sub,
                        '*' => Token::Mul,
                        '"' => tokenize_string(&mut file_ch, line)?,
                        ':' => Token::Inside,
                        '(' => Token::LeftParenthesis,
                        ')' => Token::RightParenthesis,
                        '{' => Token::LeftBrace,
                        '}' => Token::RightBrace,
                        '\n' => {
                            line += 1;
                            Token::Space(SpaceTypes::NewLine)
                        }
                        _ => Token::Invalid(ch.to_string()),
                    }
                );
            }
            current_ch = file_ch.next();
        }
    }

    Ok(tokens)
}
