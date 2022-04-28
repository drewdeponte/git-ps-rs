use std::io;
use std::fs;
use std::path;
use toml;
use super::config_dto::*;

#[derive(Debug)]
pub enum ReadConfigError {
  ReadFailed(io::Error),
  DeserializeFailed(toml::de::Error)
}

pub fn read_config_dto(path: &path::Path) -> Result<Option<ConfigDto>, ReadConfigError> {
  let config_content_result = fs::read_to_string(path);
  let content_result_option: Result<Option<String>, ReadConfigError> = match config_content_result {
    Ok(content) => Ok(Some(content)),
    Err(e) => match e.kind() {
      io::ErrorKind::NotFound => Ok(None),
      _ => Err(ReadConfigError::ReadFailed(e))
    }
  };

  match content_result_option {
    Ok(content_option) => match content_option {
      Some(content) => toml::from_str(&content).map_err(ReadConfigError::DeserializeFailed).map(|c| Some(c)),
      None => Ok(None)
    },
    Err(e) => Err(e)
  }
}
