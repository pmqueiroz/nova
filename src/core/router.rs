use std::time::{Duration, Instant};

use vte::Parser;

use crate::core::block::{BlockId, BlockState, Phase, Session};
use crate::core::grid::{ControlCommand, Grid};
use crate::sys::parser::AnsiExecutor;

pub const INTEGRATION_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
  Detecting,
  Blocks,
  Classic,
}

#[derive(Clone, Debug)]
pub struct Integration {
  pub enabled: bool,
  pub mode: Mode,
  pub preexec: bool,
  pub awaiting_echo: bool,
  pub pending_exit: Option<i32>,
  pub spawned_at: Instant,
  pub readline: bool,
}

impl Integration {
  pub fn new(enabled: bool) -> Self {
    Self {
      enabled,
      mode: if enabled {
        Mode::Detecting
      } else {
        Mode::Classic
      },
      preexec: false,
      awaiting_echo: false,
      pending_exit: None,
      spawned_at: Instant::now(),
      readline: false,
    }
  }

  pub fn for_shell(enabled: bool, shell_cmd: &str) -> Self {
    Self {
      readline: is_readline_shell(shell_cmd),
      ..Self::new(enabled)
    }
  }

  pub fn submit_bytes(&self, command: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(command.len() + 2);
    if self.readline {
      bytes.push(0x15);
    }
    bytes.extend_from_slice(command.as_bytes());
    bytes.push(b'\r');
    bytes
  }

  pub fn is_blocks(&self) -> bool {
    self.mode == Mode::Blocks
  }

  pub fn expire(&mut self, now: Instant) -> bool {
    if self.mode == Mode::Detecting && now.duration_since(self.spawned_at) >= INTEGRATION_TIMEOUT {
      self.mode = Mode::Classic;
      return true;
    }
    false
  }

  pub fn on_submit(&mut self) {
    self.awaiting_echo = !self.preexec;
  }
}

pub fn is_readline_shell(shell_cmd: &str) -> bool {
  let name = std::path::Path::new(shell_cmd)
    .file_stem()
    .and_then(|n| n.to_str())
    .unwrap_or(shell_cmd)
    .to_lowercase();
  matches!(name.as_str(), "bash" | "zsh" | "git-bash" | "sh")
}

#[derive(Default, Debug)]
pub struct Routed {
  pub controls: Vec<ControlCommand>,
  pub responses: Vec<Vec<u8>>,
  pub bell: bool,
  pub finished: Vec<(BlockId, i32)>,
  pub entered_blocks: bool,
  pub raw_changed: bool,
}

enum Target {
  Screen,
  Block(BlockId),
}

fn target(session: &Session, integ: &Integration) -> Target {
  if integ.mode != Mode::Blocks {
    return Target::Screen;
  }
  match session.phase {
    Phase::Prompt => Target::Screen,
    Phase::Raw(id) => Target::Block(id),
    Phase::Running(id) => {
      let pending = session
        .block(id)
        .is_some_and(|b| b.state == BlockState::Pending);
      if pending {
        Target::Screen
      } else {
        Target::Block(id)
      }
    }
  }
}

fn begin_output(session: &mut Session, screen: &Grid) {
  if let Some(b) = session.active_mut()
    && b.state == BlockState::Pending
  {
    b.state = BlockState::Running;
    b.output.cursor_x = 0;
    b.output.cursor_y = screen.cursor_y.min(b.output.rows.saturating_sub(1));
  }
}

fn finish(session: &mut Session, integ: &mut Integration, code: i32, out: &mut Routed) {
  if let Some(id) = session.finish(code) {
    out.finished.push((id, code));
  }
  integ.awaiting_echo = false;
}

fn handle_control(
  cmd: ControlCommand,
  session: &mut Session,
  screen: &Grid,
  integ: &mut Integration,
  out: &mut Routed,
) {
  match cmd {
    ControlCommand::PromptReady { preexec } if integ.enabled => {
      integ.preexec = preexec;
      if integ.mode == Mode::Detecting || (integ.mode == Mode::Classic && !session.is_running()) {
        integ.mode = Mode::Blocks;
        out.entered_blocks = true;
      }
      if integ.mode == Mode::Blocks && session.is_running() {
        let code = integ.pending_exit.take().unwrap_or(0);
        finish(session, integ, code, out);
      }
      integ.pending_exit = None;
    }
    ControlCommand::CommandStart if integ.mode == Mode::Blocks => {
      if session.is_running() {
        begin_output(session, screen);
        integ.awaiting_echo = false;
      }
    }
    ControlCommand::CommandComplete(code) if integ.mode == Mode::Blocks => {
      if session.is_running() {
        integ.pending_exit = Some(code as i32);
      }
      out.controls.push(ControlCommand::CommandComplete(code));
    }
    other => out.controls.push(other),
  }
}

pub fn route(
  parser: &mut Parser,
  screen: &mut Grid,
  session: &mut Session,
  integ: &mut Integration,
  bytes: &[u8],
) -> Routed {
  let mut out = Routed::default();
  for &byte in bytes {
    let was_raw = session.is_raw();
    let target = target(session, integ);
    let (bell, mut responses, controls, in_alt, on_block) = {
      let (grid, on_block): (&mut Grid, bool) = match target {
        Target::Block(id) if session.block(id).is_some() => {
          (&mut session.block_mut(id).unwrap().output, true)
        }
        _ => (&mut *screen, false),
      };
      let mut exec = AnsiExecutor {
        grid,
        bell_pending: false,
      };
      parser.advance(&mut exec, &[byte]);
      let bell = exec.bell_pending;
      let grid = exec.grid;
      let responses = std::mem::take(&mut grid.output_queue);
      let controls = std::mem::take(&mut grid.control_queue);
      (bell, responses, controls, grid.in_alt_screen(), on_block)
    };
    out.bell |= bell;
    out.responses.append(&mut responses);
    if on_block {
      if in_alt && !session.is_raw() {
        session.enter_raw();
      } else if !in_alt && session.is_raw() {
        session.leave_raw();
      }
    }
    for c in controls {
      handle_control(c, session, screen, integ, &mut out);
    }
    if integ.mode == Mode::Blocks
      && integ.awaiting_echo
      && byte == b'\n'
      && !on_block
      && session.is_running()
    {
      integ.awaiting_echo = false;
      begin_output(session, screen);
    }
    out.raw_changed |= was_raw != session.is_raw();
  }
  if integ.mode == Mode::Blocks || integ.mode == Mode::Detecting {
    session.cwd = screen.pwd.clone();
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  struct Rig {
    parser: Parser,
    screen: Grid,
    session: Session,
    integ: Integration,
  }

  impl Rig {
    fn new() -> Self {
      Self {
        parser: Parser::new(),
        screen: Grid::new(40, 10),
        session: Session::new("~".into(), 40, 10),
        integ: Integration::new(true),
      }
    }

    fn feed(&mut self, s: &str) -> Routed {
      route(
        &mut self.parser,
        &mut self.screen,
        &mut self.session,
        &mut self.integ,
        s.as_bytes(),
      )
    }

    fn submit(&mut self, cmd: &str) -> BlockId {
      let id = self.session.submit(cmd).unwrap();
      self.integ.on_submit();
      id
    }
  }

  const READY_PREEXEC: &str = "\x1b]777;prompt_ready;preexec\x07";
  const READY: &str = "\x1b]777;prompt_ready\x07";
  const START: &str = "\x1b]777;command_start\x07";

  #[test]
  fn readline_shells_get_kill_line_prefix() {
    assert_eq!(
      Integration::for_shell(true, "git-bash").submit_bytes("ls"),
      b"\x15ls\r"
    );
    assert_eq!(
      Integration::for_shell(true, "/bin/zsh").submit_bytes("ls"),
      b"\x15ls\r"
    );
    assert_eq!(
      Integration::for_shell(true, "powershell").submit_bytes("ls"),
      b"ls\r"
    );
    assert_eq!(
      Integration::for_shell(true, "cmd").submit_bytes("ls"),
      b"ls\r"
    );
  }

  #[test]
  fn disabled_integration_stays_classic() {
    let mut r = Rig::new();
    r.integ = Integration::new(false);
    let out = r.feed(&format!("{READY_PREEXEC}$ "));
    assert!(!out.entered_blocks);
    assert_eq!(r.integ.mode, Mode::Classic);
    assert_eq!(r.screen.cells[0].c.as_ref(), "$");
  }

  #[test]
  fn first_prompt_switches_to_blocks() {
    let mut r = Rig::new();
    let out = r.feed(&format!("banner\r\n{READY_PREEXEC}$ "));
    assert!(out.entered_blocks);
    assert!(r.integ.is_blocks());
    assert!(r.integ.preexec);
  }

  #[test]
  fn preexec_shell_routes_output_between_marks() {
    let mut r = Rig::new();
    r.feed(&format!("{READY_PREEXEC}$ "));
    let id = r.submit("ls");
    let out = r.feed(&format!(
      "ls\r\n{START}a.txt\r\nb.txt\r\n\x1b]777;command_complete;0\x07{READY_PREEXEC}$ "
    ));
    assert_eq!(out.finished, vec![(id, 0)]);
    let b = r.session.block(id).unwrap();
    assert_eq!(b.text(), "a.txt\nb.txt");
    assert_eq!(b.exit_code, Some(0));
    assert_eq!(r.session.phase, Phase::Prompt);
  }

  #[test]
  fn output_split_across_chunks() {
    let mut r = Rig::new();
    r.feed(READY_PREEXEC);
    let id = r.submit("seq 3");
    r.feed("seq 3\r\n\x1b]777;comm");
    r.feed("and_start\x071\r\n2");
    r.feed("\r\n3\r\n\x1b]777;command_complete;0\x07");
    assert_eq!(r.session.block(id).unwrap().text(), "1\n2\n3");
  }

  #[test]
  fn failing_command_records_exit_code_and_forwards_failure() {
    let mut r = Rig::new();
    r.feed(READY_PREEXEC);
    let id = r.submit("clsd");
    let out = r.feed(&format!(
      "{START}zsh: command not found: clsd\r\n\x1b]777;command_complete;127\x07\x1b]777;command_failure;127\x07{READY_PREEXEC}"
    ));
    assert_eq!(out.finished, vec![(id, 127)]);
    assert!(r.session.block(id).unwrap().is_error());
    assert!(out.controls.contains(&ControlCommand::CommandFailure(127)));
  }

  #[test]
  fn conpty_reordered_osc_waits_for_text_prompt_mark() {
    let mut r = Rig::new();
    r.feed("\u{F8F1}PS> ");
    let id = r.submit("echo hello");
    let out = r.feed("echo hello\r\n\u{F8F2}\x1b]777;command_complete;0\x07hello\r\n\u{F8F1}PS> ");
    assert_eq!(out.finished, vec![(id, 0)]);
    assert_eq!(r.session.block(id).unwrap().text(), "hello");
  }

  #[test]
  fn replayed_command_start_does_not_reset_block() {
    let mut r = Rig::new();
    r.feed(READY_PREEXEC);
    let id = r.submit("dir");
    r.feed("\u{F8F2}line1\r\nline2\r\n\u{F8F2}");
    assert_eq!(r.session.block(id).unwrap().text(), "line1\nline2");
  }

  #[test]
  fn exit_code_is_kept_until_prompt() {
    let mut r = Rig::new();
    r.feed("\u{F8F1}");
    let id = r.submit("clsd");
    let out = r.feed("\u{F8F2}\x1b]777;command_complete;127\x07not found\r\n");
    assert!(out.finished.is_empty());
    let out = r.feed("\u{F8F1}");
    assert_eq!(out.finished, vec![(id, 127)]);
  }

  #[test]
  fn shell_without_preexec_skips_echo_line() {
    let mut r = Rig::new();
    r.feed(&format!("{READY}C:\\>"));
    assert!(!r.integ.preexec);
    let id = r.submit("dir");
    r.feed(&format!("dir\r\n file1\r\n{READY}C:\\>"));
    let b = r.session.block(id).unwrap();
    assert_eq!(b.text(), " file1");
    assert_eq!(b.exit_code, Some(0));
  }

  #[test]
  fn alt_screen_enters_and_leaves_raw() {
    let mut r = Rig::new();
    r.feed(READY_PREEXEC);
    let id = r.submit("vim");
    let out = r.feed(&format!("{START}\x1b[?1049hEDITOR"));
    assert!(out.raw_changed);
    assert_eq!(r.session.phase, Phase::Raw(id));
    let out = r.feed("\x1b[?1049l");
    assert!(out.raw_changed);
    assert_eq!(r.session.phase, Phase::Running(id));
  }

  #[test]
  fn block_starts_at_screen_cursor_row() {
    let mut r = Rig::new();
    r.feed(&format!("l1\r\nl2\r\nl3\r\n{READY_PREEXEC}$ "));
    let id = r.submit("echo hi");
    r.feed(&format!("echo hi\r\n{START}hi\r\n"));
    let b = r.session.block(id).unwrap();
    assert_eq!(b.text(), "hi");
  }

  #[test]
  fn cwd_follows_osc7_on_prompt() {
    let mut r = Rig::new();
    r.feed(&format!(
      "\x1b]7;file://host/home/me/proj\x1b\\{READY_PREEXEC}"
    ));
    assert!(r.session.cwd.ends_with("proj"));
  }

  #[test]
  fn late_prompt_upgrades_classic_to_blocks() {
    let mut r = Rig::new();
    let later = r.integ.spawned_at + INTEGRATION_TIMEOUT;
    r.integ.expire(later);
    assert_eq!(r.integ.mode, Mode::Classic);
    let out = r.feed(READY_PREEXEC);
    assert!(out.entered_blocks);
    assert!(r.integ.is_blocks());
  }

  #[test]
  fn detection_times_out_to_classic() {
    let mut r = Rig::new();
    let later = r.integ.spawned_at + INTEGRATION_TIMEOUT;
    assert!(r.integ.expire(later));
    assert_eq!(r.integ.mode, Mode::Classic);
    let id = r.session.submit("x");
    assert!(id.is_some());
  }
}
