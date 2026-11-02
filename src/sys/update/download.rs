use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::core::install;
use crate::core::update::Asset;

fn agent() -> ureq::Agent {
  ureq::AgentBuilder::new()
    .timeout_connect(std::time::Duration::from_secs(10))
    .timeout_read(std::time::Duration::from_secs(30))
    .user_agent(super::USER_AGENT)
    .build()
}

fn sha256_file(path: &Path) -> Result<String, String> {
  let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
  let mut hasher = Sha256::new();
  std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
  Ok(format!("{:x}", hasher.finalize()))
}

pub fn fetch_verified(
  asset: &Asset,
  checksums: &Asset,
  dir: &Path,
  progress: &mut dyn FnMut(f32),
) -> Result<PathBuf, String> {
  std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
  let agent = agent();
  let sums = agent
    .get(&checksums.url)
    .call()
    .map_err(|e| e.to_string())?
    .into_string()
    .map_err(|e| e.to_string())?;
  let expected = install::parse_checksums(&sums)
    .remove(&asset.name)
    .ok_or_else(|| format!("{} is missing from checksums.txt", asset.name))?;

  let dest = dir.join(&asset.name);
  if dest.is_file() && sha256_file(&dest)? == expected {
    progress(1.0);
    return Ok(dest);
  }

  let resp = agent.get(&asset.url).call().map_err(|e| e.to_string())?;
  let total: Option<u64> = resp.header("Content-Length").and_then(|v| v.parse().ok());
  let mut reader = resp.into_reader();
  let part = dir.join(format!("{}.part", asset.name));
  let mut file = std::fs::File::create(&part).map_err(|e| e.to_string())?;
  let mut hasher = Sha256::new();
  let mut buf = vec![0u8; 64 * 1024];
  let mut done: u64 = 0;
  let mut last_pct = 0u32;
  loop {
    let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
    if n == 0 {
      break;
    }
    file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
    hasher.update(&buf[..n]);
    done += n as u64;
    if let Some(total) = total.filter(|t| *t > 0) {
      let pct = (done * 100 / total) as u32;
      if pct != last_pct {
        last_pct = pct;
        progress(done as f32 / total as f32);
      }
    }
  }
  file.flush().map_err(|e| e.to_string())?;
  drop(file);

  let got = format!("{:x}", hasher.finalize());
  if got != expected {
    let _ = std::fs::remove_file(&part);
    return Err(format!("checksum mismatch for {}", asset.name));
  }
  let _ = std::fs::remove_file(&dest);
  std::fs::rename(&part, &dest).map_err(|e| e.to_string())?;
  progress(1.0);
  Ok(dest)
}
