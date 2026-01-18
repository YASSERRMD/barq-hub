"use client";

import { useState, useEffect } from "react";
import { Activity, CheckCircle, AlertTriangle, XCircle, Server, Database, Globe, Loader2 } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { healthApi } from "@/lib/api";
import { toast } from "sonner";
import { HealthStatus } from "@/types";

export default function HealthPage() {
    const [health, setHealth] = useState<HealthStatus | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchHealth = async () => {
            try {
                const data = await healthApi.check();
                setHealth(data);
            } catch (error) {
                console.error("Failed to load health status", error);

                // Fallback for initial view if API fails (so page isn't broken during dev)
                // Remove this in pure production if strict error handling is desired
                toast.error("Failed to fetch real-time health metrics");
            } finally {
                setLoading(false);
            }
        };

        fetchHealth();
    }, []);

    const getIcon = (name: string) => {
        if (name.toLowerCase().includes("database")) return Database;
        if (name.toLowerCase().includes("redis")) return Server;
        if (name.toLowerCase().includes("llm")) return Activity;
        return Globe;
    };

    if (loading) {
        return <div className="flex justify-center items-center h-[50vh]"><Loader2 className="w-8 h-8 animate-spin text-muted-foreground" /></div>;
    }

    // Use API data if available, otherwise show error state
    const services = health?.services || [];
    const overallStatus = health?.status || "unknown";

    return (
        <div className="space-y-6">
            <div className="mb-8">
                <h1 className="text-3xl font-bold tracking-tight mb-2">System Health</h1>
                <div className="flex items-center gap-2">
                    <div className={`w-3 h-3 rounded-full ${overallStatus === 'healthy' ? 'bg-green-500' : 'bg-red-500'} animate-pulse`} />
                    <span className={`${overallStatus === 'healthy' ? 'text-green-600' : 'text-red-500'} font-medium capitalize`}>
                        System {overallStatus}
                    </span>
                    <span className="text-muted-foreground ml-2 text-sm">Version: {health?.version || '0.0.0'}</span>
                </div>
            </div>

            <div className="grid gap-6 md:grid-cols-2">
                {services.length === 0 ? (
                    <div className="col-span-2 text-center py-12 text-muted-foreground border border-dashed rounded-lg">
                        <AlertTriangle className="w-8 h-8 mx-auto mb-2 opacity-50" />
                        No services detected or connection failed
                    </div>
                ) : (
                    services.map((service) => {
                        const Icon = getIcon(service.name);
                        return (
                            <Card key={service.name} className="overflow-hidden">
                                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4 bg-muted/20">
                                    <div className="flex items-center gap-3">
                                        <div className="p-2 rounded-lg bg-background border">
                                            <Icon className="h-5 w-5 text-muted-foreground" />
                                        </div>
                                        <CardTitle className="text-base font-semibold">{service.name}</CardTitle>
                                    </div>
                                    <Badge variant={service.status === "up" || service.status === "healthy" ? "default" : "destructive"} className={
                                        service.status === "up" || service.status === "healthy"
                                            ? "bg-green-500/10 text-green-600 border-green-500/20 hover:bg-green-500/20"
                                            : "bg-red-500/10 text-red-600 border-red-500/20"
                                    }>
                                        {service.status}
                                    </Badge>
                                </CardHeader>
                                <CardContent className="pt-6 grid grid-cols-2 gap-4">
                                    <div>
                                        <p className="text-xs text-muted-foreground uppercase tracking-wider font-semibold mb-1">Status</p>
                                        <p className="text-xl font-bold capitalize">{service.status}</p>
                                    </div>
                                    <div>
                                        <p className="text-xs text-muted-foreground uppercase tracking-wider font-semibold mb-1">Latency</p>
                                        <p className="text-xl font-bold font-mono">{service.latency ? `${service.latency}ms` : '-'}</p>
                                    </div>
                                </CardContent>
                            </Card>
                        );
                    })
                )}
            </div>

            <Card>
                <CardHeader>
                    <CardTitle>Incidents</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="space-y-4">
                        <div className="flex gap-4 items-start pb-4 border-b border-border/50">
                            <CheckCircle className="h-5 w-5 text-green-500 mt-0.5" />
                            <div>
                                <p className="font-medium">Recent Check</p>
                                <p className="text-sm text-muted-foreground">
                                    System check initialized {health?.lastCheck ? formatDistanceToNow(new Date(health.lastCheck), { addSuffix: true }) : 'just now'}.
                                </p>
                            </div>
                        </div>
                    </div>
                </CardContent>
            </Card>
        </div>
    );
}
