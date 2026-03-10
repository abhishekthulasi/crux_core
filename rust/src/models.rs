use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FactTag {
    pub category: String, // "fact", "preference", "nuance", "unresolved_thread"
    pub content: String,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct Turn {
    pub id: String,
    pub parent_turn_id: Option<String>,
    pub role: String, // "user", "assistant", "system"
    pub content: String,
    pub created_at: String,
}