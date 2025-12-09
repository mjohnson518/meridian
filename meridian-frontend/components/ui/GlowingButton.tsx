import React from 'react';
import Link from 'next/link';
import { cn } from '@/lib/utils';

interface GlowingButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: 'primary' | 'secondary' | 'outline';
    size?: 'sm' | 'md' | 'lg';
    href?: string;
    children: React.ReactNode;
}

export function GlowingButton({
    variant = 'primary',
    size = 'md',
    className,
    href,
    children,
    ...props
}: GlowingButtonProps) {
    const variants = {
        primary: "bg-gradient-to-r from-emerald-500 to-teal-500 text-white shadow-[0_0_20px_-5px_rgba(16,185,129,0.5)] hover:shadow-[0_0_30px_-5px_rgba(16,185,129,0.6)] border-none hover:scale-105",
        secondary: "bg-surface text-text-primary border border-white/10 hover:bg-surface-hover hover:border-primary/50",
        outline: "bg-transparent border border-white/20 text-white hover:bg-white/5 hover:border-emerald-400/50 hover:shadow-[0_0_20px_-5px_rgba(16,185,129,0.2)]"
    };

    const sizes = {
        sm: "px-4 py-2 text-sm",
        md: "px-6 py-3 text-base",
        lg: "px-8 py-4 text-lg font-semibold"
    };

    const classes = cn(
        "relative rounded-full transition-all duration-300 ease-out active:scale-95",
        "flex items-center justify-center gap-2",
        variants[variant],
        sizes[size],
        className
    );

    if (href) {
        return (
            <Link href={href} className={classes}>
                {children}
            </Link>
        );
    }

    return (
        <button className={classes} {...props}>
            {children}
        </button>
    );
}
