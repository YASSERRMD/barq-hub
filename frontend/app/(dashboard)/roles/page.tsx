"use client";

import { Shield, Plus, Search, MoreVertical, Check } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";

const roles = [
    {
        id: "admin",
        name: "Admin",
        description: "Full system access",
        users: 1,
        permissions: ["*"]
    },
    {
        id: "developer",
        name: "Developer",
        description: "Agent and knowledge management",
        users: 3,
        permissions: ["agents:*", "knowledge:*", "workflows:*"]
    },
    {
        id: "user",
        name: "User",
        description: "Standard user access",
        users: 12,
        permissions: ["agents:read", "knowledge:read"]
    },
    {
        id: "viewer",
        name: "Viewer",
        description: "Read-only access",
        users: 8,
        permissions: ["agents:read"]
    },
];

export default function RolesPage() {
    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Roles</h1>
                    <p className="text-muted-foreground">Manage roles and permissions</p>
                </div>
                <Button>
                    <Plus className="mr-2 h-4 w-4" />
                    Create Role
                </Button>
            </div>

            <div className="flex items-center gap-4">
                <div className="relative flex-1 max-w-sm">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <Input placeholder="Search roles..." className="pl-10" />
                </div>
            </div>

            <div className="grid gap-4 md:grid-cols-2">
                {roles.map((role) => (
                    <Card key={role.id}>
                        <CardHeader className="flex flex-row items-center justify-between pb-2">
                            <div className="flex items-center gap-3">
                                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                                    <Shield className="h-5 w-5 text-primary" />
                                </div>
                                <div>
                                    <CardTitle className="text-lg">{role.name}</CardTitle>
                                    <CardDescription>{role.description}</CardDescription>
                                </div>
                            </div>
                            <Button variant="ghost" size="icon">
                                <MoreVertical className="h-4 w-4" />
                            </Button>
                        </CardHeader>
                        <CardContent>
                            <div className="flex items-center justify-between mb-3">
                                <span className="text-sm text-muted-foreground">{role.users} users</span>
                            </div>
                            <div className="flex flex-wrap gap-1">
                                {role.permissions.map((perm) => (
                                    <Badge key={perm} variant="outline" className="text-xs gap-1">
                                        <Check className="h-3 w-3" />
                                        {perm}
                                    </Badge>
                                ))}
                            </div>
                        </CardContent>
                    </Card>
                ))}
            </div>
        </div>
    );
}
