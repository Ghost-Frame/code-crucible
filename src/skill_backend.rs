use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::env;
use std::time::Duration;

#[derive(Debug)]
#[allow(dead_code)]
pub enum SkillBackendError {
    NotConfigured(String),
    RequestFailed(String),
    InvalidResponse(String),
}

impl std::fmt::Display for SkillBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillBackendError::NotConfigured(s) => write!(f, "Skill backend not configured: {}", s),
            SkillBackendError::RequestFailed(s) => write!(f, "Skill backend request failed: {}", s),
            SkillBackendError::InvalidResponse(s) => {
                write!(f, "Skill backend invalid response: {}", s)
            }
        }
    }
}

impl std::error::Error for SkillBackendError {}

pub struct SkillBackend {
    http: Client,
    base_url: String,
    api_key: Option<String>,
}

impl SkillBackend {
    /// Create a new client. Returns Err if CRUCIBLE_SKILL_URL is not set.
    pub fn new() -> Result<Self, SkillBackendError> {
        let base_url = env::var("CRUCIBLE_SKILL_URL")
            .map_err(|_| SkillBackendError::NotConfigured("CRUCIBLE_SKILL_URL not set".into()))?;
        let base_url = base_url.trim_end_matches('/').to_string();
        let api_key = env::var("CRUCIBLE_SKILL_API_KEY").ok();

        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| SkillBackendError::RequestFailed(e.to_string()))?;

        Ok(Self {
            http,
            base_url,
            api_key,
        })
    }

    fn apply_auth(
        &self,
        req: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        if let Some(key) = &self.api_key {
            req.bearer_auth(key)
        } else {
            req
        }
    }

    fn get(&self, path: &str) -> Result<Value, SkillBackendError> {
        let url = format!("{}{}", self.base_url, path);
        let req = self.http.get(&url);
        let req = self.apply_auth(req);
        let resp = req
            .send()
            .map_err(|e| SkillBackendError::RequestFailed(format!("{}: {}", url, e)))?;
        if !resp.status().is_success() {
            return Err(SkillBackendError::RequestFailed(format!(
                "{}: HTTP {}",
                url,
                resp.status()
            )));
        }
        resp.json::<Value>()
            .map_err(|e| SkillBackendError::InvalidResponse(e.to_string()))
    }

    fn post(&self, path: &str, body: Value) -> Result<Value, SkillBackendError> {
        let url = format!("{}{}", self.base_url, path);
        let req = self.http.post(&url).json(&body);
        let req = self.apply_auth(req);
        let resp = req
            .send()
            .map_err(|e| SkillBackendError::RequestFailed(format!("{}: {}", url, e)))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().unwrap_or_default();
            return Err(SkillBackendError::RequestFailed(format!(
                "{}: HTTP {} -- {}",
                url, status, body_text
            )));
        }
        resp.json::<Value>()
            .map_err(|e| SkillBackendError::InvalidResponse(e.to_string()))
    }

    // --- Skill API methods ---

    pub fn search_skills(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> Result<Value, SkillBackendError> {
        let mut body = json!({ "query": query });
        if let Some(l) = limit {
            body["limit"] = json!(l);
        }
        self.post("/skills/search", body)
    }

    #[allow(dead_code)]
    pub fn get_skill(&self, id: i64) -> Result<Value, SkillBackendError> {
        self.get(&format!("/skills/{}", id))
    }

    pub fn capture_skill(
        &self,
        description: &str,
        agent: Option<&str>,
    ) -> Result<Value, SkillBackendError> {
        let mut body = json!({ "description": description });
        if let Some(a) = agent {
            body["agent"] = json!(a);
        }
        self.post("/skills/capture", body)
    }

    pub fn record_execution(
        &self,
        skill_id: i64,
        success: bool,
        duration_ms: Option<f64>,
        error_type: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<Value, SkillBackendError> {
        let mut body = json!({ "success": success });
        if let Some(d) = duration_ms {
            body["duration_ms"] = json!(d);
        }
        if let Some(et) = error_type {
            body["error_type"] = json!(et);
        }
        if let Some(em) = error_message {
            body["error_message"] = json!(em);
        }
        self.post(&format!("/skills/{}/execute", skill_id), body)
    }

    pub fn fix_skill(&self, skill_id: i64, hint: Option<&str>) -> Result<Value, SkillBackendError> {
        let mut body = json!({});
        if let Some(h) = hint {
            body["hint"] = json!(h);
        }
        self.post(&format!("/skills/{}/fix", skill_id), body)
    }

    pub fn derive_skill(
        &self,
        parent_ids: &[i64],
        direction: &str,
        agent: Option<&str>,
    ) -> Result<Value, SkillBackendError> {
        let mut body = json!({
            "parent_ids": parent_ids,
            "direction": direction,
        });
        if let Some(a) = agent {
            body["agent"] = json!(a);
        }
        self.post("/skills/derive", body)
    }

    pub fn get_lineage(&self, skill_id: i64) -> Result<Value, SkillBackendError> {
        self.get(&format!("/skills/{}/lineage", skill_id))
    }

    #[allow(dead_code)]
    pub fn list_skills(
        &self,
        limit: usize,
        offset: usize,
        agent: Option<&str>,
    ) -> Result<Value, SkillBackendError> {
        let mut path = format!("/skills?limit={}&offset={}", limit, offset);
        if let Some(a) = agent {
            path.push_str(&format!("&agent={}", a));
        }
        self.get(&path)
    }
}
