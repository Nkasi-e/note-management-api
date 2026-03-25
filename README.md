# Note Task API

A high-performance, production-ready task management REST API built with Rust, featuring advanced optimizations, multi-cloud storage support, and comprehensive API documentation.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Performance](https://img.shields.io/badge/performance-7600%2B%20req%2Fs-brightgreen.svg)]()

## 🚀 Features
### Core Capabilities
- Task CRUD with filtering and pagination
- File uploads and task file attachments via multi-backend storage
- JWT auth with role-based access (`User` / `Admin`)
- Real-time updates over WebSocket
- Background jobs for emails, cleanup, reports, and attachment processing

### Implementation Highlights
- `bytes::Bytes` for efficient payload handling (zero-copy friendly)
- Request-scoped bump allocation (`bumpalo`) for temporary formatting/query building
- Redis caching plus a Redis-backed worker queue
- OpenAPI docs via `utoipa` + Swagger UI

## 📊 Performance Metrics

Based on benchmark results:

| Metric | Value |
|--------|-------|
| **Throughput** | 7,600+ requests/second |
| **P50 Latency** | <10ms |
| **P95 Latency** | <50ms |
| **P99 Latency** | <100ms |
| **Concurrent Users** | 100+ (tested) |

## 🛠️ Tech Stack

- **Framework**: [Axum](https://github.com/tokio-rs/axum) - Fast, ergonomic web framework
- **Database**: PostgreSQL with [SQLx](https://github.com/launchbadge/sqlx) - Compile-time checked queries
- **Cache**: Redis - In-memory data store
- **Authentication**: JWT (JSON Web Tokens)
- **Password Hashing**: Argon2 - Secure password hashing
- **Storage**: AWS S3, GCP Cloud Storage, Azure Blob Storage, Cloudinary, or Local
- **WebSocket**: Real-time communication with Axum WebSocket support
- **Documentation**: [utoipa](https://github.com/juhaku/utoipa) - OpenAPI/Swagger generation
- **Async Runtime**: [Tokio](https://tokio.rs/) - Asynchronous runtime

## 📋 Prerequisites

- **Rust**: 1.70 or higher
- **PostgreSQL**: 14 or higher
- **Redis**: 6 or higher (required; cache + worker queue)
- **Docker**: Optional (handy for running PostgreSQL and Redis locally)

## 🚀 Quick Start

### 1. Clone the Repository

```bash
git clone <repository-url>
cd note-task-api
```

### 2. Set Up Environment Variables

Create a `.env` file in the project root:

```bash
# Server
APP_HOST=127.0.0.1
APP_PORT=3001

# Database (PostgreSQL)
DATABASE_URL=postgres://postgres:password@localhost:5432/note_task_db

# Redis (required; cache + background jobs)
REDIS_URL=redis://127.0.0.1:6379
REDIS_TTL_SECS=300

# JWT Authentication
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_ISSUER=note-task-api
JWT_AUDIENCE=note-clients
JWT_EXP_MINUTES=60

# Logging (optional)
RUST_LOG=info
LOG_FORMAT=json

# Email (required by background email jobs)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@example.com
SMTP_PASSWORD=your-smtp-password-or-app-password
FROM_EMAIL=noreply@yourdomain.com

# Storage
STORAGE_BACKEND=local
UPLOAD_DIR=./uploads
MAX_FILE_SIZE=10485760
ALLOWED_EXTENSIONS=jpg,jpeg,png,gif,pdf,doc,docx,txt,zip
SERVE_FILES=true

# --- Storage backend specific options (uncomment as needed) ---

# AWS S3
# AWS_REGION=us-east-1
# AWS_S3_BUCKET=your-bucket-name
# AWS_CDN_URL=optional-cdn-url

# GCS
# GCP_BUCKET=your-bucket-name
# GCP_CDN_URL=optional-cdn-url

# Azure Blob Storage
# AZURE_ACCOUNT_NAME=your-account
# AZURE_ACCOUNT_KEY=your-key
# AZURE_CONTAINER=your-container
# AZURE_CDN_URL=optional-cdn-url

# Cloudinary
# CLOUDINARY_CLOUD_NAME=your-cloud-name
# CLOUDINARY_API_KEY=your-api-key
# CLOUDINARY_API_SECRET=your-api-secret
```

### 3. Start Database and Redis

Using Docker:

```bash
# Start PostgreSQL
docker run -d \
  --name postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=note_task_db \
  -p 5432:5432 \
  postgres:14

# Start Redis
docker run -d \
  --name redis \
  -p 6379:6379 \
  redis:6
```

### 4. Run Database Migrations

```bash
# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run
```

### 5. Build and Run

```bash
# Development mode
cargo run

# Production mode (optimized)
cargo build --release
./target/release/note-task-api
```

The API will be available at `http://localhost:3001`

### 6. Access Swagger UI

Open your browser and navigate to:

```
http://localhost:3001/swagger-ui
```

## 📖 API Documentation

### Interactive Documentation

Visit the Swagger UI at `http://localhost:3001/swagger-ui` to:
- View all available endpoints
- Test endpoints directly in the browser
- See request/response schemas
- Try authentication flows

### OpenAPI Specification

Download the OpenAPI spec at `http://localhost:3001/api-docs/openapi.json` for:
- Importing into Postman
- Generating client SDKs
- API gateway configuration
- Documentation generation

### API Endpoints

#### Health & Status
- `GET /health` - Health check
- `GET /ping` - Ping endpoint

#### Authentication
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - Login user

#### Tasks
- `POST /api/v1/tasks` - Create task
- `GET /api/v1/tasks` - List tasks (with pagination & filters)
- `GET /api/v1/tasks/{id}` - Get task by ID

#### Files
- `POST /api/v1/files/upload` - Upload file
- `GET /api/v1/files` - List user files
- `GET /api/v1/files/{id}` - Get file metadata
- `GET /api/v1/files/{id}/download` - Download file
- `DELETE /api/v1/files/{id}` - Delete file
- `GET /api/v1/files/stats` - Get file statistics
- `GET /api/v1/files/{id}/presigned-url` - Generate presigned URL

#### Users
- `POST /api/v1/users` - Create user (admin only)
- `GET /api/v1/users/{id}` - Get user by ID

## 🔐 Authentication

The API uses JWT (JSON Web Tokens) for authentication.

### Register a User

```bash
curl -X POST http://localhost:3001/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "John Doe",
    "email": "john@example.com",
    "password": "SecurePassword123!"
  }'
```

### Login

```bash
curl -X POST http://localhost:3001/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john@example.com",
    "password": "SecurePassword123!"
  }'
```

Response:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### Using the Token

Include the token in the `Authorization` header:

```bash
curl -X GET http://localhost:3001/api/v1/tasks \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## 📝 Usage Examples

### Create a Task

```bash
curl -X POST http://localhost:3001/api/v1/tasks \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Buy groceries",
    "description": "Milk, eggs, bread"
  }'
```

### Upload a File

```bash
curl -X POST http://localhost:3001/api/v1/files/upload \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -F "file=@/path/to/file.pdf"
```

### Create Task with File Attachment

```bash
# 1. Upload file first
FILE_ID=$(curl -X POST http://localhost:3001/api/v1/files/upload \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -F "file=@document.pdf" | jq -r '.data.id')

# 2. Create task with file attachment
curl -X POST http://localhost:3001/api/v1/tasks \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"Review document\",
    \"description\": \"Check the attached PDF\",
    \"attachment_ids\": [\"$FILE_ID\"]
  }"
```

### List Tasks with Filters

```bash
# Get tasks with pagination
curl "http://localhost:3001/api/v1/tasks?page=1&limit=10" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Filter by status
curl "http://localhost:3001/api/v1/tasks?status=todo" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Search tasks
curl "http://localhost:3001/api/v1/tasks?search=groceries" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Combine filters
curl "http://localhost:3001/api/v1/tasks?status=in_progress&page=1&limit=20&sort_by=created_at&sort_direction=desc" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## 🧪 Testing & Benchmarking

### Run Benchmarks

The project includes a comprehensive benchmarking suite using `wrk` and `k6`.

```bash
# Install benchmarking tools
make install-benchmark-tools

# Run all benchmarks
make benchmark

# Run specific benchmarks
make benchmark-wrk      # HTTP load testing
make benchmark-k6-load  # K6 load testing
make benchmark-k6-stress # K6 stress testing
make benchmark-k6-ws    # WebSocket testing
```

### Benchmark Results

After running benchmarks, results are saved in:
- `benchmarks/results/wrk-results.txt`
- `benchmarks/results/k6-load-results.txt`
- `benchmarks/results/k6-stress-results.txt`
- `benchmarks/results/k6-ws-results.txt`

## 🔧 Makefile Commands

```bash
# Run / type-check
make run              # Start the API server
make check            # Type-check

# SQLx migrations
make db-create       # Create DB (from DATABASE_URL)
make db-ext          # Enable pgcrypto extension
make migrate-run     # Run migrations
make migrate-info    # Show migration status

# Development loop
make watch-run       # Rebuild & restart on changes

# Benchmarks
make bench-install
make bench-seed
make bench-all
make bench-load
make bench-stress
make bench-websocket
make bench-results
make bench-clean
```

## 🏗️ Architecture

```mermaid
flowchart TD
  Client["Client"] --> Router["Axum Router"]
  Router --> Health["Health Routes: /health, /ping"]
  Router --> API["API Routes: /api/v1"]
  API --> Routes["Route Groups: auth, users, tasks, files, workers"]

  Routes --> Handlers["HTTP Handlers: src/handlers/*"]
  Handlers --> Services["Services: src/services/*"]
  Services --> Repos["Repositories: src/repositories/*"]
  Repos --> DB[(PostgreSQL)]

  Services --> Cache["RedisCache: src/cache/*"]
  Services --> Storage["StorageProvider + StorageFactory: src/storage/*"]
  Services --> WSJobNotify["Workers route integration"]

  Routes --> WS["WebSocket Routes: /ws, /ws/health"]
  WS --> WSHandler["ws_handler + WebSocketManager: src/websocket/*"]
  WSHandler --> WSNotify["WebSocket notifications"]

  WSJobNotify --> WorkerService["WorkerService + JobProcessor: src/workers/*"]
  WorkerService --> Queue["Redis sorted-set queue"]
  WorkerService --> JobProcessor["Job execution"]
  JobProcessor --> Services
  WorkerService --> WSNotify
```

### Code Structure (layers in the codebase)

- `src/main.rs`: runtime wiring (config, pools, router, starts `WorkerService`)
- `src/routes/*`: URL nesting/prefixes and which handlers are mounted
- `src/handlers/*`: HTTP handlers (request -> service calls -> responses)
- `src/services/*`: business logic (auth, tasks, users, files, email)
- `src/repositories/*`: SQLx DB access
- `src/workers/*`: background job processing driven by Redis
- `src/websocket/*`: WebSocket message routing/broadcasting
- `src/cache/*`: Redis caching helpers
- `src/storage/*`: storage backends implementing `StorageProvider`
- `src/arena.rs`: request-scoped bump allocator for temporary allocations

## 🎯 Performance Optimizations

### 1. Zero-Copy File Handling

Uses `bytes::Bytes` for efficient memory management:
- No unnecessary data copying
- Reference-counted buffers
- Efficient streaming

### 2. Arena Allocation

Custom memory allocator with `bumpalo`:
- O(1) allocation
- Bulk deallocation
- Reduced memory fragmentation

### 3. Connection Pooling

Optimized database connections:
- Connection reuse
- Configurable pool size
- Health checks

### 4. Redis Caching

Fast in-memory caching:
- Frequently accessed data
- Reduced database load
- Configurable TTL

### 5. Shared String References

Uses `Arc<str>` for frequently accessed strings:
- Reduced memory usage
- Efficient cloning
- Thread-safe sharing

## 🌐 Multi-Cloud Storage
The API selects a storage backend via `STORAGE_BACKEND` (see the environment variable section above). Each backend implements `StorageProvider` (`src/storage/*`) and is constructed by `StorageFactory`.

Supported backends:
- `local`
- `s3`
- `gcs`
- `azure`
- `cloudinary`

Presigned URLs are supported by the cloud backends. The `local` backend does not generate presigned URLs and instead returns the normal download URL.

## 🔒 Security Features

- **Password Hashing**: Argon2 algorithm with salt
- **JWT Authentication**: Secure token-based auth
- **Role-Based Access Control**: User and Admin roles
- **Input Validation**: Comprehensive request validation
- **SQL Injection Prevention**: Compile-time checked queries with SQLx
- **CORS**: Enabled (permissive; allow-all by default)

## 🐛 Troubleshooting

### Database Connection Issues

```bash
# Check if PostgreSQL is running
docker ps | grep postgres

# Check connection
psql -h localhost -U postgres -d note_task_db
```

### Redis Connection Issues

```bash
# Check if Redis is running
docker ps | grep redis

# Test connection
redis-cli ping
```

### Port Already in Use

```bash
# Find process using port 3001
lsof -i :3001

# Kill the process
kill -9 <PID>
```

## 📚 Additional Resources

- [Axum Documentation](https://docs.rs/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Tokio Documentation](https://tokio.rs/)
- [utoipa Documentation](https://docs.rs/utoipa/)
- [Swagger UI](https://swagger.io/tools/swagger-ui/)

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 👨‍💻 Author

Built with ❤️ using Rust

## 🙏 Acknowledgments

- Rust community for excellent crates and documentation
- Axum team for the amazing web framework
- SQLx team for compile-time checked queries
- utoipa team for OpenAPI generation

---

**Happy Coding! 🚀**

