"use client";

import Link from "next/link";
import {
    Cpu,
    Key,
    FlaskConical,
    Receipt,
    FileText,
    Users,
    Shield,
    Activity,
    Settings,
    ArrowRight,
    CheckCircle,
    AlertCircle,
} from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

const modules = [
    {
        name: "Providers",
        description: "Manage LLM providers and models",
        href: "/providers",
        icon: Cpu,
        color: "from-violet-500 to-purple-600",
        stat: "8 active",
        status: "healthy",
    },
    {
        name: "Applications",
        description: "API keys and access control",
        href: "/applications",
        icon: Key,
        color: "from-emerald-500 to-green-600",
        stat: "3 apps",
        status: "healthy",
    },
    {
        name: "Playground",
        description: "Test prompts and models",
        href: "/playground",
        icon: FlaskConical,
        color: "from-amber-500 to-orange-600",
        stat: "Ready",
        status: "healthy",
    },
    {
        name: "Billing",
        description: "Costs, budgets, and usage",
        href: "/billing",
        icon: Receipt,
        color: "from-pink-500 to-rose-600",
        stat: "$277",
        status: "warning",
    },
    {
        name: "Audit Logs",
        description: "Activity and security logs",
        href: "/audit",
        icon: FileText,
        color: "from-cyan-500 to-blue-600",
        stat: "2.4K",
        status: "healthy",
    },
    {
        name: "Users",
        description: "User accounts",
        href: "/users",
        icon: Users,
        color: "from-indigo-500 to-violet-600",
        stat: "24",
        status: "healthy",
    },
    {
        name: "Roles",
        description: "Permissions and RBAC",
        href: "/roles",
        icon: Shield,
        color: "from-slate-500 to-gray-600",
        stat: "4 roles",
        status: "healthy",
    },
    {
        name: "Health",
        description: "System status",
        href: "/health",
        icon: Activity,
        color: "from-green-500 to-emerald-600",
        stat: "All OK",
        status: "healthy",
    },
    {
        name: "Settings",
        description: "App configuration",
        href: "/settings",
        icon: Settings,
        color: "from-gray-500 to-slate-600",
        stat: "",
        status: "healthy",
    },
];

export default function DashboardPage() {
    return (
        <div className="space-y-8">
            {/* Welcome */}
            <div className="text-center space-y-2">
                <h1 className="text-4xl font-bold tracking-tight">
                    Welcome to <span className="bg-gradient-to-r from-violet-600 to-cyan-500 bg-clip-text text-transparent">BARQ</span>
                </h1>
                <p className="text-muted-foreground text-lg">Your AI Management Console</p>
            </div>

            {/* Status Banner */}
            <div className="flex items-center justify-center gap-4 p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
                <CheckCircle className="w-5 h-5 text-emerald-500" />
                <span className="text-sm font-medium text-emerald-700 dark:text-emerald-400">
                    All systems operational • 8 providers • 3 applications • 12.4K requests today
                </span>
            </div>

            {/* Module Grid */}
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                {modules.map((module) => (
                    <Link key={module.name} href={module.href}>
                        <Card className="group h-full hover:shadow-lg hover:shadow-primary/5 hover:border-primary/20 transition-all duration-300 cursor-pointer overflow-hidden">
                            <CardContent className="p-5">
                                <div className="flex items-start justify-between mb-4">
                                    <div className={`flex items-center justify-center w-12 h-12 rounded-xl bg-gradient-to-br ${module.color} shadow-lg`}>
                                        <module.icon className="w-6 h-6 text-white" />
                                    </div>
                                    {module.stat && (
                                        <Badge
                                            variant="secondary"
                                            className={module.status === "warning" ? "bg-amber-500/10 text-amber-600" : ""}
                                        >
                                            {module.status === "warning" && <AlertCircle className="w-3 h-3 mr-1" />}
                                            {module.stat}
                                        </Badge>
                                    )}
                                </div>
                                <div className="space-y-1">
                                    <h3 className="font-semibold text-lg group-hover:text-primary transition-colors flex items-center gap-2">
                                        {module.name}
                                        <ArrowRight className="w-4 h-4 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all" />
                                    </h3>
                                    <p className="text-sm text-muted-foreground">{module.description}</p>
                                </div>
                            </CardContent>
                        </Card>
                    </Link>
                ))}
            </div>
        </div>
    );
}
