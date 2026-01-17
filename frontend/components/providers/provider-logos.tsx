import { Cpu } from "lucide-react";

export const getProviderLogo = (providerId: string, className?: string) => {
    switch (providerId.toLowerCase()) {
        case 'openai':
            return (
                <svg role="img" viewBox="0 0 24 24" fill="#000000" className={className} xmlns="http://www.w3.org/2000/svg">
                    <path d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a1.54 1.54 0 0 1 .8312 1.32v5.64a4.4879 4.4879 0 0 1-5.2877 3.169zM6.685 8.9528a4.5 4.5 0 0 1 .5367-3.1737l.137.0804 4.7795 2.7582a.77.77 0 0 0 .7802 0l5.8328-3.3676 2.02 1.1686a1.54 1.54 0 0 1 .8312 1.32v5.64l-2.02-1.1686-3.9565-2.2851a.7948.7948 0 0 0-.7802 0l-5.8328 3.3676V8.9528zm-3.736 1.9515a4.4879 4.4879 0 0 1-.9512-4.1026l.1357.0804 4.7795 2.7582a.77.77 0 0 0 .7802 0l5.8328-3.3676V1.7377a1.54 1.54 0 0 1 1.3283.8312 4.48 4.48 0 0 1-1.0708 5.0976l-2.02 1.1674-3.9565-2.285a.7948.7948 0 0 0-.7802 0L5.3571 9.9079l-2.4081-1.3916zM1.956 12.8719a4.505 4.505 0 0 1 .532-3.1725l.1369-.0804 4.7807-2.7582a1.54 1.54 0 0 1 1.3283.8312 4.48 4.48 0 0 1-1.0708 5.0976l-2.02 1.1674-3.9565-2.285a.7948.7948 0 0 0-.7802 0l-4.7077 2.7161v-1.5162zm16.687-1.9515a4.4879 4.4879 0 0 1 .9512 4.1026l-.1357-.0804-4.7795-2.7582a.77.77 0 0 0-.7802 0l-5.8328 3.3676v4.535a1.54 1.54 0 0 1-1.3283-.8312 4.48 4.48 0 0 1 1.0708-5.0976l2.02-1.1674 3.9565 2.285a.7948.7948 0 0 0 .7802 0l4.3795-2.5292 2.4082 1.3916-.0196-3.2173zM13.266 1.7487a4.4755 4.4755 0 0 1 2.8764 1.0408l-.1419.0804-4.7783 2.7582a.7948.7948 0 0 0-.3927.6813v6.7369l-2.02-1.1686a1.54 1.54 0 0 1-.8312-1.32v-5.64a4.4879 4.4879 0 0 1 5.2877-3.169zM12 10.4566L9.6234 9.083 7.2456 10.4577 9.6222 11.8324 12 10.4566zm2.3766 1.3734-2.3754 1.3734v2.7468l2.3754-1.3734v-2.7468zm-4.7532 0v2.7468l2.3766 1.3734v-2.7468l-2.3766-1.3734z" />
                </svg>
            );
        case 'anthropic':
            // Terracotta/Orange #D97757
            return (
                <svg role="img" viewBox="0 0 24 24" fill="#D97757" className={className} xmlns="http://www.w3.org/2000/svg">
                    <path d="M17.422 17.5h-1.3c-.66 0-1.2-.42-1.42-.98l-3.32-8.32c-.36-.9-1.6-.9-1.96 0l-3.32 8.32c-.22.56-.76.98-1.42.98h-1.3L9.662 3.12c.56-1.4 2.52-1.4 3.08 0l6.38 14.38h-1.7zm-5.42-2.92l1.6-4.04 1.6 4.04h-3.2z" />
                </svg>
            );
        case 'google':
        case 'gemini':
            // Google Blue #4285F4
            return (
                <svg role="img" viewBox="0 0 24 24" fill="#4285F4" className={className} xmlns="http://www.w3.org/2000/svg">
                    <path d="M11.95 4.1c0 4.14-3.36 7.5-7.5 7.5 4.14 0 7.5 3.36 7.5 7.5 0-4.14 3.36-7.5 7.5-7.5-4.14 0-7.5-3.36-7.5-7.5z" />
                    <path d="M19.3 2.1c0 1.95-1.55 3.5-3.5 3.5 1.95 0 3.5 1.55 3.5 3.5 0-1.95 1.55-3.5 3.5-3.5-1.95 0-3.5-1.55-3.5-3.5z" opacity="0.5" />
                </svg>
            );
        case 'mistral':
            // Mistral Orange/Yellow #F19E39
            return (
                <svg role="img" viewBox="0 0 24 24" fill="#F19E39" className={className} xmlns="http://www.w3.org/2000/svg">
                    <rect x="2" y="10" width="4" height="12" />
                    <rect x="8" y="6" width="4" height="16" />
                    <rect x="14" y="2" width="4" height="20" />
                    <rect x="20" y="6" width="4" height="12" />
                </svg>
            );
        case 'cohere':
            // Cohere Purple/Coral - using #D56C6C (Coral) as distinct from others
            return (
                <svg role="img" viewBox="0 0 24 24" fill="#D56C6C" className={className} xmlns="http://www.w3.org/2000/svg">
                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 14H9v-2h2v2zm0-4H9V8h2v4zm4 4h-2v-2h2v2zm0-4h-2V8h2v4z" />
                </svg>
            );
        case 'perplexity':
            // Perplexity Teal #20808D
            return (
                <svg role="img" viewBox="0 0 24 24" fill="#20808D" className={className} xmlns="http://www.w3.org/2000/svg">
                    <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="2" fill="none" />
                    <path d="M8 12h8m-4-4v8" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
                    <circle cx="12" cy="12" r="2" fill="currentColor" />
                </svg>
            );
        case 'groq':
            // Groq Red-Orange #F55036
            return (
                <svg role="img" viewBox="0 0 24 24" fill="#F55036" className={className} xmlns="http://www.w3.org/2000/svg">
                    <path d="M3 5h18v2H3V5zm0 6h12v2H3v-2zm0 6h18v2H3v-2z" />
                </svg>
            );
        case 'together':
            // Together Blue #3B82F6
            return (
                <svg role="img" viewBox="0 0 24 24" fill="#3B82F6" className={className} xmlns="http://www.w3.org/2000/svg">
                    <circle cx="8" cy="12" r="4" />
                    <circle cx="16" cy="12" r="4" />
                </svg>
            );
        case 'azure':
            // Azure Blue #0078D4
            return (
                <svg role="img" viewBox="0 0 24 24" fill="#0078D4" className={className} xmlns="http://www.w3.org/2000/svg">
                    <path d="M11.6 3.1L5.8 17.8h3.3l2.8-8.2 5 8.2h5.5L11.6 3.1zM2.5 17.8l3.1 3.1h4L5 17.8H2.5z" />
                </svg>
            );
        default:
            return <Cpu className={className} />;
    }
};

export const LocalLogo = ({ className }: { className?: string }) => (
    <svg role="img" viewBox="0 0 24 24" fill="currentColor" className={className} xmlns="http://www.w3.org/2000/svg">
        <path d="M12 2L2 7l10 5 10-5-10-5zm0 9l2.5-1.25L12 8.5l-2.5 1.25L12 11zm0 2.5l-5-2.5-5 2.5L12 22l10-8.5-5-2.5-5 2.5z" />
    </svg>
);
