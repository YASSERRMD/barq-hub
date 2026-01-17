"use client";

import {
    Cpu,
    Database,
    Users,
    Activity,
    TrendingUp,
    DollarSign,
    FileText,
    Zap
} from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

const stats = [
    {
        title: "Active Providers",
        value: "8",
        change: "+2 this week",
        icon: Cpu,
        trend: "up",
    },
    {
        title: "Vector Databases",
        value: "3",
        change: "All connected",
        icon: Database,
        trend: "stable",
    },
    {
        title: "Total Users",
        value: "24",
        change: "+5 this month",
        icon: Users,
        trend: "up",
    },
    {
        title: "API Requests",
        value: "12.4K",
        change: "+18% from yesterday",
        icon: Activity,
        trend: "up",
    },
];

const recentActivity = [
    { action: "Provider added", detail: "OpenAI GPT-4o", time: "2 min ago" },
    { action: "User login", detail: "admin@synapse.local", time: "15 min ago" },
    { action: "Document ingested", detail: "API Documentation.pdf", time: "1 hour ago" },
    { action: "Workflow executed", detail: "Data Processing Pipeline", time: "3 hours ago" },
];

export default function DashboardPage() {
    return (
        <div className="space-y-6">
            {/* Page Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
                    <p className="text-muted-foreground">
                        Welcome to BARQ HUB - Your SYNAPSE Brain Console
                    </p>
                </div>
                <Badge variant="outline" className="gap-1">
                    <span className="h-2 w-2 rounded-full bg-green-500 animate-pulse" />
                    System Healthy
                </Badge>
            </div>

            {/* Stats Grid */}
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                {stats.map((stat) => (
                    <Card key={stat.title} className="relative overflow-hidden">
                        <CardHeader className="flex flex-row items-center justify-between pb-2">
                            <CardTitle className="text-sm font-medium text-muted-foreground">
                                {stat.title}
                            </CardTitle>
                            <stat.icon className="h-4 w-4 text-muted-foreground" />
                        </CardHeader>
                        <CardContent>
                            <div className="text-2xl font-bold">{stat.value}</div>
                            <p className="text-xs text-muted-foreground flex items-center gap-1">
                                {stat.trend === "up" && (
                                    <TrendingUp className="h-3 w-3 text-green-500" />
                                )}
                                {stat.change}
                            </p>
                        </CardContent>
                        <div className="absolute inset-x-0 bottom-0 h-1 bg-gradient-to-r from-blue-500 to-cyan-400" />
                    </Card>
                ))}
            </div>

            {/* Quick Actions & Recent Activity */}
            <div className="grid gap-6 md:grid-cols-2">
                {/* Quick Actions */}
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <Zap className="h-5 w-5 text-yellow-500" />
                            Quick Actions
                        </CardTitle>
                        <CardDescription>Common tasks and shortcuts</CardDescription>
                    </CardHeader>
                    <CardContent className="grid gap-3">
                        <QuickAction
                            icon={Cpu}
                            title="Add Provider"
                            description="Configure a new LLM provider"
                        />
                        <QuickAction
                            icon={Database}
                            title="Connect Vector DB"
                            description="Setup vector database connection"
                        />
                        <QuickAction
                            icon={FileText}
                            title="View Audit Logs"
                            description="Check recent system activity"
                        />
                        <QuickAction
                            icon={DollarSign}
                            title="Check Billing"
                            description="View costs and usage"
                        />
                    </CardContent>
                </Card>

                {/* Recent Activity */}
                <Card>
                    <CardHeader>
                        <CardTitle>Recent Activity</CardTitle>
                        <CardDescription>Latest actions in the system</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className="space-y-4">
                            {recentActivity.map((activity, i) => (
                                <div key={i} className="flex items-center gap-4">
                                    <div className="h-2 w-2 rounded-full bg-blue-500" />
                                    <div className="flex-1 space-y-1">
                                        <p className="text-sm font-medium">{activity.action}</p>
                                        <p className="text-xs text-muted-foreground">{activity.detail}</p>
                                    </div>
                                    <span className="text-xs text-muted-foreground">{activity.time}</span>
                                </div>
                            ))}
                        </div>
                    </CardContent>
                </Card>
            </div>

            {/* System Status */}
            <Card>
                <CardHeader>
                    <CardTitle>System Status</CardTitle>
                    <CardDescription>Real-time service health monitoring</CardDescription>
                </CardHeader>
                <CardContent>
                    <div className="grid gap-4 md:grid-cols-5">
                        <ServiceStatus name="Backend API" status="healthy" latency="12ms" />
                        <ServiceStatus name="PostgreSQL" status="healthy" latency="3ms" />
                        <ServiceStatus name="Redis" status="healthy" latency="1ms" />
                        <ServiceStatus name="Qdrant" status="healthy" latency="8ms" />
                        <ServiceStatus name="OpenAI" status="healthy" latency="245ms" />
                    </div>
                </CardContent>
            </Card>
        </div>
    );
}

function QuickAction({
    icon: Icon,
    title,
    description
}: {
    icon: React.ElementType;
    title: string;
    description: string;
}) {
    return (
        <button className="flex items-center gap-3 w-full p-3 rounded-lg border border-transparent hover:bg-accent hover:border-border transition-colors text-left">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                <Icon className="h-5 w-5 text-primary" />
            </div>
            <div>
                <p className="text-sm font-medium">{title}</p>
                <p className="text-xs text-muted-foreground">{description}</p>
            </div>
        </button>
    );
}

function ServiceStatus({
    name,
    status,
    latency
}: {
    name: string;
    status: "healthy" | "degraded" | "down";
    latency: string;
}) {
    const statusColors = {
        healthy: "bg-green-500",
        degraded: "bg-yellow-500",
        down: "bg-red-500",
    };

    return (
        <div className="flex items-center gap-3 p-3 rounded-lg bg-muted/50">
            <span className={`h-2.5 w-2.5 rounded-full ${statusColors[status]}`} />
            <div className="flex-1">
                <p className="text-sm font-medium">{name}</p>
                <p className="text-xs text-muted-foreground">{latency}</p>
            </div>
        </div>
    );
}
