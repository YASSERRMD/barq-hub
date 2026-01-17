"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Command, Search, Bell } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { useAuthStore } from "@/stores/auth-store";
import { ModeToggle } from "@/components/mode-toggle";
import { TopNav } from "@/components/layout/top-nav";

interface MinimalHeaderProps {
    showBack?: boolean;
}

export function MinimalHeader({ showBack = false }: MinimalHeaderProps) {
    const pathname = usePathname();
    const { user, logout } = useAuthStore();

    // Get page title from pathname
    const pageTitle = pathname === "/"
        ? "Dashboard"
        : pathname.split("/").pop()?.charAt(0).toUpperCase()! + pathname.split("/").pop()?.slice(1);

    return (
        <header className="sticky top-0 z-40 w-full border-b border-border/40 bg-background/80 backdrop-blur-xl">
            <div className="container mx-auto max-w-7xl h-16 flex items-center justify-between px-6">
                {/* Left: Branding & Title */}
                <div className="flex items-center gap-4">
                    <Link href="/" className="flex items-center gap-3 hover:opacity-80 transition-opacity">
                        <img src="/assets/logo.png" alt="BARQ HUB" className="w-8 h-8 object-contain" />
                        <span className="font-bold hidden md:inline-block bg-gradient-to-r from-violet-600 to-cyan-500 bg-clip-text text-transparent">
                            BARQ HUB
                        </span>
                    </Link>
                    <div className="h-6 w-px bg-border hidden md:block" />

                    {/* Navigation */}
                    <div className="mr-4 flex items-center">
                        <TopNav />
                    </div>
                </div>

                {/* Right: Actions & User */}
                <div className="flex items-center gap-3">
                    <div className="hidden md:flex items-center w-full max-w-[200px] mr-2">
                        <div className="relative w-full">
                            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                            <Input
                                type="search"
                                placeholder="Search..."
                                className="w-full bg-muted/40 pl-9 h-9 border-0 focus-visible:ring-1 focus-visible:ring-violet-500/50"
                            />
                        </div>
                    </div>

                    <ModeToggle />

                    <Button variant="ghost" size="icon" className="text-muted-foreground relative">
                        <Bell className="h-5 w-5" />
                        <span className="absolute top-2 right-2 w-2 h-2 rounded-full bg-red-500" />
                    </Button>

                    <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                            <Button variant="ghost" className="relative h-9 w-9 rounded-full">
                                <Avatar className="h-9 w-9 border border-border">
                                    <AvatarImage src={user?.avatar} alt={user?.name} />
                                    <AvatarFallback>{user?.name?.[0] || "A"}</AvatarFallback>
                                </Avatar>
                            </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent className="w-56" align="end" forceMount>
                            <DropdownMenuLabel className="font-normal">
                                <div className="flex flex-col space-y-1">
                                    <p className="text-sm font-medium leading-none">{user?.name || "Admin User"}</p>
                                    <p className="text-xs leading-none text-muted-foreground">
                                        {user?.email || "admin@barq.hub"}
                                    </p>
                                </div>
                            </DropdownMenuLabel>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem>
                                Profile
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                Settings
                            </DropdownMenuItem>
                            <DropdownMenuItem>
                                Billing
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem className="text-red-500 cursor-pointer" onClick={logout}>
                                Log out
                            </DropdownMenuItem>
                        </DropdownMenuContent>
                    </DropdownMenu>
                </div>
            </div>
        </header>
    );
}
