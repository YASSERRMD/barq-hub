"use client";

import { FileText, Search, Download, Filter } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";

const logs = [
    { id: "1", action: "user.login", user: "admin@synapse.local", resource: "auth", time: "2 min ago", status: "success" },
    { id: "2", action: "provider.create", user: "admin@synapse.local", resource: "openai", time: "15 min ago", status: "success" },
    { id: "3", action: "document.ingest", user: "dev@example.com", resource: "knowledge", time: "1 hour ago", status: "success" },
    { id: "4", action: "api.request", user: "api-key-xxx", resource: "chat", time: "2 hours ago", status: "error" },
    { id: "5", action: "user.update", user: "admin@synapse.local", resource: "users", time: "3 hours ago", status: "success" },
];

export default function AuditPage() {
    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Audit Logs</h1>
                    <p className="text-muted-foreground">System activity and security logs</p>
                </div>
                <Button variant="outline">
                    <Download className="mr-2 h-4 w-4" />
                    Export
                </Button>
            </div>

            <div className="flex items-center gap-4">
                <div className="relative flex-1 max-w-sm">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <Input placeholder="Search logs..." className="pl-10" />
                </div>
                <Button variant="outline" size="icon">
                    <Filter className="h-4 w-4" />
                </Button>
            </div>

            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <FileText className="h-5 w-5" />
                        Recent Activity
                    </CardTitle>
                </CardHeader>
                <CardContent>
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>Action</TableHead>
                                <TableHead>User</TableHead>
                                <TableHead>Resource</TableHead>
                                <TableHead>Time</TableHead>
                                <TableHead>Status</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {logs.map((log) => (
                                <TableRow key={log.id}>
                                    <TableCell className="font-medium">{log.action}</TableCell>
                                    <TableCell>{log.user}</TableCell>
                                    <TableCell>{log.resource}</TableCell>
                                    <TableCell>{log.time}</TableCell>
                                    <TableCell>
                                        <Badge variant={log.status === "success" ? "default" : "destructive"}>
                                            {log.status}
                                        </Badge>
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </CardContent>
            </Card>
        </div>
    );
}
