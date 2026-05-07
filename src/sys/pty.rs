use async_channel::Sender;
use portable_pty::{Child, CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::thread;

#[derive(Debug, Clone)]
pub enum PtyCommand {
  Input(Vec<u8>),
  Resize { cols: u16, rows: u16 },
}

pub struct PtyBridge {
  writer: Box<dyn Write + Send>,
  _master_pty: Box<dyn MasterPty + Send>,
  _child: Box<dyn Child + Send + Sync>,
}

impl PtyBridge {
  pub fn new(tx: Sender<Vec<u8>>, cols: u16, rows: u16) -> anyhow::Result<Self> {
    let pty_system = NativePtySystem::default();

    let pair = pty_system.openpty(PtySize {
      rows,
      cols,
      pixel_width: 0,
      pixel_height: 0,
    })?;

    #[cfg(target_os = "windows")]
    let cmd = {
      let mut c = CommandBuilder::new(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe");
      if let Ok(profile) = std::env::var("USERPROFILE") {
        c.cwd(profile);
      }
      c.env("TERM", "xterm-256color");
      let ps_prompt_script = r#"
          Set-Item function:prompt {
              $p = $PWD.ProviderPath;
              $h = [regex]::Escape($env:USERPROFILE);
              $d = $p -replace ('^' + $h), '~';
              $uri = 'file://localhost/' + ($p -replace '\\', '/');
              Write-Host -NoNewline ('{0}]7;{1}{0}{2}' -f [char]27, $uri, [char]92);
              return $d + ' λ '
          }
      "#;
      c.args([
        "-NoProfile",
        "-NoLogo",
        "-NoExit",
        "-Command",
        ps_prompt_script,
      ]);
      c
    };

    #[cfg(not(target_os = "windows"))]
    let cmd = {
      let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
      let mut c = CommandBuilder::new(shell);
      if let Ok(home) = std::env::var("HOME") {
        c.cwd(home);
      }
      c.env("TERM", "xterm-256color");
      c.env("PS1", r"\w λ ");
      c.env(
        "PROMPT_COMMAND",
        r#"printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD""#,
      );
      c
    };

    let child = pair.slave.spawn_command(cmd)?;

    let mut reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;

    thread::spawn(move || {
      let mut buffer = [0u8; 1024];
      loop {
        match reader.read(&mut buffer) {
          Ok(0) => {
            println!("[PTY] process ended (EOF).");
            break;
          }
          Ok(n) => {
            let data = buffer[..n].to_vec();
            println!("[PTY] {} bytes: {:?}", n, String::from_utf8_lossy(&data));
            if tx.send_blocking(data).is_err() {
              println!("[PTY] iced sender error: channel closed.");
              break;
            }
          }
          Err(e) => {
            println!("[PTY] fatal read error: {}", e);
            break;
          }
        }
      }
    });

    Ok(Self {
      writer,
      _master_pty: pair.master,
      _child: child,
    })
  }

  pub fn write_to_pty(&mut self, input: &[u8]) {
    let _ = self.writer.write_all(input);
  }

  pub fn resize_pty(&mut self, cols: u16, rows: u16) {
    let _ = self._master_pty.resize(PtySize {
      rows,
      cols,
      pixel_width: 0,
      pixel_height: 0,
    });
  }
}
