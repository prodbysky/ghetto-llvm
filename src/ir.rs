use crate::ast::{self};

pub struct IrGenerator {
    program: ast::AstProgram,
}

#[derive(Debug)]
pub enum IR {
    DefineVariable {
        name: String,
        t: String,
        value: ast::AstExpression
    },
    Exit { value: ast::AstExpression },
}

impl IrGenerator {
    pub fn new(mut program: ast::AstProgram) -> Self {
        program.reverse();
        Self { program }
    }

    pub fn generate(mut self) -> Vec<IR> {
        let mut ir = vec![];

        while let Some(stmt) = self.eat() {
            match stmt {
                ast::AstStatement::Exit{value} => {
                    ir.push(IR::Exit{value});
                }
                ast::AstStatement::Let {
                    value,
                    name,
                    t
                } => {
                    ir.push(IR::DefineVariable{value, t, name});
                }
            }                        
        }

        ir
    }

    fn peek(&self) -> Option<&ast::AstStatement> {
        self.program.last()
    }
    fn eat(&mut self) -> Option<ast::AstStatement> {
        self.program.pop()
    }
}
