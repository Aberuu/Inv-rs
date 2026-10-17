//! Command error definitions for the inventor-rs application.

use thiserror::Error;

/// Errors produced during command lookup or execution.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum CommandError {
    #[error("Command '{0}' was not found in the registry.")]
    NotFound(String),

    #[error("Command '{0}' is disabled in the current context: {1}")]
    Disabled(String, String),

    #[allow(dead_code)]
    #[error("Execution failed for command '{0}': {1}")]
    ExecutionFailed(String, String),
}
