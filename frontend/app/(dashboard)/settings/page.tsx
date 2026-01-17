"use client";

import { Settings, Moon, Sun, Monitor, Bell, Shield, Key, Mail, Save } from "lucide-react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Input } from "@/components/ui/input";

export default function SettingsPage() {
    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold">Settings</h1>
                    <p className="text-muted-foreground">Manage your preferences</p>
                </div>
            </div>

            <div className="grid gap-6 lg:grid-cols-2">
                {/* Appearance */}
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2 text-base">
                            <Sun className="h-4 w-4" />
                            Appearance
                        </CardTitle>
                        <CardDescription>Customize the look and feel</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <Label>Theme</Label>
                                <p className="text-xs text-muted-foreground">Select your preferred theme</p>
                            </div>
                            <div className="flex gap-1">
                                <Button variant="outline" size="sm" className="h-8 px-2">
                                    <Sun className="h-4 w-4" />
                                </Button>
                                <Button variant="outline" size="sm" className="h-8 px-2">
                                    <Moon className="h-4 w-4" />
                                </Button>
                                <Button variant="outline" size="sm" className="h-8 px-2">
                                    <Monitor className="h-4 w-4" />
                                </Button>
                            </div>
                        </div>
                    </CardContent>
                </Card>

                {/* Notifications */}
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2 text-base">
                            <Bell className="h-4 w-4" />
                            Notifications
                        </CardTitle>
                        <CardDescription>Configure notification preferences</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-3">
                        <div className="flex items-center justify-between">
                            <Label className="text-sm">Email notifications</Label>
                            <Switch />
                        </div>
                        <div className="flex items-center justify-between">
                            <Label className="text-sm">Budget alerts</Label>
                            <Switch defaultChecked />
                        </div>
                        <div className="flex items-center justify-between">
                            <Label className="text-sm">Security alerts</Label>
                            <Switch defaultChecked />
                        </div>
                    </CardContent>
                </Card>

                {/* SMTP Configuration */}
                <Card className="lg:col-span-2">
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2 text-base">
                            <Mail className="h-4 w-4" />
                            SMTP Configuration
                        </CardTitle>
                        <CardDescription>Configure email server for notifications</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className="grid gap-4 md:grid-cols-2">
                            <div className="space-y-2">
                                <Label>SMTP Host</Label>
                                <Input placeholder="smtp.example.com" />
                            </div>
                            <div className="space-y-2">
                                <Label>SMTP Port</Label>
                                <Input placeholder="587" type="number" />
                            </div>
                            <div className="space-y-2">
                                <Label>Username</Label>
                                <Input placeholder="your-email@example.com" />
                            </div>
                            <div className="space-y-2">
                                <Label>Password</Label>
                                <Input type="password" placeholder="••••••••" />
                            </div>
                            <div className="space-y-2">
                                <Label>From Email</Label>
                                <Input placeholder="noreply@barq.hub" />
                            </div>
                            <div className="space-y-2">
                                <Label>From Name</Label>
                                <Input placeholder="BARQ HUB" />
                            </div>
                        </div>
                        <div className="flex items-center gap-3 mt-4 pt-4 border-t">
                            <div className="flex items-center gap-2">
                                <Switch id="smtp-tls" defaultChecked />
                                <Label htmlFor="smtp-tls" className="text-sm">Use TLS</Label>
                            </div>
                            <div className="flex-1" />
                            <Button variant="outline" size="sm">Test Connection</Button>
                            <Button size="sm" className="gap-2">
                                <Save className="h-4 w-4" />
                                Save SMTP Settings
                            </Button>
                        </div>
                    </CardContent>
                </Card>

                {/* Security */}
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2 text-base">
                            <Shield className="h-4 w-4" />
                            Security
                        </CardTitle>
                        <CardDescription>Manage security settings</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <Label>Two-factor authentication</Label>
                                <p className="text-xs text-muted-foreground">Add extra security</p>
                            </div>
                            <Button variant="outline" size="sm">Enable</Button>
                        </div>
                        <Separator />
                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <Label>API Keys</Label>
                                <p className="text-xs text-muted-foreground">Manage access keys</p>
                            </div>
                            <Button variant="outline" size="sm" className="gap-2">
                                <Key className="h-4 w-4" />
                                Manage
                            </Button>
                        </div>
                    </CardContent>
                </Card>

                {/* Data Export */}
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2 text-base">
                            <Settings className="h-4 w-4" />
                            Data & Storage
                        </CardTitle>
                        <CardDescription>Manage your data</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <Label>Export Data</Label>
                                <p className="text-xs text-muted-foreground">Download all your data</p>
                            </div>
                            <Button variant="outline" size="sm">Export</Button>
                        </div>
                        <Separator />
                        <div className="flex items-center justify-between">
                            <div className="space-y-0.5">
                                <Label className="text-destructive">Delete Account</Label>
                                <p className="text-xs text-muted-foreground">Permanently delete account</p>
                            </div>
                            <Button variant="destructive" size="sm">Delete</Button>
                        </div>
                    </CardContent>
                </Card>
            </div>
        </div>
    );
}
