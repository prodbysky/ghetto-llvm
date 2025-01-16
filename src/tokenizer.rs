use error_stack::ResultExt;
use thiserror::Error;

#[derive(Debug)]
pub struct Tokenizer {
    source: Vec<char>,
    source_code_file_name: String,
    not_changed: String,
    offset: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BinaryOp {
    Plus,
    Minus,
    Star,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NumberTypeFlag {
    Signed,
    Floating,
    Hexadecimal,
    Binary,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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
    pub fn new(source_code: String, file_name: String) -> Self {
        let mut source: Vec<char> = source_code.chars().collect();
        source.reverse();
        Self {
            source,
            source_code_file_name: file_name,
            not_changed: source_code,
            offset: 0,
        }
    }
    // TODO: Parsing floats, signed, hexadecimal, binary numbers
    pub fn tokenize(mut self) -> TokenizerResult {
        fn location_from_offset(input: &str, offset: usize) -> Option<(usize, usize)> {
            if offset > input.len() {
                return None;
            }

            let mut newline_count = 0;
            let mut line_start = 0;

            for (index, line) in input.lines().enumerate() {
                let line_end = line_start + line.len();

                if offset >= line_start && offset <= line_end {
                    let column = offset - line_start;
                    return Some((newline_count + 1, column + 1));
                }

                line_start = line_end + 1;
                newline_count = index + 1;
            }

            None
        }
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
                Some('*') => {
                    tokens.push(Token::BinaryOperator {
                        op: BinaryOp::Star,
                        offset: self.offset,
                    });
                    self.consume();
                }
                None => {
                    return Ok(tokens);
                }
                c => {
                    let location = location_from_offset(&self.not_changed, self.offset)
                        .expect("Something really bad happened");
                    return Err(TokenizeError::UnexpectedChar).attach_printable(format!(
                        "./{}:{}:{}: unexpected character found: {c:?}",
                        self.source_code_file_name, location.0, location.1,
                    ));
                }
            }
        }
        Ok(tokens)
    }

    fn finished(&self) -> bool {
        self.source.is_empty()
    }

    fn peek(&self) -> Option<&char> {
        self.source.last()
    }
    fn consume(&mut self) -> Option<char> {
        self.offset += 1;
        self.source.pop()
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
        let tokenizer = Tokenizer::new(src, "tests::empty".to_string());
        assert_eq!(tokenizer.tokenize().unwrap(), vec![])
    }

    #[test]
    fn numbers() {
        let src = "123 69".to_string();
        let tokenizer = Tokenizer::new(src, "tests::numbers".to_string());
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
        let tokenizer = Tokenizer::new(src, "tests::operators".to_string());
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
