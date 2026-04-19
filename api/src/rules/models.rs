use serde::Serialize;

#[derive(Debug, sqlx::FromRow)]
pub struct RulesConfigRecord {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: String,
    pub config: String,
    pub disk_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct RulesConfigInfo {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<RulesConfigRecord> for RulesConfigInfo {
    fn from(r: RulesConfigRecord) -> Self {
        RulesConfigInfo {
            id: r.id,
            name: r.name,
            description: r.description,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RulesConfigDetail {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub config: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl From<RulesConfigRecord> for RulesConfigDetail {
    fn from(r: RulesConfigRecord) -> Self {
        let config: serde_json::Value = serde_json::from_str(&r.config)
            .unwrap_or(serde_json::Value::Object(Default::default()));
        RulesConfigDetail {
            id: r.id,
            name: r.name,
            description: r.description,
            config,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
