
<div align="center">
  <img src="assets/logo.png" alt="BARQ HUB Logo" width="200" height="auto" />
  <h1>BARQ HUB</h1>
  <p><strong>Enterprise AI Management Console & Orchestration Platform</strong></p>

  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
  [![Docker](https://img.shields.io/badge/docker-ready-blue.svg)]()
  [![Rust](https://img.shields.io/badge/backend-Rust%20%7C%20Axum-orange.svg)]()
  [![Next.js](https://img.shields.io/badge/frontend-Next.js%20%7C%20Tailwind-black.svg)]()
</div>

---

**BARQ HUB** is a comprehensive, enterprise-grade platform designed to orchestrate AI models, manage diverse LLM providers, and track utilization costs with precision. It offers a unified, high-performance interface for interacting with top-tier LLM providers, ensuring seamless integration, robust access control, and real-time system monitoring.

## Key Features

*   **Multi-Provider Gateway**: Seamlessly switch between **OpenAI**, **Anthropic**, **Mistral**, **Google Gemini**, and more through a unified API.
*   **Unified API Interface**: Standardized API endpoints for all your LLM needs, simplifying integration.
*   **Advanced Cost Analytics**: Detailed cost breakdown by provider, model, user, and timeframe to optimize spending.
*   **Interactive Playground**: Built-in chat interface to test, compare, and fine-tune model performance.
*   **Enterprise Governance**: Granular Role-Based Access Control (RBAC) and comprehensive API key management.
*   **System Health Monitoring**: Real-time observability of all system components and service uptime.

## Technology Stack

*   **Backend**: Rust (Axum, Tokio, SQLx) - High performance & safety
*   **Frontend**: Next.js 14, TypeScript, Tailwind CSS 4, Shadcn/ui
*   **Database**: PostgreSQL
*   **Caching**: Redis
*   **Deployment**: Docker & Docker Compose

## Getting Started

### Prerequisites

*   Docker and Docker Compose
*   Node.js (for local frontend dev)
*   Rust (for local backend dev)

### Quick Start (Docker)

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/YASSERRMD/barq-hub.git
    cd barq-hub
    ```

2.  **Start the application:**
    ```bash
    docker-compose up -d
    ```

3.  **Access the Dashboard:**
    *   **URL**: `http://localhost:4001`
    *   **Admin Credentials**:
        *   User: `admin@barq.hub`
        *   Pass: `admin123`

### Development Setup

#### Backend (Rust)
```bash
cd backend
cargo run
```

#### Frontend (Next.js)
```bash
cd frontend
npm install
npm run dev
```

## Configuration

Configuration is handled via environment variables. Copy `.env.example` to `.env` to customize:

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |
| `REDIS_URL` | Redis connection string |
| `JWT_SECRET` | Secret key for JWT generation |
| `ENCRYPTION_KEY` | Key for sensitive data encryption |

## API Documentation

The REST API is available at `http://localhost:4000/v1`.

### Key Endpoints
- **Chat**: `POST /v1/chat/completions` (OpenAI-compatible)
- **Models**: `GET /v1/models`
- **Costs**: `GET /v1/costs`



