use error_stack::ResultExt;
use thiserror::Error;

use crate::tokenizer::{self, BinaryOp};
pub struct AstParser {
    tokens: Vec<tokenizer::Token>,
}

#[derive(PartialEq, Debug)]
pub enum Node {
    Number {
        raw: String,
        flags: Vec<tokenizer::NumberTypeFlag>,
    },
    BinaryOperation {
        left: Box<Node>,
        operator: tokenizer::BinaryOp,
        right: Box<Node>,
    },
    Let {
        value: Box<Node>,
        name: String,
        t: String,
    },
    Exit {
        value: Box<Node>,
    },
}

#[derive(Debug, Error)]
pub enum AstParseError {
    #[error("invalid expression found during ast parsing")]
    InvalidExpression,
    #[error("found an expression at the top level")]
    ExpressionAtToplevel,
    #[error("invalid let statement")]
    InvalidLetStatement,
}

pub type AstParseResult = error_stack::Result<Vec<Node>, AstParseError>;

#[derive(Debug, Error)]
pub enum ExpressionParseError {
    #[error("unexpected token found when parsing `factor`, got: {found:?}")]
    InvalidFactorToken { found: Option<tokenizer::Token> },
}
pub type ExpressionParseResult = error_stack::Result<Node, ExpressionParseError>;

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
                            Node::BinaryOperation {
                                left: _,
                                operator: _,
                                right: _,
                            },
                        ) => Node::Let {
                            value: Box::new(value),
                            name,
                            t,
                        },
                        (
                            Some(tokenizer::Token::Identifier(name)),
                            Some(tokenizer::Token::Identifier(t)),
                            Node::Number { raw: _, flags: _ },
                        ) => Node::Let {
                            value: Box::new(value),
                            name,
                            t,
                        },

                        _ => {
                            return Err(AstParseError::InvalidLetStatement)
                                .attach_printable("found an invalid let statement")
                        }
                    });
                }
                tokenizer::Token::Exit => {
                    self.eat();
                    nodes.push(Node::Exit {
                        value: Box::new(
                            self.expression()
                                .change_context(AstParseError::InvalidExpression)?,
                        ),
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
                node = Node::BinaryOperation {
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
                node = Node::BinaryOperation {
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
                Ok(Node::Number { raw, flags })
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
        ast::{self, Node},
        tokenizer,
    };

    #[test]
    fn let_statement() {
        let src = "let a: u64 = (123 + 69) * 2;".to_string();
        let tokens = tokenizer::Tokenizer::new(src, "tests::let_statement".to_string())
            .tokenize()
            .unwrap();
        assert_eq!(
            ast::AstParser::new(tokens).parse().unwrap(),
            vec![Node::Let {
                value: Box::new(Node::BinaryOperation {
                    left: Box::new(Node::BinaryOperation {
                        left: Box::new(Node::Number {
                            raw: "123".to_string(),
                            flags: vec![]
                        }),
                        operator: tokenizer::BinaryOp::Plus,
                        right: Box::new(Node::Number {
                            raw: "69".to_string(),
                            flags: vec![]
                        }),
                    }),
                    operator: tokenizer::BinaryOp::Star,
                    right: Box::new(Node::Number {
                        raw: "2".to_string(),
                        flags: vec![]
                    })
                }),
                name: String::from("a"),
                t: String::from("u64"),
            },]
        )
    }
}
