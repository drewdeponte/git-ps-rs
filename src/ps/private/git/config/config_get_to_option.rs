use super::config_get_error::ConfigGetError;
use std::result::Result;

pub fn config_get_to_option<T>(
    res_val: Result<T, git2::Error>,
) -> Result<Option<T>, ConfigGetError> {
    match res_val {
        Ok(v) => Ok(Some(v)),
        Err(e) => {
            if e.class() == git2::ErrorClass::Config && e.code() == git2::ErrorCode::NotFound {
                Ok(None)
            } else {
                Err(ConfigGetError::Failed(e))
            }
        }
    }
}
