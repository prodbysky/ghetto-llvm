use crate::tokenizer;
pub struct AstParser {
    tokens: Vec<tokenizer::Token>,
}

impl AstParser {
    pub fn new(mut tokens: Vec<tokenizer::Token>) -> Self {
        tokens.reverse();
        Self { tokens }
    }
}
