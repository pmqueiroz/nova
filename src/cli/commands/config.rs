use crate::core::config;

pub fn run(args: &[String]) -> i32 {
  match args.first().map(|s| s.as_str()) {
    Some("view") => view(args.get(1..).unwrap_or_default()),
    Some("set") => set(args.get(1..).unwrap_or_default()),
    Some("help") | Some("--help") | Some("-h") => {
      eprint!("{}", usage());
      0
    }
    Some(sub) => {
      eprintln!("error: unknown subcommand '{sub}'\n\n{}", usage());
      2
    }
    None => {
      eprint!("{}", usage());
      0
    }
  }
}

fn usage() -> &'static str {
  "nova config <subcommand>

Subcommands:
  view [key]        Show full config or a specific key
  set <key> <val>   Set a config value

Keys use dot notation, e.g. general.editor, theme.font.size
"
}

fn read_toml() -> Result<(toml::Value, std::path::PathBuf), String> {
  let path = config::config_path().ok_or_else(|| "cannot determine config path".to_string())?;
  let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
  let val: toml::Value = toml::from_str(&content).map_err(|e| e.to_string())?;
  Ok((val, path))
}

fn view(args: &[String]) -> i32 {
  match read_toml() {
    Err(e) => {
      eprintln!("error: {e}");
      1
    }
    Ok((val, _)) => {
      if let Some(key) = args.first() {
        match get_nested(&val, key) {
          Some(v) => {
            println!("{}", format_value(&v));
            0
          }
          None => {
            eprintln!("error: key '{key}' not found");
            1
          }
        }
      } else {
        print!("{}", toml::to_string_pretty(&val).unwrap_or_default());
        0
      }
    }
  }
}

fn set(args: &[String]) -> i32 {
  if args.len() < 2 {
    eprintln!("usage: nova config set <key> <value>");
    return 1;
  }
  let key = &args[0];
  let raw_val = &args[1];

  let (mut val, path) = match read_toml() {
    Ok(t) => t,
    Err(e) => {
      eprintln!("error: {e}");
      return 1;
    }
  };

  let new_val = match get_nested(&val, key) {
    Some(toml::Value::Boolean(_)) => match raw_val.as_str() {
      "true" => toml::Value::Boolean(true),
      "false" => toml::Value::Boolean(false),
      _ => {
        eprintln!("error: expected boolean (true/false) for '{key}'");
        return 1;
      }
    },
    Some(toml::Value::Integer(_)) => match raw_val.parse::<i64>() {
      Ok(n) => toml::Value::Integer(n),
      Err(_) => {
        eprintln!("error: expected integer for '{key}'");
        return 1;
      }
    },
    Some(toml::Value::Float(_)) => match raw_val.parse::<f64>() {
      Ok(n) => toml::Value::Float(n),
      Err(_) => {
        eprintln!("error: expected float for '{key}'");
        return 1;
      }
    },
    None => {
      eprintln!("error: key '{key}' not found");
      return 1;
    }
    _ => toml::Value::String(raw_val.clone()),
  };

  if !set_nested(&mut val, key, new_val) {
    eprintln!("error: cannot set '{key}'");
    return 1;
  }

  match toml::to_string_pretty(&val) {
    Ok(content) => {
      if let Err(e) = std::fs::write(&path, content) {
        eprintln!("error: {e}");
        return 1;
      }
      println!("{key} = {raw_val}");
      0
    }
    Err(e) => {
      eprintln!("error: {e}");
      1
    }
  }
}

fn get_nested(val: &toml::Value, key: &str) -> Option<toml::Value> {
  let (head, rest) = match key.find('.') {
    Some(i) => (&key[..i], Some(&key[i + 1..])),
    None => (key, None),
  };
  let child = val.get(head)?;
  match rest {
    Some(r) => get_nested(child, r),
    None => Some(child.clone()),
  }
}

fn set_nested(val: &mut toml::Value, key: &str, new_val: toml::Value) -> bool {
  let (head, rest) = match key.find('.') {
    Some(i) => (&key[..i], Some(&key[i + 1..])),
    None => (key, None),
  };
  let toml::Value::Table(t) = val else {
    return false;
  };
  match rest {
    Some(r) => match t.get_mut(head) {
      Some(child) => set_nested(child, r, new_val),
      None => false,
    },
    None => {
      t.insert(head.to_string(), new_val);
      true
    }
  }
}

fn format_value(val: &toml::Value) -> String {
  match val {
    toml::Value::String(s) => s.clone(),
    toml::Value::Boolean(b) => b.to_string(),
    toml::Value::Integer(n) => n.to_string(),
    toml::Value::Float(f) => f.to_string(),
    _ => toml::to_string_pretty(val).unwrap_or_default(),
  }
}
