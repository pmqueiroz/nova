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
  pub fn new(tx: Sender<Vec<u8>>, cols: u16, rows: u16, shell: &str) -> anyhow::Result<Self> {
    let pty_system = NativePtySystem::default();

    let pair = pty_system.openpty(PtySize {
      rows,
      cols,
      pixel_width: 0,
      pixel_height: 0,
    })?;

    let cmd = build_shell_command(shell);

    let child = pair.slave.spawn_command(cmd)?;

    let mut reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;

    thread::spawn(move || {
      let mut buffer = [0u8; 1024];
      loop {
        match reader.read(&mut buffer) {
          Ok(0) => {
            break;
          }
          Ok(n) => {
            let data = buffer[..n].to_vec();
            if tx.send_blocking(data).is_err() {
              break;
            }
          }
          Err(_) => {
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

fn build_shell_command(shell: &str) -> CommandBuilder {
  #[cfg(target_os = "windows")]
  {
    let lower = shell.to_lowercase();
    let is_pwsh = lower == "pwsh" || lower.ends_with("pwsh.exe");
    let is_powershell = lower == "powershell" || lower.ends_with("powershell.exe");
    let is_cmd = lower == "cmd" || lower.ends_with("cmd.exe");
    let is_wsl = lower == "wsl" || lower.ends_with("wsl.exe");
    let is_git_bash = lower == "git-bash";

    if is_powershell || is_pwsh {
      let exe = if is_pwsh {
        "pwsh".to_string()
      } else {
        r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe".to_string()
      };
      let mut c = CommandBuilder::new(exe);
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
      c.args(["-NoProfile", "-NoLogo", "-NoExit", "-Command", ps_prompt_script]);
      c
    } else if is_cmd {
      let mut c = CommandBuilder::new(r"C:\Windows\System32\cmd.exe");
      if let Ok(profile) = std::env::var("USERPROFILE") {
        c.cwd(profile);
      }
      c
    } else if is_wsl {
      let mut c = CommandBuilder::new("wsl.exe");
      c.env("TERM", "xterm-256color");
      c.env("PS1", r"\w λ ");
      c.env(
        "PROMPT_COMMAND",
        r#"printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD""#,
      );
      c
    } else if is_git_bash {
      let exe = find_git_bash_exe()
        .unwrap_or_else(|| r"C:\Program Files\Git\bin\bash.exe".to_string());
      let mut c = CommandBuilder::new(exe);
      if let Ok(profile) = std::env::var("USERPROFILE") {
        c.cwd(profile);
      }
      c.env("TERM", "xterm-256color");
      c.env("PS1", r"\w λ ");
      c.env(
        "PROMPT_COMMAND",
        r#"printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD""#,
      );
      c.args(["--login", "-i"]);
      c
    } else {
      let mut c = CommandBuilder::new(shell);
      if let Ok(profile) = std::env::var("USERPROFILE") {
        c.cwd(profile);
      }
      c
    }
  }
  #[cfg(not(target_os = "windows"))]
  {
    let shell_name = std::path::Path::new(shell)
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or(shell);
    let mut c = CommandBuilder::new(shell);
    if let Ok(home) = std::env::var("HOME") {
      c.cwd(home);
    }
    c.env("TERM", "xterm-256color");
    if shell_name != "fish" {
      c.env("PS1", r"\w λ ");
      c.env(
        "PROMPT_COMMAND",
        r#"printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD""#,
      );
    }
    c
  }
}

#[cfg(target_os = "windows")]
fn find_git_bash_exe() -> Option<String> {
  let candidates = [
    r"C:\Program Files\Git\bin\bash.exe",
    r"C:\Program Files (x86)\Git\bin\bash.exe",
    r"C:\Git\bin\bash.exe",
  ];
  candidates
    .iter()
    .find(|p| std::path::Path::new(p).exists())
    .map(|p| p.to_string())
}
