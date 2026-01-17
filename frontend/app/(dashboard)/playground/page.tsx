"use client";

import { useState } from "react";
import { Send, Cpu, Settings } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";

export default function PlaygroundPage() {
    const [prompt, setPrompt] = useState("");
    const [response, setResponse] = useState("");

    return (
        <div className="space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Playground</h1>
                    <p className="text-muted-foreground">Test prompts with different providers</p>
                </div>
            </div>

            <div className="grid gap-6 lg:grid-cols-3">
                {/* Settings */}
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <Settings className="h-5 w-5" />
                            Settings
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="space-y-2">
                            <label className="text-sm font-medium">Provider</label>
                            <Select defaultValue="openai">
                                <SelectTrigger>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="openai">OpenAI</SelectItem>
                                    <SelectItem value="anthropic">Anthropic</SelectItem>
                                    <SelectItem value="google">Google Gemini</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div className="space-y-2">
                            <label className="text-sm font-medium">Model</label>
                            <Select defaultValue="gpt-4o">
                                <SelectTrigger>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="gpt-4o">gpt-4o</SelectItem>
                                    <SelectItem value="gpt-4-turbo">gpt-4-turbo</SelectItem>
                                    <SelectItem value="gpt-3.5-turbo">gpt-3.5-turbo</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div className="pt-4 border-t">
                            <div className="flex items-center justify-between text-sm">
                                <span className="text-muted-foreground">Est. Cost</span>
                                <Badge variant="outline">$0.00</Badge>
                            </div>
                        </div>
                    </CardContent>
                </Card>

                {/* Chat Area */}
                <div className="lg:col-span-2 space-y-4">
                    <Card className="min-h-[300px]">
                        <CardHeader>
                            <CardTitle className="flex items-center gap-2">
                                <Cpu className="h-5 w-5" />
                                Response
                            </CardTitle>
                        </CardHeader>
                        <CardContent>
                            {response ? (
                                <p className="text-sm whitespace-pre-wrap">{response}</p>
                            ) : (
                                <p className="text-sm text-muted-foreground">Response will appear here...</p>
                            )}
                        </CardContent>
                    </Card>

                    <div className="flex gap-2">
                        <Textarea
                            placeholder="Enter your prompt..."
                            value={prompt}
                            onChange={(e) => setPrompt(e.target.value)}
                            className="min-h-[100px]"
                        />
                        <Button className="self-end">
                            <Send className="h-4 w-4" />
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    );
}
