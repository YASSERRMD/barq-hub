# BARQ HUB

**AI Management Console**

Open-source console for managing LLM providers, vector databases, users, and more.

## Features

- **Dashboard** - Metrics, charts, system health
- **LLM Providers** - OpenAI, Anthropic, Google, Mistral, Groq, etc.
- **Vector Databases** - Qdrant, Pinecone, Weaviate, Chroma
- **Playground** - Test prompts live
- **Billing** - Cost tracking, budgets
- **Audit Logs** - Searchable logs with export
- **Users & Roles** - RBAC management
- **System Health** - Status monitoring
- **Dark/Light Mode** - Theme switching

## Quick Start

### Docker (Recommended)

```bash
git clone https://github.com/YASSERRMD/barq-hub.git
cd barq-hub
cp .env.example .env
docker-compose up -d --build
```

| Service | Port |
|---------|------|
| Frontend | http://localhost:4001 |
| Backend | http://localhost:4000 |
| PostgreSQL | 5433 |
| Redis | 6380 |
| Qdrant | 6335 |

### Local Development

```bash
# Start databases
docker-compose up postgres redis qdrant -d

# Backend
cargo run

# Frontend
cd frontend && npm install && npm run dev
```

## Tech Stack

- **Frontend**: Next.js 14, TypeScript, Tailwind, Shadcn/ui
- **Backend**: Rust, Axum, SQLx
- **Database**: PostgreSQL, Redis, Qdrant

## License

MIT License
