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

// #[derive(Debug)]
// pub enum FindConfigError {
//   PathExpandHomeFailed(home_dir::Error),
//   NotFile(PathBuf),
//   NotFound
// }

// pub fn find_config(repo_root: &str) -> Result<PathBuf, FindConfigError> {
//   let shared_config_path_string = format!("{}/.git-ps/config.toml", repo_root);
//   let shared_config_path = Path::new(shared_config_path_string .as_str());

//   let repository_level_config_path_string = format!("{}/.git/git-ps/config.toml", repo_root);
//   let repository_level_config_path = Path::new(repository_level_config_path_string .as_str());

//   let user_level_config_path_string = "~/.config/git-ps/config.toml".to_string();
//   let user_level_config_path = Path::new(user_level_config_path_string.as_str()).expand_home().map_err(FindConfigError::PathExpandHomeFailed)?;

//   match path_exists_and_is_file(shared_config_path) {
//     PathExistsAndIsFile::ExistsAndIsFile => Ok(shared_config_path.to_path_buf()),
//     PathExistsAndIsFile::ExistsButNotFile=> Err(FindConfigError::NotFile(shared_config_path.to_path_buf())),
//     PathExistsAndIsFile::DoesNotExist => match path_exists_and_is_file(repository_level_config_path) {
//       PathExistsAndIsFile::ExistsAndIsFile => Ok(repository_level_config_path.to_path_buf()),
//       PathExistsAndIsFile::ExistsButNotFile => Err(FindConfigError::NotFile(repository_level_config_path.to_path_buf())),
//       PathExistsAndIsFile::DoesNotExist => match path_exists_and_is_file(&user_level_config_path) {
//         PathExistsAndIsFile::ExistsAndIsFile => Ok(user_level_config_path.to_path_buf()),
//         PathExistsAndIsFile::ExistsButNotFile => Err(FindConfigError::NotFile(user_level_config_path.to_path_buf())),
//         PathExistsAndIsFile::DoesNotExist => Err(FindConfigError::NotFound)
//       }
//     }
//   }
// }


// read_config(path: Path) -> Result<Config, Error>
// merge(configA: &Config, configB: &Config) -> Config
