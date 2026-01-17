# BARQ HUB

**Lightning-fast SYNAPSE Brain Console**

Open-source management console for SYNAPSE Brain with a completely new, modern design.

## Features

- **Dashboard** - Real-time metrics, charts, system health overview
- **LLM Providers** - Manage OpenAI, Anthropic, Google, Mistral, Groq, and more
- **Vector Databases** - Configure Qdrant, Pinecone, Weaviate, Chroma
- **Playground** - Test prompts and providers live
- **Billing** - Cost tracking, budgets, usage analytics
- **Audit Logs** - Searchable logs with export to CSV/PDF
- **Users & Roles** - RBAC with permissions management
- **System Health** - Uptime, status indicators, metrics
- **Dark/Light Mode** - System preference + manual toggle

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | Next.js 14, TypeScript, Tailwind CSS, Shadcn/ui |
| Backend | Rust, Axum, SQLx |
| Database | PostgreSQL 16 |
| Cache | Redis 7 |
| Vector DB | Qdrant |
| State | Zustand, React Query |
| Charts | Recharts |

## Quick Start

### Option 1: Docker (Recommended)

```bash
# Clone the repository
git clone https://github.com/YASSERRMD/barq-hub.git
cd barq-hub

# Copy environment file and add your API keys
cp .env.example .env
nano .env  # Add your API keys

# Start all services
docker-compose up -d --build
```

**Services:**
| Service | Port |
|---------|------|
| Frontend | http://localhost:4001 |
| Backend API | http://localhost:4000 |
| PostgreSQL | localhost:5433 |
| Redis | localhost:6380 |
| Qdrant | localhost:6335 |

**Default Login:**
- Email: `admin@synapse.local`
- Password: `admin123`

### Option 2: Local Development

```bash
# Start databases only
docker-compose up postgres redis qdrant -d

# Terminal 1: Backend
cargo run

# Terminal 2: Frontend
cd frontend && npm install && npm run dev
```

## Project Structure

```
barq-hub/
├── backend/           # Rust backend source
│   ├── api/          # REST API handlers
│   ├── agents/       # Agent management
│   ├── providers/    # LLM provider adapters
│   ├── knowledge/    # RAG, embeddings
│   ├── governance/   # Auth, RBAC, audit
│   └── workflow/     # DAG workflow engine
├── frontend/          # Next.js frontend
│   ├── app/          # Pages & routes
│   ├── components/   # React components
│   ├── hooks/        # Custom hooks
│   ├── stores/       # Zustand state
│   └── types/        # TypeScript types
├── docker-compose.yml
├── Cargo.toml
├── init-db.sql
└── README.md
```

## Environment Variables

See `.env.example` for all available configuration options.

## License

MIT License

---

**BARQ HUB** - Built for the AI community
