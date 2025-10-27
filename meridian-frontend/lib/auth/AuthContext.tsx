'use client';

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import { User, Session, LoginCredentials, RegisterData, UserRole } from './types';
import authClient from './client';

interface AuthContextValue {
  user: User | null;
  session: Session | null;
  loading: boolean;
  login: (credentials: LoginCredentials) => Promise<void>;
  register: (data: RegisterData) => Promise<void>;
  logout: () => void;
  hasRole: (role: UserRole) => boolean;
  isAuthenticated: boolean;
}

const AuthContext = createContext<AuthContextValue | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [session, setSession] = useState<Session | null>(null);
  const [loading, setLoading] = useState(true);

  // Initialize session from localStorage
  useEffect(() => {
    const savedSession = authClient.getSession();
    setSession(savedSession);
    setLoading(false);
  }, []);

  // Login handler
  const login = useCallback(async (credentials: LoginCredentials) => {
    setLoading(true);
    try {
      const newSession = await authClient.login(credentials);
      setSession(newSession);
    } finally {
      setLoading(false);
    }
  }, []);

  // Register handler
  const register = useCallback(async (data: RegisterData) => {
    setLoading(true);
    try {
      const newSession = await authClient.register(data);
      setSession(newSession);
    } finally {
      setLoading(false);
    }
  }, []);

  // Logout handler
  const logout = useCallback(() => {
    authClient.logout();
    setSession(null);
  }, []);

  // Role check
  const hasRole = useCallback((role: UserRole): boolean => {
    if (!session?.user) return false;
    return session.user.role === role || session.user.role === UserRole.ADMIN;
  }, [session]);

  const value: AuthContextValue = {
    user: session?.user || null,
    session,
    loading,
    login,
    register,
    logout,
    hasRole,
    isAuthenticated: !!session,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
}

// Protected route component
export function ProtectedRoute({ 
  children, 
  requiredRole 
}: { 
  children: React.ReactNode; 
  requiredRole?: UserRole;
}) {
  const { isAuthenticated, hasRole, loading } = useAuth();

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-sacred-black"></div>
          <p className="mt-4 font-mono text-sm text-sacred-gray-600">
            Authenticating...
          </p>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    // Redirect to login
    if (typeof window !== 'undefined') {
      window.location.href = '/portal/login';
    }
    return null;
  }

  if (requiredRole && !hasRole(requiredRole)) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-2xl font-mono mb-4">Access Denied</h1>
          <p className="text-sacred-gray-600">
            You don't have permission to access this page.
          </p>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}

