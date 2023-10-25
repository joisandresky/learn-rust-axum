#![allow(unused)]

pub use self::error::{Error, Result};

use std::net::SocketAddr;

use axum::{Router, response::{Html, IntoResponse, Response}, routing::get, extract::{Query, Path}, http::{Uri, StatusCode, Method}, middleware, Json};
use ctx::Ctx;
use serde::Deserialize;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use uuid::Uuid;

mod ctx;
mod error;
mod log;
mod web;
mod model;

use crate::{model::ModelController, log::log_request};

#[tokio::main]
async fn main() -> Result<()>{
    let mc = ModelController::new().await?;

    let routes_apis = web::routes_tickets::routes(mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let routes = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api/v1", routes_apis)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(), 
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback(not_found_handler);

    let addr = SocketAddr::from(([127,0,0,1], 8080));
    println!("->> LISTENING on {addr}\n");

    axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .await
        .expect("failed to kickstart the server!");

    Ok(())
}

async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    // -- Get the eventual response error.
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // -- if client error, build the new response.
    let error_response = client_status_error
        .as_ref()
        .map((|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });

            println!("    ->> client_error_body: {client_error_body}");

            // Build the new response from the client_error_body
            (*status_code, Json(client_error_body)).into_response()
        }));

    // Build and log the server log line.
    let client_error = client_status_error.unzip().1;
    log_request(uuid, req_method, uri, ctx, service_error, client_error).await;
    
    println!();
    error_response.unwrap_or(res)
}

fn routes_hello() -> Router {
    Router::new()
        .route("/hello",get(handler_hello))
        .route("/hello2/:name", get(handler_hellow2))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("->> {:12} - handler_hello - {params:?}", "HANDLER");

    let name = params.name.as_deref().unwrap_or("World!!!");
    Html(format!("Hello <strong>{name}</strong>"))
}

async fn handler_hellow2(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:12} - handler_hello - {name:?}", "HANDLER");

    Html(format!("Hello <strong>{name}</strong>"))
}

async fn not_found_handler(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {}", uri))
}