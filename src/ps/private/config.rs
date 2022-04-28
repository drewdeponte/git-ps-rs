use std::path::Path;
use std::option::Option;
use serde::Deserialize;
use toml;
use std::fs;
use std::result::Result;

#[derive(Debug, Deserialize)]
pub struct Config {
  request_review: RequestReviewConfig
}

#[derive(Debug, Deserialize)]
pub struct RequestReviewConfig {
  require_verification: Option<bool>
}

#[derive(Debug)]
pub enum ReadConfigError {
  ReadFailed(std::io::Error),
  DeserializeFailed(toml::de::Error)
}

pub fn read_config(path: &Path) -> Result<Config, ReadConfigError> {
  let config_content = fs::read_to_string(path).map_err(ReadConfigError::ReadFailed)?;
  let config: Config = toml::from_str(&config_content).map_err(ReadConfigError::DeserializeFailed)?;
  Ok(config)
}

// read_config(path: Path) -> Result<Config, Error>
// merge(configA: &Config, configB: &Config) -> Config
