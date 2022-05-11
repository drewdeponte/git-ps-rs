use std::result::Result;
use std::time::Duration;
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
  DeserializeJsonFailed(serde_json::Error),
  VersionFromFailed(String)
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

pub fn notify_of_newer_release(color: bool) {
  if let Ok(Some(latest_release)) = newer_release_available() {
    utils::print_warn(color, format!(
r#"
  A new release of gps is available!

  {} - {}
"#, latest_release.tag_name, latest_release.html_url).as_str())
  }
}
