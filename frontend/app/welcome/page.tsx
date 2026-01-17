import Link from "next/link";
import { Zap, ArrowRight, Cpu, Key, FlaskConical, Shield, BarChart3, Sparkles } from "lucide-react";
import { Button } from "@/components/ui/button";

export default function LandingPage() {
    const features = [
        {
            icon: Cpu,
            title: "Multi-Provider LLM",
            description: "Connect to OpenAI, Anthropic, Google AI, Cohere, and more from a single interface.",
        },
        {
            icon: Key,
            title: "API Key Management",
            description: "Create and manage API keys with scopes, rate limits, and auto-rotation.",
        },
        {
            icon: FlaskConical,
            title: "Integrated Playground",
            description: "Test prompts across providers with code snippets for instant integration.",
        },
        {
            icon: Shield,
            title: "Enterprise Security",
            description: "Role-based access control, audit logs, and compliance-ready features.",
        },
        {
            icon: BarChart3,
            title: "Cost Analytics",
            description: "Track usage, set budgets, and optimize costs across all providers.",
        },
        {
            icon: Sparkles,
            title: "REST & gRPC APIs",
            description: "Production-ready APIs with comprehensive documentation and SDKs.",
        },
    ];

    return (
        <div className="min-h-screen bg-gradient-to-br from-background via-background to-muted/30">
            {/* Header */}
            <header className="fixed top-0 left-0 right-0 z-50 h-16 border-b border-border/40 bg-background/80 backdrop-blur-xl">
                <div className="container mx-auto h-full flex items-center justify-between px-6">
                    <Link href="/welcome" className="flex items-center gap-3">
                        <img src="/logo.png" alt="BARQ HUB" className="w-8 h-8 object-contain" />
                        <span className="text-xl font-bold bg-gradient-to-r from-violet-600 to-cyan-500 bg-clip-text text-transparent">
                            BARQ HUB
                        </span>
                    </Link>
                    <div className="flex items-center gap-4">
                        <Link href="/login">
                            <Button variant="ghost">Sign In</Button>
                        </Link>
                        <Link href="/register">
                            <Button className="bg-gradient-to-r from-violet-600 to-cyan-500 text-white">
                                Get Started
                            </Button>
                        </Link>
                    </div>
                </div>
            </header>

            {/* Hero */}
            <section className="pt-32 pb-20 px-6">
                <div className="container mx-auto text-center max-w-4xl">
                    <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-violet-500/10 text-violet-600 text-sm font-medium mb-6">
                        <Sparkles className="w-4 h-4" />
                        AI Management Console
                    </div>
                    <img src="/logo.png" alt="BARQ HUB Logo" className="w-24 h-24 mx-auto mb-6 object-contain animate-in fade-in zoom-in duration-500" />
                    <h1 className="text-5xl md:text-6xl font-bold tracking-tight mb-6">
                        Welcome to{" "}
                        <span className="bg-gradient-to-r from-violet-600 to-cyan-500 bg-clip-text text-transparent">
                            BARQ HUB
                        </span>
                    </h1>
                    <p className="text-xl text-muted-foreground mb-10 max-w-2xl mx-auto">
                        Enterprise-grade AI orchestration platform. Manage LLM providers, API keys,
                        and monitor usage from a single, beautiful dashboard.
                    </p>
                    <div className="flex items-center justify-center gap-4">
                        <Link href="/register">
                            <Button size="lg" className="bg-gradient-to-r from-violet-600 to-cyan-500 text-white shadow-lg shadow-violet-500/25 hover:shadow-violet-500/40 gap-2">
                                Get Started Free <ArrowRight className="w-4 h-4" />
                            </Button>
                        </Link>
                        <Link href="/login">
                            <Button size="lg" variant="outline">
                                Sign In
                            </Button>
                        </Link>
                    </div>
                </div>
            </section>

            {/* Features */}
            <section className="py-20 px-6 bg-muted/30">
                <div className="container mx-auto max-w-6xl">
                    <div className="text-center mb-16">
                        <h2 className="text-3xl font-bold mb-4">Everything you need to manage AI</h2>
                        <p className="text-muted-foreground max-w-2xl mx-auto">
                            A complete platform for managing LLM providers, API access, and usage analytics.
                        </p>
                    </div>
                    <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
                        {features.map((feature, i) => (
                            <div
                                key={i}
                                className="p-6 rounded-2xl bg-background border border-border/50 hover:border-primary/20 hover:shadow-lg transition-all"
                            >
                                <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-violet-600 to-cyan-500 flex items-center justify-center mb-4">
                                    <feature.icon className="w-6 h-6 text-white" />
                                </div>
                                <h3 className="font-semibold text-lg mb-2">{feature.title}</h3>
                                <p className="text-muted-foreground text-sm">{feature.description}</p>
                            </div>
                        ))}
                    </div>
                </div>
            </section>

            {/* CTA */}
            <section className="py-20 px-6">
                <div className="container mx-auto max-w-4xl text-center">
                    <div className="p-10 rounded-3xl bg-gradient-to-br from-violet-600 to-cyan-500 text-white">
                        <h2 className="text-3xl font-bold mb-4">Ready to get started?</h2>
                        <p className="text-white/80 mb-8 max-w-xl mx-auto">
                            Join thousands of developers and enterprises managing their AI infrastructure with BARQ HUB.
                        </p>
                        <Link href="/register">
                            <Button size="lg" className="bg-white text-violet-600 hover:bg-white/90 gap-2">
                                Create Free Account <ArrowRight className="w-4 h-4" />
                            </Button>
                        </Link>
                    </div>
                </div>
            </section>

            {/* Footer */}
            <footer className="py-8 px-6 border-t border-border/40">
                <div className="container mx-auto flex items-center justify-between text-sm text-muted-foreground">
                    <div className="flex items-center gap-2">
                        <img src="/logo.png" alt="BARQ HUB" className="w-6 h-6 object-contain" />
                        <span>BARQ HUB</span>
                    </div>
                    <p>© 2024 BARQ HUB. All rights reserved.</p>
                </div>
            </footer>
        </div>
    );
}
