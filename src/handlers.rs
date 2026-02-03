use crate::{
    structs::{CreateTask, CreateUser, Task, UpdateTodo},
    tokens::Claims,
};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::SqlitePool;

pub async fn create_user(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .expect("Password hashing failed")
        .to_string();

    let data = sqlx::query("INSERT INTO users (username, email, password) VALUES (?, ?, ?)")
        .bind(&payload.username)
        .bind(&payload.email)
        .bind(&password_hash)
        .execute(&pool)
        .await;
    match data {
        Ok(result) => {
            let user_id = result.last_insert_rowid();
            (
                StatusCode::CREATED,
                Json(serde_json::json!({"msg": "User created", "id": user_id})),
            )
        }
        Err(_) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "Username already exists"})),
        ),
    }
}

pub async fn get_todos(
    Extension(claim): Extension<Claims>,
    State(pool): State<SqlitePool>,
) -> impl IntoResponse {
    let user_id = claim.sub.parse::<i64>().unwrap();
    let data = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE user_id")
        .bind(user_id)
        .fetch_all(&pool)
        .await;

    match data {
        Ok(tasks) => (StatusCode::OK, Json(serde_json::json!({ "tasks": tasks }))),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch tasks"})),
        ),
    }
}

pub async fn post_todos(
    Extension(claims): Extension<Claims>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateTask>,
) -> impl IntoResponse {
    let user_id = claims.sub.parse::<i64>().unwrap();

    let data = sqlx::query(
        "INSERT INTO tasks (name, completed, in_progress, user_id) VALUES (?, false, false)",
    )
    .bind(&payload.name)
    .bind(user_id)
    .execute(&pool)
    .await;

    match data {
        Ok(task) => {
            let task_id = task.last_insert_rowid();
            (
                StatusCode::CREATED,
                Json(serde_json::json!({"msg": "Task created", "id": task_id})),
            )
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create task"})),
        ),
    }
}

pub async fn put_todos(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
    Json(input): Json<UpdateTodo>,
) -> impl IntoResponse {
    let query = "UPDATE tasks SET
        name = COALESCE(?, name),
        completed = COALESCE(?, completed),
        in_progress = COALESCE(?, in_progress)
        WHERE id = ?
        RETURNING id, name, completed, in_progress";

    let data = sqlx::query_as::<_, Task>(query)
        .bind(input.name)
        .bind(input.completed)
        .bind(input.in_progress)
        .bind(id)
        .fetch_optional(&pool)
        .await;

    match data {
        Ok(Some(task)) => (
            StatusCode::OK,
            Json(serde_json::json!({"msg": "Task updated", "task": task})),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"msg": "Task not found"})),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Update failed"})),
        ),
    }
}

pub async fn delete_todos(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> impl IntoResponse {
    let query = "DELETE FROM tasks WHERE id = ?";

    let data = sqlx::query(query).bind(id).execute(&pool).await;

    match data {
        Ok(res) => {
            if res.rows_affected() > 0 {
                (
                    StatusCode::OK,
                    Json(serde_json::json!({"msg": "Task deleted"})),
                )
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"msg": "Task not found"})),
                )
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Delete failed"})),
        ),
    }
}
