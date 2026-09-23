//! Check GitHub Releases for a newer version (no download/replace here).
//!
//! The host app decides how to fetch and swap the binary (Windows auto-update
//! pipeline, manual link for Linux/macOS, etc.).

use crate::AppId;
use serde::Deserialize;
use thiserror::Error;

/// Update-check errors.
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP or API failure.
    #[error("GitHub request failed: {0}")]
    Http(String),
    /// Response JSON did not match the expected shape.
    #[error("invalid GitHub release JSON: {0}")]
    Json(String),
}

/// Pointer to a GitHub repository that publishes releases.
#[derive(Debug, Clone)]
pub struct GithubRepo {
    /// Owner login, e.g. `RiasJ1Dar`.
    pub owner: String,
    /// Repository name, e.g. `TwitchDropFarm`.
    pub name: String,
}

impl GithubRepo {
    /// Create a repo pointer.
    pub fn new(owner: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
        }
    }
}

/// A newer release found on GitHub.
#[derive(Debug, Clone)]
pub struct NewerRelease {
    /// Tag name as returned by GitHub, e.g. `v1.3.0`.
    pub tag: String,
    /// Release page HTML URL.
    pub html_url: String,
    /// Optional body / release notes.
    pub body: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    draft: bool,
    prerelease: bool,
}

/// Compare `app.version` to the latest *non-draft* release tag.
///
/// Returns `Ok(None)` when current is up to date or the latest tag cannot be
/// parsed as newer. Tag may have a leading `v`. Comparison is semver-ish:
/// splits on `.` and compares numeric components; non-numeric tails are ignored
/// for ordering (good enough for `1.2` / `1.2.0` style tags).
pub fn check_latest(app: &AppId, repo: &GithubRepo) -> Result<Option<NewerRelease>, Error> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        repo.owner, repo.name
    );
    let agent = format!("{} / {}", app.name, app.version);
    let resp = ureq::get(&url)
        .set("User-Agent", &agent)
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| Error::Http(e.to_string()))?;

    let release: GhRelease = resp
        .into_json()
        .map_err(|e| Error::Json(e.to_string()))?;

    if release.draft || release.prerelease {
        return Ok(None);
    }

    let remote = strip_v(&release.tag_name);
    if is_newer(remote, &app.version) {
        Ok(Some(NewerRelease {
            tag: release.tag_name,
            html_url: release.html_url,
            body: release.body,
        }))
    } else {
        Ok(None)
    }
}

fn strip_v(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

fn is_newer(remote: &str, current: &str) -> bool {
    let r = parse_ver(remote);
    let c = parse_ver(current);
    r > c
}

fn parse_ver(s: &str) -> Vec<u64> {
    s.split('.')
        .map(|part| {
            let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse().unwrap_or(0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_compare() {
        assert!(is_newer("1.3.0", "1.2.0"));
        assert!(is_newer("1.2.1", "1.2"));
        assert!(!is_newer("1.2.0", "1.2.0"));
        assert!(!is_newer("1.1.9", "1.2"));
    }
}
