# BARQ HUB - AI Management Console

<div align="center">
  <img src="assets/logo.png" alt="BARQ HUB Logo" width="80" />
  <h1>BARQ HUB</h1>
  <p><strong>Enterprise-grade AI Orchestration & Management Platform</strong></p>
  
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Rust](https://img.shields.io/badge/backend-rust-orange.svg)](backend)
  [![Next.js](https://img.shields.io/badge/frontend-next.js-black.svg)](frontend)
  [![Docker](https://img.shields.io/badge/deployment-docker-blue.svg)](docker-compose.yml)
</div>

---

## Overview

**BARQ HUB** is a comprehensive AI management console designed for enterprises to orchestrate, monitor, and secure their AI infrastructure. It provides a unified gateway to multiple LLM providers (OpenAI, Anthropic, Google, Cohere, Mistral, Groq, Together AI), robust API key management, and detailed usage analytics.

Built with performance and security in mind, BARQ HUB leverages a high-performance **Rust** backend and a modern **Next.js** frontend.

## Key Features

- **Multi-Provider Gateway**: Unified API access to OpenAI, Anthropic, Mistral, Groq, Together AI, Google Gemini, and Cohere.
- **API Key Management**: Create keys with granular scopes, rate limits, and auto-rotation policies.
- **Real-time Analytics**: Monitor request volume, latency, and costs with interactive dashboards.
- **Enterprise Security**: Role-Based Access Control (RBAC), audit logging, and secure credential storage.
- **Interactive Playground**: Test prompts and models directly with auto-generated code snippets (Python, JS, cURL).
- **Cost Control**: Set budget limits and track spending by provider or application.
- **Production Ready**: Efficient Docker containerization and comprehensive health checks.

## Project Structure

```
barq-hub/
├── backend/              # Rust backend API
│   ├── Dockerfile        # Backend Docker configuration
│   ├── api/              # API handlers
│   ├── db/               # Database models
│   ├── providers/        # LLM provider integrations
│   └── main.rs           # Entry point
├── frontend/             # Next.js frontend
│   ├── Dockerfile        # Frontend Docker configuration
│   ├── app/              # App router pages
│   ├── components/       # UI components
│   └── lib/              # Utilities & API client
├── migrations/           # Database migrations
│   ├── init.sql          # Schema initialization
│   ├── init-db.sql       # Extended schema
│   └── seed.sql          # Sample data
├── docker-compose.yml    # Service orchestration
├── Cargo.toml            # Rust dependencies
└── README.md
```

## Prerequisites

- **Docker** & **Docker Compose** (recommended)
- **Node.js** 20+ (for local frontend development)
- **Rust** 1.75+ (for local backend development)

## Getting Started

### Quick Start with Docker (Recommended)

1. **Clone the repository**
   ```bash
   git clone https://github.com/YASSERRMD/barq-hub.git
   cd barq-hub
   ```

2. **Setup Environment**
   ```bash
   cp .env.example .env
   # Edit .env with your provider API keys if needed
   ```

3. **Start All Services**
   ```bash
   docker-compose up --build -d
   ```

4. **Verify Services are Running**
   ```bash
   docker-compose ps
   ```

5. **Access the Application**
   | Service | URL |
   |---------|-----|
   | **Admin Console** | http://localhost:4001 |
   | **API Endpoint** | http://localhost:4000/v1 |
   | **Health Check** | http://localhost:4000/v1/admin/health |
   
   **Default Admin**: `admin@barq.hub` / `admin123`

### Stopping Services

```bash
docker-compose down          # Stop containers
docker-compose down -v       # Stop and remove volumes (clears database)
```

### Rebuilding After Changes

```bash
docker-compose up -d --build backend    # Rebuild backend only
docker-compose up -d --build frontend   # Rebuild frontend only
docker-compose up -d --build            # Rebuild all services
```

### Local Development

#### Backend (Rust)
```bash
# Start database and Redis
docker-compose up postgres redis -d

# Apply migrations
psql postgres://barq:barq123@localhost:5433/barq_hub -f migrations/init.sql
psql postgres://barq:barq123@localhost:5433/barq_hub -f migrations/seed.sql

# Run backend
cd backend
cargo run
```

#### Frontend (Next.js)
```bash
cd frontend
npm install
npm run dev
```

The frontend will be available at http://localhost:3000 (dev mode).

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER__PORT` | Backend server port | `4000` |
| `DATABASE_URL` | PostgreSQL connection string | `postgres://barq:barq123@postgres:5432/barq_hub` |
| `REDIS_URL` | Redis connection string | `redis://redis:6379` |
| `JWT_SECRET` | Secret for signing auth tokens | - |
| `FRONTEND_URL` | Frontend URL for CORS | `http://localhost:4001` |

See `.env.example` for the full list.

## API Usage

### Base URL
```
http://localhost:4000/v1
```

### Authentication
All API requests require a Bearer token. Create an API key in the **API Keys** section of the console.

```bash
curl -X POST "http://localhost:4000/v1/chat/completions" \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4o",
    "provider": "openai",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

### Available Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/chat/completions` | POST | Chat with LLM providers |
| `/embeddings` | POST | Generate embeddings |
| `/provider-accounts/providers` | GET | List available providers |
| `/admin/health` | GET | System health status |
| `/admin/users/stats` | GET | User statistics |

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

1. Fork the repo
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

Distributed under the MIT License. See `LICENSE` for more information.

---

<div align="center">
  <p>Built with ❤️ by the BARQ Team</p>
</div>
