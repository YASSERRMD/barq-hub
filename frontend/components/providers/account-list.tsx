"use client";

import { useState, useEffect } from "react";
import { MoreVertical, Trash, Play, ShieldAlert, ArrowRightLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Badge } from "@/components/ui/badge";
import { providersApi } from "@/lib/api";
import { toast } from "sonner";
import { ProviderAccount } from "@/types";

interface AccountListProps {
    providerId: string;
    refreshTrigger: number;
}

export function AccountList({ providerId, refreshTrigger }: AccountListProps) {
    const [accounts, setAccounts] = useState<ProviderAccount[]>([]);
    const [isLoading, setIsLoading] = useState(true);

    const fetchAccounts = async () => {
        try {
            const data = await providersApi.getAccounts(providerId);
            // Sort by priority desc
            setAccounts(data.sort((a: any, b: any) => b.priority - a.priority));
        } catch (error) {
            console.error("Failed to fetch accounts", error);
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        fetchAccounts();
    }, [providerId, refreshTrigger]);

    const handleDelete = async (accountId: string) => {
        try {
            await providersApi.deleteAccount(providerId, accountId);
            toast.success("Account deleted");
            fetchAccounts();
        } catch (error) {
            toast.error("Failed to delete account");
        }
    };

    const handleTest = async (accountId: string) => {
        const toastId = toast.loading("Testing connection...");
        try {
            const result = await providersApi.testAccount(providerId, accountId);
            if (result.success) {
                toast.success("Connection successful", { id: toastId });
            } else {
                toast.error(`Connection failed: ${result.message}`, { id: toastId });
            }
        } catch (error) {
            toast.error("Test failed", { id: toastId });
        }
    };

    if (isLoading) return <div className="p-4 text-center text-sm text-muted-foreground">Loading accounts...</div>;

    if (accounts.length === 0) {
        return (
            <div className="p-8 text-center border-t border-border/40">
                <p className="text-muted-foreground text-sm">No API keys configured. Add one to start using this provider.</p>
            </div>
        );
    }

    return (
        <div className="divide-y divide-border/40 border-t border-border/40">
            {accounts.map((account) => (
                <div key={account.id} className="p-4 flex items-center justify-between hover:bg-muted/30 transition-colors">
                    <div className="flex items-center gap-4 flex-1">
                        <div className={`w-2 h-2 rounded-full ${account.enabled ? "bg-green-500" : "bg-muted-foreground"}`} />
                        <div className="flex-1">
                            <div className="flex items-center gap-2">
                                <span className="font-medium text-sm">{account.name}</span>
                                {account.priority >= 10 && (
                                    <Badge variant="secondary" className="text-[10px] h-5 bg-violet-500/10 text-violet-500">Primary</Badge>
                                )}
                            </div>
                            <div className="flex flex-wrap items-center gap-x-4 gap-y-1 mt-1">
                                <code className="text-xs text-muted-foreground font-mono">
                                    {account.type === "api_key" ? "sk-••••••••" : "configured"}
                                </code>
                                <span className="text-xs text-muted-foreground">•</span>
                                <span className="text-xs text-muted-foreground">Priority: {account.priority}</span>

                                {(account.quotas?.requests_per_minute || account.quotas?.requests_per_day || account.quotas?.requests_per_month) && (
                                    <>
                                        <span className="text-xs text-muted-foreground">•</span>
                                        <div className="flex items-center gap-1 text-xs text-muted-foreground">
                                            <span className="font-semibold">Limits:</span>
                                            {account.quotas?.requests_per_minute && <span>{account.quotas.requests_per_minute}/min</span>}
                                            {account.quotas?.requests_per_minute && (account.quotas?.requests_per_day || account.quotas?.requests_per_month) && <span>,</span>}
                                            {account.quotas?.requests_per_day && <span>{account.quotas.requests_per_day}/day</span>}
                                            {account.quotas?.requests_per_day && account.quotas?.requests_per_month && <span>,</span>}
                                            {account.quotas?.requests_per_month && <span>{account.quotas.requests_per_month}/mo</span>}
                                        </div>
                                    </>
                                )}

                                {(account.pricing?.input_token_price || account.pricing?.output_token_price) && (
                                    <>
                                        <span className="text-xs text-muted-foreground">•</span>
                                        <div className="flex items-center gap-1 text-xs text-muted-foreground">
                                            <span className="font-semibold">Pricing (1M):</span>
                                            <span>In: ${account.pricing?.input_token_price}</span>
                                            <span>/</span>
                                            <span>Out: ${account.pricing?.output_token_price}</span>
                                        </div>
                                    </>
                                )}
                            </div>
                        </div>
                    </div>

                    <div className="flex items-center gap-2">
                        <Button size="sm" variant="ghost" onClick={() => handleTest(account.id)}>
                            <Play className="h-4 w-4 text-muted-foreground hover:text-green-500" />
                        </Button>
                        <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                                <Button variant="ghost" size="icon" className="h-8 w-8">
                                    <MoreVertical className="h-4 w-4" />
                                </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end">
                                <DropdownMenuItem onClick={() => handleTest(account.id)}>
                                    <Play className="mr-2 h-4 w-4" /> Test Connection
                                </DropdownMenuItem>
                                <DropdownMenuItem>
                                    <ArrowRightLeft className="mr-2 h-4 w-4" /> Rotate Key
                                </DropdownMenuItem>
                                <DropdownMenuSeparator />
                                <DropdownMenuItem className="text-red-500" onClick={() => handleDelete(account.id)}>
                                    <Trash className="mr-2 h-4 w-4" /> Delete
                                </DropdownMenuItem>
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </div>
                </div>
            ))}
        </div>
    );
}
