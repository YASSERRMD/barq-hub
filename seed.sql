-- BARQ HUB Data Seeding
-- Run this script to populate the database with test data

-- 1. Additional Users
INSERT INTO users (id, email, name, password_hash, role, enabled, created_at) VALUES
('user-001', 'sarah@barq.dev', 'Sarah Engineer', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.SuQlKYS9v0sJ6m', 'user', true, NOW() - INTERVAL '30 days'),
('user-002', 'mike@barq.dev', 'Mike Analyst', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.SuQlKYS9v0sJ6m', 'viewer', true, NOW() - INTERVAL '15 days'),
('user-003', 'bot@barq.dev', 'CI/CD Bot', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.SuQlKYS9v0sJ6m', 'user', true, NOW() - INTERVAL '60 days')
ON CONFLICT (email) DO NOTHING;

-- 2. Providers (Comprehensive List as requested)
INSERT INTO provider_accounts (id, provider_id, name, enabled, priority, config, models) VALUES
-- OpenAI
('prov-openai-1', 'openai', 'OpenAI Production', true, 10, '{"type": "api_key", "api_key": "sk-proj-prod-key-123"}', '[{"id": "gpt-4-turbo", "name": "GPT-4 Turbo"}, {"id": "gpt-4o", "name": "GPT-4o"}, {"id": "gpt-3.5-turbo", "name": "GPT-3.5 Turbo"}]'),
('prov-openai-2', 'openai', 'OpenAI Dev', true, 5, '{"type": "api_key", "api_key": "sk-proj-dev-key-456"}', '[{"id": "gpt-3.5-turbo", "name": "GPT-3.5 Turbo"}]'),

-- Anthropic
('prov-anthropic-1', 'anthropic', 'Anthropic Primary', true, 9, '{"type": "api_key", "api_key": "sk-ant-key-789"}', '[{"id": "claude-3-opus-20240229", "name": "Claude 3 Opus"}, {"id": "claude-3-sonnet-20240229", "name": "Claude 3 Sonnet"}, {"id": "claude-3-haiku-20240307", "name": "Claude 3 Haiku"}]'),

-- Google
('prov-google-1', 'google', 'Google Vertex AI', true, 8, '{"type": "gcp_auth", "project_id": "barq-genai"}', '[{"id": "gemini-1.5-pro", "name": "Gemini 1.5 Pro"}, {"id": "gemini-1.5-flash", "name": "Gemini 1.5 Flash"}]'),

-- Mistral
('prov-mistral-1', 'mistral', 'Mistral API', true, 7, '{"type": "api_key", "api_key": "mistral-key-111"}', '[{"id": "mistral-large-latest", "name": "Mistral Large"}, {"id": "mistral-medium", "name": "Mistral Medium"}, {"id": "mistral-small", "name": "Mistral Small"}]'),

-- Cohere
('prov-cohere-1', 'cohere', 'Cohere Command', true, 6, '{"type": "api_key", "api_key": "cohere-key-222"}', '[{"id": "command-r-plus", "name": "Command R+"}, {"id": "command-r", "name": "Command R"}]'),
('prov-cohere-2', 'cohere', 'Cohere Embeddings', true, 10, '{"type": "api_key", "api_key": "cohere-key-222"}', '[{"id": "embed-english-v3.0", "name": "Embed English v3.0"}, {"id": "embed-multilingual-v3.0", "name": "Embed Multilingual v3.0"}]'),

-- Groq
('prov-groq-1', 'groq', 'Groq Fast Inference', true, 8, '{"type": "api_key", "api_key": "gsk-groq-key-333"}', '[{"id": "llama3-70b-8192", "name": "Llama 3 70B"}, {"id": "mixtral-8x7b-32768", "name": "Mixtral 8x7B"}]'),

-- Perplexity
('prov-perplexity-1', 'perplexity', 'Perplexity Online', true, 5, '{"type": "api_key", "api_key": "pplx-key-444"}', '[{"id": "llama-3-sonar-large-32k-online", "name": "Sonar Large Online"}, {"id": "llama-3-sonar-small-32k-chat", "name": "Sonar Small Chat"}]'),

-- Together AI
('prov-together-1', 'together', 'Together AI', true, 6, '{"type": "api_key", "api_key": "together-key-555"}', '[{"id": "meta-llama/Llama-3-70b-chat-hf", "name": "Llama 3 70B"}, {"id": "Qwen/Qwen1.5-72B-Chat", "name": "Qwen 1.5 72B"}]'),

-- Azure OpenAI
('prov-azure-1', 'azure-openai', 'Azure East US', true, 9, '{"type": "azure", "endpoint": "https://barq-eastus.openai.azure.com", "api_key": "azure-key-666"}', '[{"id": "gpt-4", "name": "GPT-4 (Azure)"}, {"id": "gpt-35-turbo", "name": "GPT-3.5 Turbo (Azure)"}]')

ON CONFLICT (id) DO NOTHING;

-- 3. Applications
INSERT INTO applications (id, name, description, api_key_hash, api_key_prefix, scopes, rate_limit, status, requests_today, created_at) VALUES
('app-prod', 'Production Web App', 'Main customer-facing application', 'hash123', 'sk-barq-prod', '["llm:chat", "embedding:create"]', 5000, 'active', 15430, NOW() - INTERVAL '60 days'),
('app-staging', 'Staging Environment', 'Pre-prod testing environment', 'hash456', 'sk-barq-stg', '["llm:chat"]', 1000, 'active', 230, NOW() - INTERVAL '45 days'),
('app-dev', 'Dev Local', 'Local development keys', 'hash789', 'sk-barq-dev', '["*"]', 100, 'active', 45, NOW() - INTERVAL '10 days'),
('app-mobile', 'Mobile App (iOS)', 'iOS Client Application', 'hash111', 'sk-barq-ios', '["llm:chat"]', 2000, 'active', 8540, NOW() - INTERVAL '20 days'),
('app-analytics', 'Analytics Worker', 'Background job for processing analytics', 'hash222', 'sk-barq-job', '["embedding:create"]', 500, 'active', 1200, NOW() - INTERVAL '30 days')
ON CONFLICT (id) DO NOTHING;

-- 4. Audit Logs (Generate some recent activity)
INSERT INTO audit_logs (user_id, user_name, action, resource, resource_id, details, ip_address, timestamp) VALUES
('admin-001', 'Admin', 'create', 'application', 'app-dev', '{"name": "Dev Local"}', '192.168.1.1', NOW() - INTERVAL '2 hours'),
('user-001', 'Sarah Engineer', 'update', 'provider', 'prov-openai-1', '{"priority": 10}', '192.168.1.5', NOW() - INTERVAL '4 hours'),
('admin-001', 'Admin', 'login', 'auth', NULL, NULL, '192.168.1.1', NOW() - INTERVAL '5 hours'),
('user-002', 'Mike Analyst', 'export', 'billing', NULL, '{"format": "csv"}', '10.0.0.5', NOW() - INTERVAL '1 day'),
('system', 'System', 'rotate_key', 'provider', 'prov-groq-1', '{"reason": "scheduled"}', 'localhost', NOW() - INTERVAL '2 days'),
('admin-001', 'Admin', 'update_settings', 'settings', 'general', NULL, '192.168.1.1', NOW() - INTERVAL '3 days')
ON CONFLICT (id) DO NOTHING;

-- 5. Cost Records (Seed data for graphs)
-- Generate 7 days of cost data
INSERT INTO cost_records (user_id, application_id, provider, model, input_tokens, output_tokens, cost, timestamp)
SELECT 
    'user-001',
    'app-prod',
    'openai',
    'gpt-4-turbo',
    (random() * 10000)::int,
    (random() * 5000)::int,
    (random() * 5)::decimal(10,6),
    NOW() - (n || ' days')::interval
FROM generate_series(0, 30) n;

INSERT INTO cost_records (user_id, application_id, provider, model, input_tokens, output_tokens, cost, timestamp)
SELECT 
    'user-001',
    'app-prod',
    'anthropic',
    'claude-3-opus-20240229',
    (random() * 15000)::int,
    (random() * 8000)::int,
    (random() * 3)::decimal(10,6),
    NOW() - (n || ' days')::interval
FROM generate_series(0, 30) n;

-- 6. Settings
INSERT INTO settings (key, value) VALUES
('general', '{"org_name": "Acme Corp", "maintenance_mode": false, "budget_limit": 5000}'),
('notifications', '{"email_alerts": true, "budget_alerts": true, "security_alerts": true}'),
('smtp', '{"host": "smtp.gmail.com", "port": 587, "username": "notifications@barq.dev", "from_email": "no-reply@barq.dev"}')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;
