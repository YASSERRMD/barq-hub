-- SYNAPSE Brain Database Initialization
-- Updated schema for full persistence

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- USERS & AUTHENTICATION
-- ============================================================================

CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(100) PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) DEFAULT 'user',
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login TIMESTAMP WITH TIME ZONE
);

CREATE TABLE IF NOT EXISTS sessions (
    id VARCHAR(100) PRIMARY KEY,
    token VARCHAR(255) UNIQUE NOT NULL,
    user_id VARCHAR(100) REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS api_keys (
    id VARCHAR(100) PRIMARY KEY,
    user_id VARCHAR(100) REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL,
    prefix VARCHAR(50) NOT NULL,
    scopes JSONB DEFAULT '[]',
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_used TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE
);

-- ============================================================================
-- APPLICATIONS (External Service API Keys)
-- ============================================================================

CREATE TABLE IF NOT EXISTS applications (
    id VARCHAR(100) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    api_key_hash VARCHAR(255) NOT NULL,
    api_key_prefix VARCHAR(50) NOT NULL,
    scopes JSONB DEFAULT '[]',
    rate_limit INTEGER DEFAULT 100,
    status VARCHAR(50) DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_used TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    requests_today BIGINT DEFAULT 0,
    requests_reset_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================================================
-- ROLES & PERMISSIONS
-- ============================================================================

CREATE TABLE IF NOT EXISTS roles (
    id VARCHAR(100) PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    permissions JSONB DEFAULT '[]',
    is_system BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id VARCHAR(100) REFERENCES users(id) ON DELETE CASCADE,
    role_id VARCHAR(100) REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

-- ============================================================================
-- PROVIDER ACCOUNTS (Multi-tier quota system)
-- ============================================================================

CREATE TABLE IF NOT EXISTS provider_accounts (
    id VARCHAR(100) PRIMARY KEY,
    provider_id VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    api_key_encrypted TEXT NOT NULL,
    enabled BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    priority INTEGER DEFAULT 0,
    
    -- Azure/Bedrock specific
    endpoint TEXT,
    region VARCHAR(100),
    deployment_name VARCHAR(255),
    
    -- Multi-tier quota configuration (JSONB for flexibility)
    quota_config JSONB DEFAULT '{}',
    -- Example: {"minute": {"limit": 100, "used": 0, "reset_at": "..."}, "hour": {...}}
    
    models JSONB DEFAULT '[]',
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================================================
-- AGENTS
-- ============================================================================

CREATE TABLE IF NOT EXISTS agents (
    id VARCHAR(100) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    system_prompt TEXT,
    
    -- Provider configurations (JSONB)
    llm_config JSONB NOT NULL DEFAULT '{}',
    embedding_config JSONB DEFAULT '{}',
    vector_db_config JSONB DEFAULT '{}',
    
    -- Knowledge
    knowledge_collection VARCHAR(255),
    
    -- Status
    enabled BOOLEAN DEFAULT true,
    status VARCHAR(50) DEFAULT 'active',
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by VARCHAR(100) REFERENCES users(id) ON DELETE SET NULL
);

-- ============================================================================
-- WORKFLOWS
-- ============================================================================

CREATE TABLE IF NOT EXISTS workflows (
    id VARCHAR(100) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    nodes JSONB DEFAULT '[]',
    edges JSONB DEFAULT '[]',
    status VARCHAR(50) DEFAULT 'draft',
    version INTEGER DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by VARCHAR(100) REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS workflow_executions (
    id VARCHAR(100) PRIMARY KEY,
    workflow_id VARCHAR(100) REFERENCES workflows(id) ON DELETE CASCADE,
    status VARCHAR(50) DEFAULT 'pending',
    variables JSONB DEFAULT '{}',
    results JSONB DEFAULT '{}',
    error TEXT,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    started_by VARCHAR(100) REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS node_executions (
    id VARCHAR(100) PRIMARY KEY,
    execution_id VARCHAR(100) REFERENCES workflow_executions(id) ON DELETE CASCADE,
    node_id VARCHAR(255) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    input JSONB DEFAULT '{}',
    output JSONB DEFAULT '{}',
    error TEXT,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE
);

-- ============================================================================
-- KNOWLEDGE BASE
-- ============================================================================

CREATE TABLE IF NOT EXISTS knowledge_collections (
    id VARCHAR(100) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    agent_id VARCHAR(100) REFERENCES agents(id) ON DELETE CASCADE,
    document_count INTEGER DEFAULT 0,
    chunk_count INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS knowledge_documents (
    id VARCHAR(100) PRIMARY KEY,
    collection_id VARCHAR(100) REFERENCES knowledge_collections(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    content_type VARCHAR(100) DEFAULT 'text/plain',
    file_size BIGINT DEFAULT 0,
    chunk_count INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_by VARCHAR(100) REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS knowledge_chunks (
    id VARCHAR(100) PRIMARY KEY,
    document_id VARCHAR(100) REFERENCES knowledge_documents(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    embedding_vector BYTEA, -- Store as binary, actual vector in Qdrant
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================================================
-- COST TRACKING & BILLING
-- ============================================================================

CREATE TABLE IF NOT EXISTS cost_entries (
    id VARCHAR(100) PRIMARY KEY,
    user_id VARCHAR(100) REFERENCES users(id) ON DELETE SET NULL,
    agent_id VARCHAR(100) REFERENCES agents(id) ON DELETE SET NULL,
    application_id VARCHAR(100) REFERENCES applications(id) ON DELETE SET NULL,
    provider VARCHAR(100) NOT NULL,
    model VARCHAR(255) NOT NULL,
    prompt_tokens INTEGER DEFAULT 0,
    completion_tokens INTEGER DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    cost DECIMAL(12, 8) DEFAULT 0,
    request_id VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS budgets (
    id VARCHAR(100) PRIMARY KEY,
    entity_type VARCHAR(50) NOT NULL, -- 'user', 'agent', 'application'
    entity_id VARCHAR(100) NOT NULL,
    monthly_limit DECIMAL(12, 4) NOT NULL,
    current_spend DECIMAL(12, 4) DEFAULT 0,
    enforce BOOLEAN DEFAULT true,
    reset_day INTEGER DEFAULT 1, -- Day of month to reset
    last_reset TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(entity_type, entity_id)
);

-- ============================================================================
-- AUDIT LOGS
-- ============================================================================

CREATE TABLE IF NOT EXISTS audit_logs (
    id VARCHAR(100) PRIMARY KEY,
    user_id VARCHAR(100) REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    resource_id VARCHAR(100),
    details JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================================================
-- TTS (Text-to-Speech) PROVIDERS
-- ============================================================================

CREATE TABLE IF NOT EXISTS tts_providers (
    id VARCHAR(100) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    provider_type VARCHAR(50) NOT NULL, -- elevenlabs, openai, google, minimax, azure
    description TEXT,
    api_base_url TEXT,
    
    -- Features
    supports_streaming BOOLEAN DEFAULT false,
    supports_realtime BOOLEAN DEFAULT false,
    supports_custom_voices BOOLEAN DEFAULT false,
    supported_languages JSONB DEFAULT '[]',
    
    -- Pricing (per 1M characters)
    price_per_million_chars DECIMAL(12, 4) DEFAULT 0,
    
    -- Status
    enabled BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tts_accounts (
    id VARCHAR(100) PRIMARY KEY,
    provider_id VARCHAR(100) REFERENCES tts_providers(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    api_key_encrypted TEXT NOT NULL,
    
    -- Provider-specific configuration
    endpoint TEXT,
    region VARCHAR(100),
    project_id VARCHAR(255),
    
    -- Quota configuration
    quota_config JSONB DEFAULT '{}',
    -- Example: {"day": {"limit": 1000000, "used": 0, "reset_at": null}}
    
    -- Status
    enabled BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    priority INTEGER DEFAULT 0,
    
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tts_voices (
    id VARCHAR(100) PRIMARY KEY,
    provider_id VARCHAR(100) REFERENCES tts_providers(id) ON DELETE CASCADE,
    voice_id VARCHAR(255) NOT NULL, -- Provider's voice ID
    name VARCHAR(255) NOT NULL,
    language VARCHAR(10) NOT NULL, -- e.g., en-US, ar-SA
    gender VARCHAR(20), -- male, female, neutral
    age VARCHAR(20), -- young, middle, senior
    style VARCHAR(100), -- conversational, news, narrative
    description TEXT,
    preview_url TEXT,
    
    -- Quality metrics
    quality_tier VARCHAR(20) DEFAULT 'standard', -- standard, premium, neural
    latency_ms INTEGER,
    
    -- Usage tracking
    usage_count BIGINT DEFAULT 0,
    
    is_custom BOOLEAN DEFAULT false,
    enabled BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================================================
-- STT (Speech-to-Text) PROVIDERS
-- ============================================================================

CREATE TABLE IF NOT EXISTS stt_providers (
    id VARCHAR(100) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    provider_type VARCHAR(50) NOT NULL, -- deepgram, openai, assemblyai, google, minimax
    description TEXT,
    api_base_url TEXT,
    
    -- Features
    supports_streaming BOOLEAN DEFAULT false,
    supports_realtime BOOLEAN DEFAULT false,
    supports_diarization BOOLEAN DEFAULT false,
    supports_punctuation BOOLEAN DEFAULT true,
    supported_languages JSONB DEFAULT '[]',
    
    -- Performance metrics
    word_error_rate DECIMAL(5, 2), -- WER percentage
    typical_latency_ms INTEGER,
    
    -- Pricing (per hour of audio)
    price_per_hour DECIMAL(12, 4) DEFAULT 0,
    
    -- Status
    enabled BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS stt_accounts (
    id VARCHAR(100) PRIMARY KEY,
    provider_id VARCHAR(100) REFERENCES stt_providers(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    api_key_encrypted TEXT NOT NULL,
    
    -- Provider-specific configuration
    endpoint TEXT,
    region VARCHAR(100),
    project_id VARCHAR(255),
    
    -- Quota configuration
    quota_config JSONB DEFAULT '{}',
    -- Example: {"day": {"limit": 36000, "used": 0, "reset_at": null}} -- seconds
    
    -- Status
    enabled BOOLEAN DEFAULT true,
    is_default BOOLEAN DEFAULT false,
    priority INTEGER DEFAULT 0,
    
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS stt_models (
    id VARCHAR(100) PRIMARY KEY,
    provider_id VARCHAR(100) REFERENCES stt_providers(id) ON DELETE CASCADE,
    model_id VARCHAR(255) NOT NULL, -- Provider's model ID
    name VARCHAR(255) NOT NULL,
    languages JSONB DEFAULT '[]', -- Supported languages
    
    -- Performance metrics
    word_error_rate DECIMAL(5, 2),
    latency_ms INTEGER,
    
    -- Features
    supports_streaming BOOLEAN DEFAULT false,
    supports_diarization BOOLEAN DEFAULT false,
    max_audio_length_seconds INTEGER,
    
    -- Quality tier
    quality_tier VARCHAR(20) DEFAULT 'standard',
    
    enabled BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================================================
-- VOICE USAGE TRACKING
-- ============================================================================

CREATE TABLE IF NOT EXISTS voice_usage (
    id VARCHAR(100) PRIMARY KEY,
    user_id VARCHAR(100) REFERENCES users(id) ON DELETE SET NULL,
    application_id VARCHAR(100) REFERENCES applications(id) ON DELETE SET NULL,
    
    -- Type: tts or stt
    usage_type VARCHAR(10) NOT NULL,
    provider_id VARCHAR(100) NOT NULL,
    model_or_voice_id VARCHAR(255),
    
    -- Metrics
    characters_processed INTEGER DEFAULT 0, -- For TTS
    audio_duration_seconds INTEGER DEFAULT 0, -- For STT
    cost DECIMAL(12, 8) DEFAULT 0,
    
    -- Request details
    request_id VARCHAR(255),
    language VARCHAR(10),
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================================================
-- WEBRTC SESSIONS
-- ============================================================================

CREATE TABLE IF NOT EXISTS webrtc_sessions (
    id VARCHAR(100) PRIMARY KEY,
    user_id VARCHAR(100) REFERENCES users(id) ON DELETE SET NULL,
    
    -- Session type: voice_chat, transcription, tts_preview
    session_type VARCHAR(50) NOT NULL,
    
    -- Provider info
    stt_provider_id VARCHAR(100),
    tts_provider_id VARCHAR(100),
    stt_model_id VARCHAR(255),
    tts_voice_id VARCHAR(255),
    
    -- Status
    status VARCHAR(50) DEFAULT 'pending', -- pending, active, completed, failed
    
    -- Metrics
    started_at TIMESTAMP WITH TIME ZONE,
    ended_at TIMESTAMP WITH TIME ZONE,
    total_audio_seconds INTEGER DEFAULT 0,
    total_characters INTEGER DEFAULT 0,
    
    -- Connection info
    ice_servers JSONB DEFAULT '[]',
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================================================
-- INDEXES
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON api_keys(prefix);
CREATE INDEX IF NOT EXISTS idx_applications_status ON applications(status);
CREATE INDEX IF NOT EXISTS idx_provider_accounts_provider ON provider_accounts(provider_id);
CREATE INDEX IF NOT EXISTS idx_provider_accounts_default ON provider_accounts(is_default);
CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);
CREATE INDEX IF NOT EXISTS idx_agents_created_by ON agents(created_by);
CREATE INDEX IF NOT EXISTS idx_workflows_status ON workflows(status);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow ON workflow_executions(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX IF NOT EXISTS idx_knowledge_collections_agent ON knowledge_collections(agent_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_documents_collection ON knowledge_documents(collection_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_chunks_document ON knowledge_chunks(document_id);
CREATE INDEX IF NOT EXISTS idx_cost_entries_user ON cost_entries(user_id);
CREATE INDEX IF NOT EXISTS idx_cost_entries_created ON cost_entries(created_at);
CREATE INDEX IF NOT EXISTS idx_cost_entries_provider ON cost_entries(provider);
CREATE INDEX IF NOT EXISTS idx_budgets_entity ON budgets(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created ON audit_logs(created_at);

-- TTS/STT indexes
CREATE INDEX IF NOT EXISTS idx_tts_providers_type ON tts_providers(provider_type);
CREATE INDEX IF NOT EXISTS idx_tts_accounts_provider ON tts_accounts(provider_id);
CREATE INDEX IF NOT EXISTS idx_tts_voices_provider ON tts_voices(provider_id);
CREATE INDEX IF NOT EXISTS idx_tts_voices_language ON tts_voices(language);
CREATE INDEX IF NOT EXISTS idx_stt_providers_type ON stt_providers(provider_type);
CREATE INDEX IF NOT EXISTS idx_stt_accounts_provider ON stt_accounts(provider_id);
CREATE INDEX IF NOT EXISTS idx_stt_models_provider ON stt_models(provider_id);
CREATE INDEX IF NOT EXISTS idx_voice_usage_user ON voice_usage(user_id);
CREATE INDEX IF NOT EXISTS idx_voice_usage_type ON voice_usage(usage_type);
CREATE INDEX IF NOT EXISTS idx_voice_usage_created ON voice_usage(created_at);
CREATE INDEX IF NOT EXISTS idx_webrtc_sessions_user ON webrtc_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_webrtc_sessions_status ON webrtc_sessions(status);

-- ============================================================================
-- DEFAULT DATA
-- ============================================================================

-- Insert default roles
INSERT INTO roles (id, name, description, permissions, is_system) VALUES
    ('role-admin', 'admin', 'Full system access', '["*"]', true),
    ('role-developer', 'developer', 'Agent and knowledge management', '["agents:*", "knowledge:*", "workflows:*", "providers:read"]', true),
    ('role-user', 'user', 'Standard user access', '["agents:read", "knowledge:read", "workflows:read"]', true),
    ('role-viewer', 'viewer', 'Read-only access', '["agents:read", "knowledge:read"]', true)
ON CONFLICT (id) DO NOTHING;

-- Insert default admin user (password: admin123 - hashed with simple hash for dev)
INSERT INTO users (id, email, name, password_hash, role, enabled) VALUES
    ('admin-001', 'admin@synapse.local', 'Admin', '0192023a7bbd73250516f069df18b500', 'admin', true)
ON CONFLICT (id) DO NOTHING;

-- Insert sample provider accounts
INSERT INTO provider_accounts (id, provider_id, name, api_key_encrypted, enabled, is_default, models, quota_config) VALUES
    ('acc-openai-001', 'openai', 'OpenAI Primary', 'encrypted:placeholder', true, true, 
     '["gpt-4o", "gpt-4-turbo", "gpt-3.5-turbo"]',
     '{"day": {"limit": 100000, "used": 0, "reset_at": null}}'),
    ('acc-anthropic-001', 'anthropic', 'Anthropic Primary', 'encrypted:placeholder', true, true,
     '["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"]',
     '{"day": {"limit": 100000, "used": 0, "reset_at": null}}')
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- TTS PROVIDERS (Top 5 for 2026)
-- ============================================================================

INSERT INTO tts_providers (id, name, provider_type, description, api_base_url, supports_streaming, supports_realtime, supports_custom_voices, supported_languages, price_per_million_chars, enabled, is_default) VALUES
    ('tts-elevenlabs', 'ElevenLabs', 'elevenlabs', 
     'Most realistic voices, 1000+ options, emotional prosody',
     'https://api.elevenlabs.io/v1',
     true, true, true, 
     '["en", "es", "fr", "de", "it", "pt", "pl", "hi", "ar", "zh", "ja", "ko"]',
     30.00, true, true),
    
    ('tts-openai', 'OpenAI TTS', 'openai',
     'Natural conversational flow, multilingual excellence',
     'https://api.openai.com/v1',
     true, true, false,
     '["en", "es", "fr", "de", "it", "pt", "ru", "zh", "ja", "ko", "ar", "hi"]',
     15.00, true, false),
    
    ('tts-google', 'Google Cloud TTS', 'google',
     'Broadcast-quality, 125+ languages, WaveNet/Neural2',
     'https://texttospeech.googleapis.com/v1',
     true, false, true,
     '["en", "es", "fr", "de", "it", "pt", "ru", "zh", "ja", "ko", "ar", "hi", "nl", "sv", "da", "no", "fi"]',
     16.00, true, false),
    
    ('tts-minimax', 'MiniMax Talkie', 'minimax',
     'Ultra-low latency, Mandarin/Arabic leader',
     'https://api.minimax.chat/v1',
     true, true, false,
     '["zh", "ar", "en", "es", "fr", "de", "ja", "ko"]',
     8.00, true, false),
    
    ('tts-azure', 'Azure Neural TTS', 'azure',
     'Enterprise-grade consistency, custom voices',
     'https://{region}.tts.speech.microsoft.com',
     true, true, true,
     '["en", "es", "fr", "de", "it", "pt", "ru", "zh", "ja", "ko", "ar", "hi", "nl", "sv"]',
     16.00, true, false)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- STT PROVIDERS (Top 5 for 2026)
-- ============================================================================

INSERT INTO stt_providers (id, name, provider_type, description, api_base_url, supports_streaming, supports_realtime, supports_diarization, supported_languages, word_error_rate, typical_latency_ms, price_per_hour, enabled, is_default) VALUES
    ('stt-deepgram', 'Deepgram Nova-3', 'deepgram',
     '11-14% WER, <300ms real-time streaming',
     'https://api.deepgram.com/v1',
     true, true, true,
     '["en", "es", "fr", "de", "it", "pt", "ru", "zh", "ja", "ko", "ar", "hi", "nl"]',
     12.5, 280, 0.25, true, true),
    
    ('stt-openai', 'OpenAI Whisper v3', 'openai',
     '11.6% WER, 100+ languages, noise robust',
     'https://api.openai.com/v1',
     true, true, false,
     '["en", "es", "fr", "de", "it", "pt", "ru", "zh", "ja", "ko", "ar", "hi", "nl", "sv", "da", "no", "fi", "pl", "cs", "hu"]',
     11.6, 500, 0.006, true, false),
    
    ('stt-assemblyai', 'AssemblyAI Universal-2', 'assemblyai',
     '14.5% WER + diarization/summaries',
     'https://api.assemblyai.com/v2',
     true, true, true,
     '["en", "es", "fr", "de", "it", "pt", "nl"]',
     14.5, 400, 0.37, true, false),
    
    ('stt-google', 'Google Cloud Chirp', 'google',
     '11.6% WER, enterprise scale',
     'https://speech.googleapis.com/v1',
     true, true, true,
     '["en", "es", "fr", "de", "it", "pt", "ru", "zh", "ja", "ko", "ar", "hi", "nl", "sv", "da", "no", "fi"]',
     11.6, 350, 0.024, true, false),
    
    ('stt-minimax', 'MiniMax STT', 'minimax',
     'Mandarin/Arabic specialist, cost-effective streaming',
     'https://api.minimax.chat/v1',
     true, true, false,
     '["zh", "ar", "en", "es", "fr", "de"]',
     10.0, 200, 0.15, true, false)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- TTS VOICES (Sample voices for each provider)
-- ============================================================================

INSERT INTO tts_voices (id, provider_id, voice_id, name, language, gender, style, quality_tier, enabled) VALUES
    -- ElevenLabs
    ('voice-el-rachel', 'tts-elevenlabs', 'rachel', 'Rachel', 'en-US', 'female', 'conversational', 'premium', true),
    ('voice-el-josh', 'tts-elevenlabs', 'josh', 'Josh', 'en-US', 'male', 'narrative', 'premium', true),
    ('voice-el-bella', 'tts-elevenlabs', 'bella', 'Bella', 'en-GB', 'female', 'conversational', 'premium', true),
    ('voice-el-adam', 'tts-elevenlabs', 'adam', 'Adam', 'en-US', 'male', 'news', 'premium', true),
    
    -- OpenAI
    ('voice-oai-alloy', 'tts-openai', 'alloy', 'Alloy', 'en-US', 'neutral', 'conversational', 'neural', true),
    ('voice-oai-echo', 'tts-openai', 'echo', 'Echo', 'en-US', 'male', 'conversational', 'neural', true),
    ('voice-oai-fable', 'tts-openai', 'fable', 'Fable', 'en-GB', 'female', 'narrative', 'neural', true),
    ('voice-oai-nova', 'tts-openai', 'nova', 'Nova', 'en-US', 'female', 'conversational', 'neural', true),
    ('voice-oai-shimmer', 'tts-openai', 'shimmer', 'Shimmer', 'en-US', 'female', 'soft', 'neural', true),
    
    -- Google
    ('voice-gc-wavenet-a', 'tts-google', 'en-US-Wavenet-A', 'WaveNet A', 'en-US', 'male', 'standard', 'neural', true),
    ('voice-gc-wavenet-c', 'tts-google', 'en-US-Wavenet-C', 'WaveNet C', 'en-US', 'female', 'standard', 'neural', true),
    ('voice-gc-neural2-a', 'tts-google', 'en-US-Neural2-A', 'Neural2 A', 'en-US', 'male', 'conversational', 'neural', true),
    ('voice-gc-ar', 'tts-google', 'ar-XA-Wavenet-A', 'Arabic WaveNet', 'ar-XA', 'female', 'standard', 'neural', true),
    
    -- MiniMax
    ('voice-mm-alice', 'tts-minimax', 'alice', 'Alice', 'en-US', 'female', 'conversational', 'premium', true),
    ('voice-mm-xiaoming', 'tts-minimax', 'xiaoming', 'Xiaoming', 'zh-CN', 'male', 'conversational', 'premium', true),
    ('voice-mm-fatima', 'tts-minimax', 'fatima', 'Fatima', 'ar-SA', 'female', 'conversational', 'premium', true),
    
    -- Azure
    ('voice-az-jenny', 'tts-azure', 'en-US-JennyNeural', 'Jenny', 'en-US', 'female', 'conversational', 'neural', true),
    ('voice-az-guy', 'tts-azure', 'en-US-GuyNeural', 'Guy', 'en-US', 'male', 'news', 'neural', true),
    ('voice-az-aria', 'tts-azure', 'en-US-AriaNeural', 'Aria', 'en-US', 'female', 'expressive', 'neural', true)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- STT MODELS
-- ============================================================================

INSERT INTO stt_models (id, provider_id, model_id, name, languages, word_error_rate, latency_ms, supports_streaming, supports_diarization, quality_tier, enabled) VALUES
    -- Deepgram
    ('model-dg-nova3', 'stt-deepgram', 'nova-3', 'Nova 3', '["en", "es", "fr", "de", "it", "pt", "zh", "ja", "ko"]', 11.0, 280, true, true, 'premium', true),
    ('model-dg-nova2', 'stt-deepgram', 'nova-2', 'Nova 2', '["en", "es", "fr", "de", "it", "pt"]', 14.0, 300, true, true, 'standard', true),
    ('model-dg-whisper', 'stt-deepgram', 'whisper-large', 'Whisper Large', '["en", "es", "fr", "de", "it", "pt", "ru", "zh", "ja"]', 12.0, 500, true, false, 'premium', true),
    
    -- OpenAI
    ('model-oai-whisper3', 'stt-openai', 'whisper-1', 'Whisper v3', '["en", "es", "fr", "de", "it", "pt", "ru", "zh", "ja", "ko", "ar", "hi"]', 11.6, 500, true, false, 'premium', true),
    
    -- AssemblyAI
    ('model-aai-best', 'stt-assemblyai', 'best', 'Best', '["en"]', 14.5, 400, true, true, 'premium', true),
    ('model-aai-nano', 'stt-assemblyai', 'nano', 'Nano (Fast)', '["en"]', 18.0, 150, true, false, 'standard', true),
    
    -- Google
    ('model-gc-chirp', 'stt-google', 'chirp', 'Chirp', '["en", "es", "fr", "de", "it", "pt", "ru", "zh", "ja"]', 11.6, 350, true, true, 'premium', true),
    ('model-gc-latest-long', 'stt-google', 'latest_long', 'Latest Long', '["en", "es", "fr", "de", "it", "pt"]', 13.0, 400, true, true, 'standard', true),
    
    -- MiniMax
    ('model-mm-standard', 'stt-minimax', 'speech-01', 'Speech 01', '["zh", "ar", "en"]', 10.0, 200, true, false, 'premium', true)
ON CONFLICT (id) DO NOTHING;

-- Grant permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO synapse;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO synapse;

