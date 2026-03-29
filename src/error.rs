//! Domain error types for LangQuest.

use std::path::PathBuf;
use thiserror::Error;

/// Errors related to configuration file operations.
#[derive(Debug, Error)]
pub enum ConfigError {
  /// Failed to read the config file from disk.
  #[error("failed to read config file {path}: {source}")]
  Read { path: PathBuf, source: std::io::Error },

  /// Failed to parse the config file as TOML.
  #[error("failed to parse config file {path}: {source}")]
  Parse { path: PathBuf, source: toml::de::Error },

  /// Failed to serialize config to TOML.
  #[error("failed to serialize config: {source}")]
  Serialize { source: toml::ser::Error },

  /// Failed to write the config file to disk.
  #[error("failed to write config file {path}: {source}")]
  Write { path: PathBuf, source: std::io::Error },
}

/// Errors related to exercise discovery and parsing.
#[derive(Debug, Error)]
pub enum ExerciseError {
  /// The `02-task.md` file is missing TOML frontmatter.
  #[error("missing TOML frontmatter in {path}")]
  MissingFrontmatter { path: PathBuf },

  /// A required frontmatter field is missing.
  #[error("missing required field '{field}' in {path}")]
  MissingField { field: &'static str, path: PathBuf },

  /// A frontmatter field has an invalid value.
  #[error("invalid value for '{field}' in {path}: {reason}")]
  InvalidField { field: &'static str, path: PathBuf, reason: String },

  /// Failed to parse TOML frontmatter.
  #[error("failed to parse frontmatter in {path}: {source}")]
  FrontmatterParse { path: PathBuf, source: toml::de::Error },

  /// No student source file found in exercise directory.
  #[error("no student source file found in {path}")]
  NoSourceFile { path: PathBuf },

  /// Failed to read an exercise file from disk.
  #[error("failed to read {path}: {source}")]
  FileRead { path: PathBuf, source: std::io::Error },

  /// Failed to parse solution.md frontmatter.
  #[error("failed to parse solution file {path}: {source}")]
  SolutionParse { path: PathBuf, source: toml::de::Error },

  /// Invalid exercise directory structure.
  #[error("invalid exercise structure at {path}: {reason}")]
  #[allow(dead_code)]
  InvalidStructure { path: PathBuf, reason: String },
}

/// Errors related to exercise verification/running.
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum RunnerError {
  /// Required external tool is not available.
  #[error("tool not found: {tool}")]
  ToolNotFound { tool: String },

  /// Command execution failed.
  #[error("failed to execute command: {source}")]
  Execution { source: std::io::Error },

  /// Command timed out.
  #[error("verification timed out after {seconds}s")]
  Timeout { seconds: u64 },

  /// Failed to parse verification output.
  #[error("failed to parse output: {reason}")]
  OutputParse { reason: String },
}
