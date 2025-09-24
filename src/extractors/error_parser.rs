use serde_json::Value;
use std::collections::HashSet;

/// Parse serde deserialization errors to provide user-friendly messages
pub fn parse_serde_error(error: &serde_json::Error) -> String {
    let error_str = error.to_string();
    
    // Common patterns for missing fields
    if error_str.contains("missing field") {
        if let Some(field_name) = extract_field_name(&error_str) {
            return format!("Missing required field: {}", field_name);
        }
        return "Missing required field in request".to_string();
    }
    
    // Invalid type errors
    if error_str.contains("invalid type") {
        if let Some(field_name) = extract_field_name(&error_str) {
            return format!("Invalid data type for field: {}", field_name);
        }
        return "Invalid data type in request".to_string();
    }
    
    // Invalid value errors
    if error_str.contains("invalid value") {
        if let Some(field_name) = extract_field_name(&error_str) {
            return format!("Invalid value for field: {}", field_name);
        }
        return "Invalid value in request".to_string();
    }
    
    // Try to extract field name from any error that mentions a field
    if let Some(field_name) = extract_field_name(&error_str) {
        return format!("Invalid data for field: {}", field_name);
    }
    
    // Fallback to original error message
    error_str
}

/// Extract field name from serde error messages
fn extract_field_name(error_str: &str) -> Option<String> {
    // Look for patterns like `field_name` or "field_name"
    let patterns = ["`", "\""];
    
    for pattern in &patterns {
        if let Some(start) = error_str.find(pattern) {
            let search_start = start + pattern.len();
            if let Some(end) = error_str[search_start..].find(pattern) {
                let field_name = &error_str[search_start..search_start + end];
                if !field_name.is_empty() && field_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Some(field_name.to_string());
                }
            }
        }
    }
    
    None
}

/// Validate JSON structure against expected fields
pub fn validate_json_structure(json_value: &Value, required_fields: &[&str]) -> Option<String> {
    if let Value::Object(obj) = json_value {
        let provided_fields: HashSet<String> = obj.keys().map(|k| k.to_string()).collect();
        let required_fields_set: HashSet<String> = required_fields.iter().map(|s| s.to_string()).collect();
        
        for required_field in required_fields_set {
            if !provided_fields.contains(&required_field) {
                return Some(format!("Missing required field: {}", required_field));
            }
        }
    }
    
    None
}
