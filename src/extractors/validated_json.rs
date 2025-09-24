use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use super::error_parser::parse_serde_error;

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> std::result::Result<Self, Self::Rejection> {
        match Json::<Value>::from_request(req, state).await {
            Ok(Json(json_value)) => {
                // Try to deserialize the JSON value to our target type
                match serde_json::from_value::<T>(json_value) {
                    Ok(value) => Ok(ValidatedJson(value)),
                    Err(err) => {
                        let error_message = parse_serde_error(&err);
                        let body = Json(json!({
                            "success": false,
                            "error": error_message,
                            "status": StatusCode::BAD_REQUEST.as_u16()
                        }));
                        Err((StatusCode::BAD_REQUEST, body).into_response())
                    }
                }
            }
            Err(rejection) => {
                let error_message = match rejection {
                    axum::extract::rejection::JsonRejection::JsonSyntaxError(err) => {
                        format!("Invalid JSON syntax: {}", err)
                    }
                    axum::extract::rejection::JsonRejection::MissingJsonContentType(_) => {
                        "Missing Content-Type: application/json header".to_string()
                    }
                    _ => format!("JSON parsing error: {}", rejection),
                };

                let body = Json(json!({
                    "success": false,
                    "error": error_message,
                    "status": StatusCode::BAD_REQUEST.as_u16()
                }));

                Err((StatusCode::BAD_REQUEST, body).into_response())
            }
        }
    }
}
