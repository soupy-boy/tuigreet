pub mod env;
pub mod parser;
mod schema;
pub mod theme;

use std::{fmt, io, path::PathBuf};

pub use schema::*;

/// Errors that can occur during configuration loading or validation
#[derive(Debug)]
pub enum ConfigError {
  /// I/O error when reading config files
  Io(io::Error),

  /// TOML parsing error
  Parse(toml::de::Error),

  /// TOML parsing error with file context
  ParseWithContext {
    file:             PathBuf,
    line:             usize,
    column:           usize,
    context:          Vec<String>,
    original_message: String,
  },
  /// General validation error with description
  Validation(String),

  /// Two options that cannot be used together
  MutuallyExclusive(String, String),

  /// Option that depends on another option
  Dependency(String),

  /// Invalid range (e.g., `min_uid` >= `max_uid`)
  InvalidRange(String),

  /// Duplicate keybindings
  DuplicateKeybindings,

  /// Invalid F-key value
  InvalidFKey(String, u8),

  /// Invalid time format string
  InvalidTimeFormat,

  /// Session wrapper executable not found
  WrapperExecutableNotFound(PathBuf),
}

impl fmt::Display for ConfigError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Io(err) => write!(f, "I/O error: {err}"),
      Self::Parse(err) => write!(f, "TOML parse error: {err}"),
      Self::ParseWithContext {
        file,
        line,
        column,
        context,
        original_message,
      } => {
        writeln!(f, "TOML parse error: {original_message}")?;
        writeln!(f, "File: {}", file.display())?;
        writeln!(f, "Line: {line}, Column: {column}")?;
        writeln!(f)?;
        for line in context {
          writeln!(f, "{line}")?;
        }
        Ok(())
      },
      Self::Validation(msg) => write!(f, "Validation error: {msg}"),
      Self::MutuallyExclusive(opt1, opt2) => {
        write!(f, "Options '{opt1}' and '{opt2}' are mutually exclusive")
      },
      Self::Dependency(msg) => write!(f, "Dependency error: {msg}"),
      Self::InvalidRange(msg) => write!(f, "Invalid range: {msg}"),
      Self::DuplicateKeybindings => {
        write!(f, "Duplicate keybindings detected")
      },
      Self::InvalidFKey(name, key) => {
        write!(
          f,
          "Invalid F-key value for '{name}': F{key} (must be F1-F12)"
        )
      },
      Self::InvalidTimeFormat => write!(f, "Invalid time format string"),
      Self::WrapperExecutableNotFound(path) => {
        write!(
          f,
          "Session wrapper executable not found: {}",
          path.display()
        )
      },
    }
  }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
  fn from(err: io::Error) -> Self {
    Self::Io(err)
  }
}

impl From<toml::de::Error> for ConfigError {
  fn from(err: toml::de::Error) -> Self {
    Self::Parse(err)
  }
}
