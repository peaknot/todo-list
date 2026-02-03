use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}
#[derive(Serialize, Deserialize, FromRow, Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTask {
    pub name: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateTodo {
    pub name: Option<String>,
    pub completed: Option<bool>,
    pub in_progress: Option<bool>,
}
#[derive(Serialize, FromRow, Debug, Clone)]
pub struct Task {
    pub id: i64,
    pub name: String,
    pub completed: bool,
    pub in_progress: bool,
    pub user_id: i64,
}
