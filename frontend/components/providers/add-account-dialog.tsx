"use client";

import { useState } from "react";
import { Plus, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { providersApi } from "@/lib/api";
import { toast } from "sonner";
import { ProviderDefinition } from "@/types";

interface AddAccountDialogProps {
    provider?: ProviderDefinition;
    providers?: ProviderDefinition[];
    onAccountAdded: () => void;
}

export function AddAccountDialog({ provider, providers, onAccountAdded }: AddAccountDialogProps) {
    const [open, setOpen] = useState(false);
    const [isLoading, setIsLoading] = useState(false);
    const [selectedProviderId, setSelectedProviderId] = useState<string>(provider?.id || "");

    // Derived active provider
    const activeProvider = provider || providers?.find(p => p.id === selectedProviderId);

    // Basic Info
    const [name, setName] = useState("");
    const [apiKey, setApiKey] = useState("");
    const [priority, setPriority] = useState("1");

    // Quotas
    const [rpm, setRpm] = useState("");
    const [rph, setRph] = useState("");
    const [rpd, setRpd] = useState("");
    const [rpmMonth, setRpmMonth] = useState("");

    // Pricing
    const [inputPrice, setInputPrice] = useState("0");
    const [outputPrice, setOutputPrice] = useState("0");

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!activeProvider) return;

        setIsLoading(true);

        try {
            await providersApi.createAccount({
                name,
                provider_id: activeProvider.id,
                config: {
                    type: "api_key",
                    api_key: apiKey,
                },
                priority: parseInt(priority),
                quotas: {
                    requests_per_minute: rpm ? parseInt(rpm) : undefined,
                    requests_per_hour: rph ? parseInt(rph) : undefined,
                    requests_per_day: rpd ? parseInt(rpd) : undefined,
                    requests_per_month: rpmMonth ? parseInt(rpmMonth) : undefined,
                },
                pricing: {
                    input_token_price: parseFloat(inputPrice),
                    output_token_price: parseFloat(outputPrice),
                    currency: "USD",
                },
                models: [],
            });
            toast.success("Account added successfully");
            setOpen(false);
            resetForm();
            onAccountAdded();
        } catch (error: any) {
            toast.error(error.response?.data?.message || "Failed to add account");
        } finally {
            setIsLoading(false);
        }
    };

    const resetForm = () => {
        setName("");
        setApiKey("");
        setPriority("1");
        setRpm("");
        setRph("");
        setRpd("");
        setRpmMonth("");
        setInputPrice("0");
        setOutputPrice("0");
        if (!provider) setSelectedProviderId("");
    };

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                {provider ? (
                    <Button size="sm" variant="outline" className="gap-2">
                        <Plus className="h-4 w-4" /> Add Key
                    </Button>
                ) : (
                    <Button className="bg-gradient-to-r from-violet-600 to-cyan-500 text-white shadow-lg shadow-violet-500/25">
                        <Plus className="h-4 w-4 mr-2" /> Add Provider
                    </Button>
                )}
            </DialogTrigger>
            <DialogContent className="max-w-2xl">
                <DialogHeader>
                    <DialogTitle>
                        {activeProvider ? `Add ${activeProvider.name} Account` : "Add Provider Account"}
                    </DialogTitle>
                    <DialogDescription>
                        Configure API key, priority, quotas, and pricing.
                    </DialogDescription>
                </DialogHeader>

                {!activeProvider && providers ? (
                    <div className="space-y-4 py-4">
                        <div className="space-y-2">
                            <Label>Select Provider Type</Label>
                            <Select value={selectedProviderId} onValueChange={setSelectedProviderId}>
                                <SelectTrigger>
                                    <SelectValue placeholder="Choose a provider..." />
                                </SelectTrigger>
                                <SelectContent>
                                    {providers.map(p => (
                                        <SelectItem key={p.id} value={p.id}>
                                            <div className="flex items-center gap-2">
                                                <span>{p.name}</span>
                                                <span className="text-xs text-muted-foreground capitalize">({p.provider_type})</span>
                                            </div>
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>
                    </div>
                ) : (
                    <form onSubmit={handleSubmit} className="space-y-4">
                        <Tabs defaultValue="general" className="w-full">
                            <TabsList className="grid w-full grid-cols-3">
                                <TabsTrigger value="general">General</TabsTrigger>
                                <TabsTrigger value="quotas">Quotas & Limits</TabsTrigger>
                                <TabsTrigger value="pricing">Pricing</TabsTrigger>
                            </TabsList>

                            <TabsContent value="general" className="space-y-4 pt-4">
                                <div className="space-y-2">
                                    <Label htmlFor="name">Account Name</Label>
                                    <Input
                                        id="name"
                                        placeholder="e.g. Production Key 1"
                                        value={name}
                                        onChange={(e) => setName(e.target.value)}
                                        required
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label htmlFor="apiKey">API Key</Label>
                                    <Input
                                        id="apiKey"
                                        type="password"
                                        placeholder={`sk-...`}
                                        value={apiKey}
                                        onChange={(e) => setApiKey(e.target.value)}
                                        required
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label htmlFor="priority">Priority</Label>
                                    <Select value={priority} onValueChange={setPriority}>
                                        <SelectTrigger>
                                            <SelectValue />
                                        </SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="10">High (Primary)</SelectItem>
                                            <SelectItem value="5">Medium</SelectItem>
                                            <SelectItem value="1">Low (Fallback)</SelectItem>
                                        </SelectContent>
                                    </Select>
                                </div>
                            </TabsContent>

                            <TabsContent value="quotas" className="space-y-4 pt-4">
                                <div className="grid grid-cols-2 gap-4">
                                    <div className="space-y-2">
                                        <Label htmlFor="rpm">Requests per Minute (RPM)</Label>
                                        <Input
                                            id="rpm"
                                            type="number"
                                            placeholder="e.g. 3500"
                                            value={rpm}
                                            onChange={(e) => setRpm(e.target.value)}
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <Label htmlFor="rph">Requests per Hour</Label>
                                        <Input
                                            id="rph"
                                            type="number"
                                            placeholder="Unlimited"
                                            value={rph}
                                            onChange={(e) => setRph(e.target.value)}
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <Label htmlFor="rpd">Requests per Day</Label>
                                        <Input
                                            id="rpd"
                                            type="number"
                                            placeholder="Unlimited"
                                            value={rpd}
                                            onChange={(e) => setRpd(e.target.value)}
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <Label htmlFor="rpmMonth">Requests per Month</Label>
                                        <Input
                                            id="rpmMonth"
                                            type="number"
                                            placeholder="Unlimited"
                                            value={rpmMonth}
                                            onChange={(e) => setRpmMonth(e.target.value)}
                                        />
                                    </div>
                                </div>
                            </TabsContent>

                            <TabsContent value="pricing" className="space-y-4 pt-4">
                                <div className="p-4 rounded-lg bg-muted text-sm text-center mb-4 text-muted-foreground">
                                    Pricing is used to estimate usage costs. Enter values in USD per 1 Million Tokens.
                                </div>
                                <div className="grid grid-cols-2 gap-4">
                                    <div className="space-y-2">
                                        <Label htmlFor="inputPrice">Input Token Price ($ / 1M)</Label>
                                        <Input
                                            id="inputPrice"
                                            type="number"
                                            step="0.01"
                                            placeholder="0.50"
                                            value={inputPrice}
                                            onChange={(e) => setInputPrice(e.target.value)}
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <Label htmlFor="outputPrice">Output Token Price ($ / 1M)</Label>
                                        <Input
                                            id="outputPrice"
                                            type="number"
                                            step="0.01"
                                            placeholder="1.50"
                                            value={outputPrice}
                                            onChange={(e) => setOutputPrice(e.target.value)}
                                        />
                                    </div>
                                </div>
                            </TabsContent>
                        </Tabs>

                        <DialogFooter>
                            <Button type="button" variant="outline" onClick={() => setOpen(false)}>
                                Cancel
                            </Button>
                            <Button type="submit" disabled={isLoading} className="bg-gradient-to-r from-violet-600 to-cyan-500 text-white">
                                {isLoading ? <Loader2 className="w-4 h-4 animate-spin" /> : "Save Account"}
                            </Button>
                        </DialogFooter>
                    </form>
                )}
            </DialogContent>
        </Dialog>
    );
}
