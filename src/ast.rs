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
}

#[derive(Debug, Error)]
pub enum AstParseError {
    #[error("invalid expression found during ast parsing")]
    InvalidExpression,
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
            nodes.push(
                self.expression()
                    .change_context(AstParseError::InvalidExpression)
                    .attach_printable("failed to parse expression")?,
            );
        }

        Ok(nodes)
    }

    fn expression(&mut self) -> ExpressionParseResult {
        let mut node = self.term()?;
        let term_operator = |token: &tokenizer::Token| match token {
            tokenizer::Token::BinaryOperator {
                op: BinaryOp::Plus,
                offset: _,
            }
            | tokenizer::Token::BinaryOperator {
                op: BinaryOp::Minus,
                offset: _,
            } => true,
            _ => false,
        };

        while self.peek().is_some_and(|token| term_operator(token)) {
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

        let factor_operator = |token: &tokenizer::Token| match token {
            tokenizer::Token::BinaryOperator {
                op: BinaryOp::Star,
                offset: _,
            } => true,
            _ => false,
        };

        while self.peek().is_some_and(|token| factor_operator(token)) {
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
                return Ok(Node::Number { raw, flags });
            }
            Some(tokenizer::Token::OpenParen) => {
                self.eat();
                let node = self.expression()?;

                if let Some(tokenizer::Token::CloseParen) = self.peek() {
                    self.eat();
                    return Ok(node);
                } else {
                    return Err(ExpressionParseError::InvalidFactorToken {
                        found: self.peek().cloned(),
                    })
                    .attach_printable("unclosed parenthesis found");
                }
            }
            _ => {
                return Err(ExpressionParseError::InvalidFactorToken {
                    found: self.peek().cloned(),
                })
                .attach_printable("failed to parse factor");
            }
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
    fn binary_expression() {
        let src = "123 + 69 * 2".to_string();
        let tokens = tokenizer::Tokenizer::new(src, "tests::binary_expression".to_string())
            .tokenize()
            .unwrap();
        assert_eq!(
            ast::AstParser::new(tokens).parse().unwrap(),
            vec![Node::BinaryOperation {
                left: Box::new(Node::Number {
                    raw: "123".to_string(),
                    flags: vec![]
                }),
                operator: tokenizer::BinaryOp::Plus,
                right: Box::new(Node::BinaryOperation {
                    left: Box::new(Node::Number {
                        raw: "69".to_string(),
                        flags: vec![]
                    }),
                    operator: tokenizer::BinaryOp::Star,
                    right: Box::new(Node::Number {
                        raw: "2".to_string(),
                        flags: vec![]
                    }),
                })
            }]
        )
    }

    #[test]
    fn parenthesized_expression() {
        let src = "(123 + 69) * 2".to_string();
        let tokens = tokenizer::Tokenizer::new(src, "tests::binary_expression".to_string())
            .tokenize()
            .unwrap();
        assert_eq!(
            ast::AstParser::new(tokens).parse().unwrap(),
            vec![Node::BinaryOperation {
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
            }]
        )
    }
}
