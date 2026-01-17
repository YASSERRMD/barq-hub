
import { SVGProps } from "react";

// OpenAI - Authentic Brand Color (Black/White usually, but we use formatted paths)
export const OpenAILogo = (props: SVGProps<SVGSVGElement>) => (
    <svg viewBox="0 0 24 24" fill="currentColor" {...props}>
        <path d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.0462 6.0462 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9723V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.3829a.0804.0804 0 0 1-.0426-.0615V2.7095a4.4992 4.4992 0 0 1 6.136 1.6417 4.49 4.49 0 0 1 .53 3.0137l-.142-.0805-4.7783-2.7629a.7948.7948 0 0 0-.7854 0L8.179 7.8909v-2.3229c.0095-.0237.0284-.0473.0473-.0568l4.8255-2.7865a4.4992 4.4992 0 0 1 5.8854 9.0264zm-5.8475 2.6731l-2.02-1.1686a.0757.0757 0 0 1-.038-.0568V7.6166a4.504 4.504 0 0 1 4.4945-4.4944 4.4755 4.4755 0 0 1 2.8716 1.0408l-.142.0805-4.783 2.7582a.7712.7712 0 0 0-.3927.6813zm6.7322-4.1063a.7664.7664 0 0 0-.3879-.6765l-5.8191-3.3543 2.0248-1.1686a.0757.0757 0 0 1 .071 0l4.8303 2.7866a4.4992 4.4992 0 0 1 1.5705 5.819 4.4755 4.4755 0 0 1-2.2897 1.958v-5.3642z" />
    </svg>
);

// Anthropic - Authentic
export const AnthropicLogo = (props: SVGProps<SVGSVGElement>) => (
    <svg viewBox="0 0 24 24" fill="currentColor" {...props}>
        <path d="M17.41 19.467H21.5L12 2 2.5 19.467h4.09l1.647-3.238h7.526l1.647 3.238Zm-6.55-12.876.71-.059 2.508 4.93H9.92l.941-4.87ZM8.97 14.502h6.059l.902 1.774H8.069l.9-1.774Z" />
    </svg>
);

// Google - Multi-color
export const GoogleLogo = (props: SVGProps<SVGSVGElement>) => (
    <svg viewBox="0 0 24 24" {...props}>
        <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" fill="#4285F4" />
        <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853" />
        <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05" />
        <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335" />
    </svg>
);

// Mistral - Brand Orange/Yellow
export const MistralLogo = (props: SVGProps<SVGSVGElement>) => (
    <svg viewBox="0 0 24 24" fill="none" {...props}>
        <path d="M8.2 14.5h-.8v-5l-2 3-2-3v5h-.8v-6.5h.8l2 3.2 2-3.2h.8v6.5zm3.6 0h.8V8h-2.5v1h1.7v5.5zm4.8 0h-.8V11h-1.8v3.5h-.8V8h3.3v6.5zm3 0h-.8v-2.3l-1.3 2.3h-.7l-1.3-2.3v2.3h-.8V8h.9l1.6 2.8L18.4 8h.9v6.5z" fill="#F19E39" />
    </svg>
);

// Cohere - Authentic
export const CohereLogo = (props: SVGProps<SVGSVGElement>) => (
    <svg viewBox="0 0 24 24" fill="currentColor" {...props}>
        <path d="M12 0c6.627 0 12 5.373 12 12s-5.373 12-12 12S0 18.627 0 12 5.373 0 12 0Zm5.4 13.9a3.9 3.9 0 1 1-7.8 0 3.9 3.9 0 0 1 7.8 0Zm-5.4-8.8a3.9 3.9 0 1 0 0 7.8 3.9 3.9 0 0 0 0-7.8Zm-7.8 4.4a3.9 3.9 0 1 1 7.8 0 3.9 3.9 0 0 1-7.8 0Z" />
    </svg>
);

// Perplexity - Brand Cyan
export const PerplexityLogo = (props: SVGProps<SVGSVGElement>) => (
    <svg viewBox="0 0 24 24" fill="currentColor" {...props}>
        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41C17.92 5.77 20 8.65 20 12c0 2.08-.81 3.98-2.11 5.39z" fill="#22B3B8" />
    </svg>
)

// Groq - Authentic
export const GroqLogo = (props: SVGProps<SVGSVGElement>) => (
    <svg viewBox="0 0 24 24" fill="currentColor" {...props}>
        <path d="M4 18h2v-2H4v2zm0-4h2v-2H4v2zm0-4h2V8H4v2zm4 8h12v-2H8v2zm0-4h12v-2H8v2zm0-4h12V8H8v2z" />
    </svg>
)

// Together - Authentic
export const TogetherLogo = (props: SVGProps<SVGSVGElement>) => (
    <svg viewBox="0 0 24 24" fill="currentColor" {...props}>
        <rect x="2" y="2" width="6" height="6" rx="1" fill="#3B82F6" />
        <rect x="10" y="2" width="6" height="6" rx="1" fill="#3B82F6" opacity="0.8" />
        <rect x="18" y="2" width="6" height="6" rx="1" fill="#3B82F6" opacity="0.6" />
        <rect x="2" y="10" width="6" height="6" rx="1" fill="#3B82F6" opacity="0.8" />
        <rect x="10" y="10" width="6" height="6" rx="1" fill="#3B82F6" opacity="0.6" />
        <rect x="2" y="18" width="6" height="6" rx="1" fill="#3B82F6" opacity="0.6" />
    </svg>
)

// Azure - Brand Blue
export const AzureLogo = (props: SVGProps<SVGSVGElement>) => (
    <svg viewBox="0 0 24 24" fill="currentColor" {...props}>
        <path d="M5.9 20.35 2 16.96l5.96-7.6 1.15 6.33-3.21 4.66zM11.69 3 5.4 12l2.35 1.7 8.52-5.75L11.69 3zm1.61 8.24-5.3 3.6 7.6 6.16 6.4-1.2-8.7-8.56z" fill="#0078D4" />
    </svg>
)

// Ollama - Authentic Shape
export const LocalLogo = (props: SVGProps<SVGSVGElement>) => (
    <svg viewBox="0 0 24 24" fill="currentColor" {...props}>
        <path d="M19.62 10.36c-.45-.48-1-1.32-2.14-1.32s-1.69.84-2.14 1.32c-.44.47-1 1.05-2.09 1.05s-1.65-.58-2.09-1.05c-.45-.48-1-1.32-2.14-1.32s-1.69.84-2.14 1.32c-.44.47-1 1.05-2.09 1.05s-2.09-.58-2.09-1.05 1-1.28 2.09-1.28c1.14 0 1.69.84 2.14 1.32.44.47 1 1.05 2.09 1.05s1.65-.58 2.09-1.05c.45-.48 1-1.32 2.14-1.32s1.69.84 2.14 1.32c.44.47 1 1.05 2.09 1.05s1.65-.58 2.09-1.05c.45-.48 1-1.32 2.14-1.32s1.69.84 2.14 1.32c.44.47 1 1.05 2.09 1.05s2.09-.58 2.09-1.05-1-1.28-2.09-1.28c-1.14 0-1.69.84-2.14 1.32-.44.47-1 1.05-2.09 1.05z" fill="currentColor" />
        <path d="M3.5 13.5a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3zm17 0a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3z" fill="currentColor" />
    </svg>
)

export function getProviderLogo(providerId: string, className?: string) {
    const props = { className: className || "w-6 h-6" };

    switch (providerId.toLowerCase()) {
        case 'openai': return <OpenAILogo {...props} />;
        case 'anthropic': return <AnthropicLogo {...props} />;
        case 'google': return <GoogleLogo {...props} />;
        case 'mistral': return <MistralLogo {...props} />;
        case 'cohere': return <CohereLogo {...props} />;
        case 'perplexity': return <PerplexityLogo {...props} />;
        case 'groq': return <GroqLogo {...props} />;
        case 'together': return <TogetherLogo {...props} />;
        case 'azure':
        case 'azure-openai': return <AzureLogo {...props} />;
        case 'local':
        case 'ollama': return <LocalLogo {...props} />;
        default: return null; // caller handles fallback
    }
}
