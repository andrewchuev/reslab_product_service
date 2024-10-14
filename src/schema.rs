use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

// List
#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub order: Option<String>,
    pub order_by: Option<String>,
}



// Create
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProductSchema {
    pub name: String,
    pub description: String,
    pub price: Option<BigDecimal>,
    pub category_id: u64,
    pub code: i32,
    pub stock: i32,
}

// Update
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateProductSchema {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<BigDecimal>,
    pub category_id: u32,
    pub code: i32,
    pub stock: i32,
}