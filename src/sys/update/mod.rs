pub mod apply;
mod download;
pub mod probe;

use std::path::PathBuf;

use crate::core::install::{self, UpdatePlan};
use crate::core::update::{self, Offer, Release};

const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const USER_AGENT: &str = concat!("nova/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone)]
pub enum Event {
  Progress(f32),
  Done(Result<PathBuf, String>),
}

pub fn latest_blocking() -> Result<Option<Release>, String> {
  let body = ureq::get(update::LATEST_RELEASE_API)
    .timeout(TIMEOUT)
    .set("Accept", "application/vnd.github+json")
    .set("User-Agent", USER_AGENT)
    .call()
    .map_err(|e| e.to_string())?
    .into_string()
    .map_err(|e| e.to_string())?;
  update::newer_release(&body, update::current_version())
}

pub fn offer_for(release: Release) -> Offer {
  let probe = probe::full();
  let install = install::detect(&probe);
  let plan = install::plan(&install, &release, probe.arch);
  Offer {
    release,
    install,
    plan,
  }
}

pub fn check_blocking() -> Result<Option<Offer>, String> {
  Ok(latest_blocking()?.map(offer_for))
}

pub async fn check() -> Result<Option<Offer>, String> {
  tokio::task::spawn_blocking(check_blocking)
    .await
    .map_err(|e| e.to_string())?
}

pub fn prepare_blocking(offer: &Offer, progress: &mut dyn FnMut(f32)) -> Result<PathBuf, String> {
  let dir = apply::version_dir(&offer.release.version)?;
  match &offer.plan {
    UpdatePlan::Stage { asset, checksums } => {
      let file = download::fetch_verified(asset, checksums, &dir, progress)?;
      apply::stage(offer, &file)?;
      Ok(file)
    }
    UpdatePlan::Delegate {
      download: Some((asset, checksums)),
      ..
    } => download::fetch_verified(asset, checksums, &dir, progress),
    _ => Ok(PathBuf::new()),
  }
}

pub fn prepare(offer: Offer) -> async_channel::Receiver<Event> {
  let (tx, rx) = async_channel::unbounded();
  std::thread::spawn(move || {
    let progress_tx = tx.clone();
    let result = prepare_blocking(&offer, &mut |p| {
      let _ = progress_tx.send_blocking(Event::Progress(p));
    });
    let _ = tx.send_blocking(Event::Done(result));
  });
  rx
}
