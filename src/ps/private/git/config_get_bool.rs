use super::config_get_error::ConfigGetError;
use super::config_get_to_option::config_get_to_option;
use git2;
use std::result::Result;

pub fn config_get_bool(config: &git2::Config, name: &str) -> Result<Option<bool>, ConfigGetError> {
    config_get_to_option(config.get_bool(name))
}
