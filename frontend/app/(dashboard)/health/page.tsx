"use client";

import { Activity, CheckCircle, XCircle, Clock, Cpu, Database, Server } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";

const services = [
    { name: "Backend API", status: "healthy", uptime: "99.9%", latency: "12ms", icon: Server },
    { name: "PostgreSQL", status: "healthy", uptime: "99.9%", latency: "3ms", icon: Database },
    { name: "Redis", status: "healthy", uptime: "100%", latency: "1ms", icon: Database },
    { name: "Qdrant", status: "healthy", uptime: "99.8%", latency: "8ms", icon: Database },
    { name: "OpenAI", status: "healthy", uptime: "99.5%", latency: "245ms", icon: Cpu },
    { name: "Anthropic", status: "degraded", uptime: "98.2%", latency: "520ms", icon: Cpu },
];

export default function HealthPage() {
    const healthyCount = services.filter(s => s.status === "healthy").length;

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">System Health</h1>
                    <p className="text-muted-foreground">Monitor service status and metrics</p>
                </div>
            </div>

            <div className="grid gap-4 md:grid-cols-3">
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">Overall Status</CardTitle>
                        <Activity className="h-4 w-4 text-green-500" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold text-green-500">Healthy</div>
                        <p className="text-xs text-muted-foreground">{healthyCount}/{services.length} services operational</p>
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">Uptime</CardTitle>
                        <Clock className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">99.7%</div>
                        <p className="text-xs text-muted-foreground">Last 30 days</p>
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">Avg Latency</CardTitle>
                        <Activity className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">132ms</div>
                        <p className="text-xs text-muted-foreground">Across all services</p>
                    </CardContent>
                </Card>
            </div>

            <Card>
                <CardHeader>
                    <CardTitle>Services</CardTitle>
                    <CardDescription>Real-time status of all services</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    {services.map((service) => (
                        <div key={service.name} className="flex items-center gap-4 p-3 rounded-lg bg-muted/50">
                            <service.icon className="h-5 w-5 text-muted-foreground" />
                            <div className="flex-1">
                                <div className="flex items-center justify-between">
                                    <span className="font-medium">{service.name}</span>
                                    <div className="flex items-center gap-2">
                                        {service.status === "healthy" ? (
                                            <CheckCircle className="h-4 w-4 text-green-500" />
                                        ) : (
                                            <XCircle className="h-4 w-4 text-yellow-500" />
                                        )}
                                        <span className={service.status === "healthy" ? "text-green-500" : "text-yellow-500"}>
                                            {service.status}
                                        </span>
                                    </div>
                                </div>
                                <div className="flex items-center gap-4 mt-1 text-sm text-muted-foreground">
                                    <span>Uptime: {service.uptime}</span>
                                    <span>Latency: {service.latency}</span>
                                </div>
                            </div>
                        </div>
                    ))}
                </CardContent>
            </Card>
        </div>
    );
}
