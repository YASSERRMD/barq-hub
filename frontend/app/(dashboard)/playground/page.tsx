"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
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
} from "lucide-react";

interface Message {
    role: "user" | "assistant";
    content: string;
}

const providers = [
    { id: "openai", name: "OpenAI", models: ["gpt-4o", "gpt-4-turbo", "gpt-3.5-turbo"] },
    { id: "anthropic", name: "Anthropic", models: ["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"] },
    { id: "google", name: "Google AI", models: ["gemini-1.5-pro", "gemini-1.5-flash"] },
    { id: "cohere", name: "Cohere", models: ["command-r-plus", "command-r"] },
];

export default function PlaygroundPage() {
    const [messages, setMessages] = useState<Message[]>([]);
    const [input, setInput] = useState("");
    const [selectedProvider, setSelectedProvider] = useState("openai");
    const [selectedModel, setSelectedModel] = useState("gpt-4o");
    const [temperature, setTemperature] = useState(0.7);
    const [maxTokens, setMaxTokens] = useState(2048);
    const [systemPrompt, setSystemPrompt] = useState("You are a helpful AI assistant.");
    const [copiedSnippet, setCopiedSnippet] = useState<string | null>(null);
    const [activeTab, setActiveTab] = useState("chat");
    const [selectedLanguage, setSelectedLanguage] = useState("python");
    const [showSettings, setShowSettings] = useState(false);
    const [isLoading, setIsLoading] = useState(false);

    const currentProvider = providers.find(p => p.id === selectedProvider);
    const baseUrl = "http://localhost:4000/v1";

    const handleSend = async () => {
        if (!input.trim()) return;

        const userMessage: Message = { role: "user", content: input };
        setMessages(prev => [...prev, userMessage]);
        setInput("");
        setIsLoading(true);

        // Demo response
        setTimeout(() => {
            const assistantMessage: Message = {
                role: "assistant",
                content: `**Demo Mode** | Using ${currentProvider?.name} (${selectedModel})\n\nThis is a simulated response. To get real AI responses:\n\n1. Configure your provider's API key in **Providers**\n2. Start the backend server\n\nYour message: "${userMessage.content}"\n\nIn production, this would be a real response from the ${selectedModel} model.`,
            };
            setMessages(prev => [...prev, assistantMessage]);
            setIsLoading(false);
        }, 1000);
    };

    const clearChat = () => setMessages([]);

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
API_KEY = "your-api-key"  # Get from Applications
BASE_URL = "${baseUrl}"

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
print(result["choices"][0]["message"]["content"])

# Embeddings
def embed(texts, model="text-embedding-3-small"):
    response = requests.post(
        f"{BASE_URL}/embeddings",
        headers=headers,
        json={"model": model, "input": texts}
    )
    return response.json()`,
        },
        javascript: {
            label: "JavaScript",
            icon: "🟨",
            code: `// BARQ API Client
const API_KEY = 'your-api-key';  // Get from Applications
const BASE_URL = '${baseUrl}';

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
console.log(result.choices[0].message.content);

// Embeddings
async function embed(texts, model = 'text-embedding-3-small') {
  const response = await fetch(\`\${BASE_URL}/embeddings\`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ model, input: texts })
  });
  return response.json();
}`,
        },
        typescript: {
            label: "TypeScript",
            icon: "🔷",
            code: `// BARQ API Client with TypeScript
interface Message {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

interface ChatResponse {
  choices: { message: { content: string } }[];
  usage: { prompt_tokens: number; completion_tokens: number };
}

class BarqClient {
  private apiKey: string;
  private baseUrl: string;

  constructor(apiKey: string, baseUrl = '${baseUrl}') {
    this.apiKey = apiKey;
    this.baseUrl = baseUrl;
  }

  private get headers() {
    return {
      'Authorization': \`Bearer \${this.apiKey}\`,
      'Content-Type': 'application/json'
    };
  }

  async chat(
    messages: Message[], 
    model = '${selectedModel}',
    provider = '${selectedProvider}'
  ): Promise<ChatResponse> {
    const response = await fetch(\`\${this.baseUrl}/chat/completions\`, {
      method: 'POST',
      headers: this.headers,
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
}

// Usage with ${currentProvider?.name || selectedProvider}
const client = new BarqClient('your-api-key');
const result = await client.chat([
  { role: 'system', content: '${systemPrompt}' },
  { role: 'user', content: 'Hello!' }
]);`,
        },
        curl: {
            label: "cURL",
            icon: "🌀",
            code: `# Chat Completion with ${currentProvider?.name || selectedProvider}
curl -X POST "${baseUrl}/chat/completions" \\
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
  }'

# Embeddings
curl -X POST "${baseUrl}/embeddings" \\
  -H "Authorization: Bearer your-api-key" \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "text-embedding-3-small",
    "input": ["Hello world", "Another text"]
  }'`,
        },
        go: {
            label: "Go",
            icon: "🐹",
            code: `package main

import (
    "bytes"
    "encoding/json"
    "fmt"
    "net/http"
)

const (
    APIKey  = "your-api-key"
    BaseURL = "${baseUrl}"
)

type Message struct {
    Role    string \`json:"role"\`
    Content string \`json:"content"\`
}

type ChatRequest struct {
    Model       string    \`json:"model"\`
    Provider    string    \`json:"provider"\`
    Messages    []Message \`json:"messages"\`
    Temperature float64   \`json:"temperature"\`
    MaxTokens   int       \`json:"max_tokens"\`
}

// Usage with ${currentProvider?.name || selectedProvider}
func main() {
    messages := []Message{
        {Role: "system", Content: "${systemPrompt}"},
        {Role: "user", Content: "Hello!"},
    }
    
    reqBody := ChatRequest{
        Model:       "${selectedModel}",
        Provider:    "${selectedProvider}",
        Messages:    messages,
        Temperature: ${temperature},
        MaxTokens:   ${maxTokens},
    }
    
    body, _ := json.Marshal(reqBody)
    req, _ := http.NewRequest("POST", BaseURL+"/chat/completions", bytes.NewBuffer(body))
    req.Header.Set("Authorization", "Bearer "+APIKey)
    req.Header.Set("Content-Type", "application/json")
    
    resp, _ := http.DefaultClient.Do(req)
    defer resp.Body.Close()
    
    fmt.Println("Response received")
}`,
        },
        rust: {
            label: "Rust",
            icon: "🦀",
            code: `use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;

const API_KEY: &str = "your-api-key";
const BASE_URL: &str = "${baseUrl}";

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    provider: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

// Usage with ${currentProvider?.name || selectedProvider}
#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, format!("Bearer {}", API_KEY).parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    
    let request = ChatRequest {
        model: "${selectedModel}".to_string(),
        provider: "${selectedProvider}".to_string(),
        messages: vec![
            Message { role: "system".into(), content: "${systemPrompt}".into() },
            Message { role: "user".into(), content: "Hello!".into() },
        ],
        temperature: ${temperature},
        max_tokens: ${maxTokens},
    };
    
    let response = client
        .post(format!("{}/chat/completions", BASE_URL))
        .headers(headers)
        .json(&request)
        .send()
        .await
        .unwrap();
    
    println!("{}", response.text().await.unwrap());
}`,
        },
    };

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold">Playground</h1>
                    <p className="text-muted-foreground">Test models and get integration code</p>
                </div>
                <div className="flex items-center gap-2">
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
                                    <Select value={selectedProvider} onValueChange={(v) => {
                                        setSelectedProvider(v);
                                        const models = providers.find(p => p.id === v)?.models || [];
                                        setSelectedModel(models[0] || "");
                                    }}>
                                        <SelectTrigger className="w-36">
                                            <SelectValue />
                                        </SelectTrigger>
                                        <SelectContent>
                                            {providers.map(p => (
                                                <SelectItem key={p.id} value={p.id}>{p.name}</SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                </div>
                                <div className="flex items-center gap-2">
                                    <Label>Model:</Label>
                                    <Select value={selectedModel} onValueChange={setSelectedModel}>
                                        <SelectTrigger className="w-40">
                                            <SelectValue />
                                        </SelectTrigger>
                                        <SelectContent>
                                            {currentProvider?.models.map(m => (
                                                <SelectItem key={m} value={m}>{m}</SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
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
                                <Badge variant="secondary" className="ml-auto">
                                    {currentProvider?.name} • {selectedModel}
                                </Badge>
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
                    <Card className="min-h-[400px] flex flex-col">
                        <CardContent className="flex-1 p-4 space-y-4 overflow-y-auto">
                            {messages.length === 0 ? (
                                <div className="h-full flex items-center justify-center text-muted-foreground">
                                    <div className="text-center space-y-2">
                                        <Bot className="w-12 h-12 mx-auto opacity-50" />
                                        <p>Start a conversation with {currentProvider?.name}</p>
                                    </div>
                                </div>
                            ) : (
                                messages.map((msg, i) => (
                                    <div key={i} className={`flex gap-3 ${msg.role === "user" ? "justify-end" : ""}`}>
                                        {msg.role === "assistant" && (
                                            <div className="w-8 h-8 rounded-full bg-gradient-to-br from-violet-600 to-cyan-500 flex items-center justify-center">
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
                                            <div className="w-8 h-8 rounded-full bg-muted flex items-center justify-center">
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
                                            <span className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce delay-100" />
                                            <span className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce delay-200" />
                                        </div>
                                    </div>
                                </div>
                            )}
                        </CardContent>

                        {/* Input */}
                        <div className="p-4 border-t">
                            <div className="flex gap-2">
                                <Input
                                    placeholder="Type a message..."
                                    value={input}
                                    onChange={e => setInput(e.target.value)}
                                    onKeyDown={e => e.key === "Enter" && !e.shiftKey && handleSend()}
                                />
                                <Button onClick={handleSend} disabled={isLoading || !input.trim()}>
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
                                    <h3 className="font-semibold">API Endpoints</h3>
                                    <p className="text-sm text-white/60">Use these with your API key from Applications</p>
                                </div>
                                <div className="ml-auto flex gap-6 text-sm">
                                    <div className="text-right">
                                        <p className="text-white/50">REST</p>
                                        <code className="text-cyan-300">{baseUrl}</code>
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
                                {codeSnippets[selectedLanguage]?.label} - {currentProvider?.name}
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
                                        Create an application in the <strong>Applications</strong> section to get your API key.
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
