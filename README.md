# BARQ HUB - AI Management Console

BARQ HUB is a comprehensive platform for managing AI models, providers, and costs. It provides a unified interface for interacting with various LLM providers, tracking usage, and managing access.

## Features

- **Multi-Provider Support**: Seamlessly switch between OpenAI, Anthropic, Mistral, and more.
- **Unified API**: Single API endpoint for all your LLM needs.
- **Cost Tracking**: Detailed cost analysis by provider, model, and user.
- **Playground**: Interactive chat interface to test models.
- **User Management**: Role-based access control and API key management.
- **System Health**: Real-time monitoring of system components.

## Getting Started

### Prerequisites

- Docker and Docker Compose
- Node.js (for local frontend development)
- Rust (for local backend development)

### Quick Start

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/YASSERRMD/barq-hub.git
    cd barq-hub
    ```

2.  **Start the application:**
    ```bash
    docker-compose up -d
    ```

3.  **Access the dashboard:**
    Open your browser and navigate to `http://localhost:4001`.

    - **Username:** `admin@barq.hub`
    - **Password:** `admin123`

### Development

#### Backend

The backend is built with Rust and Axum.

```bash
cd backend
cargo run
```

#### Frontend

The frontend is built with Next.js and Tailwind CSS.

```bash
cd frontend
npm install
npm run dev
```

## Configuration

Configuration is managed via environment variables and the `.env` file. Key variables include:

- `DATABASE_URL`: PostgreSQL connection string.
- `REDIS_URL`: Redis connection string.
- `JWT_SECRET`: Secret key for JWT tokens.
- `ENCRYPTION_KEY`: Key for encrypting sensitive data.

See `.env.example` for a full list of variables.

## API Documentation

The API is available at `http://localhost:4000/v1`.

### Key Endpoints

- `POST /v1/chat/completions`: OpenAI-compatible chat completion.
- `GET /v1/models`: List available models.
- `GET /v1/costs`: Get cost usage statistics.

For full API documentation, please refer to the OpenAPI spec (coming soon).
