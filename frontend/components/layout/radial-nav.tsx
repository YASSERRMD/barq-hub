"use client";

import { useState, useEffect } from "react";
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
    X,
} from "lucide-react";

const navigation = [
    { name: "Dashboard", href: "/", icon: LayoutDashboard, color: "bg-violet-500" },
    { name: "Providers", href: "/providers", icon: Cpu, color: "bg-purple-500" },
    { name: "Applications", href: "/applications", icon: Key, color: "bg-emerald-500" },
    { name: "Playground", href: "/playground", icon: FlaskConical, color: "bg-amber-500" },
    { name: "Billing", href: "/billing", icon: Receipt, color: "bg-pink-500" },
    { name: "Audit", href: "/audit", icon: FileText, color: "bg-cyan-500" },
    { name: "Users", href: "/users", icon: Users, color: "bg-indigo-500" },
    { name: "Roles", href: "/roles", icon: Shield, color: "bg-slate-500" },
    { name: "Health", href: "/health", icon: Activity, color: "bg-green-500" },
    { name: "Settings", href: "/settings", icon: Settings, color: "bg-gray-500" },
];

export function RadialNav() {
    const [isOpen, setIsOpen] = useState(false);
    const pathname = usePathname();

    // Close on escape
    useEffect(() => {
        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === "Escape") setIsOpen(false);
        };
        document.addEventListener("keydown", handleEscape);
        return () => document.removeEventListener("keydown", handleEscape);
    }, []);

    // Close on route change
    useEffect(() => {
        setIsOpen(false);
    }, [pathname]);

    const radius = 140; // Distance from center
    const angleStep = (2 * Math.PI) / navigation.length;

    return (
        <>
            {/* Overlay */}
            {isOpen && (
                <div
                    className="fixed inset-0 z-40 bg-background/80 backdrop-blur-sm"
                    onClick={() => setIsOpen(false)}
                />
            )}

            {/* Radial Menu */}
            <div className="fixed bottom-8 right-8 z-50">
                {/* Navigation Items */}
                <div className="relative">
                    {navigation.map((item, index) => {
                        const angle = angleStep * index - Math.PI / 2; // Start from top
                        const x = Math.cos(angle) * radius;
                        const y = Math.sin(angle) * radius;
                        const isActive = pathname === item.href;

                        return (
                            <Link
                                key={item.name}
                                href={item.href}
                                className={cn(
                                    "absolute flex items-center justify-center w-12 h-12 rounded-full shadow-lg transition-all duration-300 ease-out group",
                                    item.color,
                                    isOpen
                                        ? "opacity-100 scale-100"
                                        : "opacity-0 scale-0",
                                    isActive && "ring-2 ring-white ring-offset-2 ring-offset-background"
                                )}
                                style={{
                                    transform: isOpen
                                        ? `translate(${x}px, ${y}px)`
                                        : "translate(0, 0)",
                                    transitionDelay: isOpen ? `${index * 30}ms` : "0ms",
                                }}
                                onClick={() => setIsOpen(false)}
                            >
                                <item.icon className="w-5 h-5 text-white" />

                                {/* Tooltip */}
                                <span className="absolute right-full mr-3 px-2 py-1 text-xs font-medium text-foreground bg-popover border rounded-md shadow-md opacity-0 group-hover:opacity-100 whitespace-nowrap transition-opacity">
                                    {item.name}
                                </span>
                            </Link>
                        );
                    })}
                </div>

                {/* Center Button */}
                <button
                    onClick={() => setIsOpen(!isOpen)}
                    className={cn(
                        "relative flex items-center justify-center w-14 h-14 rounded-full shadow-xl transition-all duration-300",
                        isOpen
                            ? "bg-foreground text-background rotate-45"
                            : "bg-gradient-to-br from-violet-600 to-cyan-500 text-white"
                    )}
                >
                    {isOpen ? (
                        <X className="w-6 h-6" />
                    ) : (
                        <div className="flex flex-col gap-1">
                            <span className="w-1.5 h-1.5 rounded-full bg-white" />
                            <span className="w-1.5 h-1.5 rounded-full bg-white" />
                            <span className="w-1.5 h-1.5 rounded-full bg-white" />
                        </div>
                    )}

                    {/* Pulse animation when closed */}
                    {!isOpen && (
                        <span className="absolute inset-0 rounded-full bg-gradient-to-br from-violet-600 to-cyan-500 animate-ping opacity-20" />
                    )}
                </button>
            </div>
        </>
    );
}
