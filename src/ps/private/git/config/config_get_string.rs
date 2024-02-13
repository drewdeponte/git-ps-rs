use super::{config_get_to_option, ConfigGetError};

pub fn config_get_string(
    config: &git2::Config,
    name: &str,
) -> Result<Option<String>, ConfigGetError> {
    config_get_to_option(config.get_string(name))
}
