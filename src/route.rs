use std::sync::Arc;

use crate::handler::{render_products_page, create_product_handler, delete_product_handler, edit_product_handler, get_product_handler, product_list_handler};
use crate::{
    handler::health_check_handler,
    AppState,
};
use axum::{routing::{get, post}, Extension, Router};
use tera::Tera;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let tera = Tera::new("templates/*").unwrap();
    let shared_tera = Arc::new(tera);

    Router::new()
        .route("/", get(render_products_page)).layer(Extension(app_state.db.to_owned())).layer(Extension(shared_tera))
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
