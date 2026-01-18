import axios from "axios";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:4000/v1";

export const api = axios.create({
    baseURL: API_URL,
    headers: {
        "Content-Type": "application/json",
    },
});

// Request interceptor - add auth token
api.interceptors.request.use(
    (config) => {
        const token = typeof window !== "undefined" ? localStorage.getItem("barq_token") : null;
        if (token) {
            config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
    },
    (error) => Promise.reject(error)
);

// Response interceptor - handle auth errors
api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            // Token expired or invalid
            if (typeof window !== "undefined") {
                localStorage.removeItem("barq_token");
                localStorage.removeItem("barq_user");
                window.location.href = "/login";
            }
        }
        return Promise.reject(error);
    }
);

// Auth API
export const authApi = {
    login: async (email: string, password: string) => {
        const response = await api.post("/auth/login", { email, password });
        return response.data;
    },
    register: async (name: string, email: string, password: string) => {
        const response = await api.post("/auth/register", { name, email, password });
        return response.data;
    },
    me: async () => {
        const response = await api.get("/auth/me");
        return response.data;
    },
    logout: () => {
        localStorage.removeItem("barq_token");
        localStorage.removeItem("barq_user");
    },
};

// Providers API
export const providersApi = {
    list: async () => {
        const response = await api.get("/provider-accounts/providers");
        return response.data;
    },
    getAccounts: async (providerId: string) => {
        const response = await api.get(`/provider-accounts/${providerId}/accounts`);
        return response.data;
    },
    createAccount: async (data: any) => {
        const response = await api.post("/provider-accounts/accounts", data);
        return response.data;
    },
    updateAccount: async (providerId: string, accountId: string, data: any) => {
        const response = await api.put(`/provider-accounts/${providerId}/accounts/${accountId}`, data);
        return response.data;
    },
    deleteAccount: async (providerId: string, accountId: string) => {
        const response = await api.delete(`/provider-accounts/${providerId}/accounts/${accountId}`);
        return response.data;
    },
    testAccount: async (providerId: string, accountId: string) => {
        const response = await api.post(`/provider-accounts/${providerId}/accounts/${accountId}/test`);
        return response.data;
    },
};

// Applications API
export const applicationsApi = {
    list: async () => {
        const response = await api.get("/applications");
        return response.data;
    },
    create: async (data: any) => {
        const response = await api.post("/applications", data);
        return response.data;
    },
    update: async (id: string, data: any) => {
        const response = await api.put(`/applications/${id}`, data);
        return response.data;
    },
    delete: async (id: string) => {
        const response = await api.delete(`/applications/${id}`);
        return response.data;
    },
    rotateKey: async (id: string) => {
        const response = await api.post(`/applications/${id}/rotate-key`);
        return response.data;
    },
};

// Chat API
export const chatApi = {
    completions: async (messages: any[], model: string, provider: string, options?: any) => {
        const response = await api.post("/chat/completions", {
            model,
            provider,
            messages,
            ...options,
        });
        return response.data;
    },
};

// Embeddings API
export const embeddingsApi = {
    create: async (input: string[], model: string) => {
        const response = await api.post("/embeddings", { input, model });
        return response.data;
    },
};

// Users API
export const usersApi = {
    list: async () => {
        const response = await api.get("/admin/users");
        return response.data;
    },
    get: async (id: string) => {
        const response = await api.get(`/admin/users/${id}`);
        return response.data;
    },
    update: async (id: string, data: any) => {
        const response = await api.put(`/admin/users/${id}`, data);
        return response.data;
    },
    delete: async (id: string) => {
        const response = await api.delete(`/admin/users/${id}`);
        return response.data;
    },
};

// Roles API
export const rolesApi = {
    list: async () => {
        const response = await api.get("/roles");
        return response.data;
    },
    create: async (data: any) => {
        const response = await api.post("/roles", data);
        return response.data;
    },
    update: async (id: string, data: any) => {
        const response = await api.put(`/roles/${id}`, data);
        return response.data;
    },
    delete: async (id: string) => {
        const response = await api.delete(`/roles/${id}`);
        return response.data;
    },
};

// Audit Logs API
export const auditApi = {
    list: async (params?: { page?: number; limit?: number; action?: string }) => {
        const response = await api.get("/audit", { params });
        return response.data;
    },
};

// Health API
export const healthApi = {
    check: async () => {
        const response = await api.get("/admin/health");
        return response.data;
    },
};

// Billing API
export const billingApi = {
    getUsage: async (params?: { start?: string; end?: string }) => {
        const response = await api.get("/costs", { params });
        return response.data;
    },
    getCosts: async (params?: { start?: string; end?: string }) => {
        const response = await api.get("/costs", { params });
        return response.data;
    },
};

// Settings API
export const settingsApi = {
    get: async () => {
        const response = await api.get("/settings");
        return response.data;
    },
    update: async (data: any) => {
        const response = await api.put("/settings", data);
        return response.data;
    },
    getSmtp: async () => {
        const response = await api.get("/settings/smtp");
        return response.data;
    },
    updateSmtp: async (data: any) => {
        const response = await api.put("/settings/smtp", data);
        return response.data;
    },
    testSmtp: async () => {
        const response = await api.post("/settings/smtp/test");
        return response.data;
    },
};

export default api;
