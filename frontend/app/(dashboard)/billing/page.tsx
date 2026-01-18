"use client";

import { useState, useEffect } from "react";
import { Download, TrendingUp, DollarSign, CreditCard, Loader2 } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
    AreaChart,
    Area,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
    PieChart,
    Pie,
    Cell,
} from "recharts";
import { billingApi } from "@/lib/api";
import { toast } from "sonner";
import { BillingUsage } from "@/types";

export default function BillingPage() {
    const [usage, setUsage] = useState<BillingUsage | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchBilling = async () => {
            try {
                // Fetch usage for current month
                const data = await billingApi.getUsage();
                setUsage(data);
            } catch (error) {
                console.error("Failed to load billing data", error);
                toast.error("Failed to load usage data data");
            } finally {
                setLoading(false);
            }
        };

        fetchBilling();
    }, []);

    if (loading) {
        return <div className="flex justify-center items-center h-[50vh]"><Loader2 className="w-8 h-8 animate-spin text-muted-foreground" /></div>;
    }

    // Default empty state if no data
    const dailyData = usage?.byDay || [];
    const providerData = usage?.byProvider?.map((p, i) => ({
        ...p,
        color: ["#8b5cf6", "#0ef", "#f59e0b", "#10b981", "#ec4899"][i % 5]
    })) || [];

    const projectedCost = (usage?.totalCost || 0) * 1.2; // Simple projection
    const budget = 1000; // Static budget for now, could be from settings

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Billing & Usage</h1>
                    <p className="text-muted-foreground">Monitor costs and manage subscription</p>
                </div>
                <div className="flex gap-2">
                    <Button variant="outline">
                        <Download className="mr-2 h-4 w-4" /> Export Invoice
                    </Button>
                    <Button>Manage Plan</Button>
                </div>
            </div>

            {/* Cost Cards */}
            <div className="grid gap-4 md:grid-cols-3">
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Current Total</CardTitle>
                        <DollarSign className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">${usage?.totalCost?.toFixed(2) || "0.00"}</div>
                        <p className="text-xs text-muted-foreground mt-1">
                            {usage?.requestCount?.toLocaleString() || 0} API requests
                        </p>
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Projected End</CardTitle>
                        <TrendingUp className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">${projectedCost.toFixed(2)}</div>
                        <p className="text-xs text-muted-foreground mt-1">
                            Based on current usage
                        </p>
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Budget Remaining</CardTitle>
                        <CreditCard className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">${Math.max(0, budget - (usage?.totalCost || 0)).toFixed(2)}</div>
                        <div className="w-full bg-secondary h-2 mt-2 rounded-full overflow-hidden">
                            <div
                                className="bg-primary h-full transition-all"
                                style={{ width: `${Math.min(100, ((usage?.totalCost || 0) / budget) * 100)}%` }}
                            />
                        </div>
                    </CardContent>
                </Card>
            </div>

            {/* Charts Row */}
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <Card className="col-span-1">
                    <CardHeader>
                        <CardTitle>Daily Cost Trend</CardTitle>
                        <CardDescription>Cost accumulation over time</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className="h-[300px]">
                            <ResponsiveContainer width="100%" height="100%">
                                <AreaChart data={dailyData}>
                                    <defs>
                                        <linearGradient id="colorCost" x1="0" y1="0" x2="0" y2="1">
                                            <stop offset="5%" stopColor="#8b5cf6" stopOpacity={0.3} />
                                            <stop offset="95%" stopColor="#8b5cf6" stopOpacity={0} />
                                        </linearGradient>
                                    </defs>
                                    <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="hsl(var(--border))" />
                                    <XAxis dataKey="date" axisLine={false} tickLine={false} tick={{ fontSize: 12 }} />
                                    <YAxis prefix="$" axisLine={false} tickLine={false} tick={{ fontSize: 12 }} />
                                    <Tooltip
                                        formatter={(value: number) => [`$${value.toFixed(2)}`, "Cost"]}
                                        contentStyle={{
                                            backgroundColor: 'hsl(var(--background))',
                                            border: '1px solid hsl(var(--border))',
                                            borderRadius: '8px',
                                        }}
                                    />
                                    <Area type="monotone" dataKey="cost" stroke="#8b5cf6" fillOpacity={1} fill="url(#colorCost)" />
                                </AreaChart>
                            </ResponsiveContainer>
                        </div>
                    </CardContent>
                </Card>

                <Card className="col-span-1">
                    <CardHeader>
                        <CardTitle>Cost by Provider</CardTitle>
                        <CardDescription>Distribution of spend across LLM services</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className="h-[300px] flex items-center justify-center">
                            {providerData.length > 0 ? (
                                <ResponsiveContainer width="100%" height="100%">
                                    <PieChart>
                                        <Pie
                                            data={providerData}
                                            cx="50%"
                                            cy="50%"
                                            innerRadius={60}
                                            outerRadius={80}
                                            paddingAngle={5}
                                            dataKey="cost"
                                        >
                                            {providerData.map((entry: any, index: number) => (
                                                <Cell key={`cell-${index}`} fill={entry.color} />
                                            ))}
                                        </Pie>
                                        <Tooltip
                                            formatter={(value: number) => [`$${value.toFixed(2)}`, "Cost"]}
                                            contentStyle={{
                                                backgroundColor: 'hsl(var(--background))',
                                                border: '1px solid hsl(var(--border))',
                                                borderRadius: '8px',
                                            }}
                                        />
                                    </PieChart>
                                </ResponsiveContainer>
                            ) : (
                                <div className="text-muted-foreground text-sm">No usage data available</div>
                            )}
                        </div>
                        <div className="flex flex-wrap justify-center gap-4 mt-4">
                            {providerData.map((entry: any, index: number) => (
                                <div key={index} className="flex items-center gap-2">
                                    <div className="w-3 h-3 rounded-full" style={{ backgroundColor: entry.color }} />
                                    <span className="text-sm font-medium">{entry.provider}</span>
                                    <span className="text-sm text-muted-foreground">${entry.cost.toFixed(2)}</span>
                                </div>
                            ))}
                        </div>
                    </CardContent>
                </Card>
            </div>
        </div>
    );
}
