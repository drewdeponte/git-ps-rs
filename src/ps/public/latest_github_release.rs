use std::result::Result;
use std::time::Duration;
use std::fmt;
use ureq;
use serde::Deserialize;
use version_compare::Version;
use super::super::private::utils;

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
  pub tag_name: String,
  pub html_url: String,
  pub body: String
}

#[derive(Debug)]
pub enum LatestGitHubReleaseError {
  CallFailed(ureq::Error),
  IntoStringFailed(std::io::Error),
  DeserializeJsonFailed(serde_json::Error)
}

impl fmt::Display for LatestGitHubReleaseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::CallFailed(e) => write!(f, "request failed - {}", e),
      Self::IntoStringFailed(e) => write!(f, "failed to convert Response to a string - {}", e),
      Self::DeserializeJsonFailed(e) => write!(f, "failed to deserialize JSON response - {}", e)
    }
  }
}

pub fn latest_github_release(org: &str, repo: &str) -> Result<GitHubRelease, LatestGitHubReleaseError> {
  let body: String = ureq::get(format!("https://api.github.com/repos/{}/{}/releases/latest", org, repo).as_str())
    .set("Accept", "application/vnd.github.v3+json")
    .timeout(Duration::from_secs(1))
    .call().map_err(LatestGitHubReleaseError::CallFailed)?
    .into_string().map_err(LatestGitHubReleaseError::IntoStringFailed)?;

  let release_dto: GitHubRelease = serde_json::from_str(&body).map_err(LatestGitHubReleaseError::DeserializeJsonFailed)?;
  Ok(release_dto)
}

#[derive(Debug)]
pub enum NewerReleaseAvailableError {
  LatestGithubReleaseFailed(LatestGitHubReleaseError),
  ParseLatestVersionFailed,
  ParseCurrentVersionFailed
}


impl fmt::Display for NewerReleaseAvailableError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::LatestGithubReleaseFailed(latest_github_release_error) => write!(f, "{}", latest_github_release_error),
      Self::ParseLatestVersionFailed => write!(f, "failed to parse newer version from tag"),
      Self::ParseCurrentVersionFailed => write!(f, "failed to parse current version from CAGO_PKG_VERSION")
    }
  }
}

pub fn newer_release_available() -> Result<Option<GitHubRelease>, NewerReleaseAvailableError> {
  let latest_release = latest_github_release("uptech", "git-ps-rs").map_err(NewerReleaseAvailableError::LatestGithubReleaseFailed)?;
  let latest_version = Version::from(&latest_release.tag_name).ok_or(NewerReleaseAvailableError::ParseLatestVersionFailed)?;
  let current_version = Version::from(env!("CARGO_PKG_VERSION")).ok_or(NewerReleaseAvailableError::ParseCurrentVersionFailed)?;
  if latest_version > current_version {
    Ok(Some(latest_release))
  } else {
    Ok(None)
  }
}

pub fn notify_of_newer_release(newer_release: Option<GitHubRelease>, color: bool) {
  if let Some(latest_release) = newer_release {
    utils::print_warn(color, format!(
r#"
  A new release of gps is available!

  {} - {}
"#, latest_release.tag_name, latest_release.html_url).as_str())
  }
}
