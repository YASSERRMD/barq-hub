"use client";

import React, { useState, useEffect } from 'react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogDescription, DialogFooter } from '@/components/ui/dialog';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import {
    Plus,
    Settings,
    Trash2,
    Key,
    Server,
    Cpu,
    Cloud,
    Database,
    ChevronRight,
    RefreshCw,
    Shield,
    AlertCircle,
    Clock,
    Zap,
    RotateCcw,
    Check,
    X,
    Timer,
    Volume2,
    Mic
} from 'lucide-react';
import { getProviderLogo } from "@/components/providers/provider-logos";

// Types matching the Synapse reference implementation
type QuotaPeriod = 'minute' | 'hour' | 'day' | 'month';

interface QuotaTier {
    period: QuotaPeriod;
    tokenLimit: number;
    tokensUsed: number;
    requestLimit?: number;
    requestsUsed: number;
    secondsUntilReset: number;
}

interface ProviderAccount {
    id: string;
    name: string;
    providerId: string;
    enabled: boolean;
    isDefault: boolean;
    priority: number;
    quotas: QuotaTier[];
    models: ProviderModel[];
    blockingTier?: QuotaPeriod;
}

interface ProviderModel {
    id: string;
    name: string;
    capabilities: ModelCapability[];
    input_token_cost?: number;
    output_token_cost?: number;
}

type ModelCapability = 'llm' | 'embedding' | 'tts' | 'stt' | 'imagegeneration';

interface Provider {
    id: string;
    name: string;
    type: 'llm' | 'embedding' | 'both';
    gradient: string;
    isAWS?: boolean;
    isAzure?: boolean;
    accounts: ProviderAccount[];
    models: ProviderModel[];
}

const quotaPeriodLabels: Record<QuotaPeriod, string> = {
    minute: 'Per Minute',
    hour: 'Per Hour',
    day: 'Per Day',
    month: 'Per Month',
};

interface QuotaConfig {
    enabled: boolean;
    tokenLimit: number;
    requestLimit: number;
}

export default function ProvidersPage() {
    const [selectedId, setSelectedId] = useState<string | null>(null);
    const [isAddOpen, setIsAddOpen] = useState(false);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // New account form state with multi-tier quotas
    const [newAccount, setNewAccount] = useState({
        name: '',
        apiKey: '',
        endpoint: '',
        deploymentName: '',
        apiVersion: '2024-02-01',
        region: 'us-east-1',
        accessKeyId: '',
        secretAccessKey: '',
        priority: 0,
        quotas: {
            minute: { enabled: false, tokenLimit: 10000, requestLimit: 60 } as QuotaConfig,
            hour: { enabled: false, tokenLimit: 100000, requestLimit: 1000 } as QuotaConfig,
            day: { enabled: false, tokenLimit: 1000000, requestLimit: 10000 } as QuotaConfig,
            month: { enabled: true, tokenLimit: 10000000, requestLimit: 0 } as QuotaConfig,
        },
        models: [] as ProviderModel[],
        newModel: '',
        newModelInputCost: '',
        newModelOutputCost: '',
    });

    const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000/v1';

    // Gradient mapping
    const providerGradients: Record<string, string> = {
        'openai': 'from-emerald-500 to-teal-600',
        'anthropic': 'from-orange-500 to-amber-600',
        'azure': 'from-blue-500 to-indigo-600',
        'bedrock': 'from-amber-500 to-orange-600',
        'gemini': 'from-blue-400 to-violet-500',
        'mistral': 'from-rose-500 to-pink-600',
        'groq': 'from-purple-500 to-violet-600',
        'cohere': 'from-indigo-500 to-purple-600',
        'voyage': 'from-sky-500 to-cyan-600',
        'jina': 'from-teal-500 to-emerald-600',
        'together': 'from-cyan-500 to-blue-600',
        'local': 'from-slate-600 to-slate-800'
    };

    const [providers, setProviders] = useState<Provider[]>([]);
    const [fetchingProviders, setFetchingProviders] = useState(true);
    const [successMessage, setSuccessMessage] = useState<string | null>(null);

    // Fetch all providers and their accounts
    const fetchProvidersAndAccounts = async () => {
        setFetchingProviders(true);
        try {
            // Fetch provider definitions
            const providersRes = await fetch(`${apiUrl}/provider-accounts/providers`);
            if (!providersRes.ok) throw new Error('Failed to fetch providers');

            const providerDefs = await providersRes.json();

            // Fetch accounts for each provider
            const providersWithAccounts: Provider[] = await Promise.all(
                providerDefs
                    .filter((p: any) => p.category === 'llm' || p.category === 'embedding' || p.provider_type === 'llm' || p.provider_type === 'both') // STRICTLY LLM/Embedding only
                    .map(async (p: any) => {
                        let accounts: ProviderAccount[] = [];
                        try {
                            const accountsRes = await fetch(`${apiUrl}/provider-accounts/${p.id}/accounts`);
                            if (accountsRes.ok) {
                                const rawAccounts = await accountsRes.json();
                                accounts = rawAccounts.map((a: any) => ({
                                    id: a.id,
                                    name: a.name,
                                    providerId: a.provider_id,
                                    enabled: a.enabled,
                                    isDefault: a.is_default,
                                    priority: a.priority || 0,
                                    models: a.models || [],
                                    quotas: Object.entries(a.quotas || {}).map(([period, q]: [string, any]) => ({
                                        period: period as QuotaPeriod,
                                        tokenLimit: q.token_limit || 0,
                                        tokensUsed: q.tokens_used || 0,
                                        requestLimit: q.request_limit,
                                        requestsUsed: q.requests_used || 0,
                                        secondsUntilReset: q.seconds_until_reset || 0,
                                    })),
                                }));
                            }
                        } catch (e) {
                            console.error(`Failed to fetch accounts for ${p.id}:`, e);
                        }

                        return {
                            id: p.id,
                            name: p.name,
                            type: p.provider_type === 'both' ? 'both' :
                                p.provider_type === 'llm' ? 'llm' : 'embedding',
                            gradient: providerGradients[p.id] || 'from-slate-500 to-slate-600',
                            isAWS: p.requires_aws_config,
                            isAzure: p.requires_azure_config,
                            models: p.default_models || [],
                            accounts,
                        } as Provider;
                    })
            );

            // Sort providers alphabet
            providersWithAccounts.sort((a, b) => a.name.localeCompare(b.name));
            setProviders(providersWithAccounts);
        } catch (err: any) {
            console.error('Failed to fetch providers:', err);
            setError(err.message);
        } finally {
            setFetchingProviders(false);
        }
    };

    useEffect(() => {
        fetchProvidersAndAccounts();
    }, []);

    const openAddDialog = () => {
        const provider = providers.find(p => p.id === selectedId);
        if (provider) {
            setNewAccount(prev => ({ ...prev, models: [...provider.models] }));
        }
        setIsAddOpen(true);
    };

    const addModelToAddForm = () => {
        const name = newAccount.newModel.trim();
        if (name && !newAccount.models.some(m => m.name === name)) {
            const newModelObj: ProviderModel = {
                id: name.toLowerCase().replace(/\s+/g, '-'),
                name: name,
                capabilities: newModelCapabilities,
                input_token_cost: newAccount.newModelInputCost ? parseFloat(newAccount.newModelInputCost) : undefined,
                output_token_cost: newAccount.newModelOutputCost ? parseFloat(newAccount.newModelOutputCost) : undefined,
            };
            setNewAccount({
                ...newAccount,
                models: [...newAccount.models, newModelObj],
                newModel: '',
                newModelInputCost: '',
                newModelOutputCost: '',
            });
        }
    };

    const removeModelFromAddForm = (model: ProviderModel) => {
        setNewAccount({
            ...newAccount,
            models: newAccount.models.filter(m => m.name !== model.name),
        });
    };

    // Create account logic
    const handleCreateAccount = async () => {
        if (!selectedId || !newAccount.name) return;

        setLoading(true);
        setError(null);

        try {
            const provider = providers.find(p => p.id === selectedId);
            if (!provider) return;

            // Build config based on provider type
            let config: any;
            if (provider.isAzure) {
                config = {
                    type: 'azure',
                    endpoint: newAccount.endpoint,
                    deployment_name: newAccount.deploymentName,
                    api_version: newAccount.apiVersion,
                    api_key: newAccount.apiKey,
                };
            } else if (provider.isAWS) {
                config = {
                    type: 'aws',
                    region: newAccount.region,
                    access_key_id: newAccount.accessKeyId,
                    secret_access_key: newAccount.secretAccessKey,
                };
            } else {
                config = {
                    type: 'api_key',
                    api_key: newAccount.apiKey,
                };
            }

            // Build quotas array
            const quotas = Object.entries(newAccount.quotas)
                .filter(([_, q]) => q.enabled)
                .map(([period, q]) => ({
                    period,
                    token_limit: q.tokenLimit,
                    request_limit: q.requestLimit > 0 ? q.requestLimit : undefined,
                }));

            const response = await fetch(`${apiUrl}/provider-accounts/accounts`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name: newAccount.name,
                    provider_id: selectedId,
                    config,
                    priority: newAccount.priority,
                    models: newAccount.models.length > 0 ? newAccount.models : provider.models,
                    quotas: quotas.length > 0 ? quotas : undefined,
                }),
            });

            if (!response.ok) {
                const err = await response.json();
                throw new Error(err.message || 'Failed to create account');
            }

            const newAccountData = await response.json();

            // Success - close dialog and reset form
            setIsAddOpen(false);
            setNewAccount({
                name: '',
                apiKey: '',
                endpoint: '',
                deploymentName: '',
                apiVersion: '2024-02-01',
                region: 'us-east-1',
                accessKeyId: '',
                secretAccessKey: '',
                priority: 0,
                quotas: {
                    minute: { enabled: false, tokenLimit: 10000, requestLimit: 60 },
                    hour: { enabled: false, tokenLimit: 100000, requestLimit: 1000 },
                    day: { enabled: false, tokenLimit: 1000000, requestLimit: 10000 },
                    month: { enabled: true, tokenLimit: 10000000, requestLimit: 0 },
                },
                models: [],
                newModel: '',
                newModelInputCost: '',
                newModelOutputCost: '',
            });

            setSuccessMessage(`Account "${newAccountData.name}" created successfully!`);
            setTimeout(() => setSuccessMessage(null), 5000);
            await fetchProvidersAndAccounts();
        } catch (err: any) {
            setError(err.message || 'Failed to create account');
        } finally {
            setLoading(false);
        }
    };

    // Delete account logic
    const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
    const [accountToDelete, setAccountToDelete] = useState<{ id: string; name: string } | null>(null);
    const [deleting, setDeleting] = useState(false);

    const handleDeleteAccount = async () => {
        if (!accountToDelete) return;

        setDeleting(true);
        setError(null);

        try {
            const response = await fetch(`${apiUrl}/provider-accounts/accounts/${accountToDelete.id}`, {
                method: 'DELETE',
            });

            if (!response.ok && response.status !== 204) {
                throw new Error('Failed to delete account');
            }

            setDeleteConfirmOpen(false);
            setAccountToDelete(null);
            setSuccessMessage(`Account "${accountToDelete.name}" deleted successfully!`);
            setTimeout(() => setSuccessMessage(null), 5000);
            await fetchProvidersAndAccounts();
        } catch (err: any) {
            setError(err.message || 'Failed to delete account');
        } finally {
            setDeleting(false);
        }
    };

    // Edit account state logic (Ported from Synapse)
    const [editDialogOpen, setEditDialogOpen] = useState(false);
    const [editingAccount, setEditingAccount] = useState<ProviderAccount | null>(null);
    const [editForm, setEditForm] = useState({
        name: '',
        enabled: true,
        priority: 0,
        quotas: [] as QuotaTier[],
        models: [] as ProviderModel[],
        newModel: '',
        newModelInputCost: '',
        newModelOutputCost: '',
    });
    const [saving, setSaving] = useState(false);
    const [newModelCapabilities, setNewModelCapabilities] = useState<ModelCapability[]>(['llm']);

    const openEditDialog = (account: ProviderAccount) => {
        setEditingAccount(account);
        setEditForm({
            name: account.name,
            enabled: account.enabled,
            priority: account.priority,
            quotas: JSON.parse(JSON.stringify(account.quotas || [])),
            models: [...account.models],
            newModel: '',
            newModelInputCost: '',
            newModelOutputCost: '',
        });
        setEditDialogOpen(true);
    };

    const addModelToEditForm = () => {
        const name = editForm.newModel.trim();
        if (name && !editForm.models.some(m => m.name === name)) {
            const newModelObj: ProviderModel = {
                id: name.toLowerCase().replace(/\s+/g, '-'),
                name: name,
                capabilities: newModelCapabilities,
                input_token_cost: editForm.newModelInputCost ? parseFloat(editForm.newModelInputCost) : undefined,
                output_token_cost: editForm.newModelOutputCost ? parseFloat(editForm.newModelOutputCost) : undefined,
            };
            setEditForm({
                ...editForm,
                models: [...editForm.models, newModelObj],
                newModel: '',
                newModelInputCost: '',
                newModelOutputCost: '',
            });
        }
    };

    const removeModelFromEditForm = (model: ProviderModel) => {
        setEditForm({
            ...editForm,
            models: editForm.models.filter(m => m.name !== model.name),
        });
    };

    const handleSaveAccount = async () => {
        if (!editingAccount) return;
        setSaving(true);
        setError(null);
        try {
            const response = await fetch(`${apiUrl}/provider-accounts/accounts/${editingAccount.id}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name: editForm.name,
                    enabled: editForm.enabled,
                    priority: editForm.priority,
                    quotas: editForm.quotas,
                    models: editForm.models,
                }),
            });

            if (!response.ok) {
                const err = await response.json();
                throw new Error(err.message || 'Failed to update account');
            }

            setEditDialogOpen(false);
            setEditingAccount(null);
            setSuccessMessage(`Account "${editForm.name}" updated successfully!`);
            setTimeout(() => setSuccessMessage(null), 5000);
            await fetchProvidersAndAccounts();
        } catch (err: any) {
            setError(err.message || 'Failed to update account');
        } finally {
            setSaving(false);
        }
    };

    // --- RENDER HELPERS ---

    const selected = providers.find(p => p.id === selectedId);

    const formatTime = (seconds: number): string => {
        if (seconds < 60) return `${seconds}s`;
        if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
        if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
        return `${Math.floor(seconds / 86400)}d`;
    };

    const hasQuota = (account: ProviderAccount): boolean => {
        return !account.blockingTier && account.quotas.every(q => q.tokensUsed < q.tokenLimit);
    };

    const renderQuotaTier = (tier: QuotaTier, isBlocking: boolean) => {
        const percentage = (tier.tokensUsed / tier.tokenLimit) * 100;
        return (
            <div key={tier.period} className={`p-3 rounded-lg ${isBlocking ? 'bg-red-500/10 border border-red-500/30' : 'bg-muted/50'}`}>
                <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                        <span className="text-sm font-medium">{quotaPeriodLabels[tier.period]}</span>
                        {isBlocking && (
                            <Badge variant="destructive" className="text-xs gap-1">
                                <AlertCircle className="w-3 h-3" />
                                Exhausted
                            </Badge>
                        )}
                    </div>
                    <div className="flex items-center gap-1 text-xs text-muted-foreground">
                        <Timer className="w-3 h-3" />
                        Resets in {formatTime(tier.secondsUntilReset)}
                    </div>
                </div>

                <div className="space-y-1">
                    <div className="flex justify-between text-xs">
                        <span>Tokens</span>
                        <span className={percentage >= 100 ? 'text-red-500 font-medium' : ''}>
                            {tier.tokensUsed.toLocaleString()} / {tier.tokenLimit.toLocaleString()}
                        </span>
                    </div>
                    <div className="h-1.5 bg-muted rounded-full overflow-hidden">
                        <div
                            className={`h-full rounded-full transition-all ${percentage >= 100 ? 'bg-red-500' : percentage >= 80 ? 'bg-amber-500' : 'bg-emerald-500'}`}
                            style={{ width: `${Math.min(100, percentage)}%` }}
                        />
                    </div>
                    {tier.requestLimit && tier.requestLimit > 0 && (
                        <div className="flex justify-between text-xs text-muted-foreground mt-1">
                            <span>Requests</span>
                            <span>{tier.requestsUsed} / {tier.requestLimit}</span>
                        </div>
                    )}
                </div>
            </div>
        );
    };

    const renderAccountCard = (account: ProviderAccount) => {
        const quotaOk = hasQuota(account);
        return (
            <Card key={account.id} className={`p-5 ${!quotaOk ? 'border-amber-500/50' : ''}`}>
                <div className="flex items-start justify-between mb-4">
                    <div>
                        <div className="flex items-center gap-2">
                            <h4 className="font-semibold">{account.name}</h4>
                            {account.isDefault && <Badge>Default</Badge>}
                            {quotaOk ? (
                                <Badge variant="secondary" className="gap-1 bg-emerald-500/10 text-emerald-600">
                                    <Check className="w-3 h-3" />
                                    Active
                                </Badge>
                            ) : (
                                <Badge variant="secondary" className="gap-1 bg-amber-500/10 text-amber-600">
                                    <RotateCcw className="w-3 h-3" />
                                    Fallback Active
                                </Badge>
                            )}
                        </div>
                        <div className="flex items-center gap-2 mt-1 text-sm text-muted-foreground">
                            <span>Priority: {account.priority}</span>
                            <span className="mx-1">•</span>
                            <span>{account.quotas.length} quota tier{account.quotas.length !== 1 ? 's' : ''}</span>
                        </div>
                    </div>
                    <div className="flex gap-1">
                        <Button variant="ghost" size="icon" onClick={() => openEditDialog(account)} title="Edit account">
                            <Settings className="w-4 h-4" />
                        </Button>
                        <Button
                            variant="ghost"
                            size="icon"
                            className="text-red-600 hover:text-red-700 hover:bg-red-50 dark:hover:bg-red-950/30"
                            onClick={() => { setAccountToDelete({ id: account.id, name: account.name }); setDeleteConfirmOpen(true); }}
                            title="Delete account"
                        >
                            <Trash2 className="w-4 h-4" />
                        </Button>
                    </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-2 mb-4">
                    {account.quotas.map(tier => renderQuotaTier(tier, tier.period === account.blockingTier))}
                </div>

                <div className="space-y-2 pt-3 border-t">
                    <div className="flex items-center justify-between">
                        <Label className="text-xs font-medium text-muted-foreground flex items-center gap-2">
                            <Cpu className="w-3 h-3" />
                            Configured Models
                        </Label>
                        <Button variant="ghost" size="sm" className="h-6 text-[10px] px-2 text-primary hover:text-primary hover:bg-primary/10" onClick={() => openEditDialog(account)}>
                            Manage Models
                        </Button>
                    </div>
                    <div className="flex flex-wrap gap-2">
                        {(account.models || []).map((m, idx) => (
                            <Badge key={String(m.id || m.name || idx)} variant="outline" className="text-xs bg-slate-50 font-normal dark:bg-slate-900">
                                {String(m.name || 'Unknown')}
                            </Badge>
                        ))}
                        {(!account.models || account.models.length === 0) && <span className="text-xs text-muted-foreground italic">No models configured</span>}
                    </div>
                </div>

                {!quotaOk && (
                    <div className="mt-4 p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg flex items-start gap-2 text-sm">
                        <RotateCcw className="w-4 h-4 text-amber-500 mt-0.5 flex-shrink-0" />
                        <div>
                            <p className="text-amber-700 dark:text-amber-300 font-medium">{account.blockingTier} quota exhausted</p>
                            <p className="text-amber-600 dark:text-amber-400 text-xs mt-0.5">Traffic routed to next account. Will auto-return when quota resets.</p>
                        </div>
                    </div>
                )}
            </Card>
        );
    };

    const renderAddAccountForm = () => {
        if (!selected) return null;

        const updateQuota = (period: QuotaPeriod, field: keyof QuotaConfig, value: boolean | number) => {
            setNewAccount(prev => ({
                ...prev,
                quotas: {
                    ...prev.quotas,
                    [period]: { ...prev.quotas[period], [field]: value }
                }
            }));
        };

        return (
            <div className="space-y-4">
                <div>
                    <Label>Account Name</Label>
                    <Input className="mt-1.5" placeholder="e.g., Production, Team-A" value={newAccount.name} onChange={(e) => setNewAccount({ ...newAccount, name: e.target.value })} />
                </div>

                {selected.isAzure && (
                    <>
                        <div>
                            <Label>Azure Endpoint</Label>
                            <Input className="mt-1.5" placeholder="https://your-resource.openai.azure.com" value={newAccount.endpoint} onChange={(e) => setNewAccount({ ...newAccount, endpoint: e.target.value })} />
                        </div>
                        <div className="grid grid-cols-2 gap-4">
                            <div>
                                <Label>Deployment Name</Label>
                                <Input className="mt-1.5" placeholder="gpt-4" value={newAccount.deploymentName} onChange={(e) => setNewAccount({ ...newAccount, deploymentName: e.target.value })} />
                            </div>
                            <div>
                                <Label>API Version</Label>
                                <Select value={newAccount.apiVersion} onValueChange={(v) => setNewAccount({ ...newAccount, apiVersion: v })}>
                                    <SelectTrigger className="mt-1.5"><SelectValue /></SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="2024-02-01">2024-02-01</SelectItem>
                                        <SelectItem value="2024-05-01-preview">2024-05-01-preview</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>
                        <div>
                            <Label>API Key</Label>
                            <Input type="password" className="mt-1.5" placeholder="Azure OpenAI API Key" value={newAccount.apiKey} onChange={(e) => setNewAccount({ ...newAccount, apiKey: e.target.value })} />
                        </div>
                    </>
                )}

                {selected.isAWS && (
                    <>
                        <div>
                            <Label>AWS Region</Label>
                            <Select value={newAccount.region} onValueChange={(v) => setNewAccount({ ...newAccount, region: v })}>
                                <SelectTrigger className="mt-1.5"><SelectValue /></SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="us-east-1">US East (N. Virginia)</SelectItem>
                                    <SelectItem value="us-west-2">US West (Oregon)</SelectItem>
                                    <SelectItem value="eu-west-1">Europe (Ireland)</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div>
                            <Label>Access Key ID</Label>
                            <Input className="mt-1.5" value={newAccount.accessKeyId} onChange={(e) => setNewAccount({ ...newAccount, accessKeyId: e.target.value })} />
                        </div>
                        <div>
                            <Label>Secret Access Key</Label>
                            <Input type="password" className="mt-1.5" value={newAccount.secretAccessKey} onChange={(e) => setNewAccount({ ...newAccount, secretAccessKey: e.target.value })} />
                        </div>
                    </>
                )}

                {!selected.isAzure && !selected.isAWS && (
                    <div>
                        <Label>API Key</Label>
                        <Input type="password" className="mt-1.5" placeholder={`Enter ${selected.name} API Key`} value={newAccount.apiKey} onChange={(e) => setNewAccount({ ...newAccount, apiKey: e.target.value })} />
                    </div>
                )}

                <div>
                    <Label>Priority</Label>
                    <Input type="number" className="mt-1.5" value={newAccount.priority} onChange={(e) => setNewAccount({ ...newAccount, priority: parseInt(e.target.value) || 0 })} />
                </div>

                <div className="pt-4 border-t">
                    <Label className="flex items-center gap-2 mb-3"><Cpu className="w-4 h-4" /> Models</Label>

                    {/* Language Models List */}
                    <div className="mb-3">
                        <Label className="text-xs font-semibold text-muted-foreground mb-2 uppercase tracking-wider block">Language Models</Label>
                        <div className="border rounded-md divide-y max-h-40 overflow-y-auto bg-slate-50/50 dark:bg-slate-900/20">
                            {newAccount.models.filter(m => m.capabilities.some(c => String(c).toLowerCase() === 'llm')).map((model, idx) => (
                                <div key={String(model.name || idx)} className="grid grid-cols-12 gap-2 p-2 items-center text-sm">
                                    <div className="col-span-11 font-medium flex items-center gap-2">
                                        <span>{String(model.name || 'Unknown')}</span>
                                        {(model.input_token_cost !== undefined || model.output_token_cost !== undefined) && (
                                            <Badge variant="outline" className="text-[10px] px-1 h-5 gap-1 font-normal text-muted-foreground">
                                                <span>${model.input_token_cost || 0}</span>
                                                <span>/</span>
                                                <span>${model.output_token_cost || 0}</span>
                                            </Badge>
                                        )}
                                    </div>
                                    <div className="col-span-1 text-right">
                                        <button onClick={() => removeModelFromAddForm(model)} className="text-muted-foreground hover:text-destructive"><Trash2 className="w-4 h-4" /></button>
                                    </div>
                                </div>
                            ))}
                            {newAccount.models.filter(m => m.capabilities.some(c => String(c).toLowerCase() === 'llm')).length === 0 && (
                                <div className="p-3 text-center text-xs text-muted-foreground italic">No language models configured</div>
                            )}
                        </div>
                    </div>

                    {/* Embedding Models List */}
                    <div className="mb-3">
                        <Label className="text-xs font-semibold text-muted-foreground mb-2 uppercase tracking-wider block">Embedding Models</Label>
                        <div className="border rounded-md divide-y max-h-40 overflow-y-auto bg-slate-50/50 dark:bg-slate-900/20">
                            {newAccount.models.filter(m => m.capabilities.some(c => String(c).toLowerCase() === 'embedding')).map((model, idx) => (
                                <div key={String(model.name || idx)} className="grid grid-cols-12 gap-2 p-2 items-center text-sm">
                                    <div className="col-span-11 font-medium flex items-center gap-2">
                                        <span>{String(model.name || 'Unknown')}</span>
                                        {(model.input_token_cost !== undefined || model.output_token_cost !== undefined) && (
                                            <Badge variant="outline" className="text-[10px] px-1 h-5 gap-1 font-normal text-muted-foreground">
                                                <span>${model.input_token_cost || 0}</span>
                                                <span>/</span>
                                                <span>${model.output_token_cost || 0}</span>
                                            </Badge>
                                        )}
                                    </div>
                                    <div className="col-span-1 text-right">
                                        <button onClick={() => removeModelFromAddForm(model)} className="text-muted-foreground hover:text-destructive"><Trash2 className="w-4 h-4" /></button>
                                    </div>
                                </div>
                            ))}
                            {newAccount.models.filter(m => m.capabilities.some(c => String(c).toLowerCase() === 'embedding')).length === 0 && (
                                <div className="p-3 text-center text-xs text-muted-foreground italic">No embedding models configured</div>
                            )}
                        </div>
                    </div>

                    {/* Add New Model Form */}
                    <div className="bg-slate-50 dark:bg-slate-900 p-3 rounded-lg border space-y-2">
                        <Label className="block text-xs font-medium text-muted-foreground">ADD NEW MODEL</Label>
                        <div className="flex gap-2">
                            <Input placeholder="Model Name (e.g., gpt-4o)" value={newAccount.newModel} onChange={(e) => setNewAccount({ ...newAccount, newModel: e.target.value })} className="bg-white dark:bg-black h-8 flex-1" />
                            <select
                                className="h-8 rounded-md border border-input bg-white dark:bg-black px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                                value={newAccount.newModelType || 'llm'}
                                onChange={(e) => setNewAccount({ ...newAccount, newModelType: e.target.value })}
                            >
                                <option value="llm">LLM</option>
                                <option value="embedding">Embedding</option>
                            </select>
                        </div>
                        <div className="grid grid-cols-2 gap-2">
                            <div>
                                <Label className="text-[10px] text-muted-foreground">Input Cost ($/1M)</Label>
                                <Input type="number" placeholder="0.00" value={newAccount.newModelInputCost} onChange={(e) => setNewAccount({ ...newAccount, newModelInputCost: e.target.value })} className="bg-white dark:bg-black h-8" />
                            </div>
                            <div>
                                <Label className="text-[10px] text-muted-foreground">Output Cost ($/1M)</Label>
                                <Input type="number" placeholder="0.00" value={newAccount.newModelOutputCost} onChange={(e) => setNewAccount({ ...newAccount, newModelOutputCost: e.target.value })} className="bg-white dark:bg-black h-8" />
                            </div>
                        </div>
                        <Button
                            type="button"
                            onClick={() => {
                                const type = newAccount.newModelType || 'llm';
                                const model = {
                                    name: newAccount.newModel,
                                    capabilities: [type], // Use selected type
                                    input_token_cost: parseFloat(newAccount.newModelInputCost) || 0,
                                    output_token_cost: parseFloat(newAccount.newModelOutputCost) || 0
                                };
                                setNewAccount({
                                    ...newAccount,
                                    models: [...newAccount.models, model],
                                    newModel: '',
                                    newModelInputCost: '',
                                    newModelOutputCost: ''
                                });
                            }}
                            disabled={!newAccount.newModel.trim()}
                            size="sm"
                            className="w-full h-8"
                        >
                            <Plus className="w-4 h-4 mr-2" /> Add {newAccount.newModelType === 'embedding' ? 'Embedding' : 'LLM'} Model
                        </Button>
                    </div>
                </div>

                <div className="pt-4 border-t">
                    <div className="flex items-center gap-2 mb-4">
                        <Zap className="w-4 h-4 text-primary" />
                        <h4 className="font-medium">Quota Limits</h4>
                    </div>
                    {/* ... Quotas content ... */}

                    <div className="space-y-4">
                        {(['minute', 'hour', 'day', 'month'] as QuotaPeriod[]).map(period => (
                            <div key={period} className={`p-4 rounded-lg border ${newAccount.quotas[period].enabled ? 'border-primary/30 bg-primary/5' : 'border-border'}`}>
                                <div className="flex items-center justify-between mb-3">
                                    <div className="flex items-center gap-2">
                                        <Switch checked={newAccount.quotas[period].enabled} onCheckedChange={(v: boolean) => updateQuota(period, 'enabled', v)} />
                                        <span className="font-medium">{quotaPeriodLabels[period]}</span>
                                    </div>
                                </div>
                                {newAccount.quotas[period].enabled && (
                                    <div className="grid grid-cols-2 gap-3 mt-3">
                                        <div>
                                            <Label className="text-xs">Token Limit</Label>
                                            <Input type="number" className="mt-1 h-9" value={newAccount.quotas[period].tokenLimit} onChange={(e) => updateQuota(period, 'tokenLimit', parseInt(e.target.value) || 0)} />
                                        </div>
                                        <div>
                                            <Label className="text-xs">Request Limit</Label>
                                            <Input type="number" className="mt-1 h-9" value={newAccount.quotas[period].requestLimit} onChange={(e) => updateQuota(period, 'requestLimit', parseInt(e.target.value) || 0)} />
                                        </div>
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                </div>

                {error && <div className="p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-sm text-red-500">{error}</div>}

                <div className="flex gap-3 pt-4">
                    <Button variant="outline" className="flex-1" onClick={() => setIsAddOpen(false)}>Cancel</Button>
                    <Button className="flex-1" onClick={handleCreateAccount} disabled={loading || !newAccount.name}>
                        {loading ? 'Adding...' : 'Add Account'}
                    </Button>
                </div>
            </div>
        );
    };

    return (
        <div className="space-y-6 animate-fade-in">
            {successMessage && (
                <div className="p-4 bg-emerald-500/10 border border-emerald-500/30 rounded-lg text-sm text-emerald-600 flex items-center gap-2">
                    <Check className="w-5 h-5" />
                    {successMessage}
                </div>
            )}

            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold">Providers</h1>
                    <p className="text-muted-foreground">Configure your LLM infrastructure</p>
                </div>
                <Button variant="outline" onClick={fetchProvidersAndAccounts} disabled={fetchingProviders}>
                    <RefreshCw className={`w-4 h-4 mr-2 ${fetchingProviders ? 'animate-spin' : ''}`} />
                    Refresh
                </Button>
            </div>

            <div className="grid grid-cols-12 gap-6 mt-6">
                {/* Provider List (Left Column) */}
                <div className="col-span-12 lg:col-span-4 space-y-2">
                    {providers.map((provider) => {
                        const activeAccounts = provider.accounts.filter(a => a.enabled && hasQuota(a)).length;
                        const exhaustedAccounts = provider.accounts.filter(a => a.enabled && !hasQuota(a)).length;
                        const totalAccounts = provider.accounts.length;

                        return (
                            <div
                                key={provider.id}
                                onClick={() => setSelectedId(provider.id)}
                                className={`
                                     flex items-center gap-4 p-4 rounded-xl cursor-pointer transition-all
                                     ${selectedId === provider.id
                                        ? 'bg-primary/10 ring-2 ring-primary shadow-lg'
                                        : 'bg-card hover:bg-accent border border-border'
                                    }
                                 `}
                            >
                                <div className={`w-10 h-10 rounded-lg bg-gradient-to-br ${provider.gradient} flex items-center justify-center shadow`}>
                                    {getProviderLogo(provider.id, "w-6 h-6 text-white") || <Cpu className="w-5 h-5 text-white" />}
                                </div>
                                <div className="flex-1 min-w-0">
                                    <div className="flex items-center gap-2">
                                        <h3 className="font-medium truncate">{provider.name}</h3>
                                        {activeAccounts > 0 && <div className="w-2 h-2 rounded-full bg-emerald-500" />}
                                        {activeAccounts === 0 && exhaustedAccounts > 0 && <div className="w-2 h-2 rounded-full bg-amber-500" />}
                                    </div>
                                    <div className="flex items-center gap-2 mt-0.5">
                                        <Badge variant="secondary" className="text-xs">
                                            {provider.type === 'both' ? 'LLM + Embed' : provider.type}
                                        </Badge>
                                        {totalAccounts > 0 && (
                                            <span className="text-xs text-muted-foreground">
                                                {activeAccounts}/{totalAccounts} active
                                            </span>
                                        )}
                                    </div>
                                </div>
                                <ChevronRight className={`w-4 h-4 text-muted-foreground transition-transform ${selectedId === provider.id ? 'rotate-90' : ''}`} />
                            </div>
                        );
                    })}
                </div>

                {/* Provider Details (Right Column) */}
                <div className="col-span-12 lg:col-span-8">
                    {selected ? (
                        <div className="space-y-6">
                            <div className={`relative overflow-hidden rounded-2xl p-6 bg-gradient-to-br ${selected.gradient}`}>
                                <div className="absolute inset-0 bg-black/10" />
                                <div className="relative z-10 flex items-center justify-between">
                                    <div className="flex items-center gap-4">
                                        <div className="w-14 h-14 rounded-xl bg-white/20 backdrop-blur flex items-center justify-center">
                                            {getProviderLogo(selected.id, "w-8 h-8 text-white") || <Cpu className="w-8 h-8 text-white" />}
                                        </div>
                                        <div>
                                            <h2 className="text-2xl font-bold text-white">{selected.name}</h2>
                                            <div className="flex items-center gap-2 mt-1">
                                                {selected.isAzure && <Badge className="bg-white/20 text-white border-0"><Shield className="w-3 h-3 mr-1" />Azure</Badge>}
                                                {selected.isAWS && <Badge className="bg-white/20 text-white border-0"><Cloud className="w-3 h-3 mr-1" />AWS</Badge>}
                                                <Badge className="bg-white/20 text-white border-0">{selected.accounts.length} account(s)</Badge>
                                            </div>
                                        </div>
                                    </div>
                                    <Dialog open={isAddOpen} onOpenChange={setIsAddOpen}>
                                        <Button className="bg-white text-slate-900 hover:bg-white/90" onClick={openAddDialog}>
                                            <Plus className="w-4 h-4 mr-2" />
                                            Add Account
                                        </Button>
                                        <DialogContent className="max-w-lg max-h-[90vh] overflow-y-auto">
                                            <DialogHeader>
                                                <DialogTitle>Add {selected.name} Account</DialogTitle>
                                            </DialogHeader>
                                            {renderAddAccountForm()}
                                        </DialogContent>
                                    </Dialog>
                                </div>
                            </div>

                            {selected.accounts.length === 0 ? (
                                <Card className="p-12 text-center border-dashed">
                                    <Key className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
                                    <h3 className="text-lg font-semibold mb-2">No accounts configured</h3>
                                    <p className="text-muted-foreground mb-4">Add an account to start using this provider</p>
                                    <Button onClick={openAddDialog}>
                                        <Plus className="w-4 h-4 mr-2" />
                                        Add Account
                                    </Button>
                                </Card>
                            ) : (
                                <div className="space-y-4">
                                    <h3 className="font-semibold">Accounts ({selected.accounts.length})</h3>
                                    {selected.accounts.map(renderAccountCard)}
                                </div>
                            )}
                        </div>
                    ) : (
                        <Card className="h-full min-h-[500px] flex items-center justify-center border-dashed">
                            <div className="text-center">
                                <Settings className="w-16 h-16 mx-auto text-muted-foreground/50 mb-4" />
                                <h3 className="text-xl font-semibold mb-2">Select a Provider</h3>
                                <p className="text-muted-foreground">Choose a provider from the list to configure</p>
                            </div>
                        </Card>
                    )}
                </div>
            </div>

            {/* Dialogs */}
            <Dialog open={deleteConfirmOpen} onOpenChange={setDeleteConfirmOpen}>
                <DialogContent className="max-w-md">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2 text-destructive">
                            <AlertCircle className="w-5 h-5" />
                            Delete Account
                        </DialogTitle>
                        <DialogDescription>
                            Are you sure you want to delete the account <strong>&quot;{String(accountToDelete?.name || '')}&quot;</strong>?
                            This action cannot be undone.
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter className="gap-2 sm:gap-0">
                        <Button variant="outline" onClick={() => setDeleteConfirmOpen(false)}>Cancel</Button>
                        <Button
                            variant="destructive"
                            className="bg-red-600 hover:bg-red-700 text-white"
                            onClick={handleDeleteAccount}
                            disabled={deleting}
                        >
                            {deleting ? 'Deleting...' : 'Delete Account'}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
                <DialogContent className="max-w-md max-h-[90vh] overflow-y-auto">
                    <DialogHeader>
                        <DialogTitle>Edit Account</DialogTitle>
                    </DialogHeader>
                    <div className="space-y-4 py-4">
                        <div>
                            <Label>Account Name</Label>
                            <Input className="mt-1.5" value={editForm.name} onChange={(e) => setEditForm({ ...editForm, name: e.target.value })} placeholder="Account name" />
                        </div>
                        <div>
                            <Label>Priority</Label>
                            <Input type="number" className="mt-1.5" value={editForm.priority} onChange={(e) => setEditForm({ ...editForm, priority: parseInt(e.target.value) || 0 })} />
                        </div>
                        <div className="flex items-center justify-between">
                            <Label>Enabled</Label>
                            <Switch checked={editForm.enabled} onCheckedChange={(checked: boolean) => setEditForm({ ...editForm, enabled: checked })} />
                        </div>

                        {/* Quotas Section */}
                        <div className="pt-3 border-t">
                            <Label className="flex items-center gap-2 mb-3"><Clock className="w-4 h-4" /> Quotas & Limits</Label>
                            <div className="grid grid-cols-2 gap-3">
                                {(['minute', 'hour', 'day', 'month'] as QuotaPeriod[]).map((period) => {
                                    const tier = editForm.quotas.find(q => q.period === period);
                                    const isEnabled = !!tier;
                                    return (
                                        <div key={period} className="p-2 border rounded-md bg-slate-50 dark:bg-slate-900">
                                            <div className="flex items-center justify-between mb-2">
                                                <Label className="text-xs capitalize font-semibold">{period}</Label>
                                                <Switch
                                                    checked={isEnabled}
                                                    onCheckedChange={(checked) => {
                                                        let newQuotas = [...editForm.quotas];
                                                        if (checked) {
                                                            if (!newQuotas.find(q => q.period === period)) {
                                                                newQuotas.push({ period, tokenLimit: 100000, tokensUsed: 0, secondsUntilReset: 0, requestsUsed: 0 });
                                                            }
                                                        } else {
                                                            newQuotas = newQuotas.filter(q => q.period !== period);
                                                        }
                                                        setEditForm({ ...editForm, quotas: newQuotas });
                                                    }}
                                                    className="scale-75"
                                                />
                                            </div>
                                            {isEnabled && (
                                                <div>
                                                    <Label className="text-[10px] text-muted-foreground">Token Limit</Label>
                                                    <Input
                                                        type="number"
                                                        className="h-7 text-xs bg-white dark:bg-black"
                                                        value={tier?.tokenLimit?.toString() || ''}
                                                        onChange={(e) => {
                                                            const val = parseInt(e.target.value) || 0;
                                                            setEditForm({
                                                                ...editForm,
                                                                quotas: editForm.quotas.map(q => q.period === period ? { ...q, tokenLimit: val } : q)
                                                            });
                                                        }}
                                                    />
                                                </div>
                                            )}
                                        </div>
                                    );
                                })}
                            </div>
                        </div>

                        {/* Models Section */}
                        <div className="pt-3 border-t">
                            <Label className="flex items-center gap-2 mb-3"><Cpu className="w-4 h-4" /> Models</Label>

                            {/* Language Models List */}
                            <div className="mb-3">
                                <Label className="text-xs font-semibold text-muted-foreground mb-2 uppercase tracking-wider block">Language Models</Label>
                                <div className="border rounded-md divide-y bg-slate-50/50 dark:bg-slate-900/20">
                                    {editForm.models.filter(m => m && Array.isArray(m.capabilities) && m.capabilities.some(c => String(c).toLowerCase() === 'llm')).map((model, idx) => (
                                        <div key={String(model.name || idx)} className="grid grid-cols-12 gap-2 p-2 items-center text-sm">
                                            <div className="col-span-11 font-medium flex items-center gap-2">
                                                <span>{String(model.name || 'Unknown')}</span>
                                                {(model.input_token_cost !== undefined || model.output_token_cost !== undefined) && (
                                                    <Badge variant="outline" className="text-[10px] px-1 h-5 gap-1 font-normal text-muted-foreground">
                                                        <span>${model.input_token_cost || 0}</span>
                                                        <span>/</span>
                                                        <span>${model.output_token_cost || 0}</span>
                                                    </Badge>
                                                )}
                                            </div>
                                            <div className="col-span-1 text-right">
                                                <button onClick={() => removeModelFromEditForm(model)} className="text-muted-foreground hover:text-destructive"><Trash2 className="w-4 h-4" /></button>
                                            </div>
                                        </div>
                                    ))}
                                    {editForm.models.filter(m => m && Array.isArray(m.capabilities) && m.capabilities.some(c => String(c).toLowerCase() === 'llm')).length === 0 && (
                                        <div className="p-3 text-center text-xs text-muted-foreground italic">No language models configured</div>
                                    )}
                                </div>
                            </div>

                            {/* Embedding Models List */}
                            <div className="mb-3">
                                <Label className="text-xs font-semibold text-muted-foreground mb-2 uppercase tracking-wider block">Embedding Models</Label>
                                <div className="border rounded-md divide-y bg-slate-50/50 dark:bg-slate-900/20">
                                    {editForm.models.filter(m => m && Array.isArray(m.capabilities) && m.capabilities.some(c => String(c).toLowerCase() === 'embedding')).map((model, idx) => (
                                        <div key={String(model.name || idx)} className="grid grid-cols-12 gap-2 p-2 items-center text-sm">
                                            <div className="col-span-11 font-medium flex items-center gap-2">
                                                <span>{String(model.name || 'Unknown')}</span>
                                                {(model.input_token_cost !== undefined || model.output_token_cost !== undefined) && (
                                                    <Badge variant="outline" className="text-[10px] px-1 h-5 gap-1 font-normal text-muted-foreground">
                                                        <span>${model.input_token_cost || 0}</span>
                                                        <span>/</span>
                                                        <span>${model.output_token_cost || 0}</span>
                                                    </Badge>
                                                )}
                                            </div>
                                            <div className="col-span-1 text-right">
                                                <button onClick={() => removeModelFromEditForm(model)} className="text-muted-foreground hover:text-destructive"><Trash2 className="w-4 h-4" /></button>
                                            </div>
                                        </div>
                                    ))}
                                    {editForm.models.filter(m => m && Array.isArray(m.capabilities) && m.capabilities.some(c => String(c).toLowerCase() === 'embedding')).length === 0 && (
                                        <div className="p-3 text-center text-xs text-muted-foreground italic">No embedding models configured</div>
                                    )}
                                </div>
                            </div>

                            <div className="bg-slate-50 dark:bg-slate-900 p-3 rounded-lg border space-y-2">
                                <Label className="block text-xs font-medium text-muted-foreground">ADD NEW MODEL</Label>
                                <div className="flex gap-2">
                                    <Input placeholder="Model Name (e.g., gpt-4o)" value={editForm.newModel} onChange={(e) => setEditForm({ ...editForm, newModel: e.target.value })} className="bg-white dark:bg-black h-8 flex-1" />
                                    <select
                                        className="h-8 rounded-md border border-input bg-white dark:bg-black px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                                        value={editForm.newModelType || 'llm'}
                                        onChange={(e) => setEditForm({ ...editForm, newModelType: e.target.value })}
                                    >
                                        <option value="llm">LLM</option>
                                        <option value="embedding">Embedding</option>
                                    </select>
                                </div>
                                <div className="grid grid-cols-2 gap-2">
                                    <div>
                                        <Label className="text-[10px] text-muted-foreground">Input Cost ($/1M)</Label>
                                        <Input type="number" placeholder="0.00" value={editForm.newModelInputCost} onChange={(e) => setEditForm({ ...editForm, newModelInputCost: e.target.value })} className="bg-white dark:bg-black h-8" />
                                    </div>
                                    <div>
                                        <Label className="text-[10px] text-muted-foreground">Output Cost ($/1M)</Label>
                                        <Input type="number" placeholder="0.00" value={editForm.newModelOutputCost} onChange={(e) => setEditForm({ ...editForm, newModelOutputCost: e.target.value })} className="bg-white dark:bg-black h-8" />
                                    </div>
                                </div>
                                <Button
                                    type="button"
                                    onClick={() => {
                                        const type = editForm.newModelType || 'llm';
                                        const model = {
                                            name: editForm.newModel,
                                            capabilities: [type],
                                            input_token_cost: parseFloat(editForm.newModelInputCost) || 0,
                                            output_token_cost: parseFloat(editForm.newModelOutputCost) || 0
                                        };
                                        setEditForm({
                                            ...editForm,
                                            models: [...editForm.models, model],
                                            newModel: '',
                                            newModelInputCost: '',
                                            newModelOutputCost: ''
                                        });
                                    }}
                                    disabled={!editForm.newModel.trim()}
                                    size="sm"
                                    className="w-full h-8"
                                >
                                    <Plus className="w-4 h-4 mr-2" /> Add {editForm.newModelType === 'embedding' ? 'Embedding' : 'LLM'} Model
                                </Button>
                            </div>
                        </div>

                        {error && <div className="p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-sm text-red-500">{error}</div>}
                    </div>
                    <div className="flex gap-3 justify-end">
                        <Button variant="outline" onClick={() => setEditDialogOpen(false)}>Cancel</Button>
                        <Button onClick={handleSaveAccount} disabled={saving || !editForm.name}>{saving ? 'Saving...' : 'Save Changes'}</Button>
                    </div>
                </DialogContent>
            </Dialog>
        </div>
    );
}
