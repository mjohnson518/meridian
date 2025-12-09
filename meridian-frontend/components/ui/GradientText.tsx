import React from 'react';
import { cn } from '@/lib/utils';

interface GradientTextProps extends React.HTMLAttributes<HTMLSpanElement> {
    children: React.ReactNode;
    variant?: 'primary' | 'gold' | 'purple';
}

export function GradientText({ children, className, variant = 'primary', ...props }: GradientTextProps) {
    const variants = {
        primary: "from-secondary via-primary to-accent-blue",
        gold: "from-amber-300 via-orange-400 to-amber-500",
        purple: "from-accent-purple via-accent-pink to-accent-blue",
    };

    return (
        <span
            className={cn(
                "bg-clip-text text-transparent bg-gradient-to-r bg-[length:200%_auto] animate-gradient",
                variants[variant],
                className
            )}
            {...props}
        >
            {children}
        </span>
    );
}
