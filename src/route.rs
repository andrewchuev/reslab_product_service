use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handler::{
        create_note_handler, delete_note_handler, edit_note_handler, get_note_handler,
        health_check_handler, note_list_handler,
    },
    AppState,
};
use crate::handler::{create_product_handler, home_page_handler, product_list_handler};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(home_page_handler))
        .route("/api/healthcheck", get(health_check_handler))
        .route("/api/notes", post(create_note_handler))
        .route("/api/products", post(create_product_handler))
        .route("/api/notes", get(note_list_handler))
        .route("/api/products", get(product_list_handler))
        .route(
            "/api/notes/:id",
            get(get_note_handler)
                .patch(edit_note_handler)
                .delete(delete_note_handler),
        )
        .with_state(app_state)
}
