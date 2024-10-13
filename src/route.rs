use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::handler::{create_product_handler, delete_product_handler, edit_product_handler, get_product_handler, home_page_handler, product_list_handler};
use crate::{
    handler::health_check_handler,
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(home_page_handler))
        .route("/api/healthcheck", get(health_check_handler))
        .route("/api/products", post(create_product_handler))
        .route("/api/products", get(product_list_handler))
        .route(
            "/api/products/:id",
            get(get_product_handler)
                .patch(edit_product_handler)
                .delete(delete_product_handler),
        )
        .with_state(app_state)
}
