"use client";

import { usePathname } from "next/navigation";
import { MinimalHeader } from "@/components/layout/minimal-header";

export default function DashboardLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    const pathname = usePathname();
    const isHome = pathname === "/";

    return (
        <div className="min-h-screen bg-gradient-to-br from-background via-background to-muted/30">
            <MinimalHeader showBack={!isHome} />
            <main className="container mx-auto px-6 py-8 max-w-7xl">
                {children}
            </main>
        </div>
    );
}
