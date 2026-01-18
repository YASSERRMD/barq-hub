-- BARQ HUB Database Initialization

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(255) PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) DEFAULT 'user',
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login TIMESTAMPTZ
);

-- Create default admin user (password: admin123)
INSERT INTO users (id, email, name, password_hash, role, enabled, created_at, updated_at)
VALUES (
    'admin-001',
    'admin@barq.hub',
    'Admin',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.SuQlKYS9v0sJ6m',
    'admin',
    true,
    NOW(),
    NOW()
) ON CONFLICT (email) DO NOTHING;

-- Provider accounts table
CREATE TABLE IF NOT EXISTS provider_accounts (
    id VARCHAR(255) PRIMARY KEY,
    provider_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    api_key_encrypted TEXT,
    enabled BOOLEAN DEFAULT true,
    priority INTEGER DEFAULT 1,
    is_default BOOLEAN DEFAULT false,
    models JSONB DEFAULT '[]',
    config JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Applications table (API keys)
CREATE TABLE IF NOT EXISTS applications (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    api_key_hash VARCHAR(255) NOT NULL,
    api_key_prefix VARCHAR(50) NOT NULL,
    scopes JSONB DEFAULT '[]',
    rate_limit INTEGER DEFAULT 100,
    status VARCHAR(50) DEFAULT 'active',
    requests_today INTEGER DEFAULT 0,
    last_used TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Audit logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255),
    user_name VARCHAR(255),
    action VARCHAR(100) NOT NULL,
    resource VARCHAR(100),
    resource_id VARCHAR(255),
    details JSONB,
    ip_address VARCHAR(50),
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Roles table
CREATE TABLE IF NOT EXISTS roles (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    permissions JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default roles
INSERT INTO roles (id, name, description, permissions) VALUES
    ('role-admin', 'Admin', 'Full system access', '["*"]'),
    ('role-user', 'User', 'Standard user access', '["read", "write"]'),
    ('role-viewer', 'Viewer', 'Read-only access', '["read"]')
ON CONFLICT (name) DO NOTHING;

-- Settings table
CREATE TABLE IF NOT EXISTS settings (
    key VARCHAR(255) PRIMARY KEY,
    value JSONB NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Cost tracking table
CREATE TABLE IF NOT EXISTS cost_records (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255),
    application_id VARCHAR(255),
    provider VARCHAR(100),
    model VARCHAR(100),
    input_tokens INTEGER,
    output_tokens INTEGER,
    cost DECIMAL(10, 6),
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Settings table


-- Create indexes
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_cost_records_timestamp ON cost_records(timestamp);
CREATE INDEX IF NOT EXISTS idx_applications_status ON applications(status);
