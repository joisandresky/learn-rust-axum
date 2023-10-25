use axum::{Json, routing::post, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use tower_cookies::{Cookies, Cookie};

use crate::{Error, Result, web};

pub fn routes() -> Router {
    Router::new().route("/api/v1/login", post(api_login))
}

async fn api_login(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    // TODO: Implement Real db/auth logic
    if payload.username != "demo" || payload.password != "welcome" {
        return Err(Error::LoginFail);
    }

    // TODO: Set Cookies
    cookies.add(Cookie::new(web::AUTH_TOKEN, "user-1.exp.sign"));

    // Create success body.
    let body = Json(json!({
        "success": true,
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}