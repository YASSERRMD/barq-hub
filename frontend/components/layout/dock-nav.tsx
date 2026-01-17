"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import {
    LayoutDashboard,
    Cpu,
    Key,
    FlaskConical,
    Receipt,
    FileText,
    Users,
    Settings,
} from "lucide-react";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

const navigation = [
    { name: "Dashboard", href: "/", icon: LayoutDashboard },
    { name: "Providers", href: "/providers", icon: Cpu },
    { name: "Applications", href: "/applications", icon: Key },
    { name: "Playground", href: "/playground", icon: FlaskConical },
    { name: "Billing", href: "/billing", icon: Receipt },
    { name: "Audit", href: "/audit", icon: FileText },
    { name: "Users", href: "/users", icon: Users },
    { name: "Settings", href: "/settings", icon: Settings },
];

export function DockNav() {
    const pathname = usePathname();

    return (
        <TooltipProvider delayDuration={0}>
            <nav className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50">
                <div className="flex items-center gap-1 px-3 py-2 rounded-2xl bg-background/80 backdrop-blur-xl border shadow-xl shadow-black/10">
                    {navigation.map((item) => {
                        const isActive = pathname === item.href;
                        return (
                            <Tooltip key={item.name}>
                                <TooltipTrigger asChild>
                                    <Link
                                        href={item.href}
                                        className={cn(
                                            "flex items-center justify-center w-11 h-11 rounded-xl transition-all duration-200",
                                            isActive
                                                ? "bg-gradient-to-br from-violet-600 to-cyan-500 text-white shadow-lg shadow-violet-500/25"
                                                : "text-muted-foreground hover:text-foreground hover:bg-muted"
                                        )}
                                    >
                                        <item.icon className="w-5 h-5" />
                                    </Link>
                                </TooltipTrigger>
                                <TooltipContent side="top" className="font-medium">
                                    {item.name}
                                </TooltipContent>
                            </Tooltip>
                        );
                    })}
                </div>
            </nav>
        </TooltipProvider>
    );
}
