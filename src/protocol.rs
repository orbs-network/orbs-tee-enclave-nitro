use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeeRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeeResponse {
    pub id: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub signature: Option<String>,
    pub error: Option<String>,
}

impl TeeResponse {
    pub fn success(id: String, data: serde_json::Value) -> Self {
        Self {
            id,
            success: true,
            data: Some(data),
            signature: None,
            error: None,
        }
    }

    pub fn error(id: String, error: String) -> Self {
        Self {
            id,
            success: false,
            data: None,
            signature: None,
            error: Some(error),
        }
    }
}
