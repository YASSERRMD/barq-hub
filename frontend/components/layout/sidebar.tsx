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
    Shield,
    Activity,
    Settings,
    Zap,
    Menu,
    X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { useState } from "react";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

const navigation = [
    { name: "Dashboard", href: "/", icon: LayoutDashboard, color: "text-blue-500" },
    { name: "Providers", href: "/providers", icon: Cpu, color: "text-violet-500" },
    { name: "Applications", href: "/applications", icon: Key, color: "text-emerald-500" },
    { name: "Playground", href: "/playground", icon: FlaskConical, color: "text-amber-500" },
    { name: "Billing", href: "/billing", icon: Receipt, color: "text-pink-500" },
    { name: "Audit Logs", href: "/audit", icon: FileText, color: "text-cyan-500" },
    { name: "Users", href: "/users", icon: Users, color: "text-orange-500" },
    { name: "Roles", href: "/roles", icon: Shield, color: "text-indigo-500" },
    { name: "Health", href: "/health", icon: Activity, color: "text-green-500" },
    { name: "Settings", href: "/settings", icon: Settings, color: "text-slate-500" },
];

export function Sidebar() {
    const pathname = usePathname();
    const [expanded, setExpanded] = useState(true);

    return (
        <TooltipProvider delayDuration={0}>
            <aside
                className={cn(
                    "fixed left-0 top-0 z-50 flex flex-col h-screen border-r border-border/40 bg-gradient-to-b from-background via-background to-muted/20 transition-all duration-300 ease-out",
                    expanded ? "w-56" : "w-[68px]"
                )}
            >
                {/* Header */}
                <div className="flex items-center justify-between h-14 px-3 border-b border-border/40">
                    <Link href="/" className="flex items-center gap-2 overflow-hidden">
                        <div className="flex items-center justify-center w-9 h-9 rounded-xl bg-gradient-to-br from-violet-600 via-blue-600 to-cyan-500 shadow-lg shadow-violet-500/20">
                            <Zap className="w-5 h-5 text-white" />
                        </div>
                        {expanded && (
                            <span className="text-lg font-bold bg-gradient-to-r from-violet-600 to-cyan-500 bg-clip-text text-transparent">
                                BARQ
                            </span>
                        )}
                    </Link>
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => setExpanded(!expanded)}
                        className="h-8 w-8 shrink-0"
                    >
                        {expanded ? <X className="w-4 h-4" /> : <Menu className="w-4 h-4" />}
                    </Button>
                </div>

                {/* Navigation */}
                <nav className="flex-1 px-2 py-3 space-y-0.5 overflow-y-auto">
                    {navigation.map((item) => {
                        const isActive = pathname === item.href;
                        const NavLink = (
                            <Link
                                key={item.name}
                                href={item.href}
                                className={cn(
                                    "group flex items-center gap-3 px-2.5 py-2 rounded-lg text-sm font-medium transition-all duration-200",
                                    isActive
                                        ? "bg-primary/10 text-primary"
                                        : "text-muted-foreground hover:text-foreground hover:bg-muted/60"
                                )}
                            >
                                <div className={cn(
                                    "flex items-center justify-center w-8 h-8 rounded-lg transition-all",
                                    isActive
                                        ? "bg-primary/15"
                                        : "bg-transparent group-hover:bg-muted"
                                )}>
                                    <item.icon className={cn(
                                        "w-[18px] h-[18px] transition-colors",
                                        isActive ? item.color : ""
                                    )} />
                                </div>
                                {expanded && (
                                    <span className="truncate">{item.name}</span>
                                )}
                                {isActive && expanded && (
                                    <div className="ml-auto w-1.5 h-1.5 rounded-full bg-primary" />
                                )}
                            </Link>
                        );

                        if (!expanded) {
                            return (
                                <Tooltip key={item.name}>
                                    <TooltipTrigger asChild>{NavLink}</TooltipTrigger>
                                    <TooltipContent side="right" className="font-medium">
                                        {item.name}
                                    </TooltipContent>
                                </Tooltip>
                            );
                        }

                        return NavLink;
                    })}
                </nav>

                {/* Footer */}
                {expanded && (
                    <div className="px-3 py-3 border-t border-border/40">
                        <div className="flex items-center gap-2 px-2 py-1.5 rounded-lg bg-muted/40">
                            <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                            <span className="text-xs text-muted-foreground">All systems online</span>
                        </div>
                    </div>
                )}
            </aside>
        </TooltipProvider>
    );
}
