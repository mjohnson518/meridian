'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { useAuth } from '@/lib/auth/AuthContext';
import { SacredCard } from '@/components/sacred/Card';
import { SacredButton } from '@/components/sacred/Button';
import { Heading } from '@/components/sacred/Typography';

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

  // Redirect if already authenticated (use useEffect to avoid render phase setState)
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
    <div className="min-h-screen flex items-center justify-center bg-sacred-gray-100">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <Heading level={1} className="text-3xl mb-2" mono>
            MERIDIAN
          </Heading>
          <p className="text-sm text-sacred-gray-600 font-mono uppercase tracking-wider">
            Institutional Portal
          </p>
        </div>

        <SacredCard>
          <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
            <div>
              <label
                htmlFor="email"
                className="text-xs font-mono uppercase tracking-wider text-sacred-gray-600 block mb-2"
              >
                Email Address
              </label>
              <input
                id="email"
                type="email"
                {...register('email')}
                className={`w-full px-4 py-2 border rounded font-mono text-sm focus:outline-none focus:ring-2 transition-colors ${
                  errors.email
                    ? 'border-red-500 focus:ring-red-300'
                    : 'border-sacred-gray-200 focus:ring-sacred-gray-400'
                }`}
                placeholder="treasury@company.com"
                autoComplete="email"
                aria-invalid={!!errors.email}
                aria-describedby={errors.email ? 'email-error' : undefined}
              />
              {errors.email && (
                <p
                  id="email-error"
                  className="mt-1.5 text-xs font-mono text-red-600"
                  role="alert"
                >
                  {errors.email.message}
                </p>
              )}
            </div>

            <div>
              <label
                htmlFor="password"
                className="text-xs font-mono uppercase tracking-wider text-sacred-gray-600 block mb-2"
              >
                Password
              </label>
              <input
                id="password"
                type="password"
                {...register('password')}
                className={`w-full px-4 py-2 border rounded font-mono text-sm focus:outline-none focus:ring-2 transition-colors ${
                  errors.password
                    ? 'border-red-500 focus:ring-red-300'
                    : 'border-sacred-gray-200 focus:ring-sacred-gray-400'
                }`}
                placeholder="••••••••"
                autoComplete="current-password"
                aria-invalid={!!errors.password}
                aria-describedby={errors.password ? 'password-error' : undefined}
              />
              {errors.password && (
                <p
                  id="password-error"
                  className="mt-1.5 text-xs font-mono text-red-600"
                  role="alert"
                >
                  {errors.password.message}
                </p>
              )}
            </div>

            {error && (
              <div className="p-3 bg-red-50 border border-red-200 rounded" role="alert">
                <p className="text-xs font-mono text-red-600">{error}</p>
              </div>
            )}

            <SacredButton
              type="submit"
              variant="primary"
              fullWidth
              loading={loading}
              disabled={loading}
            >
              {loading ? 'Authenticating...' : 'Sign In →'}
            </SacredButton>
          </form>

          <div className="mt-6 pt-6 border-t border-sacred-gray-200">
            <p className="text-xs text-sacred-gray-600 text-center font-mono">
              Don't have an account?{' '}
              <a
                href="/portal/register"
                className="text-sacred-black underline hover:opacity-70"
              >
                Register
              </a>
            </p>
          </div>
        </SacredCard>

        {/* Development info - only shown in non-production with localhost */}
        {process.env.NODE_ENV !== 'production' && typeof window !== 'undefined' && window.location.hostname === 'localhost' && (
          <div className="mt-6 p-4 bg-amber-50 border border-amber-200 rounded">
            <p className="text-xs font-mono text-amber-800 mb-2">
              <strong>⚠️ Development Environment</strong>
            </p>
            <p className="text-xs text-amber-700 mb-2">
              Real backend authentication is active. Register a user or use configured test credentials.
            </p>
            <p className="text-xs text-amber-600">
              Backend: <code className="font-mono">{process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api/v1'}</code>
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
