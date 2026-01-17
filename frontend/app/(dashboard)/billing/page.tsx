"use client";

import { DollarSign, TrendingUp, CreditCard, AlertTriangle } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";

const costs = [
    { provider: "OpenAI", amount: 124.50, percentage: 45 },
    { provider: "Anthropic", amount: 89.20, percentage: 32 },
    { provider: "Google", amount: 45.30, percentage: 16 },
    { provider: "Others", amount: 18.00, percentage: 7 },
];

export default function BillingPage() {
    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Billing</h1>
                    <p className="text-muted-foreground">Track costs and manage budgets</p>
                </div>
            </div>

            <div className="grid gap-4 md:grid-cols-4">
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">Total Spend</CardTitle>
                        <DollarSign className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">$277.00</div>
                        <p className="text-xs text-muted-foreground">This month</p>
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">Budget</CardTitle>
                        <CreditCard className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">$500.00</div>
                        <Progress value={55} className="mt-2" />
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">Trend</CardTitle>
                        <TrendingUp className="h-4 w-4 text-green-500" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">-12%</div>
                        <p className="text-xs text-muted-foreground">vs last month</p>
                    </CardContent>
                </Card>
                <Card>
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <CardTitle className="text-sm font-medium">Alerts</CardTitle>
                        <AlertTriangle className="h-4 w-4 text-yellow-500" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">1</div>
                        <p className="text-xs text-muted-foreground">Budget warning</p>
                    </CardContent>
                </Card>
            </div>

            <Card>
                <CardHeader>
                    <CardTitle>Cost by Provider</CardTitle>
                    <CardDescription>Monthly breakdown by provider</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    {costs.map((cost) => (
                        <div key={cost.provider} className="space-y-2">
                            <div className="flex items-center justify-between text-sm">
                                <span>{cost.provider}</span>
                                <span className="font-medium">${cost.amount.toFixed(2)}</span>
                            </div>
                            <Progress value={cost.percentage} />
                        </div>
                    ))}
                </CardContent>
            </Card>
        </div>
    );
}
