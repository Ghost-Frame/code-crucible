pub mod approaches;
pub mod ast;
pub mod hypothesis;
pub mod session;
#[cfg(feature = "skill-backend")]
pub mod skills;
pub mod spec;
pub mod stats;
pub mod think;
pub mod verify;

use crate::json_io::Output;

pub type ToolResult = Result<Output, ToolError>;

#[derive(Debug)]
pub enum ToolError {
    MissingField(String),
    InvalidValue(String),
    DatabaseError(String),
    IoError(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::MissingField(s) => write!(f, "Missing required field: {}", s),
            ToolError::InvalidValue(s) => write!(f, "Invalid value: {}", s),
            ToolError::DatabaseError(s) => write!(f, "Database error: {}", s),
            ToolError::IoError(s) => write!(f, "I/O error: {}", s),
        }
    }
}

impl std::error::Error for ToolError {}

/// Best-effort: write the active gate state for external hook integration.
/// Failures here must not abort the caller.
pub fn set_session_active(id: &str, kind: &str) {
    let Ok(home) = std::env::var("HOME") else {
        return;
    };
    let dir = std::path::PathBuf::from(home)
        .join(".claude")
        .join("session-env");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("code-crucible-active"), format!("{}:{}", id, kind));
}
