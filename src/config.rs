use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigParseError {
    #[error("input file not provided")]
    InputNotProvided,
    #[error("executable name not found")]
    ExecutableNameNotFound,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub executable_name: String,
    pub input_file_name: String,
}

pub type ConfigResult = error_stack::Result<Config, ConfigParseError>;

impl Config {
    /// First arg should always be the compiler executable name and second one should always be the
    /// input file name
    pub fn from_args(args: &mut impl std::iter::Iterator<Item = String>) -> ConfigResult {
        Ok(Self {
            executable_name: args
                .next()
                .ok_or(ConfigParseError::ExecutableNameNotFound)?,
            input_file_name: args.next().ok_or(ConfigParseError::InputNotProvided)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn basic_input() {
        let mut args = vec!["program".to_string(), "main.ghl".to_string()].into_iter();
        let cfg = Config::from_args(&mut args).unwrap();
        assert_eq!(
            cfg,
            Config {
                executable_name: "program".to_string(),
                input_file_name: "main.ghl".to_string(),
            }
        )
    }
}
