use crate::core::update::{self, Release};

const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub fn check_blocking() -> Result<Option<Release>, String> {
  let body = ureq::get(update::LATEST_RELEASE_API)
    .timeout(TIMEOUT)
    .set("Accept", "application/vnd.github+json")
    .set("User-Agent", concat!("nova/", env!("CARGO_PKG_VERSION")))
    .call()
    .map_err(|e| e.to_string())?
    .into_string()
    .map_err(|e| e.to_string())?;
  update::newer_release(&body, update::current_version())
}

pub async fn check() -> Result<Option<Release>, String> {
  tokio::task::spawn_blocking(check_blocking)
    .await
    .map_err(|e| e.to_string())?
}
