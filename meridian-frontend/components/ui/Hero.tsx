'use client';

import { motion } from 'framer-motion';
import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface HeroProps {
  badge?: ReactNode;
  headline: string;
  subheadline?: string;
  description: string;
  ctas?: {
    primary: ReactNode;
    secondary?: ReactNode;
  };
  trustIndicators?: Array<{
    icon: ReactNode;
    text: string;
  }>;
  className?: string;
}

export function Hero({
  badge,
  headline,
  subheadline,
  description,
  ctas,
  trustIndicators,
  className,
}: HeroProps) {
  return (
    <section className={cn('relative overflow-hidden pt-32 pb-24 md:pt-40 md:pb-32 bg-white dark:bg-black', className)}>
      {/* Gradient Mesh Background */}
      <div className="absolute inset-0 gradient-mesh" />
      
      {/* Subtle Grid Pattern */}
      <div className="absolute inset-0 grid-pattern opacity-50" />
      
      <div className="relative z-10 max-w-7xl mx-auto px-8">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6 }}
          className="max-w-4xl"
        >
          {/* Badge */}
          {badge && (
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.1, duration: 0.5 }}
              className="mb-8"
            >
              {badge}
            </motion.div>
          )}

          {/* Headline */}
          <motion.h1
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2, duration: 0.5 }}
            className="text-6xl md:text-7xl lg:text-8xl font-bold tracking-tight mb-8 text-black dark:text-white leading-[1.1]"
            style={{ letterSpacing: '-0.02em' }}
          >
            {headline}
            {subheadline && (
              <span className="block mt-2 bg-gradient-to-r from-black via-gray-600 to-black dark:from-white dark:via-gray-400 dark:to-white bg-clip-text text-transparent">
                {subheadline}
              </span>
            )}
          </motion.h1>

          {/* Description */}
          <motion.p
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3, duration: 0.5 }}
            className="text-xl md:text-2xl text-gray-600 dark:text-gray-400 mb-12 leading-relaxed max-w-3xl font-normal"
          >
            {description}
          </motion.p>

          {/* CTAs */}
          {ctas && (
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.4, duration: 0.5 }}
              className="flex flex-col sm:flex-row gap-4 mb-16"
            >
              {ctas.primary}
              {ctas.secondary}
            </motion.div>
          )}

          {/* Trust Indicators */}
          {trustIndicators && trustIndicators.length > 0 && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.6, duration: 0.5 }}
              className="flex flex-wrap items-center gap-8"
            >
              {trustIndicators.map((indicator, index) => (
                <div
                  key={index}
                  className="flex items-center space-x-2 text-gray-600 dark:text-gray-400"
                >
                  {indicator.icon}
                  <span className="text-sm font-medium">{indicator.text}</span>
                </div>
              ))}
            </motion.div>
          )}
        </motion.div>
      </div>
    </section>
  );
}

