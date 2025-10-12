use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock};
use tokio::time::{sleep, timeout};
use uuid::Uuid;
use chrono::Utc;
use redis::{AsyncCommands};
use serde_json;
use tracing::{info, warn, error, debug};

use crate::cache::RedisCache;
use crate::websocket::{WebSocketManager, WsEvent};
use super::job_types::*;
use super::JobProcessor;

#[derive(Clone)]
pub struct WorkerService {
    cache: RedisCache,
    job_processor: Arc<JobProcessor>,
    config: JobQueueConfig,
    is_running: Arc<RwLock<bool>>,
    ws_manager: Option<WebSocketManager>,
}

impl WorkerService {
    pub fn new(cache: RedisCache, job_processor: JobProcessor, config: JobQueueConfig) -> Self {
        Self {
            cache,
            job_processor: Arc::new(job_processor),
            config,
            is_running: Arc::new(RwLock::new(false)),
            ws_manager: None,
        }
    }

    /// Set WebSocket manager for real-time notifications
    pub fn with_websocket(mut self, ws_manager: WebSocketManager) -> Self {
        self.ws_manager = Some(ws_manager);
        self
    }

    /// Start the worker service
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        info!("Starting background worker service with {} workers", self.config.worker_pool_size);

        // Start worker pool
        let mut handles = Vec::new();
        for worker_id in 0..self.config.worker_pool_size {
            let worker = self.clone();
            let handle = tokio::spawn(async move {
                worker.run_worker(worker_id).await;
            });
            handles.push(handle);
        }

        // Wait for all workers to complete (long-running)
        for handle in handles {
            if let Err(e) = handle.await {
                error!("Worker task failed: {}", e);
            }
        }

        Ok(())
    }

    /// Stop the worker service
    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Worker service stopped");
    }

    /// Check if the service is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Enqueue a new job
    pub async fn enqueue_job(&self, job_type: JobType, priority: JobPriority) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let job_id = Uuid::new_v4();
        let metadata = JobMetadata {
            id: job_id,
            job_type: job_type.clone(),
            status: JobStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            retry_count: 0,
            max_retries: self.config.max_retries,
            error_message: None,
            priority: priority.clone(),
        };

        // Store job metadata with zero-copy optimization
        // Serialize once and cache as Bytes for potential reuse
        let job_key = format!("job:{}", job_id);
        let _job_bytes = self.cache.set_json_bytes_with_ttl(&job_key, &metadata, 86400).await?; // 24 hours TTL

        // Add to priority queue
        let queue_key = format!("queue:{}", self.config.queue_name);
        let priority_score = priority.clone() as u32;
        let mut con = self.cache.get_connection().await?;
        let _: () = con.zadd(&queue_key, job_id.to_string(), priority_score).await?;

        info!("Enqueued job {} with priority {:?}", job_id, priority);
        
        // Send WebSocket notification
        if let Some(ref ws_manager) = self.ws_manager {
            let user_id = match &job_type {
                JobType::SendEmail(p) => Some(p.user_id),
                JobType::SendTaskReminder(p) => Some(p.user_id),
                JobType::GenerateReport(p) => Some(p.user_id),
                _ => None,
            };
            
            let event = WsEvent::JobEnqueued {
                job_id,
                job_type: format!("{:?}", job_type),
                priority: priority.clone(),
                user_id,
            };
            
            if let Some(uid) = user_id {
                ws_manager.send_to_user(&uid, event).await;
            } else {
                ws_manager.broadcast(event).await;
            }
        }
        
        Ok(job_id)
    }

    /// Get job status
    pub async fn get_job_status(&self, job_id: Uuid) -> Result<Option<JobMetadata>, Box<dyn std::error::Error + Send + Sync>> {
        let job_key = format!("job:{}", job_id);
        let job_data: Option<String> = self.cache.get_json(&job_key).await?;
        
        if let Some(data) = job_data {
            let metadata: JobMetadata = serde_json::from_str(&data)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    /// Run a single worker
    async fn run_worker(&self, worker_id: usize) {
        info!("Worker {} started", worker_id);
        
        while self.is_running().await {
            match self.process_next_job().await {
                Ok(processed) => {
                    if processed {
                        debug!("Worker {} processed a job", worker_id);
                    } else {
                        // No jobs available
                        sleep(Duration::from_secs(1)).await;
                    }
                }
                Err(e) => {
                    error!("Worker {} error: {}", worker_id, e);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
        
        info!("Worker {} stopped", worker_id);
    }

    /// Process the next job in the queue
    async fn process_next_job(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let queue_key = format!("queue:{}", self.config.queue_name);
        let mut con = self.cache.get_connection().await?;

        // Get highest priority job
        let result: Vec<(String, u32)> = con.zpopmax(&queue_key, 1).await?;
        
        if let Some((job_id_str, _)) = result.first() {
            let job_id = Uuid::parse_str(&job_id_str)?;
            let job_key = format!("job:{}", job_id);
            
            // Get job metadata
            let job_data: Option<String> = self.cache.get_json(&job_key).await?;
            if let Some(data) = job_data {
                let mut metadata: JobMetadata = serde_json::from_str(&data)?;
                
                // Update status to processing (zero-copy optimization)
                metadata.status = JobStatus::Processing;
                metadata.started_at = Some(Utc::now());
                self.cache.set_json(&job_key, &metadata).await?;

                // Send WebSocket notification for job started
                if let Some(ref ws_manager) = self.ws_manager {
                    ws_manager.broadcast(WsEvent::JobStarted {
                        job_id,
                        job_type: format!("{:?}", metadata.job_type),
                    }).await;
                }

                // Process the job with timeout
                let result = timeout(
                    Duration::from_secs(self.config.job_timeout_secs),
                    self.job_processor.process_job(metadata.job_type.clone())
                ).await;

                match result {
                    Ok(Ok(_)) => {
                        // Completed
                        metadata.status = JobStatus::Completed;
                        metadata.completed_at = Some(Utc::now());
                        info!("Job {} completed successfully", job_id);
                        
                        // Send WebSocket notification for job completed
                        if let Some(ref ws_manager) = self.ws_manager {
                            ws_manager.broadcast(WsEvent::JobCompleted {
                                job_id,
                                job_type: format!("{:?}", metadata.job_type),
                            }).await;
                        }
                    }
                    Ok(Err(e)) => {
                        // Failed
                        metadata.retry_count += 1;
                        metadata.error_message = Some(e.to_string());
                        
                        if metadata.retry_count >= metadata.max_retries {
                            metadata.status = JobStatus::Failed;
                            error!("Job {} failed permanently after {} retries: {}", job_id, metadata.retry_count, e);
                            
                            // Send WebSocket notification for job failed
                            if let Some(ref ws_manager) = self.ws_manager {
                                ws_manager.broadcast(WsEvent::JobFailed {
                                    job_id,
                                    job_type: format!("{:?}", metadata.job_type),
                                    error: e.to_string(),
                                    retry_count: metadata.retry_count,
                                }).await;
                            }
                        } else {
                            metadata.status = JobStatus::Retrying;
                            warn!("Job {} failed, retrying ({}/{})", job_id, metadata.retry_count, metadata.max_retries);
                            
                            // Send WebSocket notification for job retrying
                            if let Some(ref ws_manager) = self.ws_manager {
                                ws_manager.broadcast(WsEvent::JobRetrying {
                                    job_id,
                                    job_type: format!("{:?}", metadata.job_type),
                                    retry_count: metadata.retry_count,
                                    max_retries: metadata.max_retries,
                                }).await;
                            }
                            
                            // Re-queue with delay
                            let delay_secs = self.config.retry_delay_secs * metadata.retry_count as u64;
                            tokio::spawn({
                                let queue_key = queue_key.clone();
                                let job_id = job_id;
                                let priority_score = metadata.priority.clone() as u32;
                                let cache = self.cache.clone();
                                async move {
                                    sleep(Duration::from_secs(delay_secs)).await;
                                    if let Ok(mut con) = cache.get_connection().await {
                                        let _: () = con.zadd(&queue_key, job_id.to_string(), priority_score).await.unwrap_or_default();
                                    }
                                }
                            });
                        }
                    }
                    Err(_) => {
                        // Timed out
                        metadata.retry_count += 1;
                        metadata.error_message = Some("Job timed out".to_string());
                        
                        if metadata.retry_count >= metadata.max_retries {
                            metadata.status = JobStatus::Failed;
                            error!("Job {} timed out permanently", job_id);
                        } else {
                            metadata.status = JobStatus::Retrying;
                            warn!("Job {} timed out, retrying", job_id);
                            
                            // Re-queue with delay
                            let delay_secs = self.config.retry_delay_secs * metadata.retry_count as u64;
                            tokio::spawn({
                                let queue_key = queue_key.clone();
                                let job_id = job_id;
                                let priority_score = metadata.priority.clone() as u32;
                                let cache = self.cache.clone();
                                async move {
                                    sleep(Duration::from_secs(delay_secs)).await;
                                    if let Ok(mut con) = cache.get_connection().await {
                                        let _: () = con.zadd(&queue_key, job_id.to_string(), priority_score).await.unwrap_or_default();
                                    }
                                }
                            });
                        }
                    }
                }

                // Update job metadata (zero-copy optimization)
                self.cache.set_json_with_ttl(&job_key, &metadata, 86400).await?;
                Ok(true)
            } else {
                warn!("Job {} not found in cache", job_id);
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Get queue statistics
    pub async fn get_queue_stats(&self) -> Result<QueueStats, Box<dyn std::error::Error + Send + Sync>> {
        let queue_key = format!("queue:{}", self.config.queue_name);
        let mut con = self.cache.get_connection().await?;
        
        let pending_count: u32 = con.zcard(&queue_key).await?;
        
        Ok(QueueStats {
            pending_jobs: pending_count,
            worker_pool_size: self.config.worker_pool_size,
            is_running: self.is_running().await,
        })
    }
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub pending_jobs: u32,
    pub worker_pool_size: usize,
    pub is_running: bool,
}
