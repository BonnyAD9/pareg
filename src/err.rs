use thiserror::Error;

pub type Result<T> = std::result::Result<T, ArgError>;

#[derive(Debug, Error)]
pub enum ArgError {
    #[error("Expected next argument")]
    NoMoreArguments,
    #[error(
        "Failed to parse '{value}' into '{typ}'{}",
        if let Some(msg) = .msg {
            format!(": {msg}")
        } else {
            "".to_owned()
        }
    )]
    FailedToParse {
        typ: &'static str,
        value: String,
        msg: Option<String>,
    },
    #[error("Expected value, but there was no value")]
    NoValue,
}
