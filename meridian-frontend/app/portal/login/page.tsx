'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { motion } from 'framer-motion';
import { useAuth } from '@/lib/auth/AuthContext';
import { PortalCard } from '@/components/portal/PortalCard';
import { PortalButton } from '@/components/portal/PortalButton';
import { cn } from '@/lib/utils';

const loginSchema = z.object({
  email: z
    .string()
    .min(1, 'Email is required')
    .email('Please enter a valid email address'),
  password: z
    .string()
    .min(1, 'Password is required')
    .min(6, 'Password must be at least 6 characters'),
});

type LoginFormData = z.infer<typeof loginSchema>;

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0.1, delayChildren: 0.2 },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  visible: { opacity: 1, y: 0 },
};

const inputClasses = cn(
  "w-full px-4 py-3 rounded-xl font-mono text-sm",
  // Light mode
  "bg-gray-50 border-gray-300",
  // Dark mode - solid dark background for visibility
  "dark:bg-gray-800/90 dark:border-gray-600",
  "border",
  // Text colors
  "text-gray-900 dark:text-white",
  "placeholder-gray-500 dark:placeholder-gray-400",
  // Focus states
  "focus:outline-none focus:border-emerald-500 focus:ring-2 focus:ring-emerald-500/30",
  "dark:focus:border-emerald-500/60 dark:focus:ring-emerald-500/20",
  "transition-all duration-200"
);

const inputErrorClasses = cn(
  "w-full px-4 py-3 rounded-xl font-mono text-sm",
  // Light mode
  "bg-red-50 border-red-400",
  // Dark mode
  "dark:bg-red-500/15 dark:border-red-500/50",
  "border",
  // Text colors
  "text-gray-900 dark:text-white",
  "placeholder-gray-500 dark:placeholder-gray-400",
  // Focus states
  "focus:outline-none focus:border-red-500 focus:ring-2 focus:ring-red-500/30",
  "dark:focus:border-red-500/60 dark:focus:ring-red-500/20",
  "transition-all duration-200"
);

export default function LoginPage() {
  const router = useRouter();
  const { login, isAuthenticated } = useAuth();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginFormData>({
    resolver: zodResolver(loginSchema),
  });

  // Redirect if already authenticated
  useEffect(() => {
    if (isAuthenticated) {
      router.push('/portal/dashboard');
    }
  }, [isAuthenticated, router]);

  const onSubmit = async (data: LoginFormData) => {
    setError('');
    setLoading(true);

    try {
      await login({ email: data.email, password: data.password });
      router.push('/portal/dashboard');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-950 relative overflow-hidden transition-colors duration-300">
      {/* Background Grid - Dark mode */}
      <div
        className="fixed inset-0 pointer-events-none opacity-30 hidden dark:block"
        style={{
          backgroundImage: `linear-gradient(rgba(255,255,255,0.02) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.02) 1px, transparent 1px)`,
          backgroundSize: '24px 24px',
          maskImage: 'radial-gradient(ellipse at center, black 0%, transparent 70%)',
        }}
      />
      {/* Background Grid - Light mode */}
      <div
        className="fixed inset-0 pointer-events-none opacity-50 dark:hidden"
        style={{
          backgroundImage: `linear-gradient(rgba(0,0,0,0.02) 1px, transparent 1px), linear-gradient(90deg, rgba(0,0,0,0.02) 1px, transparent 1px)`,
          backgroundSize: '24px 24px',
          maskImage: 'radial-gradient(ellipse at center, black 0%, transparent 70%)',
        }}
      />

      {/* Decorative glow */}
      <div className="absolute top-1/4 left-1/2 -translate-x-1/2 w-[600px] h-[600px] bg-emerald-500/5 dark:bg-emerald-500/10 rounded-full blur-[120px] pointer-events-none" />

      <motion.div
        className="w-full max-w-md px-6 relative z-10"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        {/* Logo & Header */}
        <motion.div variants={itemVariants} className="text-center mb-8">
          <h1 className="text-4xl font-heading font-bold mb-3">
            <span className="bg-gradient-to-r from-emerald-400 via-teal-400 to-cyan-400 bg-clip-text text-transparent">
              MERIDIAN
            </span>
          </h1>
          <p className="text-xs text-gray-500 dark:text-gray-500 font-mono uppercase tracking-[0.3em]">
            Institutional Portal
          </p>
        </motion.div>

        {/* Login Card */}
        <motion.div variants={itemVariants}>
          <PortalCard hoverEffect={false} padding="lg">
            <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
              {/* Email Field */}
              <div>
                <label
                  htmlFor="email"
                  className="text-xs font-mono uppercase tracking-wider text-gray-600 dark:text-gray-400 block mb-2"
                >
                  Email Address
                </label>
                <input
                  id="email"
                  type="email"
                  {...register('email')}
                  className={errors.email ? inputErrorClasses : inputClasses}
                  placeholder="treasury@company.com"
                  autoComplete="email"
                  aria-invalid={!!errors.email}
                  aria-describedby={errors.email ? 'email-error' : undefined}
                />
                {errors.email && (
                  <p
                    id="email-error"
                    className="mt-2 text-xs font-mono text-red-400"
                    role="alert"
                  >
                    {errors.email.message}
                  </p>
                )}
              </div>

              {/* Password Field */}
              <div>
                <label
                  htmlFor="password"
                  className="text-xs font-mono uppercase tracking-wider text-gray-600 dark:text-gray-400 block mb-2"
                >
                  Password
                </label>
                <input
                  id="password"
                  type="password"
                  {...register('password')}
                  className={errors.password ? inputErrorClasses : inputClasses}
                  placeholder="••••••••"
                  autoComplete="current-password"
                  aria-invalid={!!errors.password}
                  aria-describedby={errors.password ? 'password-error' : undefined}
                />
                {errors.password && (
                  <p
                    id="password-error"
                    className="mt-2 text-xs font-mono text-red-400"
                    role="alert"
                  >
                    {errors.password.message}
                  </p>
                )}
              </div>

              {/* Error Alert */}
              {error && (
                <motion.div
                  initial={{ opacity: 0, y: -10 }}
                  animate={{ opacity: 1, y: 0 }}
                  className="p-4 rounded-xl bg-red-50 dark:bg-red-500/10 border border-red-200 dark:border-red-500/30"
                  role="alert"
                >
                  <p className="text-xs font-mono text-red-600 dark:text-red-400">{error}</p>
                </motion.div>
              )}

              {/* Submit Button */}
              <PortalButton
                type="submit"
                variant="primary"
                fullWidth
                loading={loading}
                disabled={loading}
                className="py-4"
              >
                {loading ? 'Authenticating...' : 'Sign In'}
                {!loading && (
                  <svg className="w-4 h-4 ml-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
                  </svg>
                )}
              </PortalButton>
            </form>

            {/* Register Link */}
            <div className="mt-6 pt-6 border-t border-gray-200 dark:border-gray-700/50">
              <p className="text-xs text-gray-500 dark:text-gray-500 text-center font-mono">
                Don't have an account?{' '}
                <a
                  href="/portal/register"
                  className="text-emerald-600 dark:text-emerald-400 hover:text-emerald-500 dark:hover:text-emerald-300 transition-colors"
                >
                  Register
                </a>
              </p>
            </div>
          </PortalCard>
        </motion.div>

        {/* Development info */}
        {process.env.NODE_ENV !== 'production' && typeof window !== 'undefined' && window.location.hostname === 'localhost' && (
          <motion.div
            variants={itemVariants}
            className="mt-6 p-4 rounded-xl bg-amber-500/10 border border-amber-500/30"
          >
            <p className="text-xs font-mono text-amber-400 mb-2 flex items-center gap-2">
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              Development Environment
            </p>
            <p className="text-xs text-amber-400/70 mb-2">
              Real backend authentication is active. Register a user or use configured test credentials.
            </p>
            <p className="text-xs text-amber-400/50">
              Backend: <code className="font-mono text-amber-300/70">{process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api/v1'}</code>
            </p>
          </motion.div>
        )}

        {/* Back to Home */}
        <motion.div variants={itemVariants} className="mt-6 text-center">
          <a
            href="/"
            className="text-xs font-mono text-gray-500 dark:text-gray-600 hover:text-gray-700 dark:hover:text-gray-400 transition-colors"
          >
            ← Back to Home
          </a>
        </motion.div>
      </motion.div>
    </div>
  );
}
