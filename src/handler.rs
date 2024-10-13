use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use axum::response::Html;
use serde_json::json;

use crate::{
    model::{NoteModel, NoteModelResponse},
    schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema},
    AppState,
};
use crate::model::{ProductModel, ProductModelResponse};
use crate::schema::CreateProductSchema;

pub async fn health_check_handler() -> impl IntoResponse {
    const MESSAGE: &str = "API Services";

    let json_response = serde_json::json!({
        "status": "ok",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn home_page_handler() -> impl IntoResponse {

    let html : String = "<h1>Hello from Axum</h1>".to_string();
    Html(html)
}

pub async fn note_list_handler(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Param
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    // // Query with macro
    // let notes = sqlx::query_as!(
    //     NoteModel,
    //     r#"SELECT * FROM notes ORDER by id LIMIT ? OFFSET ?"#,
    //     limit as i32,
    //     offset as i32
    // )
    // .fetch_all(&data.db)
    // .await
    // .map_err(|e| {
    //     let error_response = serde_json::json!({
    //         "status": "error",
    //         "message": format!("Database error: { }", e),
    //     });
    //     (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    // })?;

    // Query without macro
    let notes =
        sqlx::query_as::<_, NoteModel>(r#"SELECT * FROM notes ORDER by id LIMIT ? OFFSET ?"#)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&data.db)
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "status": "error",
                    "message": format!("Database error: { }", e),
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

    // Response
    let note_responses = notes
        .iter()
        .map(|note| to_note_response(&note))
        .collect::<Vec<NoteModelResponse>>();

    let json_response = serde_json::json!({
        "status": "ok",
        "count": note_responses.len(),
        "notes": note_responses
    });

    Ok(Json(json_response))
}


pub async fn product_list_handler(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Param
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let products =
        sqlx::query_as::<_, ProductModel>(r#"SELECT * FROM products ORDER by id LIMIT ? OFFSET ?"#)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&data.db)
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "status": "error",
                    "message": format!("Database error: { }", e),
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

    // Response
    let product_responses = products
        .iter()
        .map(|product| to_product_response(&product))
        .collect::<Vec<ProductModelResponse>>();

    let json_response = serde_json::json!({
        "status": "ok",
        "count": product_responses.len(),
        "items": product_responses
    });

    Ok(Json(json_response))
}


pub async fn create_note_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Insert
    let id = uuid::Uuid::new_v4().to_string();
    let query_result = sqlx::query(r#"INSERT INTO notes (id, title, content) VALUES (?, ?, ?)"#)
        .bind(&id)
        .bind(&body.title)
        .bind(&body.content)
        .execute(&data.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    // Duplicate err check
    if let Err(err) = query_result {
        if err.contains("Duplicate entry") {
            let error_response = serde_json::json!({
                "status": "error",
                "message": "Note already exists",
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }

        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", err)})),
        ));
    }

    // // Get insereted note by ID with query macro
    // let note = sqlx::query_as!(NoteModel, r#"SELECT * FROM notes WHERE id = ?"#, &id)
    //     .fetch_one(&data.db)
    //     .await
    //     .map_err(|e| {
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             Json(json!({"status": "error","message": format!("{:?}", e)})),
    //         )
    //     })?;

    // Get insereted note by ID without query macro
    let note = sqlx::query_as::<_, NoteModel>(r#"SELECT * FROM notes WHERE id = ?"#)
        .bind(&id)
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    let note_response = serde_json::json!({
            "status": "success",
            "data": serde_json::json!({
                "note": to_note_response(&note)
        })
    });

    Ok(Json(note_response))
}


pub async fn create_product_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateProductSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Insert
    //let id = uuid::Uuid::new_v4().to_string();
    let query_result = sqlx::query(r#"INSERT INTO products (name, description, price, category_id, code, stock, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP(), CURRENT_TIMESTAMP())"#)
        .bind(&body.name)
        .bind(&body.description)
        .bind(&body.price)
        .bind(&body.category_id)
        .bind(&body.code)
        .bind(&body.stock)
        .execute(&data.db)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    // Duplicate err check
    if let Err(err) = query_result {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", err)})),
        ));
    }

    // Get insereted note by ID without query macro

    let id = query_result.unwrap().last_insert_id();

    let product = sqlx::query_as::<_, ProductModel>(r#"SELECT * FROM products WHERE id = ?"#)
        .bind(&id)
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    let product_response = serde_json::json!({
            "status": "success",
            "data": serde_json::json!({
                "note": to_product_response(&product)
        })
    });

    Ok(Json(product_response))
}


pub async fn get_note_handler(
    Path(id): Path<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // // get using query macro
    // let query_result = sqlx::query_as!(
    //     NoteModel,
    //     r#"SELECT * FROM notes WHERE id = ?"#,
    //     &id
    // )
    // .fetch_one(&data.db)
    // .await;

    // get not using query macro
    let query_result = sqlx::query_as::<_, NoteModel>(r#"SELECT * FROM notes WHERE id = ?"#)
        .bind(&id)
        .fetch_one(&data.db)
        .await;

    // check & response
    match query_result {
        Ok(note) => {
            let note_response = serde_json::json!({
                "status": "success",
                "data": serde_json::json!({
                    "note": to_note_response(&note)
                })
            });

            return Ok(Json(note_response));
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Note with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    };
}

pub async fn edit_note_handler(
    Path(id): Path<String>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // // validate note with query macro
    // let query_result = sqlx::query_as!(
    //     NoteModel,
    //     r#"SELECT * FROM notes WHERE id = ?"#,
    //     &id
    // )
    // .fetch_one(&data.db)
    // .await;

    // validate note without query macro
    let query_result = sqlx::query_as::<_, NoteModel>(r#"SELECT * FROM notes WHERE id = ?"#)
        .bind(&id)
        .fetch_one(&data.db)
        .await;

    // fetch the result
    let note = match query_result {
        Ok(note) => note,
        Err(sqlx::Error::RowNotFound) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Note with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("{:?}", e)
                })),
            ));
        }
    };

    // parse data
    let is_published = body.is_published.unwrap_or(note.is_published != 0);
    let i8_is_published = is_published as i8;

    // Update (if empty, use old value)
    let update_result =
        sqlx::query(r#"UPDATE notes SET title = ?, content = ?, is_published = ? WHERE id = ?"#)
            .bind(&body.title.unwrap_or_else(|| note.title))
            .bind(&body.content.unwrap_or_else(|| note.content))
            .bind(i8_is_published)
            .bind(&id)
            .execute(&data.db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": "error",
                        "message": format!("{:?}", e)
                    })),
                )
            })?;

    // if no data affected (or deleted when wanted to update)
    if update_result.rows_affected() == 0 {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Note with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // // get updated data with query macro
    // let updated_note = sqlx::query_as!(
    //     NoteModel,
    //     r#"SELECT * FROM notes WHERE id = ?"#,
    //     &id
    // )
    // .fetch_one(&data.db)
    // .await
    // .map_err(|e| {
    //     (
    //         StatusCode::INTERNAL_SERVER_ERROR,
    //         Json(json!({"status": "error","message": format!("{:?}", e)})),
    //     )
    // })?;

    // get updated data without query macro
    let updated_note = sqlx::query_as::<_, NoteModel>(r#"SELECT * FROM notes WHERE id = ?"#)
        .bind(&id)
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    let note_response = serde_json::json!({
        "status": "success",
        "data": serde_json::json!({
            "note": to_note_response(&updated_note)
        })
    });

    Ok(Json(note_response))
}

pub async fn delete_note_handler(
    Path(id): Path<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // // delete with query macro
    // let query_result = sqlx::query!(r#"DELETE FROM notes WHERE id = ?"#, &id)
    //     .execute(&data.db)
    //     .await
    //     .map_err(|e| {
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             Json(json!({
    //                 "status": "error",
    //                 "message": format!("{:?}", e)
    //             })),
    //         )
    //     })?;

    // delete not using query macro
    let query_result = sqlx::query(r#"DELETE FROM notes WHERE id = ?"#)
        .bind(&id)
        .execute(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    // response
    if query_result.rows_affected() == 0 {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Note with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::OK)
}

// Convert DB Model to Response
fn to_note_response(note: &NoteModel) -> NoteModelResponse {
    NoteModelResponse {
        id: note.id.to_owned(),
        title: note.title.to_owned(),
        content: note.content.to_owned(),
        is_published: note.is_published != 0,
        created_at: note.created_at.unwrap(),
        updated_at: note.updated_at.unwrap(),
    }
}

fn to_product_response(product: &ProductModel) -> ProductModelResponse {
    ProductModelResponse {
        id: product.id.to_owned(),
        category_id: product.category_id.to_owned(),
        name: product.name.to_owned(),
        description: product.description.to_owned(),
        price: product.price.to_owned(),
        code: product.code.to_owned(),
        stock: product.stock.to_owned(),
        image: product.image.to_owned(),
        created_at: product.created_at.unwrap(),
        updated_at: product.updated_at.unwrap(),
    }
}