"use client";

import { useState, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { Skeleton } from "@/components/ui/skeleton";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import {
    Send,
    Copy,
    Check,
    Code2,
    Bot,
    User,
    RotateCcw,
    Zap,
    Settings,
    Terminal,
    AlertCircle,
    RefreshCw,
} from "lucide-react";

interface Message {
    role: "user" | "assistant" | "system";
    content: string;
}

interface ProviderAccount {
    id: string;
    name: string;
    provider_id: string;
    enabled: boolean;
    models: { id: string; name: string; capability: string }[];
}

interface Provider {
    id: string;
    name: string;
    accounts: ProviderAccount[];
}

export default function PlaygroundPage() {
    const [messages, setMessages] = useState<Message[]>([]);
    const [input, setInput] = useState("");
    const [providers, setProviders] = useState<Provider[]>([]);
    const [selectedProvider, setSelectedProvider] = useState("");
    const [selectedModel, setSelectedModel] = useState("");
    const [temperature, setTemperature] = useState(0.7);
    const [maxTokens, setMaxTokens] = useState(2048);
    const [systemPrompt, setSystemPrompt] = useState("You are a helpful AI assistant.");
    const [copiedSnippet, setCopiedSnippet] = useState<string | null>(null);
    const [activeTab, setActiveTab] = useState("chat");
    const [selectedLanguage, setSelectedLanguage] = useState("python");
    const [showSettings, setShowSettings] = useState(false);
    const [isLoading, setIsLoading] = useState(false);
    const [loadingProviders, setLoadingProviders] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const messagesEndRef = useRef<HTMLDivElement>(null);

    const apiUrl = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:4000/v1';

    // Auto-scroll to bottom when messages change
    useEffect(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [messages, isLoading]);

    // Fetch providers from database
    useEffect(() => {
        fetchProviders();
    }, []);

    const fetchProviders = async () => {
        setLoadingProviders(true);
        try {
            const response = await fetch(`${apiUrl}/provider-accounts/providers`);
            if (response.ok) {
                const providerDefs = await response.json();

                // Fetch accounts for each provider (like Providers page does)
                const providersWithAccounts: Provider[] = await Promise.all(
                    providerDefs
                        .filter((p: any) => p.provider_type === 'llm' || p.provider_type === 'both')
                        .map(async (p: any) => {
                            let accounts: ProviderAccount[] = [];
                            try {
                                const accountsRes = await fetch(`${apiUrl}/provider-accounts/${p.id}/accounts`);
                                if (accountsRes.ok) {
                                    const rawAccounts = await accountsRes.json();
                                    accounts = rawAccounts.map((a: any) => ({
                                        id: a.id,
                                        name: a.name,
                                        provider_id: a.provider_id,
                                        enabled: a.enabled,
                                        models: a.models || [],
                                    }));
                                }
                            } catch (e) {
                                console.error(`Failed to fetch accounts for ${p.id}:`, e);
                            }

                            return {
                                id: p.id,
                                name: p.name,
                                accounts,
                            } as Provider;
                        })
                );

                setProviders(providersWithAccounts);

                // Auto-select first provider with enabled accounts and models
                const firstWithModels = providersWithAccounts.find((p: Provider) =>
                    p.accounts?.some((a: ProviderAccount) => a.enabled && a.models?.length > 0)
                );
                if (firstWithModels) {
                    setSelectedProvider(firstWithModels.id);
                    const firstAccount = firstWithModels.accounts.find((a: ProviderAccount) => a.enabled && a.models?.length > 0);
                    if (firstAccount?.models?.[0]) {
                        setSelectedModel(firstAccount.models[0].id || firstAccount.models[0].name);
                    }
                } else if (providersWithAccounts.length > 0) {
                    // If no accounts with models, still select provider (user might want to use default models)
                    const firstProvider = providersWithAccounts.find(p => p.accounts && p.accounts.length > 0);
                    if (firstProvider) {
                        setSelectedProvider(firstProvider.id);
                    }
                }
            }
        } catch (err) {
            console.error("Failed to fetch providers:", err);
        } finally {
            setLoadingProviders(false);
        }
    };

    // Get current provider and its models
    const currentProvider = providers.find(p => p.id === selectedProvider);
    const availableModels = currentProvider?.accounts
        ?.filter(a => a.enabled)
        ?.flatMap(a => a.models || []) || [];

    const handleSend = async () => {
        if (!input.trim() || !selectedModel) return;

        const userMessage: Message = { role: "user", content: input };
        const newMessages = [...messages, userMessage];
        setMessages(newMessages);
        setInput("");
        setIsLoading(true);
        setError(null);

        try {
            // Build messages with system prompt
            const apiMessages = [
                { role: "system", content: systemPrompt },
                ...newMessages.filter(m => m.role !== "system")
            ];

            const response = await fetch(`${apiUrl}/chat/completions`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    model: selectedModel,
                    provider: selectedProvider,
                    messages: apiMessages,
                    temperature,
                    max_tokens: maxTokens,
                }),
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                throw new Error(errData.error?.message || errData.message || `API error: ${response.status}`);
            }

            const data = await response.json();
            const assistantContent = data.choices?.[0]?.message?.content || "No response received";

            setMessages(prev => [...prev, { role: "assistant", content: assistantContent }]);
        } catch (err: any) {
            console.error("Chat error:", err);
            setError(err.message || "Failed to get response");
            // Add error message to chat
            setMessages(prev => [...prev, {
                role: "assistant",
                content: `❌ **Error:** ${err.message}\n\nPlease check:\n- Provider API key is configured correctly\n- Model is available for your account\n- Backend server is running`
            }]);
        } finally {
            setIsLoading(false);
        }
    };

    const clearChat = () => {
        setMessages([]);
        setError(null);
    };

    const copyToClipboard = (text: string, snippetId: string) => {
        navigator.clipboard.writeText(text);
        setCopiedSnippet(snippetId);
        setTimeout(() => setCopiedSnippet(null), 2000);
    };

    const codeSnippets: Record<string, { label: string; icon: string; code: string }> = {
        python: {
            label: "Python",
            icon: "🐍",
            code: `import requests

# BARQ API Configuration
API_KEY = "your-api-key"  # Get from API Keys page
BASE_URL = "${apiUrl}"

headers = {
    "Authorization": f"Bearer {API_KEY}",
    "Content-Type": "application/json"
}

# Chat Completion with ${currentProvider?.name || selectedProvider}
def chat(messages, model="${selectedModel}", provider="${selectedProvider}"):
    response = requests.post(
        f"{BASE_URL}/chat/completions",
        headers=headers,
        json={
            "model": model,
            "provider": provider,
            "messages": messages,
            "temperature": ${temperature},
            "max_tokens": ${maxTokens}
        }
    )
    return response.json()

# Example usage
result = chat([
    {"role": "system", "content": "${systemPrompt}"},
    {"role": "user", "content": "Hello!"}
])
print(result["choices"][0]["message"]["content"])`,
        },
        javascript: {
            label: "JavaScript",
            icon: "🟨",
            code: `// BARQ API Client
const API_KEY = 'your-api-key';  // Get from API Keys page
const BASE_URL = '${apiUrl}';

const headers = {
  'Authorization': \`Bearer \${API_KEY}\`,
  'Content-Type': 'application/json'
};

// Chat Completion with ${currentProvider?.name || selectedProvider}
async function chat(messages, model = '${selectedModel}', provider = '${selectedProvider}') {
  const response = await fetch(\`\${BASE_URL}/chat/completions\`, {
    method: 'POST',
    headers,
    body: JSON.stringify({
      model,
      provider,
      messages,
      temperature: ${temperature},
      max_tokens: ${maxTokens}
    })
  });
  return response.json();
}

// Example usage
const result = await chat([
  { role: 'system', content: '${systemPrompt}' },
  { role: 'user', content: 'Hello!' }
]);
console.log(result.choices[0].message.content);`,
        },
        curl: {
            label: "cURL",
            icon: "🌀",
            code: `# Chat Completion with ${currentProvider?.name || selectedProvider}
curl -X POST "${apiUrl}/chat/completions" \\
  -H "Authorization: Bearer your-api-key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "${selectedModel}",
    "provider": "${selectedProvider}",
    "messages": [
      {"role": "system", "content": "${systemPrompt}"},
      {"role": "user", "content": "Hello!"}
    ],
    "temperature": ${temperature},
    "max_tokens": ${maxTokens}
  }'`,
        },
    };

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold">Playground</h1>
                    <p className="text-muted-foreground">Test your configured providers and get integration code</p>
                </div>
                <div className="flex items-center gap-2">
                    <Button variant="outline" size="sm" onClick={fetchProviders} disabled={loadingProviders}>
                        <RefreshCw className={`w-4 h-4 mr-2 ${loadingProviders ? 'animate-spin' : ''}`} />
                        Refresh
                    </Button>
                    <Button variant="outline" size="sm" onClick={() => setShowSettings(!showSettings)}>
                        <Settings className="w-4 h-4 mr-2" />
                        Settings
                    </Button>
                    <Button variant="outline" size="sm" onClick={clearChat}>
                        <RotateCcw className="w-4 h-4 mr-2" />
                        Clear
                    </Button>
                </div>
            </div>

            {/* No providers warning */}
            {!loadingProviders && providers.length === 0 && (
                <Alert>
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>
                        No providers configured. Go to <strong>Providers</strong> to add an account with your API key.
                    </AlertDescription>
                </Alert>
            )}

            {/* No models warning */}
            {!loadingProviders && providers.length > 0 && availableModels.length === 0 && selectedProvider && (
                <Alert>
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>
                        No models configured for this provider. Edit the account in <strong>Providers</strong> to add models.
                    </AlertDescription>
                </Alert>
            )}

            <Tabs value={activeTab} onValueChange={setActiveTab}>
                <TabsList>
                    <TabsTrigger value="chat" className="gap-2">
                        <Bot className="w-4 h-4" />
                        Chat
                    </TabsTrigger>
                    <TabsTrigger value="code" className="gap-2">
                        <Code2 className="w-4 h-4" />
                        Integration Code
                    </TabsTrigger>
                </TabsList>

                <TabsContent value="chat" className="space-y-4 mt-4">
                    {/* Provider Selection */}
                    <Card>
                        <CardContent className="py-3">
                            <div className="flex items-center gap-4 flex-wrap">
                                <div className="flex items-center gap-2">
                                    <Label>Provider:</Label>
                                    {loadingProviders ? (
                                        <Skeleton className="h-9 w-36" />
                                    ) : (
                                        <Select value={selectedProvider} onValueChange={(v) => {
                                            setSelectedProvider(v);
                                            const provider = providers.find(p => p.id === v);
                                            const firstModel = provider?.accounts
                                                ?.filter(a => a.enabled)
                                                ?.flatMap(a => a.models || [])?.[0];
                                            setSelectedModel(firstModel?.id || firstModel?.name || "");
                                        }}>
                                            <SelectTrigger className="w-36">
                                                <SelectValue placeholder="Select provider" />
                                            </SelectTrigger>
                                            <SelectContent>
                                                {providers.filter(p => p.accounts?.some(a => a.enabled)).map(p => (
                                                    <SelectItem key={p.id} value={p.id}>{p.name}</SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                    )}
                                </div>
                                <div className="flex items-center gap-2">
                                    <Label>Model:</Label>
                                    {loadingProviders ? (
                                        <Skeleton className="h-9 w-48" />
                                    ) : (
                                        <Select value={selectedModel} onValueChange={setSelectedModel}>
                                            <SelectTrigger className="w-48">
                                                <SelectValue placeholder="Select model" />
                                            </SelectTrigger>
                                            <SelectContent>
                                                {availableModels.map(m => (
                                                    <SelectItem key={m.id || m.name} value={m.id || m.name}>
                                                        {m.name || m.id}
                                                    </SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                    )}
                                </div>
                                <div className="flex items-center gap-2">
                                    <Label>Temp:</Label>
                                    <Input
                                        type="number"
                                        step="0.1"
                                        min="0"
                                        max="2"
                                        className="w-20 h-8"
                                        value={temperature}
                                        onChange={e => setTemperature(parseFloat(e.target.value))}
                                    />
                                </div>
                                {selectedProvider && selectedModel && (
                                    <Badge variant="secondary" className="ml-auto">
                                        {currentProvider?.name} • {selectedModel}
                                    </Badge>
                                )}
                            </div>
                        </CardContent>
                    </Card>

                    {/* Settings Panel */}
                    {showSettings && (
                        <Card>
                            <CardHeader className="py-3">
                                <CardTitle className="text-sm">Advanced Settings</CardTitle>
                            </CardHeader>
                            <CardContent className="space-y-3">
                                <div className="space-y-2">
                                    <Label>System Prompt</Label>
                                    <Textarea
                                        value={systemPrompt}
                                        onChange={e => setSystemPrompt(e.target.value)}
                                        rows={3}
                                    />
                                </div>
                                <div className="flex gap-4">
                                    <div className="space-y-2">
                                        <Label>Max Tokens</Label>
                                        <Input
                                            type="number"
                                            value={maxTokens}
                                            onChange={e => setMaxTokens(parseInt(e.target.value))}
                                            className="w-32"
                                        />
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    )}

                    {/* Chat Messages */}
                    <Card className="h-[500px] flex flex-col">
                        <CardContent className="flex-1 p-4 space-y-4 overflow-y-auto" style={{ maxHeight: 'calc(100% - 72px)' }}>
                            {messages.length === 0 ? (
                                <div className="h-full flex items-center justify-center text-muted-foreground">
                                    <div className="text-center space-y-2">
                                        <Bot className="w-12 h-12 mx-auto opacity-50" />
                                        <p>Start a conversation{currentProvider ? ` with ${currentProvider.name}` : ''}</p>
                                        {!selectedModel && (
                                            <p className="text-sm text-amber-600">Select a model to begin</p>
                                        )}
                                    </div>
                                </div>
                            ) : (
                                messages.map((msg, i) => (
                                    <div key={i} className={`flex gap-3 ${msg.role === "user" ? "justify-end" : ""}`}>
                                        {msg.role === "assistant" && (
                                            <div className="w-8 h-8 rounded-full bg-gradient-to-br from-violet-600 to-cyan-500 flex items-center justify-center flex-shrink-0">
                                                <Bot className="w-4 h-4 text-white" />
                                            </div>
                                        )}
                                        <div className={`max-w-[70%] p-3 rounded-lg ${msg.role === "user"
                                            ? "bg-primary text-primary-foreground"
                                            : "bg-muted"
                                            }`}>
                                            <p className="text-sm whitespace-pre-wrap">{msg.content}</p>
                                        </div>
                                        {msg.role === "user" && (
                                            <div className="w-8 h-8 rounded-full bg-muted flex items-center justify-center flex-shrink-0">
                                                <User className="w-4 h-4" />
                                            </div>
                                        )}
                                    </div>
                                ))
                            )}
                            {isLoading && (
                                <div className="flex gap-3">
                                    <div className="w-8 h-8 rounded-full bg-gradient-to-br from-violet-600 to-cyan-500 flex items-center justify-center">
                                        <Bot className="w-4 h-4 text-white animate-pulse" />
                                    </div>
                                    <div className="bg-muted p-3 rounded-lg">
                                        <div className="flex gap-1">
                                            <span className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce" />
                                            <span className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce [animation-delay:100ms]" />
                                            <span className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce [animation-delay:200ms]" />
                                        </div>
                                    </div>
                                </div>
                            )}
                            <div ref={messagesEndRef} />
                        </CardContent>

                        {/* Input */}
                        <div className="p-4 border-t">
                            <div className="flex gap-2">
                                <Input
                                    placeholder={selectedModel ? "Type a message..." : "Select a model first..."}
                                    value={input}
                                    onChange={e => setInput(e.target.value)}
                                    onKeyDown={e => e.key === "Enter" && !e.shiftKey && handleSend()}
                                    disabled={!selectedModel}
                                />
                                <Button onClick={handleSend} disabled={isLoading || !input.trim() || !selectedModel}>
                                    <Send className="w-4 h-4" />
                                </Button>
                            </div>
                        </div>
                    </Card>
                </TabsContent>

                <TabsContent value="code" className="space-y-4 mt-4">
                    {/* API Endpoints */}
                    <Card className="bg-gradient-to-r from-slate-900 to-slate-800 text-white border-0">
                        <CardContent className="py-4">
                            <div className="flex items-center gap-4">
                                <div className="p-3 rounded-xl bg-white/10">
                                    <Terminal className="w-6 h-6" />
                                </div>
                                <div>
                                    <h3 className="font-semibold">API Endpoint</h3>
                                    <p className="text-sm text-white/60">Use with your API key from API Keys page</p>
                                </div>
                                <div className="ml-auto text-right">
                                    <p className="text-white/50">Base URL</p>
                                    <code className="text-cyan-300">{apiUrl}</code>
                                </div>
                            </div>
                        </CardContent>
                    </Card>

                    {/* Language Tabs */}
                    <div className="flex gap-2 flex-wrap">
                        {Object.entries(codeSnippets).map(([lang, { label, icon }]) => (
                            <Button
                                key={lang}
                                variant={selectedLanguage === lang ? "default" : "outline"}
                                size="sm"
                                onClick={() => setSelectedLanguage(lang)}
                                className="gap-2"
                            >
                                <span>{icon}</span>
                                {label}
                            </Button>
                        ))}
                    </div>

                    {/* Code Block */}
                    <Card>
                        <CardHeader className="py-3 flex-row items-center justify-between">
                            <CardTitle className="text-sm flex items-center gap-2">
                                <Code2 className="w-4 h-4" />
                                {codeSnippets[selectedLanguage]?.label} - {currentProvider?.name || "Select Provider"}
                            </CardTitle>
                            <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => copyToClipboard(codeSnippets[selectedLanguage]?.code || "", selectedLanguage)}
                            >
                                {copiedSnippet === selectedLanguage ? (
                                    <Check className="w-4 h-4 text-emerald-500" />
                                ) : (
                                    <Copy className="w-4 h-4" />
                                )}
                            </Button>
                        </CardHeader>
                        <CardContent className="pt-0">
                            <pre className="bg-slate-950 text-slate-50 p-4 rounded-lg overflow-x-auto text-sm">
                                <code>{codeSnippets[selectedLanguage]?.code}</code>
                            </pre>
                        </CardContent>
                    </Card>

                    {/* Info */}
                    <Card className="border-amber-500/20 bg-amber-500/5">
                        <CardContent className="py-3">
                            <div className="flex items-start gap-3">
                                <Zap className="w-5 h-5 text-amber-500 mt-0.5" />
                                <div className="text-sm">
                                    <p className="font-medium text-amber-600">Get your API Key</p>
                                    <p className="text-muted-foreground">
                                        Create an API key in the <strong>API Keys</strong> section to authenticate your requests.
                                        The code snippets above are pre-configured with your current settings.
                                    </p>
                                </div>
                            </div>
                        </CardContent>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    );
}
