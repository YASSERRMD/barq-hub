
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

## How to Use

### 1. Identity & Access Management
BARQ HUB uses a robust RBAC system.
- **Users**: Manage team members in `Users` page.
- **Roles**: Define granular permissions in `Roles` page.
- **API Keys**: Create application-specific keys in `API Keys` page.
  - Click `+ Create API Key`
  - Set a rate limit (e.g., 60 req/min)
  - Copy the key immediately (it won't be shown again)
  - Use this key in the `Authorization: Bearer <KEY>` header for your apps.

### 2. Managing LLM Providers
Navigate to the **Providers** page to configure upstream AI services.
- **Enable/Disable**: Toggle providers on/off instantly.
- **Configuration**: Click `Edit` to update API keys or base URLs.
- **Load Balancing**: The system automatically load-balances between available accounts for the same provider.

### 3. Interactive Playground
Test your models before integrating them.
- Go to `Playground`
- Select a Provider (e.g., OpenAI, Groq) and a Model.
- Type your prompt and view the response, token usage, and estimated cost in real-time.
- Use the **Integration Code** tab to generate snippets for Python, Node.js, and cURL.

## API Documentation

The REST API is available at `http://localhost:4000/v1`.

### Key Endpoints
- **Chat Completions**: `POST /v1/chat/completions`
  ```json
  {
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello"}]
  }
  ```
- **Models**: `GET /v1/models`
- **Costs**: `GET /v1/costs`

*Full OpenAPI specification available at `/swagger-ui` (coming soon).*
