// Authentication client for institutional portal

import { User, Session, LoginCredentials, RegisterData, UserRole, KYCStatus } from './types';

// API URL configuration - fails fast in production if not set
const getApiBaseUrl = (): string => {
  const url = process.env.NEXT_PUBLIC_API_URL;
  if (!url && typeof window !== 'undefined' && window.location.hostname !== 'localhost') {
    console.error('[Auth] NEXT_PUBLIC_API_URL not configured for production');
    throw new Error('API configuration error: NEXT_PUBLIC_API_URL must be set in production');
  }
  return url || 'http://localhost:8080/api/v1';
};

const API_BASE_URL = getApiBaseUrl();

// Session storage keys
// SECURITY NOTE: Backend sets httpOnly cookies for primary authentication
// These localStorage keys are for:
// - SESSION_KEY: User info display and session expiry tracking (no sensitive tokens)
// - TOKEN_KEY: WebSocket authentication only (WS can't use cookies)
const SESSION_KEY = 'meridian_session';
const TOKEN_KEY = 'meridian_ws_token'; // Renamed to clarify WebSocket-only use

export const authClient = {
  // Login
  async login(credentials: LoginCredentials): Promise<Session> {
    try {
      const response = await fetch(`${API_BASE_URL}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include', // SECURITY: Include cookies for httpOnly auth
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
      console.error('[Auth] Login error:', error instanceof Error ? error.message : 'Unknown error');
      throw error;
    }
  },

  // Register
  async register(data: RegisterData): Promise<Session> {
    try {
      const response = await fetch(`${API_BASE_URL}/auth/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include', // SECURITY: Include cookies for httpOnly auth
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
      console.error('[Auth] Registration error:', error instanceof Error ? error.message : 'Unknown error');
      throw error;
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
  // SECURITY: Only store non-sensitive data for UI display
  // Actual authentication uses httpOnly cookies set by backend
  saveSession(session: Session) {
    if (typeof window !== 'undefined') {
      // Strip sensitive tokens before storing
      const safeSession = {
        user: session.user,
        expiresAt: session.expiresAt,
        // Note: accessToken/refreshToken NOT stored - handled by httpOnly cookies
      };
      localStorage.setItem(SESSION_KEY, JSON.stringify(safeSession));
      // WebSocket-only token (required because WS can't use cookies)
      localStorage.setItem(TOKEN_KEY, session.accessToken);
    }
  },

  // Create mock session for development only
  // SECURITY: This function is strictly for local development
  createMockSession(email: string, role: UserRole = UserRole.TREASURY): Session {
    // Multi-layer production detection
    const isProduction = process.env.NODE_ENV === 'production';
    const isProductionDomain = typeof window !== 'undefined' &&
      window.location.hostname !== 'localhost' &&
      !window.location.hostname.startsWith('127.') &&
      !window.location.hostname.endsWith('.local');

    if (isProduction || isProductionDomain) {
      console.error('[Auth] SECURITY: Mock sessions blocked in production');
      throw new Error('Mock sessions are not available in production');
    }

    console.warn('[Auth] Creating mock session - FOR DEVELOPMENT ONLY');

    // Use environment variable for test wallet or clearly invalid address
    const testWallet = process.env.NEXT_PUBLIC_TEST_WALLET_ADDRESS || '0x0000000000000000000000000000000000000000';

    const mockUser: User = {
      id: 'mock-user-id',
      email,
      role,
      organization: 'Mock Corporation Ltd.',
      kycStatus: KYCStatus.APPROVED,
      walletAddress: testWallet,
      createdAt: new Date(),
      lastLoginAt: new Date(),
    };

    // Generate unique mock tokens to avoid static token attacks
    const mockId = Math.random().toString(36).substring(7);

    return {
      user: mockUser,
      accessToken: `dev-only-token-${mockId}`,
      refreshToken: `dev-only-refresh-${mockId}`,
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
      // SECURITY: Use cookies for auth, fallback to header for refresh token
      const response = await fetch(`${API_BASE_URL}/auth/refresh`, {
        method: 'POST',
        credentials: 'include', // Include httpOnly cookies
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
    } catch (error) {
      console.error('[Auth] Token refresh error:', error instanceof Error ? error.message : 'Unknown error');
      this.logout();
      return null;
    }
  },
};

export default authClient;

