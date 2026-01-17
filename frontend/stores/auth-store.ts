import { create } from "zustand";
import { persist } from "zustand/middleware";
import { User } from "@/types";
import { authApi } from "@/lib/api";

interface AuthState {
    user: User | null;
    token: string | null;
    isLoading: boolean;
    isAuthenticated: boolean;

    login: (email: string, password: string) => Promise<void>;
    register: (name: string, email: string, password: string) => Promise<void>;
    logout: () => void;
    checkAuth: () => Promise<void>;
}

export const useAuthStore = create<AuthState>()(
    persist(
        (set, get) => ({
            user: null,
            token: null,
            isLoading: true,
            isAuthenticated: false,

            login: async (email: string, password: string) => {
                const response = await authApi.login(email, password);
                localStorage.setItem("barq_token", response.token);
                set({
                    user: response.user,
                    token: response.token,
                    isAuthenticated: true,
                });
            },

            register: async (name: string, email: string, password: string) => {
                const response = await authApi.register(name, email, password);
                localStorage.setItem("barq_token", response.token);
                set({
                    user: response.user,
                    token: response.token,
                    isAuthenticated: true,
                });
            },

            logout: () => {
                authApi.logout();
                set({
                    user: null,
                    token: null,
                    isAuthenticated: false,
                });
            },

            checkAuth: async () => {
                const token = localStorage.getItem("barq_token");
                if (!token) {
                    set({ isLoading: false, isAuthenticated: false });
                    return;
                }

                try {
                    const user = await authApi.me();
                    set({
                        user,
                        token,
                        isAuthenticated: true,
                        isLoading: false,
                    });
                } catch {
                    localStorage.removeItem("barq_token");
                    set({
                        user: null,
                        token: null,
                        isAuthenticated: false,
                        isLoading: false,
                    });
                }
            },
        }),
        {
            name: "barq-auth",
            partialize: (state) => ({
                user: state.user,
                token: state.token,
                isAuthenticated: state.isAuthenticated,
            }),
        }
    )
);
