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
