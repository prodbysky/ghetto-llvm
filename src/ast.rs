use error_stack::ResultExt;
use thiserror::Error;

use crate::tokenizer::{self, BinaryOp};
pub struct AstParser {
    tokens: Vec<tokenizer::Token>,
}

#[derive(PartialEq, Debug)]
pub enum AstStatement {
    Let {
        value: AstExpression,
        name: String,
        t: String,
    },
    Exit {
        value: AstExpression,
    },
}

#[derive(PartialEq, Debug)]
pub enum AstExpression {
    Number {
        raw: String,
        flags: Vec<tokenizer::NumberTypeFlag>,
    },
    BinaryOperation {
        left: Box<AstExpression>,
        operator: tokenizer::BinaryOp,
        right: Box<AstExpression>,
    },
    Identifier {
        name: String,
    },
}

impl std::fmt::Display for AstExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number { raw, flags } => {
                f.write_str(&raw)?;
            }
            Self::Identifier { name } => {
                f.write_str(&name)?;
            }
            Self::BinaryOperation {
                left,
                operator,
                right,
            } => {
                f.write_str(format!("{}", left).as_str())?;
                f.write_str(
                    format!(
                        "{}",
                        match operator {
                            BinaryOp::Plus => '+',
                            BinaryOp::Minus => '-',
                            BinaryOp::Star => '*',
                            BinaryOp::SingleEqual => '=',
                        }
                    )
                    .as_str(),
                )?;
                f.write_str(format!("{}", right).as_str())?;
            }
        }
        Ok(())
    }
}

pub type AstProgram = Vec<AstStatement>;

#[derive(Debug, Error)]
pub enum AstParseError {
    #[error("invalid expression found during ast parsing")]
    InvalidExpression,
    #[error("found an expression at the top level")]
    ExpressionAtToplevel,
    #[error("invalid let statement")]
    InvalidLetStatement,
}

pub type AstParseResult = error_stack::Result<AstProgram, AstParseError>;

#[derive(Debug, Error)]
pub enum ExpressionParseError {
    #[error("unexpected token found when parsing `factor`, got: {found:?}")]
    InvalidFactorToken { found: Option<tokenizer::Token> },
}
pub type ExpressionParseResult = error_stack::Result<AstExpression, ExpressionParseError>;

impl AstParser {
    pub fn new(mut tokens: Vec<tokenizer::Token>) -> Self {
        tokens.reverse();
        Self { tokens }
    }

    pub fn parse(&mut self) -> AstParseResult {
        let mut nodes = vec![];

        while !self.finished() {
            match self.peek().unwrap() {
                tokenizer::Token::Let => {
                    self.eat(); // Let
                    let name = self.eat();
                    self.eat(); // Colon
                    let t = self.eat();
                    self.eat(); // `=`
                    let value = self
                        .expression()
                        .change_context(AstParseError::InvalidExpression)
                        .attach_printable("found an invalid expression")?;
                    self.eat(); // `;`

                    nodes.push(match (name, t, &value) {
                        (
                            Some(tokenizer::Token::Identifier(name)),
                            Some(tokenizer::Token::Identifier(t)),
                            AstExpression::BinaryOperation {
                                left: _,
                                operator: _,
                                right: _,
                            },
                        ) => AstStatement::Let { value, name, t },
                        (
                            Some(tokenizer::Token::Identifier(name)),
                            Some(tokenizer::Token::Identifier(t)),
                            AstExpression::Number { raw: _, flags: _ },
                        ) => AstStatement::Let { value, name, t },

                        _ => {
                            return Err(AstParseError::InvalidLetStatement)
                                .attach_printable("found an invalid let statement")
                        }
                    });
                }
                tokenizer::Token::Exit => {
                    self.eat();
                    nodes.push(AstStatement::Exit {
                        value: self
                            .expression()
                            .change_context(AstParseError::InvalidExpression)?,
                    });
                }
                tokenizer::Token::Semicolon => {
                    while self
                        .peek()
                        .is_some_and(|t| matches!(t, tokenizer::Token::Semicolon))
                    {
                        self.eat();
                    }
                }
                _ => {
                    return Err(AstParseError::ExpressionAtToplevel)
                        .attach_printable("failed to parse program")
                }
            }
        }

        Ok(nodes)
    }

    fn expression(&mut self) -> ExpressionParseResult {
        let mut node = self.term()?;
        let term_operator = |token: &tokenizer::Token| {
            matches!(
                token,
                tokenizer::Token::BinaryOperator {
                    op: BinaryOp::Plus,
                    offset: _
                } | tokenizer::Token::BinaryOperator {
                    op: BinaryOp::Minus,
                    offset: _
                }
            )
        };
        while self.peek().is_some_and(term_operator) {
            if let Some(tokenizer::Token::BinaryOperator { op, offset: _ }) = self.eat() {
                node = AstExpression::BinaryOperation {
                    left: Box::new(node),
                    operator: op,
                    right: Box::new(self.term()?),
                }
            }
        }
        Ok(node)
    }

    fn term(&mut self) -> ExpressionParseResult {
        let mut node = self.factor()?;

        let factor_operator = |token: &tokenizer::Token| {
            matches!(
                token,
                tokenizer::Token::BinaryOperator {
                    op: BinaryOp::Star,
                    offset: _
                }
            )
        };

        while self.peek().is_some_and(factor_operator) {
            if let Some(tokenizer::Token::BinaryOperator { op, offset: _ }) = self.eat() {
                node = AstExpression::BinaryOperation {
                    left: Box::new(node),
                    operator: op,
                    right: Box::new(self.factor()?),
                }
            }
        }
        Ok(node)
    }

    fn factor(&mut self) -> ExpressionParseResult {
        match self.peek().cloned() {
            Some(tokenizer::Token::Number {
                raw,
                flags,
                offset: _,
            }) => {
                self.eat();
                Ok(AstExpression::Number { raw, flags })
            }
            Some(tokenizer::Token::Identifier(name)) => {
                self.eat();
                Ok(AstExpression::Identifier { name })
            }
            Some(tokenizer::Token::OpenParen) => {
                self.eat();
                let node = self.expression()?;

                if let Some(tokenizer::Token::CloseParen) = self.peek() {
                    self.eat();
                    Ok(node)
                } else {
                    Err(ExpressionParseError::InvalidFactorToken {
                        found: self.peek().cloned(),
                    })
                    .attach_printable("unclosed parenthesis found")
                }
            }
            _ => Err(ExpressionParseError::InvalidFactorToken {
                found: self.peek().cloned(),
            })
            .attach_printable("failed to parse factor"),
        }
    }

    fn finished(&self) -> bool {
        self.tokens.is_empty()
    }

    fn peek(&self) -> Option<&tokenizer::Token> {
        self.tokens.last()
    }
    fn eat(&mut self) -> Option<tokenizer::Token> {
        self.tokens.pop()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{self, AstExpression, AstStatement},
        tokenizer,
    };

    #[test]
    fn let_statement() {
        {
            let src = "let a: u64 = (123 + 69) * 2;".to_string();
            let tokens = tokenizer::Tokenizer::new(src, "tests::let_statement".to_string())
                .tokenize()
                .unwrap();

            assert_eq!(
                ast::AstParser::new(tokens).parse().unwrap(),
                vec![AstStatement::Let {
                    value: AstExpression::BinaryOperation {
                        left: Box::new(AstExpression::BinaryOperation {
                            left: Box::new(AstExpression::Number {
                                raw: "123".to_string(),
                                flags: vec![]
                            }),
                            operator: tokenizer::BinaryOp::Plus,
                            right: Box::new(AstExpression::Number {
                                raw: "69".to_string(),
                                flags: vec![]
                            }),
                        }),
                        operator: tokenizer::BinaryOp::Star,
                        right: Box::new(AstExpression::Number {
                            raw: "2".to_string(),
                            flags: vec![]
                        })
                    },
                    name: String::from("a"),
                    t: String::from("u64"),
                },]
            )
        }
    }
}
