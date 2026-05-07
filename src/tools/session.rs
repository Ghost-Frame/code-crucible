use crate::db::Database;
use crate::json_io::Output;
use crate::tools::{ToolError, ToolResult};
use chrono::Utc;
use serde::Deserialize;
use std::process::Command;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CheckpointInput {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub fn checkpoint(db: &Database, input: CheckpointInput) -> ToolResult {
    let name = input
        .name
        .ok_or_else(|| ToolError::MissingField("name".into()))?;

    let id = format!("ckpt_{}", &Uuid::new_v4().to_string()[..8]);
    let now = Utc::now().timestamp();

    // Get current git HEAD
    let git_ref = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    db.conn()
        .execute(
            r#"
            INSERT OR REPLACE INTO checkpoints (id, name, created_at, git_ref, description)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            rusqlite::params![id, name, now, git_ref, input.description],
        )
        .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

    Ok(Output::ok_with_id(
        id,
        format!("Checkpoint '{}' created", name),
    ))
}

#[derive(Deserialize)]
pub struct RollbackInput {
    pub checkpoint_name: Option<String>,
}

pub fn rollback(db: &Database, input: RollbackInput) -> ToolResult {
    let name = input
        .checkpoint_name
        .ok_or_else(|| ToolError::MissingField("checkpoint_name".into()))?;

    let git_ref: Option<String> = db
        .conn()
        .query_row(
            "SELECT git_ref FROM checkpoints WHERE name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )
        .map_err(|_| ToolError::InvalidValue(format!("Checkpoint not found: {}", name)))?;

    if let Some(ref git_hash) = git_ref {
        let status = Command::new("git")
            .args(["checkout", git_hash])
            .status()
            .map_err(|e| ToolError::IoError(e.to_string()))?;

        if !status.success() {
            return Err(ToolError::IoError("git checkout failed".into()));
        }
    }

    Ok(Output::ok(format!("Rolled back to checkpoint '{}'", name)))
}

#[derive(Deserialize)]
pub struct SessionLearnInput {
    pub discovery: Option<String>,
    pub context: Option<String>,
    pub tags: Option<Vec<String>>,
    #[cfg(feature = "skill-backend")]
    pub capture_as_skill: Option<bool>,
    pub spec_id: Option<String>,
}

pub fn session_learn(db: &Database, input: SessionLearnInput) -> ToolResult {
    #[cfg(feature = "skill-backend")]
    let capture_as_skill = input.capture_as_skill;
    let spec_id = input.spec_id;

    let discovery = input
        .discovery
        .ok_or_else(|| ToolError::MissingField("discovery".into()))?;

    let id = format!("learn_{}", &Uuid::new_v4().to_string()[..8]);
    let now = Utc::now().timestamp();

    db.conn()
        .execute(
            r#"
            INSERT INTO session_learns (id, created_at, discovery, context, tags, spec_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            rusqlite::params![
                id,
                now,
                discovery,
                input.context,
                input.tags.map(|t| serde_json::to_string(&t).unwrap()),
                spec_id,
            ],
        )
        .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

    #[cfg(feature = "skill-backend")]
    let skill_info = {
        let mut info = None;
        if capture_as_skill.unwrap_or(false) {
            if let Ok(client) = crate::skill_backend::SkillBackend::new() {
                match client.capture_skill(&discovery, Some("code-crucible")) {
                    Ok(v) => {
                        info = Some(v);
                    }
                    Err(e) => {
                        eprintln!("warning: skill capture failed: {}", e);
                    }
                }
            }
        }
        info
    };
    #[cfg(not(feature = "skill-backend"))]
    let skill_info: Option<serde_json::Value> = None;

    let mut output = Output::ok_with_id(id, "Learning recorded");
    if let Some(info) = skill_info {
        output.data = Some(serde_json::json!({
            "skill_captured": true,
            "skill": info,
        }));
    }
    Ok(output)
}

#[derive(Deserialize)]
pub struct SessionRecallInput {
    pub query: Option<String>,
    pub limit: Option<usize>,
}

pub fn session_recall(db: &Database, input: SessionRecallInput) -> ToolResult {
    let query = input.query.unwrap_or_default();
    let limit = input.limit.unwrap_or(10);

    let mut stmt = db
        .conn()
        .prepare(
            r#"
            SELECT id, discovery, context, tags
            FROM session_learns
            WHERE discovery LIKE ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )
        .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

    let pattern = format!("%{}%", query);
    let rows = stmt
        .query_map(rusqlite::params![pattern, limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "discovery": row.get::<_, String>(1)?,
                "context": row.get::<_, Option<String>>(2)?,
                "tags": row.get::<_, Option<String>>(3)?,
            }))
        })
        .map_err(|e| ToolError::DatabaseError(e.to_string()))?;

    let results: Vec<_> = rows.filter_map(|r| r.ok()).collect();

    let mut output = Output::ok(format!("Found {} learnings", results.len()));
    output.data = Some(serde_json::json!({ "results": results }));
    Ok(output)
}
