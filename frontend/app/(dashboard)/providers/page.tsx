"use client";

import { Cpu, Plus, Search, MoreVertical } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";

const providers = [
    { id: "openai", name: "OpenAI", models: ["gpt-4o", "gpt-4-turbo", "gpt-3.5-turbo"], status: "active" },
    { id: "anthropic", name: "Anthropic", models: ["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"], status: "active" },
    { id: "google", name: "Google Gemini", models: ["gemini-1.5-pro", "gemini-1.5-flash"], status: "active" },
    { id: "mistral", name: "Mistral", models: ["mistral-large", "codestral"], status: "inactive" },
];

export default function ProvidersPage() {
    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Providers</h1>
                    <p className="text-muted-foreground">Manage your LLM providers</p>
                </div>
                <Button>
                    <Plus className="mr-2 h-4 w-4" />
                    Add Provider
                </Button>
            </div>

            <div className="flex items-center gap-4">
                <div className="relative flex-1 max-w-sm">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <Input placeholder="Search providers..." className="pl-10" />
                </div>
            </div>

            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {providers.map((provider) => (
                    <Card key={provider.id}>
                        <CardHeader className="flex flex-row items-center justify-between pb-2">
                            <div className="flex items-center gap-3">
                                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                                    <Cpu className="h-5 w-5 text-primary" />
                                </div>
                                <div>
                                    <CardTitle className="text-lg">{provider.name}</CardTitle>
                                    <Badge variant={provider.status === "active" ? "default" : "secondary"}>
                                        {provider.status}
                                    </Badge>
                                </div>
                            </div>
                            <Button variant="ghost" size="icon">
                                <MoreVertical className="h-4 w-4" />
                            </Button>
                        </CardHeader>
                        <CardContent>
                            <CardDescription className="mb-2">Available Models:</CardDescription>
                            <div className="flex flex-wrap gap-1">
                                {provider.models.map((model) => (
                                    <Badge key={model} variant="outline" className="text-xs">
                                        {model}
                                    </Badge>
                                ))}
                            </div>
                        </CardContent>
                    </Card>
                ))}
            </div>
        </div>
    );
}
