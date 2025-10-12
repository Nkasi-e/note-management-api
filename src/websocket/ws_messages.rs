use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::workers::JobPriority;

/// WebSocket message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub event: WsEvent,
    pub timestamp: DateTime<Utc>,
}

impl WsMessage {
    pub fn new(event: WsEvent) -> Self {
        Self {
            event,
            timestamp: Utc::now(),
        }
    }
}

/// WebSocket event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsEvent {
    // Connection events
    Connected {
        client_id: Uuid,
        message: String,
    },
    Ping,
    Pong,
    
    // Room events
    JoinedRoom {
        room: String,
        client_id: Uuid,
    },
    LeftRoom {
        room: String,
        client_id: Uuid,
    },
    RoomMessage {
        room: String,
        message: String,
    },
    
    // Client commands (from client to server)
    JoinRoom {
        room: String,
    },
    LeaveRoom {
        room: String,
    },
    
    // Job events
    JobEnqueued {
        job_id: Uuid,
        job_type: String,
        priority: JobPriority,
        user_id: Option<Uuid>,
    },
    JobStarted {
        job_id: Uuid,
        job_type: String,
    },
    JobCompleted {
        job_id: Uuid,
        job_type: String,
    },
    JobFailed {
        job_id: Uuid,
        job_type: String,
        error: String,
        retry_count: u32,
    },
    JobRetrying {
        job_id: Uuid,
        job_type: String,
        retry_count: u32,
        max_retries: u32,
    },
    
    // Task events
    TaskCreated {
        task_id: Uuid,
        title: String,
        user_id: Uuid,
    },
    TaskUpdated {
        task_id: Uuid,
        title: String,
        user_id: Uuid,
    },
    TaskDeleted {
        task_id: Uuid,
        user_id: Uuid,
    },
    
    // User events
    UserRegistered {
        user_id: Uuid,
        email: String,
    },
    
    // System events
    SystemNotification {
        message: String,
        level: NotificationLevel,
    },
    
    // Error events
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WsEventType {
    Job,
    Task,
    User,
    System,
    Connection,
}

impl WsEvent {
    pub fn event_type(&self) -> WsEventType {
        match self {
            WsEvent::JobEnqueued { .. }
            | WsEvent::JobStarted { .. }
            | WsEvent::JobCompleted { .. }
            | WsEvent::JobFailed { .. }
            | WsEvent::JobRetrying { .. } => WsEventType::Job,
            
            WsEvent::TaskCreated { .. }
            | WsEvent::TaskUpdated { .. }
            | WsEvent::TaskDeleted { .. } => WsEventType::Task,
            
            WsEvent::UserRegistered { .. } => WsEventType::User,
            
            WsEvent::SystemNotification { .. } => WsEventType::System,
            
            WsEvent::Connected { .. }
            | WsEvent::Ping
            | WsEvent::Pong
            | WsEvent::JoinedRoom { .. }
            | WsEvent::LeftRoom { .. }
            | WsEvent::RoomMessage { .. }
            | WsEvent::JoinRoom { .. }
            | WsEvent::LeaveRoom { .. }
            | WsEvent::Error { .. } => WsEventType::Connection,
        }
    }
    
    /// Get the user ID associated with this event, if any
    pub fn user_id(&self) -> Option<Uuid> {
        match self {
            WsEvent::JobEnqueued { user_id, .. } => *user_id,
            WsEvent::TaskCreated { user_id, .. }
            | WsEvent::TaskUpdated { user_id, .. }
            | WsEvent::TaskDeleted { user_id, .. } => Some(*user_id),
            WsEvent::UserRegistered { user_id, .. } => Some(*user_id),
            _ => None,
        }
    }
}

