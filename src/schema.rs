use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

// List
#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}


// Create
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateNoteSchema {
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_published: Option<bool>,
}

// Update
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateNoteSchema {
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_published: Option<bool>,
}




// Create
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateProductSchema {
    pub name: String,
    pub description: String,
    pub price: Option<BigDecimal>,
    pub category_id: u64,
    pub code: u64,
    pub stock: u64,
}

// Update
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateProductSchema {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<BigDecimal>,
    pub category_id: Option<u64>,
    pub code: Option<u64>,
    pub stock: Option<u64>,
}