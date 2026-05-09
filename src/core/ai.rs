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
  let rows = &grid.cells;
  let prompt_row = (0..rows.len())
    .rev()
    .find(|&r| rows[r].iter().any(|c| c.c == 'λ'));
  let start = match prompt_row {
    Some(r) => r + 1,
    None => return String::new(),
  };
  rows[start..]
    .iter()
    .map(|row| {
      row
        .iter()
        .map(|c| c.c)
        .collect::<String>()
        .trim_end()
        .to_string()
    })
    .filter(|l| !l.is_empty())
    .collect::<Vec<_>>()
    .join("\n")
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
