use super::super::private::utils;
use serde::Deserialize;
use std::fmt;
use std::result::Result;
use std::time::Duration;
use ureq;
use version_compare::Version;

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub html_url: String,
    pub body: String,
}

#[derive(Debug)]
pub enum LatestGitHubReleaseError {
    Call(Box<ureq::Error>),
    IntoString(std::io::Error),
    DeserializeJson(serde_json::Error),
}

impl fmt::Display for LatestGitHubReleaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Call(e) => write!(f, "request failed - {}", e),
            Self::IntoString(e) => {
                write!(f, "failed to convert Response to a string - {}", e)
            }
            Self::DeserializeJson(e) => {
                write!(f, "failed to deserialize JSON response - {}", e)
            }
        }
    }
}

pub fn latest_github_release(
    org: &str,
    repo: &str,
) -> Result<GitHubRelease, LatestGitHubReleaseError> {
    let body: String = ureq::get(
        format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            org, repo
        )
        .as_str(),
    )
    .set("Accept", "application/vnd.github.v3+json")
    .timeout(Duration::from_secs(1))
    .call()
    .map_err(|e| LatestGitHubReleaseError::Call(e.into()))?
    .into_string()
    .map_err(LatestGitHubReleaseError::IntoString)?;

    let release_dto: GitHubRelease =
        serde_json::from_str(&body).map_err(LatestGitHubReleaseError::DeserializeJson)?;
    Ok(release_dto)
}

#[derive(Debug)]
pub enum NewerReleaseAvailableError {
    LatestGithubRelease(LatestGitHubReleaseError),
    ParseLatestVersion,
    ParseCurrentVersion,
}

impl fmt::Display for NewerReleaseAvailableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LatestGithubRelease(latest_github_release_error) => {
                write!(f, "{}", latest_github_release_error)
            }
            Self::ParseLatestVersion => write!(f, "failed to parse newer version from tag"),
            Self::ParseCurrentVersion => {
                write!(f, "failed to parse current version from CAGO_PKG_VERSION")
            }
        }
    }
}

pub fn newer_release_available() -> Result<Option<GitHubRelease>, NewerReleaseAvailableError> {
    let latest_release = latest_github_release("uptech", "git-ps-rs")
        .map_err(NewerReleaseAvailableError::LatestGithubRelease)?;
    let latest_version = Version::from(&latest_release.tag_name)
        .ok_or(NewerReleaseAvailableError::ParseLatestVersion)?;
    let current_version = Version::from(env!("CARGO_PKG_VERSION"))
        .ok_or(NewerReleaseAvailableError::ParseCurrentVersion)?;
    if latest_version > current_version {
        Ok(Some(latest_release))
    } else {
        Ok(None)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn notify_of_newer_release(newer_release: Option<GitHubRelease>, color: bool) {
    if let Some(latest_release) = newer_release {
        utils::print_warn(
            color,
            format!(
                r#"
  A new release of gps is available!

  {} - {}
"#,
                latest_release.tag_name, latest_release.html_url
            )
            .as_str(),
        )
    }
}

#[cfg(target_os = "macos")]
pub fn notify_of_newer_release(newer_release: Option<GitHubRelease>, color: bool) {
    if let Some(latest_release) = newer_release {
        utils::print_warn(
            color,
            format!(
                r#"
  A new release of gps is available!

  {} - {}

  To upgrade, run: brew update && brew upgrade git-ps-rs
"#,
                latest_release.tag_name, latest_release.html_url
            )
            .as_str(),
        )
    }
}
