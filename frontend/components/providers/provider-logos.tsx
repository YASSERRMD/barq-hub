import Image from "next/image";
import { Cpu } from "lucide-react";

const PROVIDER_LOGOS: Record<string, string> = {
    openai: "/providers/openai.svg",
    anthropic: "/providers/anthropic.svg",
    google: "/providers/google.svg",
    gemini: "/providers/google.svg",
    mistral: "/providers/mistral.svg",
    cohere: "/providers/cohere.svg",
    perplexity: "/providers/perplexity.svg",
    groq: "/providers/groq.svg",
    together: "/providers/together.svg",
    azure: "/providers/azure.svg",
};

export const getProviderLogo = (providerId: string, className?: string) => {
    const src = PROVIDER_LOGOS[providerId.toLowerCase()];

    if (src) {
        // className usually contains width/height (e.g. w-6 h-6)
        // We ensure a default size if none provided, and use a relative wrapper for the Image
        return (
            <div className={`relative ${className || 'w-6 h-6'} select-none`}>
                <Image
                    src={src}
                    alt={`${providerId} logo`}
                    fill
                    className="object-contain"
                    sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw"
                />
            </div>
        );
    }

    return <Cpu className={className} />;
};

export const LocalLogo = ({ className }: { className?: string }) => (
    <svg role="img" viewBox="0 0 24 24" fill="currentColor" className={className} xmlns="http://www.w3.org/2000/svg">
        <path d="M12 2L2 7l10 5 10-5-10-5zm0 9l2.5-1.25L12 8.5l-2.5 1.25L12 11zm0 2.5l-5-2.5-5 2.5L12 22l10-8.5-5-2.5-5 2.5z" />
    </svg>
);
