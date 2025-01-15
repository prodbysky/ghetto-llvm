use error_stack::ResultExt;
use thiserror::Error;

#[derive(Debug)]
pub enum BinaryOp {
    Plus,
    Minus,
}

#[derive(Debug)]
pub enum Token {
    Number { raw: String, t: NumberType },
    BinaryOperator(BinaryOp),
}

#[derive(Debug)]
pub enum NumberType {
    U64,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("input file not provided")]
    InputNotProvided,
}

#[derive(Debug, Error)]
#[error("something bad happenned...")]
pub struct CompilerError;

fn main() -> error_stack::Result<(), CompilerError> {
    let mut args = std::env::args();
    let _program_name = args.next().expect("something non-standard happened");
    let name = args
        .next()
        .ok_or(ConfigError::InputNotProvided)
        .attach_printable("failed to parse config")
        .change_context(CompilerError)?;

    let input = std::fs::read_to_string(&name)
        .attach_printable(format!("failed to open and/read input file: {name}"))
        .change_context(CompilerError)?;
    let mut input: std::collections::VecDeque<char> = input.chars().collect::<Vec<char>>().into();

    let mut tokens = vec![];

    while !input.is_empty() {
        while input.front().is_some_and(|c| c.is_whitespace()) {
            input.pop_front();
        }
        if input.front().is_some_and(|c| c.is_ascii_digit()) {
            let mut buffer = String::new();

            while input.front().is_some_and(|c| c.is_ascii_digit()) {
                buffer.push(input.pop_front().unwrap());
            }
            tokens.push(Token::Number {
                raw: buffer,
                t: NumberType::U64,
            });
        }

        if input.front().is_some() {
            match input.pop_front().unwrap() {
                '+' => tokens.push(Token::BinaryOperator(BinaryOp::Plus)),
                '-' => tokens.push(Token::BinaryOperator(BinaryOp::Minus)),
                _ => {}
            }
        }
    }

    dbg!(tokens);

    Ok(())
}
