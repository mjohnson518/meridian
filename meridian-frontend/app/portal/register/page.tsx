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
import { UserRole } from '@/lib/auth/types';
import { cn } from '@/lib/utils';

const registerSchema = z.object({
  email: z.string().min(1, 'Email is required').email('Enter a valid email'),
  password: z.string().min(8, 'Password must be at least 8 characters'),
  confirmPassword: z.string().min(1, 'Please confirm your password'),
  organizationName: z.string().min(2, 'Organization name is required'),
  role: z.nativeEnum(UserRole),
}).refine(d => d.password === d.confirmPassword, {
  message: 'Passwords do not match',
  path: ['confirmPassword'],
});

type RegisterFormData = z.infer<typeof registerSchema>;

const containerVariants = {
  hidden: { opacity: 0 },
  visible: { opacity: 1, transition: { staggerChildren: 0.1, delayChildren: 0.2 } },
};

const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  visible: { opacity: 1, y: 0 },
};

const inputClasses = cn(
  'w-full px-4 py-3 rounded-xl font-mono text-sm',
  'bg-gray-50 border-gray-300 dark:bg-gray-800/90 dark:border-gray-600',
  'border text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400',
  'focus:outline-none focus:border-emerald-500 focus:ring-2 focus:ring-emerald-500/30',
  'dark:focus:border-emerald-500/60 dark:focus:ring-emerald-500/20',
  'transition-all duration-200'
);

const inputErrorClasses = cn(
  'w-full px-4 py-3 rounded-xl font-mono text-sm',
  'bg-red-50 border-red-400 dark:bg-red-500/15 dark:border-red-500/50',
  'border text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400',
  'focus:outline-none focus:border-red-500 focus:ring-2 focus:ring-red-500/30',
  'transition-all duration-200'
);

export default function RegisterPage() {
  const router = useRouter();
  const { register: registerUser, isAuthenticated } = useAuth();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<RegisterFormData>({
    resolver: zodResolver(registerSchema),
    defaultValues: { role: UserRole.TREASURY },
  });

  useEffect(() => {
    if (isAuthenticated) {
      router.push('/portal/dashboard');
    }
  }, [isAuthenticated, router]);

  const onSubmit = async (data: RegisterFormData) => {
    setError('');
    setLoading(true);
    try {
      await registerUser({
        email: data.email,
        password: data.password,
        organizationName: data.organizationName,
        role: data.role,
      });
      router.push('/portal/dashboard');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Registration failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-950 relative overflow-hidden transition-colors duration-300">
      <div
        className="fixed inset-0 pointer-events-none opacity-30 hidden dark:block"
        style={{
          backgroundImage: `linear-gradient(rgba(255,255,255,0.02) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.02) 1px, transparent 1px)`,
          backgroundSize: '24px 24px',
          maskImage: 'radial-gradient(ellipse at center, black 0%, transparent 70%)',
        }}
      />
      <div className="absolute top-1/4 left-1/2 -translate-x-1/2 w-[600px] h-[600px] bg-emerald-500/5 dark:bg-emerald-500/10 rounded-full blur-[120px] pointer-events-none" />

      <motion.div
        className="w-full max-w-md px-6 relative z-10 py-12"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        <motion.div variants={itemVariants} className="text-center mb-8">
          <h1 className="text-4xl font-heading font-bold mb-3">
            <span className="bg-gradient-to-r from-emerald-400 via-teal-400 to-cyan-400 bg-clip-text text-transparent">
              MERIDIAN
            </span>
          </h1>
          <p className="text-xs text-gray-500 font-mono uppercase tracking-[0.3em]">
            Create Institutional Account
          </p>
        </motion.div>

        <motion.div variants={itemVariants}>
          <PortalCard hoverEffect={false} padding="lg">
            <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
              {error && (
                <div className="p-3 rounded-xl bg-red-500/10 border border-red-500/30 text-red-400 font-mono text-xs">
                  {error}
                </div>
              )}

              <div>
                <label className="text-xs font-mono uppercase tracking-wider text-gray-600 dark:text-gray-400 block mb-2">
                  Organization Name
                </label>
                <input
                  type="text"
                  {...register('organizationName')}
                  className={errors.organizationName ? inputErrorClasses : inputClasses}
                  placeholder="Acme Capital Ltd"
                  autoComplete="organization"
                />
                {errors.organizationName && (
                  <p className="text-xs text-red-400 font-mono mt-1">{errors.organizationName.message}</p>
                )}
              </div>

              <div>
                <label className="text-xs font-mono uppercase tracking-wider text-gray-600 dark:text-gray-400 block mb-2">
                  Email Address
                </label>
                <input
                  type="email"
                  {...register('email')}
                  className={errors.email ? inputErrorClasses : inputClasses}
                  placeholder="treasury@company.com"
                  autoComplete="email"
                />
                {errors.email && (
                  <p className="text-xs text-red-400 font-mono mt-1">{errors.email.message}</p>
                )}
              </div>

              <div>
                <label className="text-xs font-mono uppercase tracking-wider text-gray-600 dark:text-gray-400 block mb-2">
                  Role
                </label>
                <select
                  {...register('role')}
                  className={cn(inputClasses, 'cursor-pointer')}
                >
                  <option value={UserRole.TREASURY}>Treasury</option>
                  <option value={UserRole.COMPLIANCE}>Compliance</option>
                  <option value={UserRole.VIEWER}>Viewer</option>
                  <option value={UserRole.ADMIN}>Admin</option>
                </select>
              </div>

              <div>
                <label className="text-xs font-mono uppercase tracking-wider text-gray-600 dark:text-gray-400 block mb-2">
                  Password
                </label>
                <input
                  type="password"
                  {...register('password')}
                  className={errors.password ? inputErrorClasses : inputClasses}
                  placeholder="Minimum 8 characters"
                  autoComplete="new-password"
                />
                {errors.password && (
                  <p className="text-xs text-red-400 font-mono mt-1">{errors.password.message}</p>
                )}
              </div>

              <div>
                <label className="text-xs font-mono uppercase tracking-wider text-gray-600 dark:text-gray-400 block mb-2">
                  Confirm Password
                </label>
                <input
                  type="password"
                  {...register('confirmPassword')}
                  className={errors.confirmPassword ? inputErrorClasses : inputClasses}
                  placeholder="Re-enter password"
                  autoComplete="new-password"
                />
                {errors.confirmPassword && (
                  <p className="text-xs text-red-400 font-mono mt-1">{errors.confirmPassword.message}</p>
                )}
              </div>

              <PortalButton
                type="submit"
                variant="primary"
                fullWidth
                loading={loading}
                className="mt-2"
              >
                {loading ? 'Creating account...' : 'Create Account'}
              </PortalButton>
            </form>
          </PortalCard>
        </motion.div>

        <motion.p variants={itemVariants} className="text-center text-xs text-gray-500 font-mono mt-6">
          Already have an account?{' '}
          <a href="/portal/login" className="text-emerald-400 hover:text-emerald-300 transition-colors">
            Sign in
          </a>
        </motion.p>
      </motion.div>
    </div>
  );
}
