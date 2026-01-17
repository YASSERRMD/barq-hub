"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useAuthStore } from "@/stores/auth-store";

export function useAuth() {
    const { user, isAuthenticated, isLoading, login, register, logout, checkAuth } = useAuthStore();
    const router = useRouter();

    useEffect(() => {
        checkAuth();
    }, [checkAuth]);

    const handleLogin = async (email: string, password: string) => {
        await login(email, password);
        router.push("/");
    };

    const handleRegister = async (name: string, email: string, password: string) => {
        await register(name, email, password);
        router.push("/");
    };

    const handleLogout = () => {
        logout();
        router.push("/login");
    };

    return {
        user,
        isAuthenticated,
        isLoading,
        login: handleLogin,
        register: handleRegister,
        logout: handleLogout,
    };
}

export function useRequireAuth() {
    const { isAuthenticated, isLoading } = useAuthStore();
    const router = useRouter();

    useEffect(() => {
        if (!isLoading && !isAuthenticated) {
            router.push("/login");
        }
    }, [isAuthenticated, isLoading, router]);

    return { isAuthenticated, isLoading };
}
