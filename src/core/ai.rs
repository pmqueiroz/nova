use anthropic::client::ClientBuilder;
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};
use openai_api_rust::chat::{ChatApi, ChatBody};
use openai_api_rust::{Auth, Message as OaiMessage, OpenAI, Role as OaiRole};

use crate::core::{config::AiProvider, grid::Grid};

pub struct AiQuery {
  pub question: String,
  pub context: String,
  pub provider: AiProvider,
  pub model: String,
  pub api_key: String,
  pub base_url: Option<String>,
  pub shell: String,
  pub os: String,
}

pub fn extract_last_output(grid: &Grid) -> String {
  let prompt_row = (0..grid.rows)
    .rev()
    .find(|&r| grid.row(r).iter().any(|c| c.c.as_str() == "λ"));
  let start = match prompt_row {
    Some(r) => r + 1,
    None => return String::new(),
  };
  (start..grid.rows)
    .map(|r| {
      grid
        .row(r)
        .iter()
        .flat_map(|c| c.c.chars())
        .collect::<String>()
        .trim_end()
        .to_string()
    })
    .filter(|l| !l.is_empty())
    .collect::<Vec<_>>()
    .join("\n")
}

pub fn split_answer(text: &str) -> (String, Option<String>) {
  let mut body = String::new();
  let mut cmd: Option<String> = None;
  let mut in_code = false;
  for line in text.lines() {
    if line.trim_start().starts_with("```") {
      in_code = !in_code;
      continue;
    }
    if in_code {
      if cmd.is_none() && !line.trim().is_empty() {
        cmd = Some(line.trim().trim_start_matches("$ ").to_string());
      }
      continue;
    }
    if !line.trim().is_empty() {
      if !body.is_empty() {
        body.push(' ');
      }
      body.push_str(line.trim());
    }
  }
  (body.replace("**", ""), cmd)
}

const DIAGNOSE_MAX_LINES: usize = 60;
const DIAGNOSE_MAX_CHARS: usize = 4000;
const NO_MATCH_PROGRAMS: &[&str] = &[
  "grep",
  "egrep",
  "fgrep",
  "rg",
  "ag",
  "diff",
  "cmp",
  "test",
  "[",
  "[[",
  "false",
  "which",
  "where",
  "where.exe",
  "command",
  "type",
  "pgrep",
  "findstr",
];

#[derive(Debug, PartialEq)]
pub struct Diagnosis {
  pub message: String,
  pub command: Option<String>,
}

fn program_of(cmd: &str) -> &str {
  let first = cmd.split_whitespace().next().unwrap_or("");
  first.rsplit(['/', '\\']).next().unwrap_or(first)
}

pub fn should_diagnose(cmd: Option<&str>, code: i32, output: &str) -> bool {
  if code == 0 || output.trim().is_empty() {
    return false;
  }
  let Some(cmd) = cmd.map(str::trim).filter(|c| !c.is_empty()) else {
    return true;
  };
  if cmd.contains('|') || cmd.contains("&&") || cmd.contains("||") {
    return true;
  }
  let program = program_of(cmd).to_ascii_lowercase();
  !(code == 1 && NO_MATCH_PROGRAMS.contains(&program.as_str()))
}

fn output_tail(output: &str) -> String {
  let lines: Vec<&str> = output.lines().collect();
  let tail = lines[lines.len().saturating_sub(DIAGNOSE_MAX_LINES)..].join("\n");
  match tail.char_indices().rev().nth(DIAGNOSE_MAX_CHARS - 1) {
    Some((i, _)) => tail[i..].to_string(),
    None => tail,
  }
}

pub fn diagnose_question(cmd: Option<&str>, code: i32, output: &str, shell: &str) -> String {
  let command = match cmd {
    Some(c) => format!("Command: `{c}`\n"),
    None => String::new(),
  };
  format!(
    "A command just exited non-zero in the user's terminal. An inline hint may be shown under it.\n\
     {command}Exit code: {code}\n\
     Output:\n{output}\n\n\
     Most non-zero exits need no hint. Answer {{\"skip\": true}} when ANY of these hold:\n\
     - the exit is expected or intentional: a search/compare/check that found nothing or found a difference (grep, rg, diff, test, linters, test runners reporting failures), a deliberate `exit N`, a cancelled or interrupted command, a command run on purpose to show an error;\n\
     - the output already states the cause and the fix plainly (usage text, a clear message naming the problem);\n\
     - you cannot point to a specific, concrete cause, or the only thing you could say is what the exit code means.\n\
     Otherwise write the hint:\n\
     - name the concrete cause in this case (e.g. \"`cdd` is a typo for `cd`\"), never a generic meaning of the exit code and never a restatement of the error text;\n\
     - give a corrected command for {shell} that runs as-is, keeping the user's arguments, paths and intent;\n\
     - if there is no command that would fix it, prefer skipping.\n\n\
     Reply with ONLY a JSON object, no markdown, no code fences, either\n\
     {{\"skip\": true}}\n\
     or\n\
     {{\"message\": \"one short sentence, commands and names in backticks\", \"command\": \"corrected command\" or null}}",
    output = output_tail(output),
  )
}

pub fn parse_diagnosis(text: &str) -> Option<Diagnosis> {
  let json = match (text.find('{'), text.rfind('}')) {
    (Some(a), Some(b)) if a < b => serde_json::from_str::<serde_json::Value>(&text[a..=b]).ok(),
    _ => None,
  };
  let (message, command) = match json {
    Some(v) => {
      if v.get("skip").and_then(|s| s.as_bool()) == Some(true) {
        return None;
      }
      let message = v
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or_default()
        .trim()
        .to_string();
      let command = v
        .get("command")
        .or_else(|| v.get("command_to_solve"))
        .and_then(|c| c.as_str())
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty());
      (message, command)
    }
    None => split_answer(text),
  };
  if message.is_empty() {
    return None;
  }
  Some(Diagnosis { message, command })
}

pub async fn query(q: AiQuery) -> Result<String, String> {
  if q.api_key.is_empty() {
    return Err("No API key configured. Add one in Settings → AI.".into());
  }
  match q.provider {
    AiProvider::Anthropic => query_anthropic(q).await,
    AiProvider::OpenAi => query_openai(q).await,
  }
}

async fn query_anthropic(q: AiQuery) -> Result<String, String> {
  let system = build_system_prompt(&q.os, &q.shell);
  let user_content = build_user_message(&q.context, &q.question);

  let request = MessagesRequestBuilder::default()
    .model(q.model.clone())
    .max_tokens(1024usize)
    .system(system)
    .messages(vec![Message {
      role: Role::User,
      content: vec![ContentBlock::Text { text: user_content }],
    }])
    .stream(false)
    .stop_sequences(vec![])
    .build()
    .map_err(|e| e.to_string())?;

  let api_base = q
    .base_url
    .clone()
    .unwrap_or_else(|| "https://api.anthropic.com".to_string());
  let client = ClientBuilder::default()
    .api_key(q.api_key.clone())
    .api_base(api_base)
    .build()
    .map_err(|e| e.to_string())?;

  let response = client.messages(request).await.map_err(|e| e.to_string())?;

  response
    .content
    .into_iter()
    .find_map(|block| {
      if let ContentBlock::Text { text } = block {
        Some(text)
      } else {
        None
      }
    })
    .ok_or_else(|| "No text in response".to_string())
}

async fn query_openai(q: AiQuery) -> Result<String, String> {
  let mut base = q
    .base_url
    .clone()
    .unwrap_or_else(|| "https://api.openai.com/v1/".to_string());
  if !base.ends_with('/') {
    base.push('/');
  }

  tokio::task::spawn_blocking(move || {
    let auth = Auth::new(&q.api_key);
    let client = OpenAI::new(auth, &base);

    let system = build_system_prompt(&q.os, &q.shell);
    let body = ChatBody {
      model: q.model,
      messages: vec![
        OaiMessage {
          role: OaiRole::System,
          content: system,
        },
        OaiMessage {
          role: OaiRole::User,
          content: build_user_message(&q.context, &q.question),
        },
      ],
      max_tokens: Some(1024),
      temperature: None,
      top_p: None,
      n: None,
      stream: Some(false),
      stop: None,
      presence_penalty: None,
      frequency_penalty: None,
      logit_bias: None,
      user: None,
    };

    client
      .chat_completion_create(&body)
      .map_err(|e| e.to_string())
      .and_then(|completion| {
        completion
          .choices
          .into_iter()
          .next()
          .and_then(|choice| choice.message)
          .map(|msg| msg.content)
          .ok_or_else(|| "No content in response".to_string())
      })
  })
  .await
  .map_err(|e| format!("Task error: {}", e))?
}

fn build_system_prompt(os: &str, shell: &str) -> String {
  format!(
    "You are a helpful terminal assistant. OS: {os}. Shell: {shell}.\n\
     Be concise and actionable. Use markdown formatting.\n\
     For ALL commands or code, use fenced code blocks with a language tag:\n\
     ```bash\ncommand here\n```\n\
     Use bash/sh for shell commands, python for Python, etc. \
     Never include commands outside of code blocks."
  )
}

fn build_user_message(context: &str, question: &str) -> String {
  if context.is_empty() {
    question.to_string()
  } else {
    format!("Context:\n{}\n\nQuestion: {}", context, question)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn split_answer_takes_first_command_and_flattens_prose() {
    let (body, cmd) = split_answer(
      "`clsd` is a **typo**.
Try this:
```bash
$ cls
```
```bash
other
```",
    );
    assert_eq!(body, "`clsd` is a typo. Try this:");
    assert_eq!(cmd.as_deref(), Some("cls"));
  }

  #[test]
  fn should_diagnose_skips_expected_failures() {
    assert!(!should_diagnose(Some("grep foo bar.txt"), 1, "x"));
    assert!(!should_diagnose(Some("/usr/bin/diff a b"), 1, "< a\n> b"));
    assert!(!should_diagnose(Some("cargo build"), 101, "  "));
    assert!(should_diagnose(
      Some("grep foo missing.txt"),
      2,
      "No such file"
    ));
    assert!(should_diagnose(
      Some("cat x | grep y"),
      1,
      "cat: x: No such file"
    ));
    assert!(should_diagnose(Some("cdd foo"), 127, "cdd: not found"));
    assert!(should_diagnose(None, 1, "error"));
  }

  #[test]
  fn diagnose_question_includes_command_and_tail() {
    let output = (0..100)
      .map(|i| format!("line{i}"))
      .collect::<Vec<_>>()
      .join("\n");
    let q = diagnose_question(Some("cdd foo"), 127, &output, "pwsh");
    assert!(q.contains("`cdd foo`") && q.contains("127") && q.contains("pwsh"));
    assert!(q.contains("line99") && !q.contains("line10\n"));
  }

  #[test]
  fn parse_diagnosis_handles_skip_json_and_fences() {
    assert_eq!(parse_diagnosis("{\"skip\": true}"), None);
    assert_eq!(
      parse_diagnosis("{\"message\": \"\", \"command\": null}"),
      None
    );
    assert_eq!(
      parse_diagnosis(
        "```json\n{\"message\": \"`cdd` is a typo for `cd`.\", \"command\": \"cd foo\"}\n```"
      ),
      Some(Diagnosis {
        message: "`cdd` is a typo for `cd`.".into(),
        command: Some("cd foo".into()),
      })
    );
    assert_eq!(
      parse_diagnosis("Missing dep.\n```bash\nnpm i\n```"),
      Some(Diagnosis {
        message: "Missing dep.".into(),
        command: Some("npm i".into()),
      })
    );
  }

  #[test]
  fn split_answer_without_code() {
    let (body, cmd) = split_answer("Just text.");
    assert_eq!(body, "Just text.");
    assert!(cmd.is_none());
  }
}
