use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tower_http::trace::{
    TraceLayer, 
    DefaultOnRequest, 
    DefaultOnResponse, 
    DefaultMakeSpan
};
use tracing::{info, debug, Level};

pub fn logging_middleware() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}

pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();

    // Log request details
    info!(
        method = %method,
        uri = %uri,
        "Incoming request"
    );

    // Log headers (excluding sensitive ones)
    if !headers.is_empty() {
        debug!("Request headers: {:?}", filter_sensitive_headers(headers));
    }

    // Process request
    let response = next.run(request).await;

    // Log response
    info!(
        status = %response.status(),
        "Request completed"
    );

    response
}

fn filter_sensitive_headers(headers: axum::http::HeaderMap) -> Vec<(String, String)> {
    let sensitive_headers = ["authorization", "cookie", "x-api-key"];
    
    headers
        .iter()
        .filter(|(name, _)| {
            let header_name = name.as_str().to_lowercase();
            !sensitive_headers.contains(&header_name.as_str())
        })
        .map(|(name, value)| {
            (
                name.to_string(),
                value.to_str().unwrap_or("[non-utf8]").to_string()
            )
        })
        .collect()
}