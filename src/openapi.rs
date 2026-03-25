// ============================================================================
// OpenAPI/Swagger Documentation Configuration
// ============================================================================
// This module configures the OpenAPI specification for our API using utoipa.
// It defines what schemas (data models) are documented and how they appear
// in the Swagger UI.
//
// Key concepts:
// - OpenApi trait: Generates the OpenAPI spec from Rust code
// - ToSchema trait: Makes a struct documentable in OpenAPI
// - Swagger UI: Interactive web interface for exploring the API
// ============================================================================

use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

// Import all the types we want to document in the OpenAPI spec.
// Each of these types has the `ToSchema` derive macro, which tells utoipa
// how to convert them into OpenAPI schema definitions.
use crate::domain::{
    User, CreateUserRequest,
    Task, CreateTaskRequest, TaskWithAttachments,
    FileMetadata, UploadFileResponse, FileStats,
    PaginationParams, TaskFilters, TaskQueryParams,
    PaginationMeta, PaginatedResponse,
};
use crate::domain::error::ApiErrorResponse;
use crate::domain::user::UserRole;
use crate::domain::task::TaskStatus;
use crate::services::auth_service::{RegisterRequest, LoginRequest, TokenResponse};

// ============================================================================
// ApiDoc Struct - The Main OpenAPI Configuration
// ============================================================================
// This struct uses the `OpenApi` derive macro to automatically generate
// the OpenAPI specification. The #[openapi(...)] attribute configures
// what gets included in the spec.
#[derive(OpenApi)]
#[openapi(
    // ========================================================================
    // PATHS - API Endpoints
    // ========================================================================
    // This section lists all the endpoints that have #[utoipa::path] attributes.
    // These will appear in Swagger UI with "Try it out" buttons.
    //
    // To add more endpoints:
    // 1. Add #[utoipa::path] to the handler function
    // 2. Add the path here: crate::handlers::module_name::function_name
    // 3. Rebuild and the endpoint will appear in Swagger UI
    paths(
        // Health check endpoints
        crate::handlers::health_handlers::health,
        crate::handlers::health_handlers::ping,
        
        // Authentication endpoints
        crate::handlers::auth_handlers::register,
        crate::handlers::auth_handlers::login,
        
        // Task endpoints
        crate::handlers::task_handlers::create_task,
        crate::handlers::task_handlers::get_task,
        crate::handlers::task_handlers::get_tasks,
        
        // File endpoints
        crate::handlers::file_handlers::upload_file,
        crate::handlers::file_handlers::list_user_files,
        crate::handlers::file_handlers::get_file_metadata,
        crate::handlers::file_handlers::download_file,
        crate::handlers::file_handlers::delete_file,
        crate::handlers::file_handlers::get_user_stats,
        crate::handlers::file_handlers::generate_presigned_url,
        
        // User endpoints
        crate::handlers::user_handlers::create_user,
        crate::handlers::user_handlers::get_user,
    ),
    
    // ========================================================================
    // COMPONENTS - Data Models/Schemas
    // ========================================================================
    // The components section defines all the data structures (schemas) that
    // are used in the API. Each schema listed here must have the `ToSchema`
    // derive macro on its definition.
    components(
        schemas(
            // ================================================================
            // User-related schemas
            // ================================================================
            User,              // The user entity with id, name, email, role, etc.
            UserRole,          // Enum: User or Admin
            CreateUserRequest, // Request body for creating a new user
            
            // ================================================================
            // Task-related schemas
            // ================================================================
            Task,                  // The task entity with id, title, description, status, etc.
            TaskStatus,            // Enum: Todo, InProgress, Done
            CreateTaskRequest,     // Request body for creating a new task
            TaskWithAttachments,   // Task with attached file IDs
            
            // ================================================================
            // File-related schemas
            // ================================================================
            FileMetadata,       // File information stored in database
            UploadFileResponse, // Response after uploading a file
            FileStats,          // File upload statistics
            
            // ================================================================
            // Pagination and filtering schemas
            // ================================================================
            PaginationParams,      // Page number, limit, sort options
            TaskFilters,           // Filter tasks by status, user, date, search
            TaskQueryParams,       // Combined pagination + filters
            PaginationMeta,        // Metadata about pagination (total pages, etc.)
            PaginatedResponse<Task>, // Generic paginated response wrapper
            
            // ================================================================
            // Error response schema
            // ================================================================
            ApiErrorResponse,   // Standard error response format
            
            // ================================================================
            // Authentication schemas
            // ================================================================
            RegisterRequest,    // Request body for user registration
            LoginRequest,       // Request body for user login
            TokenResponse,      // Response containing JWT token
        )
    ),
    
    // ========================================================================
    // MODIFIERS - Custom OpenAPI Modifications
    // ========================================================================
    // The SecurityAddon modifier adds JWT Bearer authentication configuration
    // to the OpenAPI spec. This enables the "Authorize" button in Swagger UI.
    modifiers(&SecurityAddon),
    
    // ========================================================================
    // TAGS - Endpoint Grouping
    // ========================================================================
    // Tags are used to group related endpoints together in the Swagger UI.
    // When you add endpoint documentation later, you'll assign each endpoint
    // to one of these tags.
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "tasks", description = "Task management endpoints"),
        (name = "files", description = "File upload and management endpoints"),
        (name = "users", description = "User management endpoints"),
    ),
    // ========================================================================
    // INFO - API Metadata
    // ========================================================================
    // This section provides general information about the API that appears
    // at the top of the Swagger UI documentation.
    info(
        title = "Note Task API",
        version = "0.1.0",
        description = "A high-performance task management API built with Rust, Axum, and PostgreSQL.\n\n## Features\n\n- 🚀 **High Performance**: Built with Rust for blazing-fast performance (7,000+ req/s)\n- 🔐 **Secure Authentication**: JWT-based authentication with role-based access control\n- 📝 **Task Management**: Create, read, update, and delete tasks with rich metadata\n- 📎 **File Attachments**: Upload and attach files to tasks\n- 🔍 **Advanced Filtering**: Filter and search tasks with pagination\n- ⚡️ **Optimized**: Zero-copy file handling, arena allocation for WebSocket\n- 🌐 **Multi-Cloud Storage**: Support for AWS S3, GCP, Azure, and Cloudinary\n- 📊 **Real-time Updates**: WebSocket support for live notifications\n\n## Performance\n\n- **Throughput**: 7,600+ requests/second\n- **P95 Latency**: <50ms\n- **P99 Latency**: <100ms\n- **Optimizations**: Zero-copy (bytes::Bytes), Arena allocation (bumpalo), Arc<str> sharing\n\n## Authentication\n\nMost endpoints require authentication. Include the JWT token in the `Authorization` header:\n\n```\nAuthorization: Bearer <your_jwt_token>\n```\n\nObtain a token by registering or logging in through the `/api/v1/auth/register` or `/api/v1/auth/login` endpoints.",
        contact(
            name = "API Support",
            email = "support@example.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    
    // ========================================================================
    // SERVERS - API Server URLs
    // ========================================================================
    // Define the different server environments where the API can be accessed.
    // Users can switch between these in the Swagger UI dropdown.
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "http://localhost:3001", description = "Local development server (alt port)"),
        (url = "https://api.example.com", description = "Production server"),
    )
)]
pub struct ApiDoc;

// ============================================================================
// SecurityAddon - JWT Bearer Authentication Configuration
// ============================================================================
// This struct implements the `Modify` trait to add custom security schemes
// to the OpenAPI specification. It configures JWT Bearer authentication,
// which enables the "Authorize" button in Swagger UI.
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    /// Modifies the OpenAPI spec to add JWT Bearer authentication
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // Check if the components section exists (it should always exist)
        if let Some(components) = openapi.components.as_mut() {
            // Add a security scheme named "bearer_auth"
            // This will appear as the "Authorize" button in Swagger UI
            components.add_security_scheme(
                "bearer_auth", // The name referenced in endpoint security requirements
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer) // Use Bearer token authentication
                        .bearer_format("JWT")            // Specify JWT format
                        .description(Some("Enter your JWT token")) // Help text for users
                        .build(),
                ),
            )
        }
    }
}

