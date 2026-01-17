"use client";

import { useState, useEffect } from "react";
import { Key, Plus, Search, Copy, Check, RefreshCw, Settings, Trash2, Globe, Terminal, Zap, Shield, Loader2 } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { applicationsApi } from "@/lib/api";
import { toast } from "sonner";
import { Application } from "@/types";
import { formatDistanceToNow } from "date-fns";

export default function ApplicationsPage() {
    const [applications, setApplications] = useState<Application[]>([]);
    const [loading, setLoading] = useState(true);
    const [copied, setCopied] = useState<string | null>(null);

    useEffect(() => {
        const fetchApps = async () => {
            try {
                const data = await applicationsApi.list();
                setApplications(data);
            } catch (error) {
                console.error("Failed to load applications", error);
                toast.error("Failed to load applications");
            } finally {
                setLoading(false);
            }
        };

        fetchApps();
    }, []);

    const copyKey = (id: string, key: string) => {
        navigator.clipboard.writeText(key);
        setCopied(id);
        setTimeout(() => setCopied(null), 2000);
    };

    const statusColors: Record<string, string> = {
        active: "bg-emerald-500/10 text-emerald-600 border-emerald-500/20",
        suspended: "bg-amber-500/10 text-amber-600 border-amber-500/20",
        expired: "bg-red-500/10 text-red-600 border-red-500/20",
    };

    if (loading) {
        return <div className="flex justify-center items-center h-[50vh]"><Loader2 className="w-8 h-8 animate-spin text-muted-foreground" /></div>;
    }

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold">Applications</h1>
                    <p className="text-muted-foreground">Manage API access for external services</p>
                </div>
                <Button className="bg-gradient-to-r from-violet-600 to-blue-600 text-white shadow-lg shadow-violet-500/25 hover:shadow-violet-500/40 transition-shadow">
                    <Plus className="mr-2 h-4 w-4" />
                    Create Application
                </Button>
            </div>

            {/* Stats */}
            <div className="grid grid-cols-4 gap-4">
                <Card className="border-0 bg-gradient-to-br from-violet-500/10 to-transparent">
                    <CardContent className="pt-4">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-violet-500/20">
                                <Key className="w-5 h-5 text-violet-600" />
                            </div>
                            <div>
                                <p className="text-2xl font-bold">{applications.length}</p>
                                <p className="text-xs text-muted-foreground">Applications</p>
                            </div>
                        </div>
                    </CardContent>
                </Card>
                <Card className="border-0 bg-gradient-to-br from-emerald-500/10 to-transparent">
                    <CardContent className="pt-4">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-emerald-500/20">
                                <Check className="w-5 h-5 text-emerald-600" />
                            </div>
                            <div>
                                <p className="text-2xl font-bold">{applications.filter(a => a.status === "active").length}</p>
                                <p className="text-xs text-muted-foreground">Active</p>
                            </div>
                        </div>
                    </CardContent>
                </Card>
                <Card className="border-0 bg-gradient-to-br from-blue-500/10 to-transparent">
                    <CardContent className="pt-4">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-blue-500/20">
                                <Zap className="w-5 h-5 text-blue-600" />
                            </div>
                            <div>
                                <p className="text-2xl font-bold">{applications.reduce((acc, a) => acc + (a.requestsToday || 0), 0).toLocaleString()}</p>
                                <p className="text-xs text-muted-foreground">Requests Today</p>
                            </div>
                        </div>
                    </CardContent>
                </Card>
                <Card className="border-0 bg-gradient-to-br from-cyan-500/10 to-transparent">
                    <CardContent className="pt-4">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-cyan-500/20">
                                <Globe className="w-5 h-5 text-cyan-600" />
                            </div>
                            <div>
                                <p className="text-2xl font-bold">REST + gRPC</p>
                                <p className="text-xs text-muted-foreground">Protocols</p>
                            </div>
                        </div>
                    </CardContent>
                </Card>
            </div>

            {/* API Endpoints Banner */}
            <Card className="border-0 bg-gradient-to-r from-slate-900 to-slate-800 text-white overflow-hidden">
                <CardContent className="py-4">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-4">
                            <div className="p-3 rounded-xl bg-white/10">
                                <Terminal className="w-6 h-6" />
                            </div>
                            <div>
                                <h3 className="font-semibold">API Endpoints</h3>
                                <p className="text-sm text-white/60">Use these endpoints with your API keys</p>
                            </div>
                        </div>
                        <div className="flex gap-6 text-sm">
                            <div className="text-right">
                                <p className="text-white/50">REST</p>
                                <code className="text-cyan-300">http://localhost:4000/v1</code>
                            </div>
                            <div className="h-8 w-px bg-white/20" />
                            <div className="text-right">
                                <p className="text-white/50">gRPC</p>
                                <code className="text-violet-300">localhost:4000</code>
                            </div>
                        </div>
                    </div>
                </CardContent>
            </Card>

            {/* Search */}
            <div className="flex items-center gap-4">
                <div className="relative flex-1 max-w-sm">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <Input placeholder="Search applications..." className="pl-10" />
                </div>
            </div>

            {/* Applications List */}
            <div className="space-y-4">
                {applications.length === 0 ? (
                    <div className="text-center py-12 text-muted-foreground bg-muted/20 rounded-lg">
                        No applications found. Create one to get started.
                    </div>
                ) : (
                    applications.map((app) => (
                        <Card key={app.id} className="overflow-hidden">
                            <CardContent className="p-5">
                                <div className="flex items-start justify-between">
                                    <div className="flex items-start gap-4">
                                        <div className={`p-3 rounded-xl ${app.status === "active" ? "bg-emerald-500/10" :
                                            app.status === "suspended" ? "bg-amber-500/10" : "bg-red-500/10"
                                            }`}>
                                            <Key className={`w-6 h-6 ${app.status === "active" ? "text-emerald-600" :
                                                app.status === "suspended" ? "text-amber-600" : "text-red-600"
                                                }`} />
                                        </div>
                                        <div>
                                            <div className="flex items-center gap-2">
                                                <h4 className="font-semibold">{app.name}</h4>
                                                <Badge variant="outline" className={statusColors[app.status || 'active']}>
                                                    {app.status}
                                                </Badge>
                                            </div>
                                            <p className="text-sm text-muted-foreground mt-0.5">{app.description}</p>

                                            {/* Key & Stats */}
                                            <div className="flex items-center gap-4 mt-3 text-sm text-muted-foreground">
                                                <button
                                                    onClick={() => copyKey(app.id, app.apiKeyPrefix)}
                                                    className="flex items-center gap-1 hover:text-foreground transition-colors"
                                                >
                                                    <code className="text-xs bg-muted px-1.5 py-0.5 rounded">{app.apiKeyPrefix}</code>
                                                    {copied === app.id ? (
                                                        <Check className="w-3 h-3 text-emerald-500" />
                                                    ) : (
                                                        <Copy className="w-3 h-3" />
                                                    )}
                                                </button>
                                                <span>•</span>
                                                <span className="flex items-center gap-1">
                                                    <Zap className="w-3 h-3" />
                                                    {app.requestsToday?.toLocaleString() || 0} reqs today
                                                </span>
                                            </div>

                                            {/* Scopes */}
                                            <div className="flex flex-wrap gap-1.5 mt-3">
                                                {app.scopes.map((scope) => (
                                                    <Badge key={scope} variant="secondary" className="text-xs gap-1">
                                                        <Shield className="w-3 h-3" />
                                                        {scope}
                                                    </Badge>
                                                ))}
                                            </div>
                                        </div>
                                    </div>

                                    {/* Actions */}
                                    <div className="flex gap-2">
                                        <Button variant="outline" size="sm">
                                            <RefreshCw className="w-4 h-4 mr-1" />
                                            Rotate
                                        </Button>
                                        <Button variant="outline" size="icon" className="h-8 w-8">
                                            <Settings className="w-4 h-4" />
                                        </Button>
                                        <Button variant="outline" size="icon" className="h-8 w-8 text-destructive hover:text-destructive">
                                            <Trash2 className="w-4 h-4" />
                                        </Button>
                                    </div>
                                </div>

                                {/* Rate Limit Progress */}
                                <div className="mt-4 pt-4 border-t">
                                    <div className="flex items-center justify-between text-sm mb-1">
                                        <span className="text-muted-foreground">Rate Limit Usage</span>
                                        <span>{Math.min(100, ((app.requestsToday || 0) / (app.rateLimit * 60 || 1)) * 100).toFixed(0)}%</span>
                                    </div>
                                    <Progress value={Math.min(100, ((app.requestsToday || 0) / (app.rateLimit * 60 || 1)) * 100)} className="h-1.5" />
                                    <p className="text-xs text-muted-foreground mt-1">{app.rateLimit} requests/min limit</p>
                                </div>
                            </CardContent>
                        </Card>
                    ))
                )}
            </div>
        </div>
    );
}
