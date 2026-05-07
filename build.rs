fn main() {
  println!("cargo:rerun-if-changed=assets/icons/");

  #[cfg(target_os = "windows")]
  embed_windows_icon();
}

#[cfg(target_os = "windows")]
fn embed_windows_icon() {
  let sizes = [16u32, 24, 32, 48, 64, 128, 256];
  let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

  for size in sizes {
    let path = format!("assets/icons/windows/nova-win-{}.png", size);
    let file = std::fs::File::open(&path)
      .unwrap_or_else(|e| panic!("failed to open {path}: {e}"));
    let image = ico::IconImage::read_png(file)
      .unwrap_or_else(|e| panic!("failed to decode {path}: {e}"));
    icon_dir.add_entry(
      ico::IconDirEntry::encode(&image)
        .unwrap_or_else(|e| panic!("failed to encode {path}: {e}")),
    );
  }

  let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
  let ico_path = format!("{out_dir}/nova.ico");
  let file = std::fs::File::create(&ico_path)
    .unwrap_or_else(|e| panic!("failed to create {ico_path}: {e}"));
  icon_dir
    .write(file)
    .unwrap_or_else(|e| panic!("failed to write ico: {e}"));

  let mut res = winres::WindowsResource::new();
  res.set_icon(&ico_path);
  res.compile().unwrap_or_else(|e| panic!("winres compile failed: {e}"));
}
