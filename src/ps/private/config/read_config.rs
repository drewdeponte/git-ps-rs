use super::ConfigDto;
use std::fs;
use std::io;
use std::path;
use toml;

#[derive(Debug)]
pub enum ReadConfigError {
    ReadFailed(io::Error),
    DeserializeFailed(toml::de::Error),
}

impl std::fmt::Display for ReadConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeserializeFailed(e) => {
                write!(f, "failed to deserialize the patch stack config, {}", e)
            }
            Self::ReadFailed(e) => write!(f, "failed to read the patch stack config, {}", e),
        }
    }
}

impl std::error::Error for ReadConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadFailed(e) => Some(e),
            Self::DeserializeFailed(e) => Some(e),
        }
    }
}

pub fn read_config_dto(path: &path::Path) -> Result<Option<ConfigDto>, ReadConfigError> {
    let config_content_result = fs::read_to_string(path);
    let content_result_option: Result<Option<String>, ReadConfigError> = match config_content_result
    {
        Ok(content) => Ok(Some(content)),
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => Ok(None),
            _ => Err(ReadConfigError::ReadFailed(e)),
        },
    };

    match content_result_option {
        Ok(content_option) => match content_option {
            Some(content) => toml::from_str(&content)
                .map_err(ReadConfigError::DeserializeFailed)
                .map(Some),
            None => Ok(None),
        },
        Err(e) => Err(e),
    }
}
