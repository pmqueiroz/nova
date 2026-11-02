pub fn til_home(pwd: &str) -> String {
  let home_dir = std::env::var("HOME")
    .or_else(|_| std::env::var("USERPROFILE"))
    .unwrap_or_default();

  if pwd.starts_with(&home_dir) {
    pwd.replacen(&home_dir, "~", 1)
  } else {
    pwd.to_owned()
  }
}

pub fn dir_name(pwd: &str) -> String {
  let path = til_home(pwd);
  path
    .trim_end_matches(['/', '\\'])
    .rsplit(['/', '\\'])
    .next()
    .filter(|name| !name.is_empty())
    .map(str::to_owned)
    .unwrap_or(path)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dir_name_keeps_only_the_last_segment() {
    assert_eq!(dir_name("/usr/local/bin"), "bin");
    assert_eq!(dir_name(r"C:\Users\me\projects\nova\"), "nova");
    assert_eq!(dir_name("/"), "/");
  }
}
