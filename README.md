# Note Task API

A high-performance, production-ready task management REST API built with Rust, featuring advanced optimizations, multi-cloud storage support, and comprehensive API documentation.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Performance](https://img.shields.io/badge/performance-7600%2B%20req%2Fs-brightgreen.svg)]()

## 🚀 Features

### Core Functionality
- **Task Management**: Create, read, update, and delete tasks with rich metadata
- **File Attachments**: Upload and attach files to tasks with multi-cloud storage support
- **User Authentication**: JWT-based authentication with role-based access control (User/Admin)
- **Advanced Filtering**: Search, filter, and paginate tasks with flexible query parameters
- **Real-time Updates**: WebSocket support for live notifications and updates

### Performance Optimizations
- **Zero-Copy File Handling**: Uses `bytes::Bytes` for efficient memory management
- **Arena Allocation**: Custom memory allocator with `bumpalo` for temporary objects
- **Shared String References**: `Arc<str>` for frequently accessed strings
- **Redis Caching**: Fast in-memory caching for frequently accessed data
- **Connection Pooling**: Optimized database connection management with SQLx

### Storage & Infrastructure
- **Multi-Cloud Storage**: Support for AWS S3, Google Cloud Storage, Azure Blob Storage, and Cloudinary
- **Presigned URLs**: Secure, time-limited file access without authentication
- **Local Storage**: Fallback option for development and testing
- **Background Jobs**: Asynchronous job processing for emails and cleanup tasks
- **Database Migrations**: Automated schema management with SQLx

### Developer Experience
- **Interactive API Documentation**: Swagger UI with "Try it out" functionality for all endpoints
- **Comprehensive Testing**: Benchmarking suite with `wrk` and `k6`
- **Makefile Commands**: Easy-to-use commands for common tasks
- **Validation**: Request validation with detailed error messages
- **Logging**: Structured logging with `tracing`

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
- **Redis**: 6 or higher (optional, for caching)
- **Docker**: For running PostgreSQL and Redis (optional)

## 🚀 Quick Start

### 1. Clone the Repository

```bash
git clone <repository-url>
cd note-task-api
```

### 2. Set Up Environment Variables

Create a `.env` file in the project root:

```bash
# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=3001

# Database Configuration
DATABASE_URL=postgresql://postgres:password@localhost:5432/note_task_db

# Redis Configuration (optional)
REDIS_URL=redis://localhost:6379

# JWT Configuration
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_EXPIRATION=86400

# Storage Configuration
# Options: local, s3, gcs, azure, cloudinary
STORAGE_PROVIDER=local
UPLOAD_DIR=./uploads

# AWS S3 (if using S3)
# AWS_REGION=us-east-1
# AWS_ACCESS_KEY_ID=your-access-key
# AWS_SECRET_ACCESS_KEY=your-secret-key
# S3_BUCKET_NAME=your-bucket-name

# GCP Cloud Storage (if using GCS)
# GCS_BUCKET_NAME=your-bucket-name
# GCS_CREDENTIALS_PATH=./path/to/credentials.json

# Azure Blob Storage (if using Azure)
# AZURE_STORAGE_ACCOUNT=your-account
# AZURE_STORAGE_ACCESS_KEY=your-key
# AZURE_CONTAINER_NAME=your-container

# Cloudinary (if using Cloudinary)
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
# Development
make dev              # Run in development mode
make build            # Build the project
make release          # Build optimized release

# Database
make db-setup         # Set up database
make db-migrate       # Run migrations
make db-reset         # Reset database

# Testing
make test             # Run tests
make benchmark        # Run all benchmarks

# Utilities
make clean            # Clean build artifacts
make fmt              # Format code
make lint             # Run linter
```

## 🏗️ Project Structure

```
note-task-api/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── lib.rs                  # Library root
│   ├── config/                 # Configuration management
│   │   ├── mod.rs
│   │   └── settings.rs
│   ├── domain/                 # Domain models
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── task.rs
│   │   ├── file.rs
│   │   ├── error.rs
│   │   └── pagination.rs
│   ├── handlers/               # HTTP request handlers
│   │   ├── mod.rs
│   │   ├── auth_handlers.rs
│   │   ├── task_handlers.rs
│   │   ├── file_handlers.rs
│   │   ├── user_handlers.rs
│   │   └── health_handlers.rs
│   ├── services/               # Business logic
│   │   ├── mod.rs
│   │   ├── auth_service.rs
│   │   ├── task_service.rs
│   │   ├── file_service.rs
│   │   ├── user_service.rs
│   │   └── email_service.rs
│   ├── repositories/           # Data access layer
│   │   ├── mod.rs
│   │   ├── user_repository.rs
│   │   ├── task_repository.rs
│   │   └── file_repository.rs
│   ├── middleware/             # Custom middleware
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   └── logging.rs
│   ├── storage/                # Storage providers
│   │   ├── mod.rs
│   │   ├── local.rs
│   │   ├── s3.rs
│   │   ├── gcs.rs
│   │   ├── azure.rs
│   │   └── cloudinary.rs
│   ├── workers/                # Background jobs
│   │   ├── mod.rs
│   │   ├── worker_service.rs
│   │   └── job_processor.rs
│   ├── websocket/              # WebSocket support
│   │   ├── mod.rs
│   │   └── manager.rs
│   ├── cache/                  # Redis caching
│   │   ├── mod.rs
│   │   └── redis_cache.rs
│   ├── arena/                  # Arena allocation
│   │   ├── mod.rs
│   │   └── query_builder.rs
│   ├── openapi.rs              # OpenAPI/Swagger config
│   ├── routes.rs               # Route definitions
│   ├── validation.rs           # Request validation
│   └── extractors.rs           # Custom extractors
├── migrations/                 # Database migrations
├── benchmarks/                 # Benchmark scripts
│   ├── wrk/
│   └── k6/
├── Cargo.toml                  # Rust dependencies
├── Makefile                    # Build commands
├── .env.example                # Environment variables template
└── README.md                   # This file
```

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

The API supports multiple storage providers. Switch between them by changing the `STORAGE_PROVIDER` environment variable.

### Local Storage (Development)

```env
STORAGE_PROVIDER=local
UPLOAD_DIR=./uploads
```

### AWS S3

```env
STORAGE_PROVIDER=s3
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=your-access-key
AWS_SECRET_ACCESS_KEY=your-secret-key
S3_BUCKET_NAME=your-bucket-name
```

### Google Cloud Storage

```env
STORAGE_PROVIDER=gcs
GCS_BUCKET_NAME=your-bucket-name
GCS_CREDENTIALS_PATH=./credentials.json
```

### Azure Blob Storage

```env
STORAGE_PROVIDER=azure
AZURE_STORAGE_ACCOUNT=your-account
AZURE_STORAGE_ACCESS_KEY=your-key
AZURE_CONTAINER_NAME=your-container
```

### Cloudinary

```env
STORAGE_PROVIDER=cloudinary
CLOUDINARY_CLOUD_NAME=your-cloud-name
CLOUDINARY_API_KEY=your-api-key
CLOUDINARY_API_SECRET=your-api-secret
```

## 🔒 Security Features

- **Password Hashing**: Argon2 algorithm with salt
- **JWT Authentication**: Secure token-based auth
- **Role-Based Access Control**: User and Admin roles
- **Input Validation**: Comprehensive request validation
- **SQL Injection Prevention**: Compile-time checked queries with SQLx
- **CORS**: Configurable cross-origin resource sharing
- **Rate Limiting**: (TODO) Prevent abuse

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

