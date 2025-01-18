mod ast;
mod config;
mod ir;
mod tokenizer;

use clap::Parser;
use error_stack::ResultExt;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("something bad happenned...")]
pub struct CompilerError;

fn main() -> error_stack::Result<(), CompilerError> {
    let config = config::Config::parse();
    let input = std::fs::read_to_string(&config.input_file_name)
        .attach_printable(format!(
            "failed to open and/read input file: {}",
            config.input_file_name
        ))
        .change_context(CompilerError)?;

    let tokenizer = tokenizer::Tokenizer::new(input, config.input_file_name);
    let tokens = tokenizer
        .tokenize()
        .change_context(CompilerError)
        .attach_printable("failed to tokenize source code")?;

    if config.dump_tokens {
        std::fs::write(config.tokens_out_name, format!("{:#?}", tokens))
            .change_context(CompilerError)
            .attach_printable("failed to dump tokens to file")?;
    }
    let mut ast_parser = ast::AstParser::new(tokens);
    let ast = ast_parser
        .parse()
        .change_context(CompilerError)
        .attach_printable("failed to parse the ast tree")?;
    if config.dump_ast {
        std::fs::write(config.ast_out_name, format!("{:#?}", ast))
            .change_context(CompilerError)
            .attach_printable("failed to dump ast to file")?;
    }
    let mut ir_generator = ir::IrGenerator::new(ast);
    let ir = ir_generator.generate();
    dbg!(ir);
    Ok(())
}
