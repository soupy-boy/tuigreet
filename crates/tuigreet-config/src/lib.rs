pub mod config;
pub mod error;
pub mod loader;
pub mod validation;

#[cfg(test)] mod tests;

use crate::{config::Config, error::ConfigError};

/// Successful config load: resolved config plus non-fatal warnings.
pub type ConfigOk = (Config, Vec<ConfigError>);

/// Partial config plus fatal errors and non-fatal warnings.
pub type ConfigErr = Box<(Config, Vec<ConfigError>, Vec<ConfigError>)>;

/// The full result type returned by `load_config*` and `validate`.
pub type LoadResult = Result<ConfigOk, ConfigErr>;
