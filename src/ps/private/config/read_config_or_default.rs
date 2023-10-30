use super::config_dto::*;
use super::read_config::*;
use std::path;

#[derive(Debug)]
pub enum ReadConfigDtoOrDefaultError {
    ReadConfigFailed(ReadConfigError),
}

impl std::fmt::Display for ReadConfigDtoOrDefaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadConfigFailed(e) => {
                write!(f, "read config dto failed, {}", e)
            }
        }
    }
}

impl std::error::Error for ReadConfigDtoOrDefaultError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadConfigFailed(e) => Some(e),
        }
    }
}

pub fn read_config_dto_or_default(
    path: &path::Path,
) -> Result<ConfigDto, ReadConfigDtoOrDefaultError> {
    let config: ConfigDto = read_config_dto(path)
        .map(|config_option| config_option.unwrap_or_default())
        .map_err(ReadConfigDtoOrDefaultError::ReadConfigFailed)?;
    Ok(config)
}
