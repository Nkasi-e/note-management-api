use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
    Extension,
    http::StatusCode,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use uuid::Uuid;
use tracing::{info, warn, debug};
use jsonwebtoken::{decode, DecodingKey, Validation};

use super::ws_manager::WebSocketManager;
use super::ws_messages::{WsEvent, WsMessage};
use crate::middleware::CurrentUser;
use crate::middleware::auth::Claims;
use crate::config::settings::AuthConfig;

#[derive(Debug, Deserialize)]
pub struct WsQueryParams {
    pub token: Option<String>,
}

/// WebSocket handler - upgrades HTTP connection to WebSocket
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsQueryParams>,
    State(auth_config): State<AuthConfig>,
    Extension(ws_manager): Extension<WebSocketManager>,
    user: Option<Extension<CurrentUser>>,
) -> Result<Response, (StatusCode, String)> {
    // Try to get user_id from middleware first (if authenticated via header)
    let mut user_id = user.map(|u| u.id);
    
    // If not authenticated via header, try query parameter token
    if user_id.is_none() {
        if let Some(token) = params.token {
            user_id = verify_token(&token, &auth_config)?;
        }
    }
    
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, ws_manager, user_id)))
}

/// Verify JWT token and extract user_id
fn verify_token(token: &str, auth_config: &AuthConfig) -> Result<Option<Uuid>, (StatusCode, String)> {
    let decoding_key = DecodingKey::from_secret(auth_config.jwt_secret.as_bytes());
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.set_issuer(&[&auth_config.issuer]);
    validation.set_audience(&[&auth_config.audience]);
    
    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            match Uuid::parse_str(&token_data.claims.sub) {
                Ok(user_id) => Ok(Some(user_id)),
                Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid user ID in token".to_string())),
            }
        }
        Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string())),
    }
}

/// Handle an individual WebSocket connection
async fn handle_socket(socket: WebSocket, ws_manager: WebSocketManager, user_id: Option<Uuid>) {
    let client_id = Uuid::new_v4();
    
    info!("New WebSocket connection: {} (user: {:?})", client_id, user_id);
    
    // Split the socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();
    
    // Create a channel for sending messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    
    // Register the client
    ws_manager.register_client(client_id, user_id, tx).await;
    
    // Send connection confirmation
    let connection_event = WsEvent::Connected {
        client_id,
        message: "Connected to Note Task API WebSocket".to_string(),
    };
    
    if let Ok(json) = serde_json::to_string(&WsMessage::new(connection_event)) {
        let _ = sender.send(Message::Text(json)).await;
    }
    
    // Spawn a task to handle outgoing messages
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });
    
    // Spawn a task to handle incoming messages
    let ws_manager_clone = ws_manager.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Err(e) = handle_client_message(msg, &client_id, &ws_manager_clone).await {
                warn!("Error handling client message: {}", e);
            }
        }
    });
    
    // Wait for either task to finish (which means the connection is closed)
    tokio::select! {
        _ = (&mut send_task) => {
            debug!("Send task completed for client {}", client_id);
            recv_task.abort();
        },
        _ = (&mut recv_task) => {
            debug!("Receive task completed for client {}", client_id);
            send_task.abort();
        },
    }
    
    // Unregister the client
    ws_manager.unregister_client(&client_id).await;
    info!("WebSocket connection closed: {}", client_id);
}

/// Handle messages received from a client
async fn handle_client_message(
    msg: Message,
    client_id: &Uuid,
    ws_manager: &WebSocketManager,
) -> Result<(), String> {
    match msg {
        Message::Text(text) => {
            debug!("Received text message from {}: {}", client_id, text);
            
            // Try to parse as WsMessage
            if let Ok(ws_message) = serde_json::from_str::<WsMessage>(&text) {
                match ws_message.event {
                    WsEvent::Ping => {
                        // Respond with pong
                        ws_manager
                            .send_to_client(client_id, WsEvent::Pong)
                            .await
                            .map_err(|e| format!("Failed to send pong: {}", e))?;
                    }
                    WsEvent::JoinRoom { room } => {
                        // Client wants to join a room
                        match ws_manager.join_room(client_id, room.clone()).await {
                            Ok(_) => {
                                // Notify the client they joined
                                ws_manager
                                    .send_to_client(client_id, WsEvent::JoinedRoom {
                                        room: room.clone(),
                                        client_id: *client_id,
                                    })
                                    .await
                                    .ok();
                                
                                // Notify others in the room
                                ws_manager
                                    .broadcast_to_room(&room, WsEvent::RoomMessage {
                                        room: room.clone(),
                                        message: format!("Client {} joined the room", client_id),
                                    })
                                    .await;
                            }
                            Err(e) => {
                                warn!("Failed to join room: {}", e);
                                ws_manager
                                    .send_to_client(client_id, WsEvent::Error {
                                        message: format!("Failed to join room: {}", e),
                                    })
                                    .await
                                    .ok();
                            }
                        }
                    }
                    WsEvent::LeaveRoom { room } => {
                        // Client wants to leave a room
                        match ws_manager.leave_room(client_id, &room).await {
                            Ok(_) => {
                                // Notify the client they left
                                ws_manager
                                    .send_to_client(client_id, WsEvent::LeftRoom {
                                        room: room.clone(),
                                        client_id: *client_id,
                                    })
                                    .await
                                    .ok();
                                
                                // Notify others in the room
                                ws_manager
                                    .broadcast_to_room(&room, WsEvent::RoomMessage {
                                        room: room.clone(),
                                        message: format!("Client {} left the room", client_id),
                                    })
                                    .await;
                            }
                            Err(e) => {
                                warn!("Failed to leave room: {}", e);
                            }
                        }
                    }
                    _ => {
                        debug!("Received event from client {}: {:?}", client_id, ws_message.event);
                    }
                }
            }
        }
        Message::Binary(_) => {
            debug!("Received binary message from {}", client_id);
        }
        Message::Ping(_data) => {
            debug!("Received ping from {}", client_id);
            // Axum automatically handles pong responses
        }
        Message::Pong(_) => {
            debug!("Received pong from {}", client_id);
        }
        Message::Close(_) => {
            info!("Client {} requested close", client_id);
        }
    }
    
    Ok(())
}

/// Health check for WebSocket connections
pub async fn ws_health(
    Extension(ws_manager): Extension<WebSocketManager>,
) -> axum::response::Json<serde_json::Value> {
    let client_count = ws_manager.client_count().await;
    
    axum::response::Json(serde_json::json!({
        "success": true,
        "websocket": {
            "connected_clients": client_count,
            "status": "operational"
        }
    }))
}

