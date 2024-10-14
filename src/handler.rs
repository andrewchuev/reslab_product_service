use std::sync::Arc;

use axum::{extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, Extension, Json};

use axum::response::Html;
use serde_json::json;
use sqlx::MySqlPool;
use tera::{Context, Tera};
use crate::model::{ProductModel, ProductModelResponse};
use crate::schema::{CreateProductSchema, UpdateProductSchema};
use crate::{
    schema::FilterOptions,
    AppState,
};

pub async fn health_check_handler() -> impl IntoResponse {
    const MESSAGE: &str = "API Product Service";

    let json_response = serde_json::json!({
        "status": "ok",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn product_list_handler(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Param
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;
    let order = opts.order.unwrap_or("asc".to_string());
    let order_by = opts.order_by.unwrap_or("id".to_string());


    let query = format!(
        "SELECT * FROM products ORDER BY {} {} LIMIT ? OFFSET ?",
        order_by,
        order,
    );

    let products =
        sqlx::query_as::<_, ProductModel>(&query)
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


    if let Err(err) = query_result {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", err)})),
        ));
    }


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
                "product": to_product_response(&product)
        })
    });

    Ok(Json(product_response))
}


pub async fn get_product_handler(
    Path(id): Path<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as::<_, ProductModel>(r#"SELECT * FROM products WHERE id = ?"#)
        .bind(&id)
        .fetch_one(&data.db)
        .await;


    match query_result {
        Ok(product) => {
            let product_response = serde_json::json!({
                "status": "success",
                "data": serde_json::json!({
                    "product": to_product_response(&product)
                })
            });

            Ok(Json(product_response))
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Product with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    }
}


pub async fn edit_product_handler(
    Path(id): Path<u32>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateProductSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    println!("id: {}", id);
    let query_result = sqlx::query_as::<_, ProductModel>(r#"SELECT * FROM products WHERE id = ?"#)
        .bind(&id)
        .fetch_one(&data.db)
        .await;


    let product = match query_result {
        Ok(product) => product,
        Err(sqlx::Error::RowNotFound) => {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Product with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message1": format!("{:?}", e)
                })),
            ));
        }
    };


    let update_result =
        sqlx::query(r#"UPDATE products SET name = ?, description = ?, price = ?, category_id = ? WHERE id = ?"#)
            .bind(&body.name.unwrap_or_else(|| product.name))
            .bind(body.description.clone().or(product.description.clone()))
            .bind(&body.price.unwrap_or_else(|| product.price))
            .bind(&body.category_id)
            .bind(&id)
            .execute(&data.db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": "error",
                        "message2": format!("{:?}", e)
                    })),
                )
            })?;


    if update_result.rows_affected() == 0 {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Product with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }


    let updated_product = sqlx::query_as::<_, ProductModel>(r#"SELECT * FROM products WHERE id = ?"#)
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
            "product": to_product_response(&updated_product)
        })
    });

    Ok(Json(product_response))
}


pub async fn delete_product_handler(
    Path(id): Path<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query(r#"DELETE FROM products WHERE id = ?"#)
        .bind(&id)
        .execute(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;


    if query_result.rows_affected() == 0 {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Product with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::OK)
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


pub async fn render_products_page(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(pool): Extension<MySqlPool>,  // Pool для доступа к базе данных
) -> Html<String> {
    // Запрос продуктов из базы данных напрямую
    let products = sqlx::query_as!(
        ProductModel,
        "SELECT *  FROM products"
    )
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch products");

    let mut context = Context::new();
    context.insert("products", &products);

    let rendered = tera.render("products.html", &context)
        .expect("Failed to render template");

    Html(rendered)
}

