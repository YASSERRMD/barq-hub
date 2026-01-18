"use client";

import { useState, useEffect } from "react";
import { Key, Plus, Search, Copy, Check, RefreshCw, Settings, Trash2, Globe, Terminal, Zap, Shield, Loader2, AlertCircle } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { applicationsApi } from "@/lib/api";
import { toast } from "sonner";
import { Application } from "@/types";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogDescription } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";

export default function ApiKeysPage() {
    const [applications, setApplications] = useState<Application[]>([]);
    const [loading, setLoading] = useState(true);
    const [copied, setCopied] = useState<string | null>(null);

    // Create & Edit State
    const [isCreateOpen, setIsCreateOpen] = useState(false);
    const [isEditOpen, setIsEditOpen] = useState(false);
    const [isDeleteOpen, setIsDeleteOpen] = useState(false);
    const [selectedApp, setSelectedApp] = useState<Application | null>(null);
    const [newApp, setNewApp] = useState({
        name: "",
        description: "",
        rateLimit: 60,
        scopes: ["read", "write"] as string[] // simplified for now
    });
    const [creating, setCreating] = useState(false);

    useEffect(() => {
        fetchApps();
    }, []);

    const fetchApps = async () => {
        try {
            const data = await applicationsApi.list();
            setApplications(data);
        } catch (error) {
            console.error("Failed to load API keys", error);
            toast.error("Failed to load API keys");
        } finally {
            setLoading(false);
        }
    };

    const handleRotate = async (appId: string, appName: string) => {
        if (!confirm(`Are you sure you want to rotate the key for "${appName}"? The old key will stop working immediately.`)) return;

        try {
            const data = await applicationsApi.rotateKey(appId);
            // Show the new key
            toast.success("API Key rotated successfully");
            setCopied(appId); // Highlight/reuse copy logic concept if needed, or just show modal

            // For rotation, we ideally want to show the new key to the user.
            // The backend returns { api_key: "..." }
            // Let's reuse the Create Success idea or a custom alert.
            // For now, let's copy to clipboard automatically and notify.
            navigator.clipboard.writeText(data.api_key);
            toast.message("New API Key copied to clipboard", {
                description: "Make sure to update your applications immediately."
            });

            fetchApps();
        } catch (error) {
            console.error("Failed to rotate key", error);
            toast.error("Failed to rotate API key");
        }
    };

    const openEdit = (app: Application) => {
        setSelectedApp(app);
        setNewApp({
            name: app.name,
            description: app.description || "",
            rateLimit: app.rateLimit,
            scopes: app.scopes
        });
        setIsEditOpen(true);
    };

    const handleUpdate = async () => {
        if (!selectedApp) return;
        setCreating(true);
        try {
            await applicationsApi.update(selectedApp.id, {
                name: newApp.name,
                description: newApp.description,
                rateLimit: newApp.rateLimit,
                scopes: newApp.scopes
            });
            toast.success("Application updated successfully");
            setIsEditOpen(false);
            fetchApps();
        } catch (error) {
            console.error("Failed to update", error);
            toast.error("Failed to update application");
        } finally {
            setCreating(false);
        }
    };

    const handleCreate = async () => {
        if (!newApp.name) {
            toast.error("Name is required");
            return;
        }
        setCreating(true);
        try {
            const res = await applicationsApi.create(newApp);
            toast.success("API Key created successfully");

            // Should show the key to user
            // We can reuse a dialog or just copy it
            navigator.clipboard.writeText(res.api_key);
            toast.message("API Key copied to clipboard", {
                description: "This is the only time you will see this key."
            });

            setIsCreateOpen(false);
            setNewApp({ name: "", description: "", rateLimit: 60, scopes: ["read", "write"] });
            fetchApps();
        } catch (error) {
            console.error("Failed to create API key", error);
            toast.error("Failed to create API key");
        } finally {
            setCreating(false);
        }
    };

    const handleDelete = async () => {
        if (!selectedApp) return;
        try {
            await applicationsApi.delete(selectedApp.id);
            toast.success("API Key deleted");
            setIsDeleteOpen(false);
            fetchApps();
        } catch (error) {
            console.error("Failed to delete", error);
            toast.error("Failed to delete API key");
        }
    };

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
                    <h1 className="text-2xl font-bold">API Keys</h1>
                    <p className="text-muted-foreground">Manage API access tokens and applications</p>
                </div>
                <Button onClick={() => {
                    setNewApp({ name: "", description: "", rateLimit: 60, scopes: ["read", "write"] });
                    setIsCreateOpen(true);
                }} className="bg-gradient-to-r from-violet-600 to-blue-600 text-white shadow-lg shadow-violet-500/25 hover:shadow-violet-500/40 transition-shadow">
                    <Plus className="mr-2 h-4 w-4" />
                    Create API Key
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
                                <p className="text-xs text-muted-foreground">Active Keys</p>
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
                                <p className="text-xs text-muted-foreground">Operational</p>
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
                    <Input placeholder="Search API keys..." className="pl-10" />
                </div>
            </div>

            {/* Applications List */}
            <div className="space-y-4">
                {applications.length === 0 ? (
                    <div className="text-center py-12 text-muted-foreground bg-muted/20 rounded-lg">
                        <Key className="w-12 h-12 mx-auto text-muted-foreground/50 mb-3" />
                        <h3 className="text-lg font-medium mb-1">No API Keys Found</h3>
                        <p className="mb-4 text-sm">Create an API key to start making requests.</p>
                        <Button onClick={() => setIsCreateOpen(true)} variant="outline">Create API Key</Button>
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
                                            <p className="text-sm text-muted-foreground mt-0.5">{app.description || "No description provided"}</p>

                                            {/* Key & Stats */}
                                            <div className="flex items-center gap-4 mt-3 text-sm text-muted-foreground">
                                                <button
                                                    onClick={() => copyKey(app.id, app.apiKeyPrefix)}
                                                    className="flex items-center gap-1 hover:text-foreground transition-colors"
                                                >
                                                    <code className="text-xs bg-muted px-1.5 py-0.5 rounded">{app.apiKeyPrefix}****************</code>
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
                                        <Button
                                            variant="outline"
                                            size="sm"
                                            onClick={() => handleRotate(app.id, app.name)}
                                        >
                                            <RefreshCw className="w-4 h-4 mr-1" />
                                            Rotate
                                        </Button>
                                        <Button
                                            variant="outline"
                                            size="icon"
                                            className="h-8 w-8"
                                            onClick={() => openEdit(app)}
                                        >
                                            <Settings className="w-4 h-4" />
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            className="h-8 w-8 text-red-600 hover:text-red-700 hover:bg-red-50 dark:hover:bg-red-950/30"
                                            onClick={() => { setSelectedApp(app); setIsDeleteOpen(true); }}
                                        >
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

            {/* Create Dialog */}
            <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Create API Key</DialogTitle>
                        <DialogDescription>
                            Generate a new API key for your application.
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4 py-4">
                        <div className="space-y-2">
                            <Label>Application Name</Label>
                            <Input
                                placeholder="My Awesome App"
                                value={newApp.name}
                                onChange={(e) => setNewApp({ ...newApp, name: e.target.value })}
                            />
                        </div>
                        <div className="space-y-2">
                            <Label>Description</Label>
                            <Input
                                placeholder="Used for backend integration..."
                                value={newApp.description}
                                onChange={(e) => setNewApp({ ...newApp, description: e.target.value })}
                            />
                        </div>
                        <div className="space-y-2">
                            <Label>Rate Limit (requests/min)</Label>
                            <Input
                                type="number"
                                value={newApp.rateLimit}
                                onChange={(e) => setNewApp({ ...newApp, rateLimit: parseInt(e.target.value) || 60 })}
                            />
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setIsCreateOpen(false)}>Cancel</Button>
                        <Button
                            onClick={handleCreate}
                            disabled={creating || !newApp.name}
                            className="bg-primary text-primary-foreground"
                        >
                            {creating ? "Creating..." : "Create Key"}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            {/* Edit Dialog */}
            <Dialog open={isEditOpen} onOpenChange={setIsEditOpen}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Edit Application</DialogTitle>
                        <DialogDescription>
                            Update settings for <strong>{selectedApp?.name}</strong>.
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4 py-4">
                        <div className="space-y-2">
                            <Label>Application Name</Label>
                            <Input
                                placeholder="My Awesome App"
                                value={newApp.name}
                                onChange={(e) => setNewApp({ ...newApp, name: e.target.value })}
                            />
                        </div>
                        <div className="space-y-2">
                            <Label>Description</Label>
                            <Input
                                placeholder="Used for backend integration..."
                                value={newApp.description}
                                onChange={(e) => setNewApp({ ...newApp, description: e.target.value })}
                            />
                        </div>
                        <div className="space-y-2">
                            <Label>Rate Limit (requests/min)</Label>
                            <Input
                                type="number"
                                value={newApp.rateLimit}
                                onChange={(e) => setNewApp({ ...newApp, rateLimit: parseInt(e.target.value) || 60 })}
                            />
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setIsEditOpen(false)}>Cancel</Button>
                        <Button
                            onClick={handleUpdate}
                            disabled={creating || !newApp.name}
                            className="bg-primary text-primary-foreground"
                        >
                            {creating ? "Saving..." : "Save Changes"}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            {/* Delete Dialog */}
            <Dialog open={isDeleteOpen} onOpenChange={setIsDeleteOpen}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle className="text-red-600 flex items-center gap-2">
                            <AlertCircle className="w-5 h-5" />
                            Revoke API Key
                        </DialogTitle>
                        <DialogDescription>
                            Are you sure you want to revoke the key for <strong>{selectedApp?.name}</strong>?
                            This will immediately stop all requests using this key.
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setIsDeleteOpen(false)}>Cancel</Button>
                        <Button variant="destructive" onClick={handleDelete}>Revoke Key</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    );
}
