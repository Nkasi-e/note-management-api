// Extractors module - custom Axum extractors
pub mod error_parser;
pub mod validated_json;

// Re-export commonly used extractors
pub use validated_json::ValidatedJson;
