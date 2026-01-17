// User types
export interface User {
    id: string;
    name: string;
    email: string;
    role: string;
    avatar?: string;
    createdAt: string;
    updatedAt: string;
}

export interface AuthResponse {
    token: string;
    user: User;
}

// Provider types
export interface ProviderDefinition {
    id: string;
    name: string;
    category: "llm_embedding" | "vector_db";
    provider_type: "llm" | "embedding" | "both" | "vector_db";
    requires_azure_config: boolean;
    requires_aws_config: boolean;
    default_models: ProviderModel[];
    supported_quota_periods: string[];
}

export interface ProviderModel {
    id: string;
    name: string;
    capabilities: string[];
}

export interface ProviderQuota {
    requests_per_minute?: number;
    requests_per_hour?: number;
    requests_per_day?: number;
    requests_per_month?: number;
    tokens_per_minute?: number;
    tokens_per_hour?: number;
    tokens_per_day?: number;
    tokens_per_month?: number;
}

export interface ProviderPricing {
    input_token_price: number; // Price per 1M tokens
    output_token_price: number; // Price per 1M tokens
    currency: string;
}

export interface ProviderAccount {
    id: string;
    providerId: string;
    name: string;
    type: "api_key" | "azure" | "aws";
    enabled: boolean;
    isDefault: boolean;
    priority: number;
    quotas?: ProviderQuota;
    pricing?: ProviderPricing;
    models: ProviderModel[];
    createdAt: string;
    updatedAt: string;
}

// Application types
export interface Application {
    id: string;
    name: string;
    description: string;
    apiKeyPrefix: string;
    apiKey?: string;
    scopes: string[];
    rateLimit: number;
    status: "active" | "suspended" | "expired";
    requestsToday: number;
    lastUsed?: string;
    expiresAt?: string;
    createdAt: string;
    updatedAt: string;
}

export interface CreateApplicationRequest {
    name: string;
    description?: string;
    scopes: string[];
    rateLimit?: number;
    expiresAt?: string;
}

// Role types
export interface Role {
    id: string;
    name: string;
    description: string;
    permissions: string[];
    userCount: number;
    createdAt: string;
    updatedAt: string;
}

// Audit types
export interface AuditLog {
    id: string;
    userId: string;
    userName: string;
    action: string;
    resource: string;
    resourceId?: string;
    details?: Record<string, any>;
    ipAddress: string;
    timestamp: string;
}

// Health types
export interface HealthStatus {
    status: "healthy" | "degraded" | "unhealthy";
    services: {
        name: string;
        status: "up" | "down";
        latency?: number;
        lastCheck: string;
    }[];
    uptime: number;
    version: string;
}

// Billing types
export interface BillingUsage {
    totalCost: number;
    totalRequests: number;
    byProvider: {
        provider: string;
        cost: number;
        requests: number;
    }[];
    byDay: {
        date: string;
        cost: number;
        requests: number;
    }[];
}

// Settings types
export interface Settings {
    theme: "light" | "dark" | "system";
    emailNotifications: boolean;
    budgetAlerts: boolean;
    securityAlerts: boolean;
    budgetLimit?: number;
}

export interface SmtpSettings {
    host: string;
    port: number;
    username: string;
    password: string;
    fromEmail: string;
    fromName: string;
    useTls: boolean;
}

// Chat types
export interface Message {
    role: "system" | "user" | "assistant";
    content: string;
}

export interface ChatRequest {
    model: string;
    provider: string;
    messages: Message[];
    temperature?: number;
    maxTokens?: number;
    stream?: boolean;
}

export interface ChatResponse {
    id: string;
    model: string;
    choices: {
        index: number;
        message: Message;
        finishReason: string;
    }[];
    usage: {
        promptTokens: number;
        completionTokens: number;
        totalTokens: number;
    };
}

// Embedding types
export interface EmbeddingRequest {
    input: string[];
    model: string;
}

export interface EmbeddingResponse {
    data: {
        index: number;
        embedding: number[];
    }[];
    model: string;
    usage: {
        promptTokens: number;
        totalTokens: number;
    };
}
