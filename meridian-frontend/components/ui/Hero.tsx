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
    <section className={cn('relative overflow-hidden bg-white dark:bg-black', className)}>
      {/* Gradient Mesh Background */}
      <div className="absolute inset-0 gradient-mesh opacity-30" />
      
      {/* Subtle Grid Pattern */}
      <div className="absolute inset-0 grid-pattern opacity-20" />
      
      <div className="relative z-10 max-w-7xl mx-auto px-6 sm:px-8 lg:px-12">
        <div className="py-24 sm:py-32 lg:py-40">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, ease: [0.4, 0, 0.2, 1] }}
            className="max-w-5xl"
          >
            {/* Badge */}
            {badge && (
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.1, duration: 0.5 }}
                className="mb-8 sm:mb-10"
              >
                {badge}
              </motion.div>
            )}

            {/* Headline */}
            <motion.h1
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.2, duration: 0.5 }}
              className="text-5xl sm:text-6xl md:text-7xl lg:text-8xl font-bold mb-6 sm:mb-8 text-black dark:text-white leading-[1.1] tracking-tight"
              style={{ letterSpacing: '-0.02em', fontWeight: 800 }}
            >
              {headline}
              {subheadline && (
                <span className="block mt-3 sm:mt-4 bg-gradient-to-r from-black via-gray-700 to-black dark:from-white dark:via-gray-300 dark:to-white bg-clip-text text-transparent">
                  {subheadline}
                </span>
              )}
            </motion.h1>

            {/* Description */}
            <motion.p
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3, duration: 0.5 }}
              className="text-lg sm:text-xl md:text-2xl text-gray-600 dark:text-gray-400 mb-8 sm:mb-12 leading-relaxed max-w-3xl font-normal"
            >
              {description}
            </motion.p>

            {/* CTAs */}
            {ctas && (
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.4, duration: 0.5 }}
                className="flex flex-col sm:flex-row gap-4 sm:gap-6 mb-12 sm:mb-16"
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
                className="flex flex-wrap items-center gap-6 sm:gap-8"
              >
                {trustIndicators.map((indicator, index) => (
                  <div
                    key={index}
                    className="flex items-center space-x-2 text-sm sm:text-base text-gray-600 dark:text-gray-400"
                  >
                    {indicator.icon}
                    <span className="font-medium">{indicator.text}</span>
                  </div>
                ))}
              </motion.div>
            )}
          </motion.div>
        </div>
      </div>
    </section>
  );
}
