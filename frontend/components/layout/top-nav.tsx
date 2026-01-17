"use client";

import * as React from "react";
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
    Activity,
    Menu,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
    Sheet,
    SheetContent,
    SheetHeader,
    SheetTitle,
    SheetTrigger,
} from "@/components/ui/sheet";
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from "@/components/ui/tooltip";

const navigation = [
    { name: "Dashboard", href: "/", icon: LayoutDashboard },
    { name: "Providers", href: "/providers", icon: Cpu },
    { name: "Applications", href: "/applications", icon: Key },
    { name: "Playground", href: "/playground", icon: FlaskConical },
    { name: "Billing", href: "/billing", icon: Receipt },
    { name: "Audit", href: "/audit", icon: FileText },
    { name: "Users", href: "/users", icon: Users },
    { name: "Health", href: "/health", icon: Activity },
    { name: "Settings", href: "/settings", icon: Settings },
];

export function TopNav() {
    const pathname = usePathname();

    return (
        <>
            {/* Desktop Navigation */}
            <nav className="hidden lg:flex items-center gap-1">
                <TooltipProvider delayDuration={0}>
                    {navigation.map((item) => {
                        const isActive = pathname === item.href;
                        return (
                            <Tooltip key={item.href}>
                                <TooltipTrigger asChild>
                                    <Link
                                        href={item.href}
                                        className={cn(
                                            "flex items-center justify-center p-2 rounded-md transition-colors hover:bg-muted hover:text-foreground group relative",
                                            isActive
                                                ? "bg-muted text-foreground"
                                                : "text-muted-foreground"
                                        )}
                                    >
                                        <item.icon className="w-5 h-5" />
                                        <span className="sr-only">{item.name}</span>
                                        {isActive && (
                                            <span className="absolute -bottom-1 left-1/2 -translate-x-1/2 w-1 h-1 rounded-full bg-violet-500" />
                                        )}
                                    </Link>
                                </TooltipTrigger>
                                <TooltipContent>
                                    <p>{item.name}</p>
                                </TooltipContent>
                            </Tooltip>
                        );
                    })}
                </TooltipProvider>
            </nav>

            {/* Mobile Navigation */}
            <div className="lg:hidden">
                <Sheet>
                    <SheetTrigger asChild>
                        <Button variant="ghost" size="icon" className="md:hidden">
                            <Menu className="h-5 w-5" />
                            <span className="sr-only">Toggle menu</span>
                        </Button>
                    </SheetTrigger>
                    <SheetContent side="left" className="w-[300px] sm:w-[400px] pr-0">
                        <SheetHeader className="mb-6 px-4 text-left">
                            <div className="flex items-center gap-3">
                                <img src="/assets/logo.png" alt="BARQ HUB" className="w-8 h-8 object-contain" />
                                <SheetTitle className="font-bold bg-gradient-to-r from-violet-600 to-cyan-500 bg-clip-text text-transparent">
                                    BARQ HUB
                                </SheetTitle>
                            </div>
                        </SheetHeader>
                        <div className="flex flex-col gap-1 px-2">
                            {navigation.map((item) => {
                                const isActive = pathname === item.href;
                                return (
                                    <Link
                                        key={item.href}
                                        href={item.href}
                                        className={cn(
                                            "flex items-center gap-3 px-3 py-3 rounded-md text-base font-medium transition-colors hover:bg-muted hover:text-foreground",
                                            isActive
                                                ? "bg-muted/50 text-foreground border-l-2 border-violet-500"
                                                : "text-muted-foreground"
                                        )}
                                    >
                                        <item.icon className="w-5 h-5" />
                                        {item.name}
                                    </Link>
                                );
                            })}
                        </div>
                    </SheetContent>
                </Sheet>
            </div>
        </>
    );
}
