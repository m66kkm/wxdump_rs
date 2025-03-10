use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Standard API response format
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Create a new successful API response
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    /// Create a new successful API response with no data
    pub fn success_no_data() -> ApiResponse<()> {
        ApiResponse {
            code: 0,
            message: "success".to_string(),
            data: None,
        }
    }

    /// Create a new error API response
    pub fn error(code: i32, message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            code,
            message: message.into(),
            data: None,
        }
    }
}

/// Pagination parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: usize,
    pub page_size: usize,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

impl PaginationParams {
    /// Get the offset for SQL queries
    pub fn offset(&self) -> usize {
        (self.page - 1) * self.page_size
    }

    /// Get the limit for SQL queries
    pub fn limit(&self) -> usize {
        self.page_size
    }
}

/// Pagination result
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationResult<T> {
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
    pub items: Vec<T>,
}

impl<T> PaginationResult<T> {
    /// Create a new pagination result
    pub fn new(items: Vec<T>, total: usize, params: &PaginationParams) -> Self {
        let total_pages = (total + params.page_size - 1) / params.page_size;

        Self {
            total,
            page: params.page,
            page_size: params.page_size,
            total_pages,
            items,
        }
    }
}

/// Convert a JSON value to a specific type
pub fn json_to<T>(value: Value) -> Result<T, serde_json::Error>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(value)
}

/// Convert a specific type to a JSON value
pub fn to_json<T>(value: &T) -> Result<Value, serde_json::Error>
where
    T: Serialize,
{
    serde_json::to_value(value)
}

/// Convert a HashMap to a JSON object
pub fn hashmap_to_json<K, V>(map: &HashMap<K, V>) -> Result<Value, serde_json::Error>
where
    K: Serialize,
    V: Serialize,
{
    serde_json::to_value(map)
}

/// Convert a JSON object to a HashMap
pub fn json_to_hashmap<K, V>(value: Value) -> Result<HashMap<K, V>, serde_json::Error>
where
    K: for<'de> Deserialize<'de> + std::hash::Hash + Eq,
    V: for<'de> Deserialize<'de>,
{
    serde_json::from_value(value)
}
