use super::read_config::*;
use super::config_dto::*;
use std::path;

#[derive(Debug)]
pub enum ReadConfigDtoOrDefaultError {
  ReadConfigFailed(ReadConfigError)
}

pub fn read_config_dto_or_default(path: &path::Path) -> Result<ConfigDto, ReadConfigDtoOrDefaultError> {
  let config: ConfigDto = read_config_dto(path)
    .map(|config_option| config_option.unwrap_or_default())
    .map_err(ReadConfigDtoOrDefaultError::ReadConfigFailed)?;
  Ok(config)
}
