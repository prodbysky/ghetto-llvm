use error_stack::ResultExt;
use thiserror::Error;

#[derive(Debug)]
pub struct Tokenizer {
    source: std::collections::VecDeque<char>,
    offset: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Plus,
    Minus,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NumberTypeFlag {
    Signed,
    Floating,
    Hexadecimal,
    Binary,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Number {
        raw: String,
        flags: Vec<NumberTypeFlag>,
        offset: usize,
    },
    BinaryOperator {
        op: BinaryOp,
        offset: usize,
    },
}

#[derive(Debug, Error)]
pub enum TokenizeError {
    #[error("unexpected char found")]
    UnexpectedChar,
}

pub type TokenizerResult = error_stack::Result<Vec<Token>, TokenizeError>;

impl Tokenizer {
    pub fn new(source_code: String) -> Self {
        Self {
            source: source_code.chars().collect::<Vec<_>>().into(),
            offset: 0,
        }
    }
    // TODO: Parsing floats, signed, hexadecimal, binary numbers
    pub fn tokenize(mut self) -> TokenizerResult {
        let mut tokens = vec![];
        while !self.finished() {
            self.trim_whitespace();
            if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                let mut buffer = String::new();
                let offset = self.offset;
                while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    buffer.push(self.consume().unwrap());
                }
                tokens.push(Token::Number {
                    raw: buffer,
                    flags: vec![],
                    offset,
                });
                continue;
            }
            match self.peek() {
                Some('+') => {
                    tokens.push(Token::BinaryOperator {
                        op: BinaryOp::Plus,
                        offset: self.offset,
                    });
                    self.consume();
                }
                Some('-') => {
                    tokens.push(Token::BinaryOperator {
                        op: BinaryOp::Minus,
                        offset: self.offset,
                    });
                    self.consume();
                }
                None => {
                    return Ok(tokens);
                }
                c => {
                    return Err(TokenizeError::UnexpectedChar)
                        .attach_printable(format!("unexpected character found: {c:?}"))
                }
            }
        }
        Ok(tokens)
    }

    fn finished(&self) -> bool {
        self.source.is_empty()
    }

    fn peek(&self) -> Option<&char> {
        self.source.front()
    }
    fn consume(&mut self) -> Option<char> {
        self.offset += 1;
        self.source.pop_front()
    }

    fn trim_whitespace(&mut self) {
        while self.peek().is_some_and(|c| c.is_whitespace()) {
            self.consume();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::{BinaryOp, Token};

    use super::Tokenizer;

    #[test]
    fn empty() {
        let src = "".to_string();
        let tokenizer = Tokenizer::new(src);
        assert_eq!(tokenizer.tokenize().unwrap(), vec![])
    }

    #[test]
    fn numbers() {
        let src = "123 69".to_string();
        let tokenizer = Tokenizer::new(src);
        assert_eq!(
            tokenizer.tokenize().unwrap(),
            vec![
                Token::Number {
                    raw: "123".to_lowercase(),
                    flags: vec![],
                    offset: 0
                },
                Token::Number {
                    raw: "69".to_lowercase(),
                    flags: vec![],
                    offset: 4
                },
            ]
        )
    }
    #[test]
    fn operators() {
        let src = "- + -".to_string();
        let tokenizer = Tokenizer::new(src);
        assert_eq!(
            tokenizer.tokenize().unwrap(),
            vec![
                Token::BinaryOperator {
                    op: BinaryOp::Minus,
                    offset: 0
                },
                Token::BinaryOperator {
                    op: BinaryOp::Plus,
                    offset: 2
                },
                Token::BinaryOperator {
                    op: BinaryOp::Minus,
                    offset: 4
                },
            ]
        )
    }
}
