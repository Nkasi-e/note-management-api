// WebSocket module for real-time updates
pub mod ws_manager;
pub mod ws_messages;
pub mod ws_handler;

// Re-export commonly used types
pub use ws_manager::WebSocketManager;
pub use ws_messages::{WsMessage, WsEvent, WsEventType};
pub use ws_handler::ws_handler;

