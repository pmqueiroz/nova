use async_channel::Sender;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::thread;

pub struct PtyBridge {
  master: Box<dyn Write + Send>,
}

impl PtyBridge {
  pub fn new(tx: Sender<Vec<u8>>) -> anyhow::Result<Self> {
    let pty_system = NativePtySystem::default();

    let pair = pty_system.openpty(PtySize {
      rows: 24,
      cols: 80,
      pixel_width: 0,
      pixel_height: 0,
    })?;

    #[cfg(target_os = "windows")]
    let cmd = CommandBuilder::new("powershell.exe");

    #[cfg(not(target_os = "windows"))]
    let cmd = CommandBuilder::new(std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string()));

    let _child = pair.slave.spawn_command(cmd)?;

    let mut reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;

    thread::spawn(move || {
      let mut buffer = [0u8; 1024];
      loop {
        match reader.read(&mut buffer) {
          Ok(0) => break, // EOF
          Ok(n) => {
            let data = buffer[..n].to_vec();
            if tx.try_send(data).is_err() {
              break;
            }
          }
          Err(_) => break,
        }
      }
    });

    Ok(Self { master: writer })
  }

  pub fn write_to_pty(&mut self, input: &[u8]) {
    let _ = self.master.write_all(input);
  }
}
