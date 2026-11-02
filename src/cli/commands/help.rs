pub fn usage() -> &'static str {
  "nova <command>

Commands:
  ask      Open Ask AI modal (Nova terminal only)
  config   View or set configuration values
  explain  Explain output using AI (Nova terminal only)
  help     Show this help
  update   Check for a newer release (--install to update)

Flags:
  --version, -v   Print version
  --check-update  Same as 'nova update'
"
}
