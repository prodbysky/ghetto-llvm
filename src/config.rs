use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigParseError {
    #[error("input file not provided")]
    InputNotProvided,
    #[error("executable name not found")]
    ExecutableNameNotFound,
}

pub struct Config {
    pub executable_name: String,
    pub input_file_name: String,
}

pub type ConfigResult = error_stack::Result<Config, ConfigParseError>;

impl Config {
    // First arg should always be the compiler executable name and second one should always be the
    // input file name
    pub fn from_args(args: &mut std::env::Args) -> ConfigResult {
        Ok(Self {
            executable_name: args
                .next()
                .ok_or(ConfigParseError::ExecutableNameNotFound)?,
            input_file_name: args.next().ok_or(ConfigParseError::InputNotProvided)?,
        })
    }
}
