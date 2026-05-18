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
  pub fn new(
    tx: Sender<Vec<u8>>,
    cols: u16,
    rows: u16,
    shell: &str,
    initial_cwd: Option<&str>,
    initial_command: Option<&str>,
  ) -> anyhow::Result<Self> {
    let pty_system = NativePtySystem::default();

    let pair = pty_system.openpty(PtySize {
      rows,
      cols,
      pixel_width: 0,
      pixel_height: 0,
    })?;

    let cmd = if let Some(cmd) = initial_command {
      build_direct_command(cmd, initial_cwd)
    } else {
      build_shell_command(shell, initial_cwd)
    };

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

fn accent_rgb() -> (u8, u8, u8) {
  let hex = &crate::core::config::get().theme.colors.accent;
  let h = hex.trim_start_matches('#');
  if h.len() >= 6 {
    let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(123);
    let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(147);
    let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(253);
    (r, g, b)
  } else {
    (123, 147, 253)
  }
}

fn build_direct_command(cmd: &str, initial_cwd: Option<&str>) -> CommandBuilder {
  let mut parts = cmd.split_whitespace();
  let exe = parts.next().unwrap_or(cmd);
  let mut c = CommandBuilder::new(exe);
  c.args(parts);
  if let Some(dir) = initial_cwd {
    c.cwd(dir);
  }
  c.env("TERM", "xterm-256color");
  c.env("COLORTERM", "truecolor");
  c.env("NOVA_TERMINAL", "1");
  c.env("TERM_PROGRAM", "Nova");
  c
}

fn build_shell_command(shell: &str, initial_cwd: Option<&str>) -> CommandBuilder {
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
      if let Some(dir) = initial_cwd {
        c.cwd(dir);
      }
      c.env("TERM", "xterm-256color");
      c.env("COLORTERM", "truecolor");
      c.env("NOVA_TERMINAL", "1");
      c.env("TERM_PROGRAM", "Nova");
      let (ar, ag, ab) = accent_rgb();
      let ps_prompt_script = format!(
        r#"
          Set-Item function:prompt {{
              if (-not $global:__nova_prompt_count) {{ $global:__nova_prompt_count = $true; $global:LASTEXITCODE = 0; $exitCode = 0; $is_first = $true }} else {{ $exitCode = $global:LASTEXITCODE; $global:LASTEXITCODE = 0; $is_first = $false }}
              $p = $PWD.ProviderPath;
              $h = [regex]::Escape($env:USERPROFILE);
              $d = $p -replace ('^' + $h), '~';
              $uri = 'file://localhost/' + ($p -replace '\\', '/');
              $ESC = [char]27;
              Write-Host -NoNewline ('{{0}}]7;{{1}}{{0}}{{2}}' -f [char]27, $uri, [char]92);
              $complete = if (-not $is_first) {{ "$ESC]777;command_complete;$exitCode$([char]7)" }} else {{ "" }}
              $diag = if ($exitCode -ne 0) {{ "$ESC]777;command_failure;$exitCode$([char]7)" }} else {{ "" }}
              return ($complete + $diag + $ESC + '[38;2;128;128;128m' + $d + $ESC + '[0m ' + $ESC + '[38;2;{ar};{ag};{ab}mλ' + $ESC + '[0m ')
          }}
          if (-not (Get-Command ssh -CommandType Function -ErrorAction SilentlyContinue)) {{
              function global:ssh {{
                  $fwa = 'b','c','D','E','e','F','I','i','J','L','l','m','o','p','Q','R','S','W','w'
                  $skip = $false; $host_arg = $null
                  foreach ($a in $args) {{
                      if ($skip) {{ $skip = $false; continue }}
                      if ($a -match '^-([a-zA-Z])$' -and ($fwa -contains $Matches[1])) {{ $skip = $true }}
                      elseif ($a -notmatch '^-') {{ $host_arg = $a; break }}
                  }}
                  if ($host_arg) {{ Write-Host -NoNewline ('{{0}}]7;ssh://{{1}}{{0}}{{2}}' -f [char]27, $host_arg, [char]92) }}
                  $ssh_exe = (Get-Command -Name ssh -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1).Source
                  if ($ssh_exe) {{ & $ssh_exe @args }} else {{ Write-Error 'ssh not found' }}
                  $p2 = $PWD.ProviderPath; $u2 = 'file://localhost/' + ($p2 -replace '\\', '/')
                  Write-Host -NoNewline ('{{0}}]7;ssh://{{1}}{{0}}{{2}}' -f [char]27, $u2, [char]92)
              }}
          }}
      "#
      );
      c.args([
        "-NoProfile",
        "-NoLogo",
        "-NoExit",
        "-Command",
        ps_prompt_script.as_str(),
      ]);
      c
    } else if is_cmd {
      let mut c = CommandBuilder::new(r"C:\Windows\System32\cmd.exe");
      if let Some(dir) = initial_cwd {
        c.cwd(dir);
      }
      c.env("NOVA_TERMINAL", "1");
      c.env("TERM_PROGRAM", "Nova");
      c
    } else if is_wsl {
      let mut c = CommandBuilder::new("wsl.exe");
      if let Some(dir) = initial_cwd {
        c.cwd(dir);
      }
      c.env("TERM", "xterm-256color");
      c.env("COLORTERM", "truecolor");
      c.env("NOVA_TERMINAL", "1");
      c.env("TERM_PROGRAM", "Nova");
      let (ar, ag, ab) = accent_rgb();
      c.env(
        "PS1",
        format!(
          "\\[\\e[38;2;128;128;128m\\]\\w\\[\\e[0m\\] \\[\\e[38;2;{ar};{ag};{ab}m\\]λ\\[\\e[0m\\] "
        ),
      );
      c.env(
        "PROMPT_COMMAND",
        r#"__nova_exit_code=$?
if ! declare -f __nova_ssh > /dev/null 2>&1; then
  __nova_ssh() { local h="" s=false; for a in "$@"; do $s && { s=false; continue; }; case "$a" in -b|-c|-D|-E|-e|-F|-I|-i|-J|-L|-l|-m|-o|-p|-Q|-R|-S|-W|-w) s=true;; -*) ;; *) h="$a"; break;; esac; done; [ -n "$h" ] && printf "\033]7;ssh://%s\033\\" "$h"; command ssh "$@"; printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD"; }
  ssh() { __nova_ssh "$@"; }
fi
if ! declare -f __nova_osc7 > /dev/null 2>&1; then
  __nova_osc7() { printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD"; }
fi
__nova_osc7
if [ -z "${__nova_first_prompt_done+x}" ]; then
  __nova_first_prompt_done=1
else
  printf "\033]777;command_complete;$__nova_exit_code\a"
fi
if [ "$__nova_exit_code" -ne 0 ]; then
  printf "\033]777;command_failure;$__nova_exit_code\a"
fi"#,
      );
      c
    } else if is_git_bash {
      let exe =
        find_git_bash_exe().unwrap_or_else(|| r"C:\Program Files\Git\bin\bash.exe".to_string());
      let mut c = CommandBuilder::new(exe);
      if let Some(dir) = initial_cwd {
        c.cwd(dir);
      }
      c.env("TERM", "xterm-256color");
      c.env("COLORTERM", "truecolor");
      c.env("NOVA_TERMINAL", "1");
      c.env("TERM_PROGRAM", "Nova");
      let (ar, ag, ab) = accent_rgb();
      c.env(
        "NOVA_PS1",
        format!(
          "\\[\\e[38;2;128;128;128m\\]\\w\\[\\e[0m\\] \\[\\e[38;2;{ar};{ag};{ab}m\\]λ\\[\\e[0m\\] "
        ),
      );
      c.env(
        "NOVA_PROMPT_COMMAND",
        r#"__nova_exit_code=$?
if ! declare -f __nova_ssh > /dev/null 2>&1; then
  __nova_ssh() { local h="" s=false; for a in "$@"; do $s && { s=false; continue; }; case "$a" in -b|-c|-D|-E|-e|-F|-I|-i|-J|-L|-l|-m|-o|-p|-Q|-R|-S|-W|-w) s=true;; -*) ;; *) h="$a"; break;; esac; done; [ -n "$h" ] && printf "\033]7;ssh://%s\033\\" "$h"; command ssh "$@"; printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD"; }
  ssh() { __nova_ssh "$@"; }
fi
if ! declare -f __nova_osc7 > /dev/null 2>&1; then
  __nova_osc7() { printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD"; }
fi
__nova_osc7
if [ -z "${__nova_first_prompt_done+x}" ]; then
  __nova_first_prompt_done=1
else
  printf "\033]777;command_complete;$__nova_exit_code\a"
fi
if [ "$__nova_exit_code" -ne 0 ]; then
  printf "\033]777;command_failure;$__nova_exit_code\a"
fi"#,
      );
      if let Some((nova_home, real_home)) = nova_git_bash_home() {
        c.env(
          "OLDHOME",
          windows_path_to_unix(std::path::Path::new(&real_home)),
        );
        c.env("HOME", windows_path_to_unix(&nova_home));
      }
      c.args(["--login", "-i"]);
      c
    } else {
      let mut c = CommandBuilder::new(shell);
      if let Some(dir) = initial_cwd {
        c.cwd(dir);
      }
      c.env("NOVA_TERMINAL", "1");
      c.env("TERM_PROGRAM", "Nova");
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
    if let Some(dir) = initial_cwd {
      c.cwd(dir);
    }

    match shell_name {
      "bash" => {
        if let Some((nova_home, real_home)) = nova_bash_home() {
          c.env("OLDHOME", real_home);
          c.env("HOME", nova_home);
        }
        c.args(["-l", "-i"]);
      }
      "zsh" => {
        if let Some(zdotdir) = nova_zsh_zdotdir() {
          c.env("ZDOTDIR", zdotdir);
        }
        c.args(["-l", "-i"]);
      }
      "fish" => {
        c.args(["-l"]);
      }
      _ => {}
    }

    c.env("TERM", "xterm-256color");
    c.env("COLORTERM", "truecolor");
    c.env("NOVA_TERMINAL", "1");
    c.env("TERM_PROGRAM", "Nova");
    if shell_name != "fish" {
      let (ar, ag, ab) = accent_rgb();
      c.env(
        "PS1",
        format!(
          "\\[\\e[38;2;128;128;128m\\]\\w\\[\\e[0m\\] \\[\\e[38;2;{ar};{ag};{ab}m\\]λ\\[\\e[0m\\] "
        ),
      );
      c.env(
        "PROMPT_COMMAND",
        r#"__nova_exit_code=$?
if ! declare -f __nova_ssh > /dev/null 2>&1; then
  __nova_ssh() { local h="" s=false; for a in "$@"; do $s && { s=false; continue; }; case "$a" in -b|-c|-D|-E|-e|-F|-I|-i|-J|-L|-l|-m|-o|-p|-Q|-R|-S|-W|-w) s=true;; -*) ;; *) h="$a"; break;; esac; done; [ -n "$h" ] && printf "\033]7;ssh://%s\033\\" "$h"; command ssh "$@"; printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD"; }
  ssh() { __nova_ssh "$@"; }
fi
if ! declare -f __nova_osc7 > /dev/null 2>&1; then
  __nova_osc7() { printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD"; }
fi
__nova_osc7
if [ -z "${__nova_first_prompt_done+x}" ]; then
  __nova_first_prompt_done=1
else
  printf "\033]777;command_complete;$__nova_exit_code\a"
fi
if [ "$__nova_exit_code" -ne 0 ]; then
  printf "\033]777;command_failure;$__nova_exit_code\a"
fi"#,
      );

      c.env(
        "NOVA_PS1",
        format!(
          "\\[\\e[38;2;128;128;128m\\]\\w\\[\\e[0m\\] \\[\\e[38;2;{ar};{ag};{ab}m\\]λ\\[\\e[0m\\] "
        ),
      );
      c.env(
        "NOVA_PROMPT_COMMAND",
        r#"__nova_exit_code=$?
if ! declare -f __nova_ssh > /dev/null 2>&1; then
  __nova_ssh() { local h="" s=false; for a in "$@"; do $s && { s=false; continue; }; case "$a" in -b|-c|-D|-E|-e|-F|-I|-i|-J|-L|-l|-m|-o|-p|-Q|-R|-S|-W|-w) s=true;; -*) ;; *) h="$a"; break;; esac; done; [ -n "$h" ] && printf "\033]7;ssh://%s\033\\" "$h"; command ssh "$@"; printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD"; }
  ssh() { __nova_ssh "$@"; }
fi
if ! declare -f __nova_osc7 > /dev/null 2>&1; then
  __nova_osc7() { printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD"; }
fi
__nova_osc7
if [ -z "${__nova_first_prompt_done+x}" ]; then
  __nova_first_prompt_done=1
else
  printf "\033]777;command_complete;$__nova_exit_code\a"
fi
if [ "$__nova_exit_code" -ne 0 ]; then
  printf "\033]777;command_failure;$__nova_exit_code\a"
fi"#,
      );
    }
    c
  }
}

#[cfg(not(target_os = "windows"))]
fn nova_zsh_zdotdir() -> Option<std::path::PathBuf> {
  let home = std::env::var("HOME").ok()?;
  let dir = std::path::PathBuf::from(&home)
    .join(".cache")
    .join("nova")
    .join("zsh-init");
  std::fs::create_dir_all(&dir).ok()?;

  let zshenv = "[[ -f \"$HOME/.zshenv\" ]] && source \"$HOME/.zshenv\"\n";
  let zprofile = "[[ -f \"$HOME/.zprofile\" ]] && source \"$HOME/.zprofile\"\n";
  let zshrc = "[[ -f \"$HOME/.zshrc\" ]] && source \"$HOME/.zshrc\"\nautoload -Uz add-zsh-hook 2>/dev/null\n__nova_precmd() { eval \"$NOVA_PROMPT_COMMAND\"; }\nadd-zsh-hook precmd __nova_precmd 2>/dev/null\n";

  std::fs::write(dir.join(".zshenv"), zshenv).ok()?;
  std::fs::write(dir.join(".zprofile"), zprofile).ok()?;
  std::fs::write(dir.join(".zshrc"), zshrc).ok()?;
  Some(dir)
}

#[cfg(not(target_os = "windows"))]
fn nova_bash_home() -> Option<(std::path::PathBuf, String)> {
  let home = std::env::var("HOME").ok()?;
  let dir = std::path::PathBuf::from(&home)
    .join(".cache")
    .join("nova")
    .join("bash-init");
  std::fs::create_dir_all(&dir).ok()?;

  let bash_profile = r#"export HOME="$OLDHOME"
unset PROMPT_COMMAND
if [[ -f "$HOME/.bash_profile" ]]; then
  source "$HOME/.bash_profile"
elif [[ -f "$HOME/.bash_login" ]]; then
  source "$HOME/.bash_login"
elif [[ -f "$HOME/.profile" ]]; then
  source "$HOME/.profile"
fi
PROMPT_COMMAND="${PROMPT_COMMAND:+${PROMPT_COMMAND}; }eval \"\$NOVA_PROMPT_COMMAND\""
PS1="$NOVA_PS1"
"#;
  let bashrc = r#"export HOME="$OLDHOME"
unset PROMPT_COMMAND
[[ -f "$HOME/.bashrc" ]] && source "$HOME/.bashrc"
PROMPT_COMMAND="${PROMPT_COMMAND:+${PROMPT_COMMAND}; }eval \"\$NOVA_PROMPT_COMMAND\""
PS1="$NOVA_PS1"
"#;

  std::fs::write(dir.join(".bash_profile"), bash_profile).ok()?;
  std::fs::write(dir.join(".bashrc"), bashrc).ok()?;
  Some((dir, home))
}

#[cfg(target_os = "windows")]
fn windows_path_to_unix(path: &std::path::Path) -> String {
  let s = path.to_string_lossy();
  let mut chars = s.chars();
  match (chars.next(), chars.next(), chars.next()) {
    (Some(drive), Some(':'), Some('\\')) => {
      format!(
        "/{}/{}",
        drive.to_ascii_lowercase(),
        &s[3..].replace('\\', "/")
      )
    }
    _ => s.replace('\\', "/").to_string(),
  }
}

#[cfg(target_os = "windows")]
fn nova_git_bash_home() -> Option<(std::path::PathBuf, String)> {
  let home = std::env::var("USERPROFILE").ok()?;
  let dir = std::path::PathBuf::from(&home)
    .join(".cache")
    .join("nova")
    .join("bash-init");
  std::fs::create_dir_all(&dir).ok()?;

  let prompt_script = r#"__nova_exit_code=$?
if ! declare -f __nova_ssh > /dev/null 2>&1; then
  __nova_ssh() { local h="" s=false; for a in "$@"; do $s && { s=false; continue; }; case "$a" in -b|-c|-D|-E|-e|-F|-I|-i|-J|-L|-l|-m|-o|-p|-Q|-R|-S|-W|-w) s=true;; -*) ;; *) h="$a"; break;; esac; done; [ -n "$h" ] && printf "\033]7;ssh://%s\033\\" "$h"; command ssh "$@"; printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD"; }
  ssh() { __nova_ssh "$@"; }
fi
if ! declare -f __nova_osc7 > /dev/null 2>&1; then
  __nova_osc7() { printf "\033]7;file://%s%s\033\\" "$HOSTNAME" "$PWD"; }
fi
__nova_osc7
if [ -z "${__nova_first_prompt_done+x}" ]; then
  __nova_first_prompt_done=1
else
  printf "\033]777;command_complete;$__nova_exit_code\a"
fi
if [ "$__nova_exit_code" -ne 0 ]; then
  printf "\033]777;command_failure;$__nova_exit_code\a"
fi
"#;
  std::fs::write(dir.join("nova_prompt.sh"), prompt_script).ok()?;

  let dir_unix = windows_path_to_unix(&dir);
  let bash_profile = format!(
    "export HOME=\"$OLDHOME\"\n\
     unset PROMPT_COMMAND\n\
     if [[ -f \"$HOME/.bash_profile\" ]]; then\n\
     \x20 source \"$HOME/.bash_profile\"\n\
     elif [[ -f \"$HOME/.bash_login\" ]]; then\n\
     \x20 source \"$HOME/.bash_login\"\n\
     elif [[ -f \"$HOME/.profile\" ]]; then\n\
     \x20 source \"$HOME/.profile\"\n\
     fi\n\
     PROMPT_COMMAND=\"${{PROMPT_COMMAND:+${{PROMPT_COMMAND}}; }}source {dir_unix}/nova_prompt.sh\"\n\
     PS1=\"$NOVA_PS1\"\n"
  );
  let bashrc = format!(
    "export HOME=\"$OLDHOME\"\n\
     unset PROMPT_COMMAND\n\
     [[ -f \"$HOME/.bashrc\" ]] && source \"$HOME/.bashrc\"\n\
     PROMPT_COMMAND=\"${{PROMPT_COMMAND:+${{PROMPT_COMMAND}}; }}source {dir_unix}/nova_prompt.sh\"\n\
     PS1=\"$NOVA_PS1\"\n"
  );

  std::fs::write(dir.join(".bash_profile"), bash_profile).ok()?;
  std::fs::write(dir.join(".bashrc"), bashrc).ok()?;
  Some((dir, home))
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
