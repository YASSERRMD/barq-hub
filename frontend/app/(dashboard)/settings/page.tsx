"use client";

import { useState, useEffect } from "react";
import { Save, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { settingsApi } from "@/lib/api";
import { toast } from "sonner";
import { Settings, SmtpSettings } from "@/types";

export default function SettingsPage() {
    const [settings, setSettings] = useState<Settings | null>(null);
    const [smtp, setSmtp] = useState<SmtpSettings | null>(null);
    const [loading, setLoading] = useState(true);
    const [saving, setSaving] = useState(false);

    useEffect(() => {
        const loadSettings = async () => {
            try {
                const [settingsData, smtpData] = await Promise.all([
                    settingsApi.get(),
                    settingsApi.getSmtp()
                ]);
                setSettings(settingsData);
                setSmtp(smtpData);
            } catch (error) {
                console.error("Failed to load settings", error);
                toast.error("Failed to load settings");
            } finally {
                setLoading(false);
            }
        };

        loadSettings();
    }, []);

    const handleSaveSettings = async () => {
        if (!settings) return;
        setSaving(true);
        try {
            await settingsApi.update(settings);
            toast.success("Settings saved successfully");
        } catch (error) {
            toast.error("Failed to save settings");
        } finally {
            setSaving(false);
        }
    };

    const handleSaveSmtp = async () => {
        if (!smtp) return;
        setSaving(true);
        try {
            await settingsApi.updateSmtp(smtp);
            toast.success("SMTP settings saved");
        } catch (error) {
            toast.error("Failed to save SMTP settings");
        } finally {
            setSaving(false);
        }
    };

    const handleTestSmtp = async () => {
        const toastId = toast.loading("Testing SMTP connection...");
        try {
            await settingsApi.testSmtp();
            toast.success("SMTP connection successful", { id: toastId });
        } catch (error) {
            toast.error("SMTP connection failed", { id: toastId });
        }
    };

    if (loading) {
        return <div className="flex justify-center items-center h-[50vh]"><Loader2 className="w-8 h-8 animate-spin text-muted-foreground" /></div>;
    }

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Settings</h1>
                    <p className="text-muted-foreground">Manage your application preferences</p>
                </div>
            </div>

            <Tabs defaultValue="general" className="w-full">
                <TabsList className="mb-4">
                    <TabsTrigger value="general">General</TabsTrigger>
                    <TabsTrigger value="notifications">Notifications</TabsTrigger>
                    <TabsTrigger value="smtp">SMTP Configuration</TabsTrigger>
                    <TabsTrigger value="security">Security</TabsTrigger>
                </TabsList>

                <TabsContent value="general">
                    <Card>
                        <CardHeader>
                            <CardTitle>General Settings</CardTitle>
                            <CardDescription>Configure basic application settings.</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="space-y-2">
                                <Label>Organization Name</Label>
                                <Input defaultValue="Acme Corp" disabled />
                            </div>
                            <div className="flex items-center justify-between">
                                <div className="space-y-0.5">
                                    <Label>Budget Monthly Limit ($)</Label>
                                    <p className="text-sm text-muted-foreground">Hard cap for spending alerts</p>
                                </div>
                                <Input
                                    type="number"
                                    className="w-[120px]"
                                    value={settings?.budgetLimit || ""}
                                    onChange={(e) => setSettings(prev => prev ? { ...prev, budgetLimit: parseFloat(e.target.value) } : null)}
                                />
                            </div>
                        </CardContent>
                        <CardFooter>
                            <Button onClick={handleSaveSettings} disabled={saving}>
                                {saving ? <Loader2 className="w-4 h-4 animate-spin mr-2" /> : <Save className="w-4 h-4 mr-2" />}
                                Save Changes
                            </Button>
                        </CardFooter>
                    </Card>
                </TabsContent>

                <TabsContent value="notifications">
                    <Card>
                        <CardHeader>
                            <CardTitle>Notification Preferences</CardTitle>
                            <CardDescription>Choose what alerts you want to receive.</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="flex items-center justify-between">
                                <Label>Email Notifications</Label>
                                <Switch
                                    checked={settings?.emailNotifications}
                                    onCheckedChange={(c) => setSettings(prev => prev ? { ...prev, emailNotifications: c } : null)}
                                />
                            </div>
                            <div className="flex items-center justify-between">
                                <Label>Budget Alerts</Label>
                                <Switch
                                    checked={settings?.budgetAlerts}
                                    onCheckedChange={(c) => setSettings(prev => prev ? { ...prev, budgetAlerts: c } : null)}
                                />
                            </div>
                            <div className="flex items-center justify-between">
                                <Label>Security Alerts</Label>
                                <Switch
                                    checked={settings?.securityAlerts}
                                    onCheckedChange={(c) => setSettings(prev => prev ? { ...prev, securityAlerts: c } : null)}
                                />
                            </div>
                        </CardContent>
                        <CardFooter>
                            <Button onClick={handleSaveSettings} disabled={saving}>
                                {saving ? <Loader2 className="w-4 h-4 animate-spin mr-2" /> : <Save className="w-4 h-4 mr-2" />}
                                Save Changes
                            </Button>
                        </CardFooter>
                    </Card>
                </TabsContent>

                <TabsContent value="smtp">
                    <Card>
                        <CardHeader>
                            <CardTitle>SMTP Server</CardTitle>
                            <CardDescription>Configure email server for system notifications.</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-2">
                                    <Label>Host</Label>
                                    <Input
                                        value={smtp?.host || ""}
                                        onChange={(e) => setSmtp(prev => prev ? { ...prev, host: e.target.value } : null)}
                                        placeholder="smtp.example.com"
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label>Port</Label>
                                    <Input
                                        value={smtp?.port || ""}
                                        onChange={(e) => setSmtp(prev => prev ? { ...prev, port: parseInt(e.target.value) } : null)}
                                        placeholder="587"
                                    />
                                </div>
                            </div>
                            <div className="space-y-2">
                                <Label>Username</Label>
                                <Input
                                    value={smtp?.username || ""}
                                    onChange={(e) => setSmtp(prev => prev ? { ...prev, username: e.target.value } : null)}
                                />
                            </div>
                            <div className="space-y-2">
                                <Label>Password</Label>
                                <Input
                                    type="password"
                                    value={smtp?.password || ""}
                                    onChange={(e) => setSmtp(prev => prev ? { ...prev, password: e.target.value } : null)}
                                />
                            </div>
                            <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-2">
                                    <Label>From Email</Label>
                                    <Input
                                        value={smtp?.fromEmail || ""}
                                        onChange={(e) => setSmtp(prev => prev ? { ...prev, fromEmail: e.target.value } : null)}
                                        placeholder="noreply@barq.hub"
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label>From Name</Label>
                                    <Input
                                        value={smtp?.fromName || ""}
                                        onChange={(e) => setSmtp(prev => prev ? { ...prev, fromName: e.target.value } : null)}
                                        placeholder="BARQ System"
                                    />
                                </div>
                            </div>
                        </CardContent>
                        <CardFooter className="justify-between">
                            <Button variant="outline" onClick={handleTestSmtp}>Test Connection</Button>
                            <Button onClick={handleSaveSmtp} disabled={saving}>
                                {saving ? <Loader2 className="w-4 h-4 animate-spin mr-2" /> : "Save SMTP Settings"}
                            </Button>
                        </CardFooter>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    );
}
