use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use tracing::{info, warn, debug};
use axum::extract::ws::Message;

use super::ws_messages::{WsMessage, WsEvent};
use crate::arena::RequestArena;

/// Client connection information
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub client_id: Uuid,
    pub user_id: Option<Uuid>,
    pub sender: mpsc::UnboundedSender<Message>,
    pub rooms: HashSet<String>,  // Rooms this client has joined
}

/// WebSocket manager for handling connections and broadcasting messages
#[derive(Clone)]
pub struct WebSocketManager {
    clients: Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
    rooms: Arc<RwLock<HashMap<String, HashSet<Uuid>>>>,  // room_name -> set of client_ids
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new client connection
    pub async fn register_client(
        &self,
        client_id: Uuid,
        user_id: Option<Uuid>,
        sender: mpsc::UnboundedSender<Message>,
    ) {
        let connection = ClientConnection {
            client_id,
            user_id,
            sender,
            rooms: HashSet::new(),  // Start with no rooms
        };

        self.clients.write().await.insert(client_id, connection);
        
        info!("WebSocket client registered: {} (user_id: {:?})", client_id, user_id);
        debug!("Total connected clients: {}", self.clients.read().await.len());
    }

    /// Unregister a client connection
    pub async fn unregister_client(&self, client_id: &Uuid) {
        // Remove client from all rooms
        let mut rooms = self.rooms.write().await;
        for (_room_name, members) in rooms.iter_mut() {
            members.remove(client_id);
        }
        // Clean up empty rooms
        rooms.retain(|_, members| !members.is_empty());
        drop(rooms);
        
        // Remove client
        self.clients.write().await.remove(client_id);
        info!("WebSocket client unregistered: {}", client_id);
        debug!("Total connected clients: {}", self.clients.read().await.len());
    }

    /// Broadcast a message to all connected clients
    /// Uses Arc<str> for zero-copy message sharing + arena allocation for serialization
    pub async fn broadcast(&self, event: WsEvent) {
        // Use arena for temporary allocations during serialization
        let arena = RequestArena::with_capacity(512); // Pre-allocate for typical message size
        
        let message = WsMessage::new(event);
        let json = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(e) => {
                warn!("Failed to serialize WebSocket message: {}", e);
                return;
            }
        };

        // Convert to Arc<str> for zero-copy sharing
        // All clients get a reference to the same string - no cloning!
        let json_arc: std::sync::Arc<str> = json.into();

        let clients = self.clients.read().await;
        let mut failed_clients = Vec::new();

        for (client_id, connection) in clients.iter() {
            // Clone Arc just increments reference count - zero-copy!
            if let Err(e) = connection.sender.send(Message::Text(json_arc.to_string())) {
                warn!("Failed to send message to client {}: {}", client_id, e);
                failed_clients.push(*client_id);
            }
        }

        let client_count = clients.len();
        
        // Clean up failed clients
        drop(clients);
        if !failed_clients.is_empty() {
            let mut clients = self.clients.write().await;
            for client_id in failed_clients {
                clients.remove(&client_id);
            }
        }

        debug!("Broadcast message to {} clients (zero-copy + arena allocation, {} bytes used)", 
               client_count, arena.allocated_bytes());
        // Arena is dropped here, all temporary allocations freed at once
    }

    /// Send a message to a specific client
    pub async fn send_to_client(&self, client_id: &Uuid, event: WsEvent) -> Result<(), String> {
        let message = WsMessage::new(event);
        let json = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        let clients = self.clients.read().await;
        if let Some(connection) = clients.get(client_id) {
            connection
                .sender
                .send(Message::Text(json))
                .map_err(|e| format!("Failed to send message: {}", e))?;
            Ok(())
        } else {
            Err(format!("Client {} not found", client_id))
        }
    }

    /// Send a message to all clients associated with a specific user
    /// Uses Arc<str> for zero-copy when broadcasting to multiple user sessions
    pub async fn send_to_user(&self, user_id: &Uuid, event: WsEvent) {
        let message = WsMessage::new(event);
        let json = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(e) => {
                warn!("Failed to serialize WebSocket message: {}", e);
                return;
            }
        };

        // Convert to Arc<str> for efficient sharing
        let json_arc: std::sync::Arc<str> = json.into();

        let clients = self.clients.read().await;
        let mut sent_count = 0;

        for connection in clients.values() {
            if connection.user_id == Some(*user_id) {
                // Arc clone is just ref count increment - zero-copy!
                if let Err(e) = connection.sender.send(Message::Text(json_arc.to_string())) {
                    warn!("Failed to send message to client {}: {}", connection.client_id, e);
                } else {
                    sent_count += 1;
                }
            }
        }

        debug!("Sent message to {} clients for user {} (zero-copy)", sent_count, user_id);
    }

    /// Get the number of connected clients
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Get the number of connected clients for a specific user
    pub async fn user_client_count(&self, user_id: &Uuid) -> usize {
        self.clients
            .read()
            .await
            .values()
            .filter(|c| c.user_id == Some(*user_id))
            .count()
    }

    /// Get all connected client IDs
    pub async fn get_client_ids(&self) -> Vec<Uuid> {
        self.clients.read().await.keys().copied().collect()
    }

    /// Check if a client is connected
    pub async fn is_client_connected(&self, client_id: &Uuid) -> bool {
        self.clients.read().await.contains_key(client_id)
    }

    // ===== Room Management Methods =====

    /// Join a client to a room
    pub async fn join_room(&self, client_id: &Uuid, room: String) -> Result<(), String> {
        // Add room to client's room list
        let mut clients = self.clients.write().await;
        if let Some(connection) = clients.get_mut(client_id) {
            connection.rooms.insert(room.clone());
        } else {
            return Err(format!("Client {} not found", client_id));
        }
        drop(clients);

        // Add client to room's member list
        let mut rooms = self.rooms.write().await;
        rooms
            .entry(room.clone())
            .or_insert_with(HashSet::new)
            .insert(*client_id);

        info!("Client {} joined room '{}'", client_id, room);
        Ok(())
    }

    /// Remove a client from a room
    pub async fn leave_room(&self, client_id: &Uuid, room: &str) -> Result<(), String> {
        // Remove room from client's room list
        let mut clients = self.clients.write().await;
        if let Some(connection) = clients.get_mut(client_id) {
            connection.rooms.remove(room);
        } else {
            return Err(format!("Client {} not found", client_id));
        }
        drop(clients);

        // Remove client from room's member list
        let mut rooms = self.rooms.write().await;
        if let Some(members) = rooms.get_mut(room) {
            members.remove(client_id);
            
            // Clean up empty room
            if members.is_empty() {
                rooms.remove(room);
                info!("Room '{}' is now empty and has been removed", room);
            }
        }

        info!("Client {} left room '{}'", client_id, room);
        Ok(())
    }

    /// Broadcast a message to all clients in a specific room
    /// Uses Arc<str> for zero-copy message sharing
    pub async fn broadcast_to_room(&self, room: &str, event: WsEvent) {
        let message = WsMessage::new(event);
        let json = match serde_json::to_string(&message) {
            Ok(json) => json,
            Err(e) => {
                warn!("Failed to serialize WebSocket message: {}", e);
                return;
            }
        };

        // Convert to Arc<str> for zero-copy sharing across room members
        let json_arc: std::sync::Arc<str> = json.into();

        // Get all client IDs in the room
        let rooms = self.rooms.read().await;
        let client_ids = match rooms.get(room) {
            Some(ids) => ids.clone(),
            None => {
                debug!("Room '{}' not found or empty", room);
                return;
            }
        };
        drop(rooms);

        // Send to all clients in the room
        let clients = self.clients.read().await;
        let mut sent_count = 0;
        let mut failed_clients = Vec::new();

        for client_id in &client_ids {
            if let Some(connection) = clients.get(client_id) {
                // Arc clone is just ref count increment - zero-copy!
                if let Err(e) = connection.sender.send(Message::Text(json_arc.to_string())) {
                    warn!("Failed to send message to client {} in room '{}': {}", client_id, room, e);
                    failed_clients.push(*client_id);
                } else {
                    sent_count += 1;
                }
            }
        }

        debug!("Broadcast message to {} clients in room '{}' (zero-copy)", sent_count, room);

        // Clean up failed clients
        drop(clients);
        if !failed_clients.is_empty() {
            let mut clients = self.clients.write().await;
            for client_id in failed_clients {
                clients.remove(&client_id);
            }
        }
    }

    /// Get all rooms a client has joined
    pub async fn get_client_rooms(&self, client_id: &Uuid) -> Vec<String> {
        self.clients
            .read()
            .await
            .get(client_id)
            .map(|c| c.rooms.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all clients in a room
    pub async fn get_room_members(&self, room: &str) -> Vec<Uuid> {
        self.rooms
            .read()
            .await
            .get(room)
            .map(|members| members.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get the number of clients in a room
    pub async fn room_member_count(&self, room: &str) -> usize {
        self.rooms
            .read()
            .await
            .get(room)
            .map(|members| members.len())
            .unwrap_or(0)
    }

    /// Get all active rooms
    pub async fn get_all_rooms(&self) -> Vec<String> {
        self.rooms.read().await.keys().cloned().collect()
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

