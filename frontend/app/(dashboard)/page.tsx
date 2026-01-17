"use client";

import { useEffect, useState } from "react";
import {
    Activity,
    Zap,
    Share2,
    Server,
    Clock,
    Database,
    TrendingUp,
    TrendingDown,
    ArrowUpRight,
    ArrowDownRight,
    DollarSign,
    Users
} from "lucide-react";
import {
    AreaChart,
    Area,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
    BarChart,
    Bar,
    LineChart,
    Line,
} from "recharts";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useAuthStore } from "@/stores/auth-store";

// Mock data for charts
const requestData = [
    { time: "00:00", requests: 120, errors: 2 },
    { time: "04:00", requests: 80, errors: 1 },
    { time: "08:00", requests: 450, errors: 5 },
    { time: "12:00", requests: 980, errors: 12 },
    { time: "16:00", requests: 850, errors: 8 },
    { time: "20:00", requests: 340, errors: 3 },
    { time: "23:59", requests: 190, errors: 2 },
];

const costData = [
    { provider: "OpenAI", cost: 124.50 },
    { provider: "Anthropic", cost: 89.20 },
    { provider: "Mistral", cost: 45.00 },
    { provider: "Cohere", cost: 32.10 },
];

export default function DashboardPage() {
    const { user } = useAuthStore();
    const [mounted, setMounted] = useState(false);

    useEffect(() => {
        setMounted(true);
    }, []);

    if (!mounted) return null;

    return (
        <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
            {/* Hero Section */}
            <div className="relative overflow-hidden rounded-3xl bg-gradient-to-br from-violet-600 to-cyan-500 p-8 text-white shadow-xl shadow-violet-500/20">
                <div className="absolute top-0 right-0 p-12 opacity-10">
                    <Zap className="w-64 h-64 rotate-12" />
                </div>
                <div className="relative z-10">
                    <div className="flex items-center gap-3 mb-4">
                        <Badge variant="outline" className="border-white/30 text-white bg-white/10 backdrop-blur-md">
                            <div className="w-2 h-2 rounded-full bg-green-400 mr-2 animate-pulse" />
                            System Operational
                        </Badge>
                        <Badge variant="outline" className="border-white/30 text-white bg-white/10 backdrop-blur-md">
                            v0.1.0-beta
                        </Badge>
                    </div>
                    <h1 className="text-4xl md:text-5xl font-bold mb-4 tracking-tight">
                        Welcome back, {user?.name || "Commander"}
                    </h1>
                    <p className="text-white/80 text-lg max-w-2xl mb-8">
                        Your AI infrastructure is performing optimally.
                        Request volume is up <span className="text-white font-semibold">12%</span> from yesterday.
                    </p>
                    <div className="flex gap-4">
                        <Button variant="secondary" className="gap-2 shadow-lg shadow-black/10">
                            <Activity className="w-4 h-4" /> View Live Metrics
                        </Button>
                        <Button variant="outline" className="bg-white/10 border-white/20 text-white hover:bg-white/20 gap-2">
                            <Share2 className="w-4 h-4" /> Connection Status
                        </Button>
                    </div>
                </div>
            </div>

            {/* Quick Stats Row */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                {[
                    { label: "Total Requests", value: "24.5k", change: "+12%", trend: "up", icon: Activity, color: "text-blue-500" },
                    { label: "Avg Latency", value: "142ms", change: "-5%", trend: "down", icon: Clock, color: "text-green-500" },
                    { label: "Est. Cost", value: "$342.50", change: "+2%", trend: "up", icon: DollarSign, color: "text-amber-500" },
                    { label: "Active Users", value: "842", change: "+18%", trend: "up", icon: Users, color: "text-violet-500" },
                ].map((stat, i) => (
                    <Card key={i} className="border-0 bg-background/50 backdrop-blur-sm border-t border-white/10 shadow-lg shadow-black/5 hover:bg-muted/50 transition-all">
                        <CardContent className="p-6">
                            <div className="flex justify-between items-start mb-4">
                                <div className={`p-3 rounded-2xl bg-background shadow-inner ${stat.color}`}>
                                    <stat.icon className="w-6 h-6" />
                                </div>
                                <div className={`flex items-center gap-1 text-xs font-medium px-2 py-1 rounded-full ${stat.trend === "up"
                                        ? "bg-green-500/10 text-green-600"
                                        : "bg-red-500/10 text-red-600"
                                    }`}>
                                    {stat.trend === "up" ? <ArrowUpRight className="w-3 h-3" /> : <ArrowDownRight className="w-3 h-3" />}
                                    {stat.change}
                                </div>
                            </div>
                            <h3 className="text-3xl font-bold mb-1">{stat.value}</h3>
                            <p className="text-muted-foreground text-sm">{stat.label}</p>
                        </CardContent>
                    </Card>
                ))}
            </div>

            {/* Main Charts Section */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                {/* Request Volume Chart */}
                <Card className="lg:col-span-2 border-0 bg-background/50 backdrop-blur-sm shadow-xl shadow-black/5">
                    <CardContent className="p-6">
                        <div className="flex items-center justify-between mb-8">
                            <div>
                                <h3 className="text-lg font-semibold">Traffic Overview</h3>
                                <p className="text-sm text-muted-foreground">Request volume and error rates over time</p>
                            </div>
                            <div className="flex gap-2">
                                <Button size="sm" variant="outline" className="h-8">24h</Button>
                                <Button size="sm" variant="ghost" className="h-8">7d</Button>
                                <Button size="sm" variant="ghost" className="h-8">30d</Button>
                            </div>
                        </div>
                        <div className="h-[300px] w-full">
                            <ResponsiveContainer width="100%" height="100%">
                                <AreaChart data={requestData}>
                                    <defs>
                                        <linearGradient id="colorRequests" x1="0" y1="0" x2="0" y2="1">
                                            <stop offset="5%" stopColor="#8b5cf6" stopOpacity={0.3} />
                                            <stop offset="95%" stopColor="#8b5cf6" stopOpacity={0} />
                                        </linearGradient>
                                    </defs>
                                    <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="hsl(var(--border))" />
                                    <XAxis
                                        dataKey="time"
                                        axisLine={false}
                                        tickLine={false}
                                        tick={{ fill: 'hsl(var(--muted-foreground))', fontSize: 12 }}
                                        dy={10}
                                    />
                                    <YAxis
                                        axisLine={false}
                                        tickLine={false}
                                        tick={{ fill: 'hsl(var(--muted-foreground))', fontSize: 12 }}
                                    />
                                    <Tooltip
                                        contentStyle={{
                                            backgroundColor: 'hsl(var(--background))',
                                            border: '1px solid hsl(var(--border))',
                                            borderRadius: '8px',
                                        }}
                                    />
                                    <Area
                                        type="monotone"
                                        dataKey="requests"
                                        stroke="#8b5cf6"
                                        strokeWidth={3}
                                        fillOpacity={1}
                                        fill="url(#colorRequests)"
                                    />
                                </AreaChart>
                            </ResponsiveContainer>
                        </div>
                    </CardContent>
                </Card>

                {/* Quick Status & Cost */}
                <div className="space-y-6">
                    {/* System Health */}
                    <Card className="border-0 bg-background/50 backdrop-blur-sm shadow-xl shadow-black/5">
                        <CardContent className="p-6">
                            <h3 className="text-lg font-semibold mb-6">System Health</h3>
                            <div className="space-y-4">
                                {[
                                    { name: "API Gateway", status: "Operational", color: "bg-green-500" },
                                    { name: "Database", status: "Operational", color: "bg-green-500" },
                                    { name: "Redis Cache", status: "Operational", color: "bg-green-500" },
                                    { name: "LLM Providers", status: "Degraded", color: "bg-amber-500" },
                                ].map((service, i) => (
                                    <div key={i} className="flex items-center justify-between">
                                        <div className="flex items-center gap-3">
                                            <div className={`w-2 h-2 rounded-full ${service.color}`} />
                                            <span className="font-medium">{service.name}</span>
                                        </div>
                                        <span className="text-sm text-muted-foreground">{service.status}</span>
                                    </div>
                                ))}
                            </div>
                        </CardContent>
                    </Card>

                    {/* Cost Distribution */}
                    <Card className="border-0 bg-background/50 backdrop-blur-sm shadow-xl shadow-black/5">
                        <CardContent className="p-6">
                            <h3 className="text-lg font-semibold mb-6">Cost by Provider</h3>
                            <div className="space-y-4">
                                {costData.map((item, i) => (
                                    <div key={i} className="space-y-2">
                                        <div className="flex justify-between text-sm">
                                            <span className="font-medium">{item.provider}</span>
                                            <span className="text-muted-foreground">${item.cost.toFixed(2)}</span>
                                        </div>
                                        <div className="h-2 rounded-full bg-muted overflow-hidden">
                                            <div
                                                className="h-full bg-gradient-to-r from-violet-600 to-cyan-500 rounded-full"
                                                style={{ width: `${(item.cost / 150) * 100}%` }}
                                            />
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </CardContent>
                    </Card>
                </div>
            </div>

            {/* Recent Activity Ticker */}
            <Card className="border-0 bg-muted/30">
                <CardContent className="p-4 flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <Database className="w-5 h-5 text-muted-foreground" />
                        <div className="h-4 w-px bg-border" />
                        <p className="text-sm text-muted-foreground">
                            <span className="font-medium text-foreground">Latest:</span> API Key created for "Marketing Bot" by admin@barq.hub
                        </p>
                    </div>
                    <p className="text-xs text-muted-foreground font-mono">2 mins ago</p>
                </CardContent>
            </Card>
        </div>
    );
}
