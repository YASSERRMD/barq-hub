"use client";

import { useState, useEffect } from "react";
import { Search, Filter, Download, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { auditApi } from "@/lib/api";
import { toast } from "sonner";
import { AuditLog } from "@/types";
import { formatDistanceToNow } from "date-fns";

export default function AuditPage() {
    const [logs, setLogs] = useState<AuditLog[]>([]);
    const [loading, setLoading] = useState(true);
    const [searchTerm, setSearchTerm] = useState("");

    useEffect(() => {
        const fetchLogs = async () => {
            try {
                // In a real app we would pass search/filter params here
                const data = await auditApi.list();
                setLogs(data);
            } catch (error) {
                console.error("Failed to load audit logs", error);
                toast.error("Failed to load audit logs");
            } finally {
                setLoading(false);
            }
        };

        fetchLogs();
    }, []); // Add dependencies when implementing real search

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Audit Logs</h1>
                    <p className="text-muted-foreground">Track system activity and security events</p>
                </div>
                <Button variant="outline">
                    <Download className="mr-2 h-4 w-4" /> Export CSV
                </Button>
            </div>

            {/* Filters */}
            <div className="flex flex-col md:flex-row gap-4">
                <div className="relative flex-1">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <Input
                        placeholder="Search logs..."
                        className="pl-10"
                        value={searchTerm}
                        onChange={(e) => setSearchTerm(e.target.value)}
                    />
                </div>
                <div className="flex gap-2">
                    <Select defaultValue="all">
                        <SelectTrigger className="w-[180px]">
                            <SelectValue placeholder="Event Type" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Events</SelectItem>
                            <SelectItem value="auth">Authentication</SelectItem>
                            <SelectItem value="api">API Keys</SelectItem>
                            <SelectItem value="provider">Providers</SelectItem>
                        </SelectContent>
                    </Select>
                    <Select defaultValue="7d">
                        <SelectTrigger className="w-[180px]">
                            <SelectValue placeholder="Time Range" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="24h">Last 24 Hours</SelectItem>
                            <SelectItem value="7d">Last 7 Days</SelectItem>
                            <SelectItem value="30d">Last 30 Days</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
            </div>

            {/* Logs Table */}
            <div className="rounded-md border border-border/40 bg-background/50 backdrop-blur-sm">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>Event</TableHead>
                            <TableHead>Actor</TableHead>
                            <TableHead>Resource</TableHead>
                            <TableHead className="text-right">Time</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {loading ? (
                            <TableRow>
                                <TableCell colSpan={4} className="h-24 text-center">
                                    <Loader2 className="w-6 h-6 animate-spin mx-auto" />
                                </TableCell>
                            </TableRow>
                        ) : logs.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={4} className="h-24 text-center text-muted-foreground">
                                    No logs found.
                                </TableCell>
                            </TableRow>
                        ) : (
                            logs.map((log) => (
                                <TableRow key={log.id}>
                                    <TableCell className="font-medium">
                                        <div className="flex items-center gap-2">
                                            <code className="text-xs bg-muted px-1.5 py-0.5 rounded">{log.action}</code>
                                        </div>
                                    </TableCell>
                                    <TableCell>{log.userName || log.userId}</TableCell>
                                    <TableCell>{log.resource}</TableCell>
                                    <TableCell className="text-right text-muted-foreground">
                                        {formatDistanceToNow(new Date(log.timestamp), { addSuffix: true })}
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </div>
        </div>
    );
}
