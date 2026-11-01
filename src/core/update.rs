use semver::Version;
use serde::Deserialize;

pub const LATEST_RELEASE_API: &str = "https://api.github.com/repos/pmqueiroz/nova/releases/latest";
pub const RELEASES_PAGE: &str = "https://github.com/pmqueiroz/nova/releases/latest";
pub const CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
  pub version: String,
  pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
  Unknown,
  Checking,
  UpToDate,
  Available(Release),
  Failed(String),
}

#[derive(Deserialize)]
struct GithubRelease {
  tag_name: String,
  html_url: Option<String>,
  #[serde(default)]
  draft: bool,
  #[serde(default)]
  prerelease: bool,
}

pub fn current_version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}

pub fn parse_version(tag: &str) -> Option<Version> {
  let t = tag.trim();
  Version::parse(t.strip_prefix('v').unwrap_or(t)).ok()
}

pub fn newer_release(body: &str, current: &str) -> Result<Option<Release>, String> {
  let release: GithubRelease =
    serde_json::from_str(body).map_err(|e| format!("unexpected release response: {e}"))?;
  if release.draft || release.prerelease {
    return Ok(None);
  }
  let latest = parse_version(&release.tag_name)
    .ok_or_else(|| format!("invalid release tag '{}'", release.tag_name))?;
  let current = parse_version(current).ok_or_else(|| format!("invalid version '{current}'"))?;
  if latest <= current {
    return Ok(None);
  }
  Ok(Some(Release {
    version: latest.to_string(),
    url: release
      .html_url
      .unwrap_or_else(|| RELEASES_PAGE.to_string()),
  }))
}

pub fn status_from(result: Result<Option<Release>, String>) -> Status {
  match result {
    Ok(Some(r)) => Status::Available(r),
    Ok(None) => Status::UpToDate,
    Err(e) => Status::Failed(e),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn body(tag: &str) -> String {
    format!(
      r#"{{"tag_name":"{tag}","html_url":"https://example.com/{tag}","draft":false,"prerelease":false}}"#
    )
  }

  #[test]
  fn newer_tag_is_reported() {
    let r = newer_release(&body("v1.2.0"), "1.1.9").unwrap().unwrap();
    assert_eq!(r.version, "1.2.0");
    assert_eq!(r.url, "https://example.com/v1.2.0");
  }

  #[test]
  fn same_or_older_tag_is_up_to_date() {
    assert_eq!(newer_release(&body("v1.0.0"), "1.0.0").unwrap(), None);
    assert_eq!(newer_release(&body("0.9.0"), "1.0.0").unwrap(), None);
  }

  #[test]
  fn semver_ordering_not_string_ordering() {
    assert!(newer_release(&body("v1.10.0"), "1.9.0").unwrap().is_some());
  }

  #[test]
  fn prerelease_and_draft_are_ignored() {
    let pre = r#"{"tag_name":"v9.0.0","prerelease":true}"#;
    let draft = r#"{"tag_name":"v9.0.0","draft":true}"#;
    assert_eq!(newer_release(pre, "1.0.0").unwrap(), None);
    assert_eq!(newer_release(draft, "1.0.0").unwrap(), None);
  }

  #[test]
  fn missing_url_falls_back_to_releases_page() {
    let r = newer_release(r#"{"tag_name":"v2.0.0"}"#, "1.0.0")
      .unwrap()
      .unwrap();
    assert_eq!(r.url, RELEASES_PAGE);
  }

  #[test]
  fn bad_payloads_are_errors() {
    assert!(newer_release("not json", "1.0.0").is_err());
    assert!(newer_release(r#"{"tag_name":"nightly"}"#, "1.0.0").is_err());
  }

  #[test]
  fn compiled_version_parses() {
    assert!(parse_version(current_version()).is_some());
  }
}
