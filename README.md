# BARQ HUB

AI Management Console

## Overview

BARQ HUB is an open-source console for managing LLM providers, vector databases, users, and system resources.

## Features

| Module | Description |
|--------|-------------|
| Dashboard | System overview and metrics |
| Providers | LLM provider management |
| Vector DBs | Vector database configuration |
| Playground | Prompt testing interface |
| Billing | Cost tracking and budgets |
| Audit Logs | Activity logging and export |
| Users | User account management |
| Roles | RBAC permission management |
| Health | System status monitoring |
| Settings | Application preferences |

## Requirements

- Docker and Docker Compose
- Node.js 20+ (for local development)
- Rust 1.75+ (for backend development)

## Installation

```bash
git clone https://github.com/YASSERRMD/barq-hub.git
cd barq-hub
cp .env.example .env
docker-compose up -d --build
```

## Services

| Service | Port |
|---------|------|
| Frontend | 4001 |
| Backend | 4000 |
| PostgreSQL | 5433 |
| Redis | 6380 |
| Qdrant | 6335 |

## Development

```bash
# Start databases
docker-compose up postgres redis qdrant -d

# Backend
cargo run

# Frontend
cd frontend && npm install && npm run dev
```

## Tech Stack

- Frontend: Next.js 14, TypeScript, Tailwind CSS, Shadcn/ui
- Backend: Rust, Axum, SQLx
- Database: PostgreSQL, Redis, Qdrant

## License

MIT
