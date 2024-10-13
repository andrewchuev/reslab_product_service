use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;

// For sqlx
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct ProductModel {
    pub id: u64,
    pub category_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub price: BigDecimal,
    pub code: i64,
    pub stock: i64,
    pub image: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

// For json response
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ProductModelResponse {
    pub id: u64,
    pub category_id: u64,
    pub name: String,
    pub description: Option<String>,
    pub price: BigDecimal,
    pub code: i64,
    pub stock: i64,
    pub image: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}