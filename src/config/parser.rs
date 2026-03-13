use std::{
  collections::HashSet,
  fs,
  path::{Path, PathBuf},
};

use dirs::config_dir;

use crate::config::{Config, ConfigError};

/// Load configuration from CLI path, user config, or system config.
///
/// # Arguments
///
/// * `cli_config_path` - Optional explicit config file path from CLI
///
/// # Returns
///
/// Merged configuration from system and user configurations, or CLI config if
/// specified
///
/// # Errors
///
/// Returns error if config file cannot be read or parsed
pub fn load_config(
  cli_config_path: Option<&Path>,
) -> Result<Config, ConfigError> {
  if let Some(path) = cli_config_path {
    // If CLI config path is provided, use only that file
    return load_config_from_path(path);
  }

  // Load system and user configs
  let system_config = load_system_config();
  let user_config = load_user_config();

  // Start with system config (if available)
  let mut config = system_config.unwrap_or_default();

  // Merge user config over system config
  if let Ok(user_cfg) = user_config {
    merge_configs(&mut config, user_cfg);
  }

  Ok(config)
}

/// Load configuration from a specific path.
///
/// # Errors
///
/// Returns error if file cannot be read or contains invalid TOML
fn load_config_from_path(path: &Path) -> Result<Config, ConfigError> {
  let content = fs::read_to_string(path)?;
  match toml::from_str::<Config>(&content) {
    Ok(config) => Ok(config),
    Err(e) => Err(toml_error(path, &content, e)),
  }
}

/// Create a TOML error with file/line context for better error messages.
///
/// # Arguments
///
/// * `path` - Path to the config file
/// * `content` - Full file content
/// * `original_error` - TOML parsing error
///
/// # Returns
///
/// [`ConfigError`] with line numbers and additiona context
fn toml_error(
  path: &Path,
  content: &str,
  original_error: toml::de::Error,
) -> ConfigError {
  // Extract location information from the original error
  if let Some(span) = original_error.span() {
    let lines: Vec<&str> = content.lines().collect();
    let line_num = content[..span.start].lines().count();
    let col_num =
      span.start - content[..span.start].rfind('\n').map_or(0, |n| n + 1);

    // Create context around the error
    let context_start = line_num.saturating_sub(2);
    let context_end = std::cmp::min(line_num + 3, lines.len());

    let mut context_lines = Vec::new();
    for (i, line) in lines[context_start..context_end].iter().enumerate() {
      let actual_line_num = context_start + i + 1;
      if actual_line_num == line_num + 1 {
        let prefix = format!("  > {:3}:{}  ", actual_line_num, col_num + 1);
        context_lines.push(format!("{}{}", prefix, line));
        // Add arrow pointing to error column if reasonable column position
        if col_num < 80 {
          let prefix_len = prefix.chars().count();
          context_lines.push(format!("{}^", " ".repeat(prefix_len)));
        }
      } else {
        context_lines.push(format!("    {:3}:    {}", actual_line_num, line));
      }
    }

    // Return error w/ context
    return ConfigError::ParseWithContext {
      file:             path.to_path_buf(),
      line:             line_num + 1,
      column:           col_num + 1,
      context:          context_lines,
      original_message: original_error.message().to_string(),
    };
  }

  // Fall back to original error if no span info
  ConfigError::Parse(original_error)
}

/// Load system configuration from /etc/tuigreet/config.toml.
fn load_system_config() -> Result<Config, ConfigError> {
  let path = PathBuf::from("/etc/tuigreet/config.toml");
  if path.exists() {
    load_config_from_path(&path)
  } else {
    Ok(Config::default())
  }
}

/// Load user configuration from XDG config directory
/// (`~/.config/tuigreet/config.toml`).
fn load_user_config() -> Result<Config, ConfigError> {
  if let Some(config_dir) = config_dir() {
    let path = config_dir.join("tuigreet").join("config.toml");
    if path.exists() {
      return load_config_from_path(&path);
    }
  }
  Ok(Config::default())
}

/// Merge source config into destination, preserving non-default values from
/// source.
///
/// Only overwrites destination fields if source value differs from default.
pub fn merge_configs(dest: &mut Config, src: Config) {
  let defaults = Config::default();

  // General config
  if src.general.debug != defaults.general.debug {
    dest.general.debug = src.general.debug;
  }
  if src.general.log_file != defaults.general.log_file {
    dest.general.log_file = src.general.log_file;
  }

  // Session config
  if src.session.command != defaults.session.command {
    dest.session.command = src.session.command;
  }
  if src.session.sessions_dirs != defaults.session.sessions_dirs {
    dest.session.sessions_dirs = src.session.sessions_dirs;
  }
  if src.session.xsessions_dirs != defaults.session.xsessions_dirs {
    dest.session.xsessions_dirs = src.session.xsessions_dirs;
  }
  if src.session.session_wrapper != defaults.session.session_wrapper {
    dest.session.session_wrapper = src.session.session_wrapper;
  }
  if src.session.xsession_wrapper != defaults.session.xsession_wrapper {
    dest.session.xsession_wrapper = src.session.xsession_wrapper;
  }
  if src.session.environments != defaults.session.environments {
    dest.session.environments = src.session.environments;
  }

  // Display config
  if src.display.show_time != defaults.display.show_time {
    dest.display.show_time = src.display.show_time;
  }
  if src.display.time_format != defaults.display.time_format {
    dest.display.time_format = src.display.time_format;
  }
  if src.display.greeting != defaults.display.greeting {
    dest.display.greeting = src.display.greeting;
  }
  if src.display.show_title != defaults.display.show_title {
    dest.display.show_title = src.display.show_title;
  }
  if src.display.issue != defaults.display.issue {
    dest.display.issue = src.display.issue;
  }
  // AlignGreeting implements PartialEq through derive, so (hopefully) this
  // works correctly
  if src.display.align_greeting != defaults.display.align_greeting {
    dest.display.align_greeting = src.display.align_greeting;
  }

  // Remember config
  if src.remember.username != defaults.remember.username {
    dest.remember.username = src.remember.username;
  }
  if src.remember.session != defaults.remember.session {
    dest.remember.session = src.remember.session;
  }
  if src.remember.user_session != defaults.remember.user_session {
    dest.remember.user_session = src.remember.user_session;
  }

  // User menu config
  if src.user_menu.enabled != defaults.user_menu.enabled {
    dest.user_menu.enabled = src.user_menu.enabled;
  }
  if src.user_menu.min_uid != defaults.user_menu.min_uid {
    dest.user_menu.min_uid = src.user_menu.min_uid;
  }
  if src.user_menu.max_uid != defaults.user_menu.max_uid {
    dest.user_menu.max_uid = src.user_menu.max_uid;
  }

  // Secret config
  if src.secret.mode != defaults.secret.mode {
    dest.secret.mode = src.secret.mode;
  }
  if src.secret.characters != defaults.secret.characters {
    dest.secret.characters = src.secret.characters;
  }

  // Layout config
  if src.layout.width != defaults.layout.width {
    dest.layout.width = src.layout.width;
  }
  if src.layout.window_padding != defaults.layout.window_padding {
    dest.layout.window_padding = src.layout.window_padding;
  }
  if src.layout.container_padding != defaults.layout.container_padding {
    dest.layout.container_padding = src.layout.container_padding;
  }
  if src.layout.prompt_padding != defaults.layout.prompt_padding {
    dest.layout.prompt_padding = src.layout.prompt_padding;
  }

  // Widget config
  if src.layout.widgets.time_position != defaults.layout.widgets.time_position {
    dest.layout.widgets.time_position = src.layout.widgets.time_position;
  }
  if src.layout.widgets.status_position
    != defaults.layout.widgets.status_position
  {
    dest.layout.widgets.status_position = src.layout.widgets.status_position;
  }

  // Power config
  if src.power.use_setsid != defaults.power.use_setsid {
    dest.power.use_setsid = src.power.use_setsid;
  }

  // Keybindings config
  if src.keybindings.command != defaults.keybindings.command {
    dest.keybindings.command = src.keybindings.command;
  }
  if src.keybindings.sessions != defaults.keybindings.sessions {
    dest.keybindings.sessions = src.keybindings.sessions;
  }
  if src.keybindings.power != defaults.keybindings.power {
    dest.keybindings.power = src.keybindings.power;
  }

  // Outputs config. If source defines any outputs, they fully replace the
  // destination list rather than merging element-by-element.
  if !src.outputs.is_empty() {
    dest.outputs = src.outputs;
  }

  // Terminal size override
  if src.terminal.cols.is_some() {
    dest.terminal.cols = src.terminal.cols;
  }
  if src.terminal.rows.is_some() {
    dest.terminal.rows = src.terminal.rows;
  }

  // Theme config
  // We merge individual fields if they're different from defaults
  if src.theme.border != defaults.theme.border {
    dest.theme.border = src.theme.border;
  }
  if src.theme.text != defaults.theme.text {
    dest.theme.text = src.theme.text;
  }
  if src.theme.time != defaults.theme.time {
    dest.theme.time = src.theme.time;
  }
  if src.theme.container != defaults.theme.container {
    dest.theme.container = src.theme.container;
  }
  if src.theme.title != defaults.theme.title {
    dest.theme.title = src.theme.title;
  }
  if src.theme.greet != defaults.theme.greet {
    dest.theme.greet = src.theme.greet;
  }
  if src.theme.prompt != defaults.theme.prompt {
    dest.theme.prompt = src.theme.prompt;
  }
  if src.theme.input != defaults.theme.input {
    dest.theme.input = src.theme.input;
  }
  if src.theme.action != defaults.theme.action {
    dest.theme.action = src.theme.action;
  }
  if src.theme.button != defaults.theme.button {
    dest.theme.button = src.theme.button;
  }
}

impl Config {
  /// Validate the configuration
  pub fn validate(
    &self,
    validate_wrappers: bool,
  ) -> Result<Vec<String>, ConfigError> {
    let mut warnings = Vec::new();

    // Check mutually exclusive options
    if self.display.issue && self.display.greeting.is_some() {
      return Err(ConfigError::MutuallyExclusive(
        "display.issue".to_string(),
        "display.greeting".to_string(),
      ));
    }

    // Check dependencies
    if self.remember.user_session && !self.remember.username {
      return Err(ConfigError::Dependency(
        "remember.user_session requires remember.username".to_string(),
      ));
    }

    // Check UID ranges
    if self.user_menu.min_uid > self.user_menu.max_uid {
      return Err(ConfigError::InvalidRange(
        "user_menu.min_uid must not exceed user_menu.max_uid".to_string(),
      ));
    }

    // Check keybindings are distinct
    let keys = [
      self.keybindings.command,
      self.keybindings.sessions,
      self.keybindings.power,
    ];
    if keys.iter().collect::<HashSet<_>>().len() != keys.len() {
      return Err(ConfigError::DuplicateKeybindings);
    }

    // Check F-key ranges
    for (name, key) in [
      ("command", self.keybindings.command),
      ("sessions", self.keybindings.sessions),
      ("power", self.keybindings.power),
    ] {
      if !(1..=12).contains(&key) {
        return Err(ConfigError::InvalidFKey(name.to_string(), key));
      }
    }

    // Validate time format if provided
    if let Some(ref format) = self.display.time_format
      && chrono::format::StrftimeItems::new(format)
        .any(|item| matches!(item, chrono::format::Item::Error))
    {
      return Err(ConfigError::InvalidTimeFormat);
    }

    // Validate session wrapper executables if requested
    if validate_wrappers {
      if let Some(ref wrapper) = self.session.session_wrapper {
        self.validate_wrapper_command(wrapper)?;
      }
      if let Some(ref wrapper) = self.session.xsession_wrapper {
        self.validate_wrapper_command(wrapper)?;
      }
    }

    // Validate [[outputs]] entries
    {
      let primary_count = self.outputs.iter().filter(|o| o.primary).count();
      if primary_count > 1 {
        return Err(ConfigError::Validation(format!(
          "At most one output may be marked `primary = true`, but \
           {primary_count} are"
        )));
      }

      for output in &self.outputs {
        if output.connector.contains('/') || output.connector.contains("..") {
          return Err(ConfigError::Validation(format!(
            "Output connector name '{}' must not contain path separators",
            output.connector
          )));
        }
        if output.connector.is_empty() {
          return Err(ConfigError::Validation(
            "Output connector name must not be empty".to_string(),
          ));
        }
      }

      // Warn if [[outputs]] is configured but all are disabled
      if !self.outputs.is_empty() && self.outputs.iter().all(|o| !o.enabled) {
        warnings.push(
          "All [[outputs]] entries have `enabled = false`; no output will be \
           used for terminal sizing"
            .to_string(),
        );
      }
    }

    // Validate [terminal].
    // Both cols and rows must be set together
    if let Some(reason) = self.terminal.invalid_reason() {
      return Err(ConfigError::Validation(reason));
    }

    // Add validation warnings for potentially problematic configurations
    self.check_warnings(&mut warnings);

    Ok(warnings)
  }

  /// Check for configuration warnings
  fn check_warnings(&self, warnings: &mut Vec<String>) {
    // Warn about excessively high padding values
    if let Some(padding) = self.layout.window_padding
      && padding > 10
    {
      warnings.push(format!(
        "window_padding is very high ({}), this may cause display issues",
        padding
      ));
    }

    if let Some(padding) = self.layout.container_padding
      && padding > 10
    {
      warnings.push(format!(
        "container_padding is very high ({}), this may cause display issues",
        padding
      ));
    }

    // Warn about very wide width settings
    if self.layout.width > 200 {
      warnings.push(format!(
        "width is very high ({}), this may cause display issues on smaller \
         terminals",
        self.layout.width
      ));
    }

    // Warn if user menu is enabled but UID range might be empty
    if self.user_menu.enabled && self.user_menu.min_uid > 65000 {
      warnings.push(
        "user_menu.min_uid is very high, you may not see any users".to_string(),
      );
    }

    // Warn about potentially conflicting session directories
    let mut all_session_dirs = self.session.sessions_dirs.clone();
    all_session_dirs.extend(self.session.xsessions_dirs.clone());

    if all_session_dirs.len()
      != all_session_dirs
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len()
    {
      warnings.push(
        "Duplicate session directories detected in sessions_dirs and \
         xsessions_dirs"
          .to_string(),
      );
    }

    // Warn about hidden widgets but enabled features
    if matches!(
      self.layout.widgets.time_position,
      crate::config::WidgetPosition::Hidden
    ) && self.display.show_time
    {
      warnings
        .push("time widget is hidden but show_time is enabled".to_string());
    }

    // Warn about potential security issues with power commands
    if let Some(ref cmd) = self.power.shutdown
      && !self.power.use_setsid
      && !cmd.contains("sudo")
      && !cmd.contains("doas")
    {
      warnings.push(
        "shutdown command without setsid or privilege escalation may fail"
          .to_string(),
      );
    }

    if let Some(ref cmd) = self.power.reboot
      && !self.power.use_setsid
      && !cmd.contains("sudo")
      && !cmd.contains("doas")
    {
      warnings.push(
        "reboot command without setsid or privilege escalation may fail"
          .to_string(),
      );
    }

    // Warn about empty session directories
    if self.session.sessions_dirs.is_empty()
      && self.session.xsessions_dirs.is_empty()
    {
      warnings.push(
        "No session directories configured, users may not be able to select \
         sessions"
          .to_string(),
      );
    }

    // Warn about potentially invalid time formats
    if let Some(ref format) = self.display.time_format
      && format.is_empty()
    {
      warnings.push(
        "time_format is empty, this will result in no time display".to_string(),
      );
    }

    // Warn about conflicting remember options
    if self.remember.session && self.remember.user_session {
      // This should be caught as an error above, but just in case
      warnings.push(
        "Both remember.session and remember.user_session are enabled"
          .to_string(),
      );
    }
  }

  /// Validate that a wrapper command exists and is executable
  fn validate_wrapper_command(&self, command: &str) -> Result<(), ConfigError> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
      return Err(ConfigError::WrapperExecutableNotFound(PathBuf::from(
        command,
      )));
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if let Some(executable) = parts.first() {
      // Check if it's an absolute path
      let path = PathBuf::from(executable);
      if path.is_absolute() {
        if !path.exists() || !is_executable(&path) {
          return Err(ConfigError::WrapperExecutableNotFound(path));
        }
      } else {
        // Search in PATH
        if !command_exists(executable) {
          return Err(ConfigError::WrapperExecutableNotFound(PathBuf::from(
            executable,
          )));
        }
      }
    }
    Ok(())
  }
}

/// Check if a file is executable
fn is_executable(path: &Path) -> bool {
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(path) {
      let permissions = metadata.permissions();
      return permissions.mode() & 0o111 != 0;
    }
  }

  #[cfg(not(unix))]
  {
    // On non-Unix systems, just check if file exists
    return path.exists();
  }

  false
}

/// Check if a command exists in PATH
fn command_exists(command: &str) -> bool {
  if let Ok(path) = std::env::var("PATH") {
    for dir in std::env::split_paths(&path) {
      let full_path = dir.join(command);
      if full_path.exists() && is_executable(&full_path) {
        return true;
      }
    }
  }
  false
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_mutual_exclusive_remember_flags() {
    let toml_content = r#"
[remember]
username = true
session = true
user_session = true
"#;

    let config: Config =
      toml::from_str(toml_content).expect("Failed to parse TOML");

    assert!(config.remember.session);
    assert!(config.remember.user_session);

    // Both flags being set is now a warning, not a hard error, so that the
    // rest of the config is still applied. user_session takes behavioral
    // precedence when both are true.
    let result = config.validate(false);
    assert!(
      result.is_ok(),
      "Both remember.session and remember.user_session being true should \
       produce a warning, not an error"
    );

    let warnings = result.unwrap();
    assert!(
      warnings.iter().any(|w| {
        w.contains("remember.session") && w.contains("remember.user_session")
      }),
      "Expected a warning about conflicting remember options, got: {:?}",
      warnings
    );
  }

  #[test]
  fn test_keybindings_distinctness_in_config() {
    let toml_content = r#"
[keybindings]
command = 3
sessions = 3
power = 7
"#;

    let config: Config =
      toml::from_str(toml_content).expect("Failed to parse TOML");
    let validation_result = config.validate(false);

    match validation_result {
      Err(ConfigError::DuplicateKeybindings) => {},
      _ => {
        panic!(
          "Expected DuplicateKeybindings error, got: {:?}",
          validation_result
        );
      },
    }
  }

  #[test]
  fn test_session_config_default_consistency() {
    let default_config = Config::default();

    let partial_toml = r#"
[session]
command = "test"
"#;
    let partial_config: Config =
      toml::from_str(partial_toml).expect("Failed to parse partial TOML");

    assert_eq!(
      default_config.session.sessions_dirs,
      partial_config.session.sessions_dirs,
      "Default and partially deserialized sessions_dirs should match"
    );
  }

  #[test]
  fn test_power_config_default_consistency() {
    let default_config = Config::default();

    let partial_toml = r#"
[power]
shutdown = "poweroff"
"#;
    let partial_config: Config =
      toml::from_str(partial_toml).expect("Failed to parse partial TOML");

    assert_eq!(
      default_config.power.use_setsid, partial_config.power.use_setsid,
      "Default and partially deserialized use_setsid should match"
    );
  }

  #[test]
  fn test_wrapper_validation_empty_string() {
    let empty_wrapper = r#"
[session]
session_wrapper = ""
"#;

    let mut config: Config =
      toml::from_str(empty_wrapper).expect("Failed to parse TOML");

    config.session.xsession_wrapper = None;

    let result = config.validate(true);

    assert!(
      result.is_err(),
      "Empty wrapper command should fail validation"
    );
  }

  // [[outputs]] validation
  #[test]
  fn test_outputs_toml_roundtrip() {
    let toml_content = r#"
[[outputs]]
connector = "DP-1"
primary = true

[[outputs]]
connector = "HDMI-A-1"
enabled = false
"#;
    let config: Config =
      toml::from_str(toml_content).expect("Failed to parse [[outputs]] TOML");

    assert_eq!(config.outputs.len(), 2);
    assert_eq!(config.outputs[0].connector, "DP-1");
    assert!(config.outputs[0].primary);
    assert!(config.outputs[0].enabled); // default = true
    assert_eq!(config.outputs[1].connector, "HDMI-A-1");
    assert!(!config.outputs[1].primary); // default = false
    assert!(!config.outputs[1].enabled);

    // Validation should pass
    assert!(config.validate(false).is_ok());
  }

  #[test]
  fn test_outputs_multiple_primary_is_error() {
    let toml_content = r#"
[[outputs]]
connector = "DP-1"
primary = true

[[outputs]]
connector = "HDMI-A-1"
primary = true
"#;
    let config: Config =
      toml::from_str(toml_content).expect("Failed to parse TOML");
    let result = config.validate(false);
    assert!(
      matches!(result, Err(ConfigError::Validation(_))),
      "Multiple primary outputs should be a Validation error, got: {:?}",
      result
    );
  }

  #[test]
  fn test_outputs_empty_connector_is_error() {
    let toml_content = r#"
[[outputs]]
connector = ""
"#;
    let config: Config =
      toml::from_str(toml_content).expect("Failed to parse TOML");
    let result = config.validate(false);
    assert!(
      matches!(result, Err(ConfigError::Validation(_))),
      "Empty connector name should be a Validation error, got: {:?}",
      result
    );
  }

  #[test]
  fn test_outputs_path_separator_in_connector_is_error() {
    for bad in &["../DP-1", "/sys/class/drm/DP-1", "foo/bar"] {
      let config: Config =
        toml::from_str(&format!("[[outputs]]\nconnector = \"{}\"\n", bad))
          .expect("Failed to parse TOML");
      let result = config.validate(false);
      assert!(
        matches!(result, Err(ConfigError::Validation(_))),
        "Connector '{}' with path separator should be a Validation error, \
         got: {:?}",
        bad,
        result
      );
    }
  }

  #[test]
  fn test_outputs_valid_connector_names() {
    // Typical DRM connector name patterns that must pass
    for good in &[
      "DP-1",
      "HDMI-A-1",
      "DisplayPort-2",
      "eDP-1",
      "VGA-1",
      "DVI-D-1",
    ] {
      let config: Config =
        toml::from_str(&format!("[[outputs]]\nconnector = \"{}\"\n", good))
          .expect("Failed to parse TOML");
      assert!(
        config.validate(false).is_ok(),
        "Connector '{}' should be valid, but validation failed",
        good
      );
    }
  }

  #[test]
  fn test_outputs_all_disabled_is_warning() {
    let toml_content = r#"
[[outputs]]
connector = "DP-1"
enabled = false

[[outputs]]
connector = "HDMI-A-1"
enabled = false
"#;
    let config: Config =
      toml::from_str(toml_content).expect("Failed to parse TOML");
    let result = config.validate(false);
    assert!(
      result.is_ok(),
      "All-disabled outputs should not be an error"
    );
    let warnings = result.unwrap();
    assert!(
      warnings.iter().any(|w| w.contains("enabled = false")),
      "Expected a warning about all outputs being disabled, got: {:?}",
      warnings
    );
  }

  #[test]
  fn test_outputs_single_primary_passes() {
    let toml_content = r#"
[[outputs]]
connector = "DP-1"
primary = true

[[outputs]]
connector = "HDMI-A-1"
"#;
    let config: Config =
      toml::from_str(toml_content).expect("Failed to parse TOML");
    assert!(config.validate(false).is_ok());
  }

  // [terminal] validation
  #[test]
  fn test_terminal_both_set_passes() {
    let toml_content = r#"
[terminal]
cols = 237
rows = 52
"#;
    let config: Config =
      toml::from_str(toml_content).expect("Failed to parse TOML");
    assert_eq!(config.terminal.cols, Some(237));
    assert_eq!(config.terminal.rows, Some(52));
    assert!(config.validate(false).is_ok());
  }

  #[test]
  fn test_terminal_cols_without_rows_is_error() {
    let toml_content = r#"
[terminal]
cols = 237
"#;
    let config: Config =
      toml::from_str(toml_content).expect("Failed to parse TOML");
    let result = config.validate(false);
    assert!(
      matches!(result, Err(ConfigError::Validation(_))),
      "cols without rows should be a Validation error, got: {:?}",
      result
    );
  }

  #[test]
  fn test_terminal_rows_without_cols_is_error() {
    let toml_content = r#"
[terminal]
rows = 52
"#;
    let config: Config =
      toml::from_str(toml_content).expect("Failed to parse TOML");
    let result = config.validate(false);
    assert!(
      matches!(result, Err(ConfigError::Validation(_))),
      "rows without cols should be a Validation error, got: {:?}",
      result
    );
  }

  #[test]
  fn test_terminal_neither_set_passes() {
    let config = Config::default();
    assert!(config.validate(false).is_ok());
  }

  #[test]
  fn test_terminal_zero_cols_is_error() {
    let mut config = Config::default();
    config.terminal.cols = Some(0);
    config.terminal.rows = Some(52);
    let result = config.validate(false);
    assert!(
      matches!(result, Err(ConfigError::Validation(_))),
      "cols = 0 should be a Validation error, got: {:?}",
      result
    );
  }

  #[test]
  fn test_terminal_zero_rows_is_error() {
    let mut config = Config::default();
    config.terminal.cols = Some(237);
    config.terminal.rows = Some(0);
    let result = config.validate(false);
    assert!(
      matches!(result, Err(ConfigError::Validation(_))),
      "rows = 0 should be a Validation error, got: {:?}",
      result
    );
  }

  #[test]
  fn test_wrapper_validation_whitespace_only() {
    let whitespace_wrapper = r#"
[session]
session_wrapper = "   "
"#;

    let mut config: Config =
      toml::from_str(whitespace_wrapper).expect("Failed to parse TOML");

    config.session.xsession_wrapper = None;

    let result = config.validate(true);

    assert!(
      result.is_err(),
      "Whitespace-only wrapper command should fail validation"
    );
  }
}
