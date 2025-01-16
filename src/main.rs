mod config;
mod tokenizer;
use error_stack::ResultExt;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("something bad happenned...")]
pub struct CompilerError;

fn main() -> error_stack::Result<(), CompilerError> {
    let config = config::Config::from_args(&mut std::env::args())
        .change_context(CompilerError)
        .attach_printable("failed to parse compiler config from cli args")?;
    let input = std::fs::read_to_string(&config.input_file_name)
        .attach_printable(format!(
            "failed to open and/read input file: {}",
            config.input_file_name
        ))
        .change_context(CompilerError)?;

    let tokenizer = tokenizer::Tokenizer::new(input, config.input_file_name);
    let mut tokens = tokenizer
        .tokenize()
        .change_context(CompilerError)
        .attach_printable("failed to tokenize source code")?;

    dbg!(tokens);

    Ok(())
}
