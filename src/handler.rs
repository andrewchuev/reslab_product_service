use std::sync::Arc;

use axum::{extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, Extension, Json};

use crate::model::{ProductModel, ProductModelResponse};
use crate::schema::{CreateProductSchema, UpdateProductSchema};
use crate::{
    schema::FilterOptions,
    AppState,
};
use axum::response::Html;
use bigdecimal::{BigDecimal, ToPrimitive};
use serde_json::json;
use sqlx::MySqlPool;
use tera::{Context, Tera};

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
    // Параметры фильтрации
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;
    let order = opts.order.unwrap_or_else(|| "asc".to_string());
    let order_by = opts.order_by.unwrap_or_else(|| "id".to_string());

    let query = format!(
        "SELECT * FROM products ORDER BY {} {} LIMIT ? OFFSET ?",
        order_by, order,
    );

    let products = sqlx::query_as::<_, ProductModel>(&query)
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: {:?}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let product_responses = products
        .iter()
        .map(|product| to_product_response(product))
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
    // Вставка данных в базу
    let query_result = sqlx::query(
        r#"INSERT INTO products (name, description, price, category_id, code, stock, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP(), CURRENT_TIMESTAMP())"#,
    )
        .bind(&body.name)
        .bind(&body.description)
        .bind(&body.price)
        .bind(&body.category_id)
        .bind(&body.code)
        .bind(&body.stock)
        .execute(&data.db)
        .await;

    let query_result = match query_result {
        Ok(result) => result,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "message": format!("Database insert error: {:?}", err) })),
            ));
        }
    };

    // Получение ID вставленной записи
    let id = query_result.last_insert_id();

    // Получение продукта из базы данных
    let product = sqlx::query_as::<_, ProductModel>(r#"SELECT * FROM products WHERE id = ?"#)
        .bind(&id)
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "message": format!("Failed to fetch product: {:?}", e) })),
            )
        })?;

    // Формирование ответа
    let product_response = serde_json::json!({
        "status": "success",
        "data": {
            "product": to_product_response(&product)
        }
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
        id: product.id,
        category_id: product.category_id,
        name: product.name.clone(),
        description: Option::from(product.description.clone().unwrap_or_default()),
        price: BigDecimal::try_from(product.price.to_f64().unwrap_or_default()).unwrap(),
        code: Option::from(product.code.unwrap_or_default()),
        stock: Option::from(product.stock.unwrap_or_default()),
        image: Option::from(product.image.clone().unwrap_or_default()),
        created_at: product.created_at.unwrap_or_else(|| chrono::Utc::now()),
        updated_at: product.updated_at.unwrap_or_else(|| chrono::Utc::now()),
    }
}
pub async fn render_products_page(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(pool): Extension<MySqlPool>,
) -> Html<String> {
    // Запрос продуктов из базы данных напрямую
    let products = sqlx::query_as!(
        ProductModel,
        "SELECT *  FROM products order by price desc"
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

