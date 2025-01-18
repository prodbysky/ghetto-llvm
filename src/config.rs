use clap::Parser;

/// Compiler / interpreter for the ghetto-llvm language
#[derive(Debug, PartialEq, Eq, Parser)]
pub struct Config {
    /// The name of the source code file
    #[arg(short)]
    pub input_file_name: String,

    /// Dump ast to file
    #[arg(long)]
    pub dump_ast: bool,

    /// File name to which the AST should be dumped
    #[arg(long = "ast_out", default_value_t = String::from("out.ghl_ast"))]
    pub ast_out_name: String,

    /// Dump tokens to file
    #[arg(long)]
    pub dump_tokens: bool,

    /// File name to which the tokens should be dumped
    #[arg(long = "tokens_out", default_value_t = String::from("out.ghl_tokens"))]
    pub tokens_out_name: String,

    /// Dump c code to file
    #[arg(long)]
    pub dump_c: bool,

    /// File name to which the C code should be dumped
    #[arg(long = "c_out", default_value_t = String::from("out.c"))]
    pub c_out_name: String,
}
