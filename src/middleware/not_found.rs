use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;


pub async fn json_404_middleware(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let res = next.run(req).await;
    if res.status() != StatusCode::NOT_FOUND {
        return res;
    }
    let body = Json(json!({
        "success": false,
        "error": format!("Route {} not found", path),
        "status": StatusCode::NOT_FOUND.as_u16(),
    }));
    (StatusCode::NOT_FOUND, body).into_response()
}


