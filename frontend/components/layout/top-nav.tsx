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
    Bell,
    Search,
    User,
    LogOut,
    Command,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { ThemeToggle } from "@/components/theme-toggle";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";

const navigation = [
    { name: "Dashboard", href: "/", icon: LayoutDashboard },
    { name: "Providers", href: "/providers", icon: Cpu },
    { name: "Applications", href: "/applications", icon: Key },
    { name: "Playground", href: "/playground", icon: FlaskConical },
    { name: "Billing", href: "/billing", icon: Receipt },
    { name: "Audit", href: "/audit", icon: FileText },
    { name: "Users", href: "/users", icon: Users },
    { name: "Roles", href: "/roles", icon: Shield },
    { name: "Health", href: "/health", icon: Activity },
    { name: "Settings", href: "/settings", icon: Settings },
];

export function TopNav() {
    const pathname = usePathname();

    return (
        <header className="fixed top-0 left-0 right-0 z-50 h-16 border-b border-border/50 bg-background/95 backdrop-blur-xl supports-[backdrop-filter]:bg-background/60">
            <div className="flex h-full items-center px-6">
                {/* Logo */}
                <Link href="/" className="flex items-center gap-2 mr-8">
                    <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-gradient-to-br from-violet-600 to-cyan-500">
                        <Zap className="w-4 h-4 text-white" />
                    </div>
                    <span className="text-lg font-bold">BARQ</span>
                </Link>

                {/* Navigation */}
                <nav className="flex items-center gap-1">
                    {navigation.slice(0, 6).map((item) => {
                        const isActive = pathname === item.href;
                        return (
                            <Link
                                key={item.name}
                                href={item.href}
                                className={cn(
                                    "flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium transition-colors",
                                    isActive
                                        ? "bg-primary/10 text-primary"
                                        : "text-muted-foreground hover:text-foreground hover:bg-muted"
                                )}
                            >
                                <item.icon className="w-4 h-4" />
                                {item.name}
                            </Link>
                        );
                    })}

                    {/* More dropdown for remaining items */}
                    <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="sm" className="gap-1.5 text-muted-foreground">
                                More
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="start">
                            {navigation.slice(6).map((item) => (
                                <DropdownMenuItem key={item.name} asChild>
                                    <Link href={item.href} className="flex items-center gap-2">
                                        <item.icon className="w-4 h-4" />
                                        {item.name}
                                    </Link>
                                </DropdownMenuItem>
                            ))}
                        </DropdownMenuContent>
                    </DropdownMenu>
                </nav>

                {/* Spacer */}
                <div className="flex-1" />

                {/* Search */}
                <button className="flex items-center gap-2 px-3 py-1.5 mr-2 text-sm text-muted-foreground rounded-lg border border-border/50 bg-muted/30 hover:bg-muted/50 transition-colors">
                    <Search className="w-4 h-4" />
                    <span className="hidden sm:inline">Search...</span>
                    <kbd className="hidden sm:inline-flex ml-2 h-5 items-center gap-1 rounded border border-border/50 bg-muted px-1.5 font-mono text-[10px] font-medium">
                        <Command className="w-3 h-3" />K
                    </kbd>
                </button>

                {/* Actions */}
                <div className="flex items-center gap-1">
                    <Button variant="ghost" size="icon" className="relative h-8 w-8">
                        <Bell className="h-4 w-4" />
                        <span className="absolute top-1 right-1 w-2 h-2 rounded-full bg-red-500" />
                    </Button>

                    <ThemeToggle />

                    <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                            <Button variant="ghost" className="h-8 w-8 rounded-full">
                                <Avatar className="h-7 w-7">
                                    <AvatarFallback className="bg-gradient-to-br from-violet-600 to-cyan-500 text-white text-xs">
                                        AD
                                    </AvatarFallback>
                                </Avatar>
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end" className="w-56">
                            <DropdownMenuLabel>
                                <div className="flex flex-col space-y-1">
                                    <p className="text-sm font-medium">Admin</p>
                                    <p className="text-xs text-muted-foreground">admin@barq.hub</p>
                                </div>
                            </DropdownMenuLabel>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem>
                                <User className="mr-2 h-4 w-4" />
                                Profile
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                <Settings className="mr-2 h-4 w-4" />
                                Settings
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem className="text-destructive">
                                <LogOut className="mr-2 h-4 w-4" />
                                Log out
                            </DropdownMenuItem>
                        </DropdownMenuContent>
                    </DropdownMenu>
                </div>
            </div>
        </header>
    );
}
