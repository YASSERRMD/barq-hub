"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import {
    LayoutDashboard,
    Cpu,
    Database,
    FlaskConical,
    Receipt,
    FileText,
    Users,
    Shield,
    Activity,
    Settings,
    Zap,
    ChevronLeft,
    ChevronRight,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { useState } from "react";
import { Separator } from "@/components/ui/separator";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";

const navigation = [
    { name: "Dashboard", href: "/", icon: LayoutDashboard },
    { name: "Providers", href: "/providers", icon: Cpu },
    { name: "Vector DBs", href: "/vectordbs", icon: Database },
    { name: "Playground", href: "/playground", icon: FlaskConical },
    { name: "Billing", href: "/billing", icon: Receipt },
    { name: "Audit Logs", href: "/audit", icon: FileText },
    { name: "Users", href: "/users", icon: Users },
    { name: "Roles", href: "/roles", icon: Shield },
    { name: "Health", href: "/health", icon: Activity },
    { name: "Settings", href: "/settings", icon: Settings },
];

export function Sidebar() {
    const pathname = usePathname();
    const [collapsed, setCollapsed] = useState(false);

    return (
        <TooltipProvider delayDuration={0}>
            <aside
                className={cn(
                    "flex flex-col h-screen bg-sidebar border-r border-sidebar-border transition-all duration-300 ease-in-out",
                    collapsed ? "w-16" : "w-64"
                )}
            >
                {/* Logo */}
                <div className="flex items-center h-16 px-4 border-b border-sidebar-border">
                    <Link href="/" className="flex items-center gap-2 overflow-hidden">
                        <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-gradient-to-br from-blue-500 to-cyan-400">
                            <Zap className="w-5 h-5 text-white" />
                        </div>
                        {!collapsed && (
                            <div className="flex flex-col">
                                <span className="text-lg font-bold text-sidebar-foreground tracking-tight">
                                    BARQ HUB
                                </span>
                                <span className="text-[10px] text-muted-foreground -mt-1">
                                    SYNAPSE Brain Console
                                </span>
                            </div>
                        )}
                    </Link>
                </div>

                {/* Navigation */}
                <nav className="flex-1 px-2 py-4 space-y-1 overflow-y-auto">
                    {navigation.map((item) => {
                        const isActive = pathname === item.href;
                        const NavItem = (
                            <Link
                                key={item.name}
                                href={item.href}
                                className={cn(
                                    "flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-200",
                                    isActive
                                        ? "bg-sidebar-accent text-sidebar-accent-foreground shadow-sm"
                                        : "text-sidebar-foreground/70 hover:text-sidebar-foreground hover:bg-sidebar-accent/50"
                                )}
                            >
                                <item.icon className={cn("w-5 h-5 flex-shrink-0", isActive && "text-blue-500")} />
                                {!collapsed && <span>{item.name}</span>}
                            </Link>
                        );

                        if (collapsed) {
                            return (
                                <Tooltip key={item.name}>
                                    <TooltipTrigger asChild>{NavItem}</TooltipTrigger>
                                    <TooltipContent side="right" className="font-medium">
                                        {item.name}
                                    </TooltipContent>
                                </Tooltip>
                            );
                        }

                        return NavItem;
                    })}
                </nav>

                <Separator />

                {/* Collapse Toggle */}
                <div className="p-2">
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setCollapsed(!collapsed)}
                        className="w-full justify-center"
                    >
                        {collapsed ? (
                            <ChevronRight className="w-4 h-4" />
                        ) : (
                            <>
                                <ChevronLeft className="w-4 h-4 mr-2" />
                                <span>Collapse</span>
                            </>
                        )}
                    </Button>
                </div>
            </aside>
        </TooltipProvider>
    );
}
