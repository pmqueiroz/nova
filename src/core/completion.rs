use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
  pub name: String,
  pub dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word<'a> {
  pub start: usize,
  pub raw: &'a str,
  pub command: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Target {
  dir_text: String,
  prefix: String,
  quote: Option<char>,
  sep: char,
}

pub struct Context<'a> {
  pub cwd: &'a Path,
  pub home: Option<PathBuf>,
  pub posix: bool,
  pub history: Vec<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
  pub head: String,
  pub options: Vec<String>,
  pub index: Option<usize>,
}

impl Completion {
  pub fn step(&mut self, delta: i32) -> String {
    let len = self.options.len() as i64;
    let next = match self.index {
      None if delta < 0 => len - 1,
      None => 0,
      Some(i) => (i as i64 + i64::from(delta)).rem_euclid(len),
    } as usize;
    self.index = Some(next);
    format!("{}{}", self.head, self.options[next])
  }
}

const IGNORE_CASE: bool = cfg!(windows);
const COMMAND_SEPARATORS: [char; 4] = ['|', ';', '&', '('];
const NEEDS_QUOTE: [char; 10] = ['&', '(', ')', ';', '|', '<', '>', '\'', '`', '$'];

pub fn current_word(line: &str) -> Word<'_> {
  let mut quote: Option<char> = None;
  let mut start = 0;
  for (i, c) in line.char_indices() {
    match quote {
      Some(q) if c == q => quote = None,
      Some(_) => {}
      None if c == '"' || c == '\'' => quote = Some(c),
      None if c.is_whitespace() => start = i + c.len_utf8(),
      None => {}
    }
  }
  let before = line[..start].trim_end();
  Word {
    start,
    raw: &line[start..],
    command: before.is_empty() || before.ends_with(COMMAND_SEPARATORS),
  }
}

fn is_separator(c: char, posix: bool) -> bool {
  c == '/' || (cfg!(windows) && !posix && c == '\\')
}

fn default_separator(posix: bool) -> char {
  if posix || !cfg!(windows) { '/' } else { '\\' }
}

fn unquote(raw: &str) -> (Option<char>, &str) {
  let Some(q) = raw.chars().next().filter(|c| *c == '"' || *c == '\'') else {
    return (None, raw);
  };
  let body = &raw[1..];
  (Some(q), body.strip_suffix(q).unwrap_or(body))
}

fn parse_target(raw: &str, posix: bool) -> Target {
  let (quote, body) = unquote(raw);
  match body.char_indices().rfind(|(_, c)| is_separator(*c, posix)) {
    Some((i, sep)) => Target {
      dir_text: body[..=i].to_string(),
      prefix: body[i + 1..].to_string(),
      quote,
      sep,
    },
    None => Target {
      dir_text: String::new(),
      prefix: body.to_string(),
      quote,
      sep: default_separator(posix),
    },
  }
}

fn msys_drive_path(text: &str) -> Option<PathBuf> {
  let rest = text.strip_prefix('/')?;
  let mut chars = rest.chars();
  let drive = chars.next().filter(char::is_ascii_alphabetic)?;
  let tail = chars.as_str();
  if !(tail.is_empty() || tail.starts_with('/')) {
    return None;
  }
  Some(PathBuf::from(format!(
    "{}:\\{}",
    drive.to_ascii_uppercase(),
    tail.trim_start_matches('/')
  )))
}

fn resolve_dir(dir_text: &str, ctx: &Context) -> PathBuf {
  if dir_text.is_empty() {
    return ctx.cwd.to_path_buf();
  }
  if let Some(rest) = dir_text.strip_prefix('~')
    && (rest.is_empty() || rest.starts_with(|c| is_separator(c, ctx.posix)))
    && let Some(home) = &ctx.home
  {
    return home.join(rest.trim_start_matches(|c| is_separator(c, ctx.posix)));
  }
  if cfg!(windows)
    && ctx.posix
    && let Some(p) = msys_drive_path(dir_text)
  {
    return p;
  }
  let path = Path::new(dir_text);
  if path.is_absolute() {
    path.to_path_buf()
  } else {
    ctx.cwd.join(path)
  }
}

fn starts_with(name: &str, prefix: &str) -> bool {
  if IGNORE_CASE {
    name.to_lowercase().starts_with(&prefix.to_lowercase())
  } else {
    name.starts_with(prefix)
  }
}

fn compare_names(a: &str, b: &str) -> Ordering {
  a.to_lowercase()
    .cmp(&b.to_lowercase())
    .then_with(|| a.cmp(b))
}

fn same_name(a: &str, b: &str) -> bool {
  if IGNORE_CASE {
    a.eq_ignore_ascii_case(b)
  } else {
    a == b
  }
}

pub fn list_dir(dir: &Path) -> Vec<Candidate> {
  let Ok(entries) = fs::read_dir(dir) else {
    return Vec::new();
  };
  entries
    .flatten()
    .filter_map(|e| {
      let name = e.file_name().into_string().ok()?;
      let dir = fs::metadata(e.path()).is_ok_and(|m| m.is_dir());
      Some(Candidate { name, dir })
    })
    .collect()
}

#[cfg(windows)]
fn executable_stem(path: &Path, exts: &[String]) -> Option<String> {
  let ext = path.extension()?.to_str()?.to_lowercase();
  if !exts.contains(&ext) {
    return None;
  }
  path.file_stem()?.to_str().map(str::to_string)
}

#[cfg(windows)]
fn path_commands() -> Vec<String> {
  let exts: Vec<String> = std::env::var("PATHEXT")
    .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into())
    .split(';')
    .map(|e| e.trim_start_matches('.').to_lowercase())
    .filter(|e| !e.is_empty())
    .collect();
  path_dirs()
    .iter()
    .flat_map(|d| fs::read_dir(d).into_iter().flatten().flatten())
    .filter_map(|e| executable_stem(&e.path(), &exts))
    .collect()
}

#[cfg(not(windows))]
fn path_commands() -> Vec<String> {
  use std::os::unix::fs::PermissionsExt;
  path_dirs()
    .iter()
    .flat_map(|d| fs::read_dir(d).into_iter().flatten().flatten())
    .filter(|e| {
      fs::metadata(e.path()).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    })
    .filter_map(|e| e.file_name().into_string().ok())
    .collect()
}

fn path_dirs() -> Vec<PathBuf> {
  std::env::var_os("PATH")
    .map(|p| std::env::split_paths(&p).collect())
    .unwrap_or_default()
}

fn history_commands<'a>(history: &[&'a str]) -> impl Iterator<Item = &'a str> {
  history.iter().filter_map(|h| h.split_whitespace().next())
}

fn needs_quote(text: &str) -> bool {
  text
    .chars()
    .any(|c| c.is_whitespace() || NEEDS_QUOTE.contains(&c))
}

fn render(body: &str, quote: Option<char>, closed: bool) -> String {
  let quote = quote.or_else(|| needs_quote(body).then_some('"'));
  match (quote, closed) {
    (Some(q), true) => format!("{q}{body}{q}"),
    (Some(q), false) => format!("{q}{body}"),
    (None, _) => body.to_string(),
  }
}

fn common_prefix(names: &[&str]) -> String {
  let Some(first) = names.first() else {
    return String::new();
  };
  let mut end = first.len();
  for name in &names[1..] {
    let shared = first
      .char_indices()
      .zip(name.chars())
      .take_while(|((_, a), b)| {
        if IGNORE_CASE {
          a.to_lowercase().eq(b.to_lowercase())
        } else {
          a == b
        }
      })
      .last()
      .map_or(0, |((i, a), _)| i + a.len_utf8());
    end = end.min(shared);
  }
  first[..end].to_string()
}

fn gather(word: &Word, target: &Target, ctx: &Context) -> Vec<Candidate> {
  let mut found: Vec<Candidate> = if word.command && target.dir_text.is_empty() {
    if target.prefix.is_empty() {
      return Vec::new();
    }
    history_commands(&ctx.history)
      .map(str::to_string)
      .chain(path_commands())
      .map(|name| Candidate { name, dir: false })
      .collect()
  } else {
    list_dir(&resolve_dir(&target.dir_text, ctx))
  };
  let hidden_ok = target.prefix.starts_with('.');
  found.retain(|c| starts_with(&c.name, &target.prefix) && (hidden_ok || !c.name.starts_with('.')));
  found.sort_by(|a, b| compare_names(&a.name, &b.name));
  found.dedup_by(|a, b| same_name(&a.name, &b.name));
  found
}

fn full_body(target: &Target, c: &Candidate) -> String {
  let mut body = format!("{}{}", target.dir_text, c.name);
  if c.dir {
    body.push(target.sep);
  }
  body
}

pub fn complete(line: &str, ctx: &Context) -> Option<(String, Option<Completion>)> {
  let word = current_word(line);
  let target = parse_target(word.raw, ctx.posix);
  let found = gather(&word, &target, ctx);
  let head = &line[..word.start];
  match found.as_slice() {
    [] => None,
    [only] => {
      let mut text = format!(
        "{head}{}",
        render(&full_body(&target, only), target.quote, true)
      );
      if !only.dir {
        text.push(' ');
      }
      Some((text, None))
    }
    many => {
      let options: Vec<String> = many
        .iter()
        .map(|c| render(&full_body(&target, c), target.quote, true))
        .collect();
      let names: Vec<&str> = many.iter().map(|c| c.name.as_str()).collect();
      let shared = common_prefix(&names);
      let mut cycle = Completion {
        head: head.to_string(),
        options,
        index: None,
      };
      if shared.chars().count() > target.prefix.chars().count() {
        let body = format!("{}{}", target.dir_text, shared);
        Some((
          format!("{head}{}", render(&body, target.quote, false)),
          Some(cycle),
        ))
      } else {
        let text = cycle.step(1);
        Some((text, Some(cycle)))
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  struct TempTree(PathBuf);

  impl TempTree {
    fn new(name: &str, files: &[&str], dirs: &[&str]) -> Self {
      let root =
        std::env::temp_dir().join(format!("nova-completion-{name}-{}", std::process::id()));
      let _ = fs::remove_dir_all(&root);
      fs::create_dir_all(&root).unwrap();
      for d in dirs {
        fs::create_dir_all(root.join(d)).unwrap();
      }
      for f in files {
        fs::write(root.join(f), b"").unwrap();
      }
      Self(root)
    }

    fn ctx(&self) -> Context<'_> {
      Context {
        cwd: &self.0,
        home: Some(self.0.clone()),
        posix: true,
        history: Vec::new(),
      }
    }
  }

  impl Drop for TempTree {
    fn drop(&mut self) {
      let _ = fs::remove_dir_all(&self.0);
    }
  }

  #[test]
  fn word_starts_after_last_unquoted_space() {
    let w = current_word("cat \"my fi");
    assert_eq!(w.raw, "\"my fi");
    assert!(!w.command);
    assert!(current_word("gi").command);
    assert!(current_word("ls | gr").command);
    assert_eq!(current_word("ls ").raw, "");
  }

  #[test]
  fn target_splits_dir_and_prefix() {
    let t = parse_target("src/ui/co", true);
    assert_eq!(t.dir_text, "src/ui/");
    assert_eq!(t.prefix, "co");
    assert_eq!(t.sep, '/');
    let q = parse_target("'My Dir/x'", true);
    assert_eq!(q.quote, Some('\''));
    assert_eq!(q.dir_text, "My Dir/");
    assert_eq!(q.prefix, "x");
  }

  #[test]
  fn unique_file_gets_trailing_space_and_dir_gets_separator() {
    let t = TempTree::new("unique", &["readme.md"], &["src"]);
    let ctx = t.ctx();
    assert_eq!(
      complete("cat rea", &ctx),
      Some(("cat readme.md ".into(), None))
    );
    assert_eq!(complete("cd s", &ctx), Some(("cd src/".into(), None)));
  }

  #[test]
  fn shared_prefix_first_then_cycles() {
    let t = TempTree::new("cycle", &["notes-a.txt", "notes-b.txt"], &[]);
    let ctx = t.ctx();
    let (text, cycle) = complete("cat n", &ctx).unwrap();
    assert_eq!(text, "cat notes-");
    let mut cycle = cycle.unwrap();
    assert_eq!(cycle.step(1), "cat notes-a.txt");
    assert_eq!(cycle.step(1), "cat notes-b.txt");
    assert_eq!(cycle.step(1), "cat notes-a.txt");
    assert_eq!(cycle.step(-1), "cat notes-b.txt");
  }

  #[test]
  fn names_with_spaces_are_quoted() {
    let t = TempTree::new("quote", &[], &["My Dir"]);
    let ctx = t.ctx();
    assert_eq!(
      complete("cd My", &ctx),
      Some(("cd \"My Dir/\"".into(), None))
    );
  }

  #[test]
  fn hidden_entries_need_a_dot_prefix() {
    let t = TempTree::new("hidden", &[".env", "env.sh"], &[]);
    let ctx = t.ctx();
    assert_eq!(complete("cat e", &ctx), Some(("cat env.sh ".into(), None)));
    assert_eq!(complete("cat .e", &ctx), Some(("cat .env ".into(), None)));
  }

  #[test]
  fn tilde_resolves_to_home() {
    let t = TempTree::new("tilde", &["profile"], &[]);
    let ctx = t.ctx();
    assert_eq!(
      complete("cat ~/pro", &ctx),
      Some(("cat ~/profile ".into(), None))
    );
  }

  #[test]
  fn command_position_uses_history() {
    let t = TempTree::new("history", &[], &[]);
    let mut ctx = t.ctx();
    ctx.history = vec!["zzqnova-tool --flag"];
    assert_eq!(
      complete("zzqnova-t", &ctx),
      Some(("zzqnova-tool ".into(), None))
    );
    assert_eq!(complete("", &ctx), None);
  }

  #[test]
  fn msys_paths_map_to_drives() {
    assert_eq!(
      msys_drive_path("/c/Users/"),
      Some(PathBuf::from("C:\\Users/"))
    );
    assert_eq!(msys_drive_path("/usr/"), None);
  }

  #[test]
  fn common_prefix_respects_char_boundaries() {
    assert_eq!(common_prefix(&["ação1", "ação2"]), "ação");
    assert_eq!(common_prefix(&["a", "b"]), "");
  }
}
