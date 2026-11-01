fn main() {
  println!("cargo:rerun-if-changed=assets/icons/");

  #[cfg(target_os = "windows")]
  embed_windows_icon();
}

#[cfg(target_os = "windows")]
fn embed_windows_icon() {
  let mut res = winres::WindowsResource::new();
  res.set_icon("assets/icons/windows/nova.ico");
  res
    .compile()
    .unwrap_or_else(|e| panic!("winres compile failed: {e}"));
}
