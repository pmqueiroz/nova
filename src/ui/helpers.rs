pub fn til_home(pwd: &String) -> String {
  let home_dir = std::env::var("HOME")
    .or_else(|_| std::env::var("USERPROFILE"))
    .unwrap_or_default();

  if pwd.starts_with(&home_dir) {
    pwd.replacen(&home_dir, "~", 1)
  } else {
    pwd.clone()
  }
}

pub fn basename(path: &String) -> String {
  std::path::Path::new(path)
    .file_name()
    .map(|n| n.to_string_lossy().to_string())
    .unwrap_or_else(|| path.clone())
}

pub fn truncate(content: &String, max_length: usize) -> String {
  if content.chars().count() > max_length {
    let cut: String = content.chars().take(max_length - 3).collect();

    return format!("{}...", cut);
  }

  return content.clone();
}
