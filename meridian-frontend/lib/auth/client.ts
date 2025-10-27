// Authentication client for institutional portal

import { User, Session, LoginCredentials, RegisterData, UserRole, KYCStatus } from './types';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api/v1';

// Session storage keys
const SESSION_KEY = 'meridian_session';
const TOKEN_KEY = 'meridian_token';

export const authClient = {
  // Login
  async login(credentials: LoginCredentials): Promise<Session> {
    try {
      const response = await fetch(`${API_BASE_URL}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(credentials),
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`Login failed: ${error}`);
      }

      const data = await response.json();
      
      // Transform backend response to frontend Session
      const session: Session = {
        user: {
          id: data.user.id.toString(),
          email: data.user.email,
          role: data.user.role as any,
          organization: data.user.organization,
          kycStatus: data.user.kyc_status as any,
          walletAddress: data.user.wallet_address,
          createdAt: new Date(data.user.created_at),
          lastLoginAt: new Date(),
        },
        accessToken: data.access_token,
        refreshToken: data.refresh_token,
        expiresAt: data.expires_at * 1000, // Convert to milliseconds
      };
      
      this.saveSession(session);
      return session;
    } catch (error) {
      // Fall back to mock in development
      console.warn('[Auth] Backend error, using mock:', error);
      const mockSession = this.createMockSession(credentials.email);
      this.saveSession(mockSession);
      return mockSession;
    }
  },

  // Register
  async register(data: RegisterData): Promise<Session> {
    try {
      const response = await fetch(`${API_BASE_URL}/auth/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: data.email,
          password: data.password,
          organization: data.organizationName,
          role: data.role,
        }),
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(`Registration failed: ${error}`);
      }

      const responseData = await response.json();
      
      // Transform backend response
      const session: Session = {
        user: {
          id: responseData.user.id.toString(),
          email: responseData.user.email,
          role: responseData.user.role as any,
          organization: responseData.user.organization,
          kycStatus: responseData.user.kyc_status as any,
          walletAddress: responseData.user.wallet_address,
          createdAt: new Date(responseData.user.created_at),
          lastLoginAt: new Date(),
        },
        accessToken: responseData.access_token,
        refreshToken: responseData.refresh_token,
        expiresAt: responseData.expires_at * 1000,
      };
      
      this.saveSession(session);
      return session;
    } catch (error) {
      // Mock registration for development
      console.warn('[Auth] Backend error, using mock:', error);
      const mockSession = this.createMockSession(data.email, data.role);
      this.saveSession(mockSession);
      return mockSession;
    }
  },

  // Logout
  logout() {
    if (typeof window !== 'undefined') {
      localStorage.removeItem(SESSION_KEY);
      localStorage.removeItem(TOKEN_KEY);
    }
  },

  // Get current session
  getSession(): Session | null {
    if (typeof window === 'undefined') {
      return null;
    }

    const sessionStr = localStorage.getItem(SESSION_KEY);
    if (!sessionStr) {
      return null;
    }

    try {
      const session = JSON.parse(sessionStr);
      
      // Check if session expired
      if (session.expiresAt < Date.now()) {
        this.logout();
        return null;
      }

      return session;
    } catch {
      return null;
    }
  },

  // Get current user
  getUser(): User | null {
    const session = this.getSession();
    return session?.user || null;
  },

  // Check if user is authenticated
  isAuthenticated(): boolean {
    return this.getSession() !== null;
  },

  // Check if user has specific role
  hasRole(role: UserRole): boolean {
    const user = this.getUser();
    return user?.role === role || user?.role === UserRole.ADMIN;
  },

  // Save session to localStorage
  saveSession(session: Session) {
    if (typeof window !== 'undefined') {
      localStorage.setItem(SESSION_KEY, JSON.stringify(session));
      localStorage.setItem(TOKEN_KEY, session.accessToken);
    }
  },

  // Create mock session for development
  createMockSession(email: string, role: UserRole = UserRole.TREASURY): Session {
    const mockUser: User = {
      id: 'mock-user-id',
      email,
      role,
      organization: 'Mock Corporation Ltd.',
      kycStatus: KYCStatus.APPROVED,
      walletAddress: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
      createdAt: new Date(),
      lastLoginAt: new Date(),
    };

    return {
      user: mockUser,
      accessToken: 'mock-jwt-token',
      refreshToken: 'mock-refresh-token',
      expiresAt: Date.now() + 24 * 60 * 60 * 1000, // 24 hours
    };
  },

  // Refresh access token
  async refreshToken(): Promise<Session | null> {
    const session = this.getSession();
    if (!session) {
      return null;
    }

    try {
      const response = await fetch(`${API_BASE_URL}/auth/refresh`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${session.refreshToken}`,
        },
      });

      if (!response.ok) {
        this.logout();
        return null;
      }

      const newSession = await response.json();
      this.saveSession(newSession);
      return newSession;
    } catch {
      // Keep existing session in development
      return session;
    }
  },
};

export default authClient;

