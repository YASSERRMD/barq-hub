"use client";

import { Bell, Search, User, LogOut, Settings, Command } from "lucide-react";
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
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";

export function Header() {
    return (
        <header className="sticky top-0 z-40 h-14 flex items-center justify-between px-4 border-b border-border/40 bg-background/80 backdrop-blur-xl">
            {/* Search */}
            <button className="flex items-center gap-2 px-3 py-1.5 text-sm text-muted-foreground rounded-lg border border-border/50 bg-muted/30 hover:bg-muted/50 transition-colors">
                <Search className="w-4 h-4" />
                <span>Search...</span>
                <kbd className="ml-4 pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border border-border/50 bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground">
                    <Command className="w-3 h-3" />K
                </kbd>
            </button>

            {/* Actions */}
            <div className="flex items-center gap-1">
                {/* Notifications */}
                <Button variant="ghost" size="icon" className="relative h-9 w-9 rounded-lg">
                    <Bell className="h-4 w-4" />
                    <span className="absolute top-1.5 right-1.5 w-2 h-2 rounded-full bg-red-500" />
                </Button>

                {/* Theme Toggle */}
                <ThemeToggle />

                {/* User Menu */}
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button variant="ghost" className="relative h-9 w-9 rounded-lg">
                            <Avatar className="h-8 w-8">
                                <AvatarImage src="/images/avatar.png" alt="User" />
                                <AvatarFallback className="bg-gradient-to-br from-violet-600 to-cyan-500 text-white text-xs font-semibold">
                                    AD
                                </AvatarFallback>
                            </Avatar>
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent className="w-56" align="end" forceMount>
                        <DropdownMenuLabel className="font-normal">
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
                        <DropdownMenuItem className="text-destructive focus:text-destructive">
                            <LogOut className="mr-2 h-4 w-4" />
                            Log out
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </div>
        </header>
    );
}
