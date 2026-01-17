"use client";

import { Database, Plus, Search, MoreVertical, CheckCircle, XCircle } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";

const vectorDbs = [
    { id: "qdrant", name: "Qdrant", url: "localhost:6335", collections: 5, status: "connected" },
    { id: "pinecone", name: "Pinecone", url: "us-east-1.pinecone.io", collections: 3, status: "connected" },
    { id: "weaviate", name: "Weaviate", url: "localhost:8080", collections: 2, status: "disconnected" },
];

export default function VectorDBsPage() {
    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Vector Databases</h1>
                    <p className="text-muted-foreground">Manage vector database connections</p>
                </div>
                <Button>
                    <Plus className="mr-2 h-4 w-4" />
                    Add Database
                </Button>
            </div>

            <div className="flex items-center gap-4">
                <div className="relative flex-1 max-w-sm">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <Input placeholder="Search databases..." className="pl-10" />
                </div>
            </div>

            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {vectorDbs.map((db) => (
                    <Card key={db.id}>
                        <CardHeader className="flex flex-row items-center justify-between pb-2">
                            <div className="flex items-center gap-3">
                                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                                    <Database className="h-5 w-5 text-primary" />
                                </div>
                                <div>
                                    <CardTitle className="text-lg">{db.name}</CardTitle>
                                    <CardDescription className="text-xs">{db.url}</CardDescription>
                                </div>
                            </div>
                            <Button variant="ghost" size="icon">
                                <MoreVertical className="h-4 w-4" />
                            </Button>
                        </CardHeader>
                        <CardContent>
                            <div className="flex items-center justify-between">
                                <span className="text-sm text-muted-foreground">{db.collections} collections</span>
                                <Badge variant={db.status === "connected" ? "default" : "destructive"} className="gap-1">
                                    {db.status === "connected" ? (
                                        <CheckCircle className="h-3 w-3" />
                                    ) : (
                                        <XCircle className="h-3 w-3" />
                                    )}
                                    {db.status}
                                </Badge>
                            </div>
                        </CardContent>
                    </Card>
                ))}
            </div>
        </div>
    );
}
