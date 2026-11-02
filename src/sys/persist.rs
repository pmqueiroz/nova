use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver, Sender};

type Write = (PathBuf, Vec<u8>);

static WRITER: OnceLock<Sender<Write>> = OnceLock::new();

pub fn write_in_background(path: PathBuf, data: Vec<u8>) {
  let tx = WRITER.get_or_init(|| {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || run(rx));
    tx
  });
  if let Err(mpsc::SendError((path, data))) = tx.send((path, data)) {
    let _ = std::fs::write(path, data);
  }
}

fn run(rx: Receiver<Write>) {
  while let Ok((path, data)) = rx.recv() {
    let mut latest: HashMap<PathBuf, Vec<u8>> = HashMap::from([(path, data)]);
    while let Ok((path, data)) = rx.try_recv() {
      latest.insert(path, data);
    }
    for (path, data) in latest {
      let _ = std::fs::write(path, data);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn last_write_to_a_path_wins() {
    let dir = std::env::temp_dir().join(format!("nova-persist-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("out.bin");
    for i in 0..50u8 {
      write_in_background(file.clone(), vec![i]);
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::fs::read(&file).ok() != Some(vec![49]) && std::time::Instant::now() < deadline {
      std::thread::yield_now();
    }
    assert_eq!(std::fs::read(&file).unwrap(), vec![49]);
    let _ = std::fs::remove_dir_all(dir);
  }
}
