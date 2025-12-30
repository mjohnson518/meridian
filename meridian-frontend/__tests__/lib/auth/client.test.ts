import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { authClient } from '@/lib/auth/client';
import { UserRole, KYCStatus, Session, User } from '@/lib/auth/types';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

describe('authClient', () => {
  const mockUser: User = {
    id: 'test-user-id',
    email: 'test@example.com',
    role: UserRole.TREASURY,
    organization: 'Test Corp',
    kycStatus: KYCStatus.APPROVED,
    walletAddress: '0x1234567890abcdef1234567890abcdef12345678',
    createdAt: new Date('2024-01-01'),
    lastLoginAt: new Date('2024-06-01'),
  };

  const mockSession: Session = {
    user: mockUser,
    accessToken: 'mock-access-token',
    refreshToken: 'mock-refresh-token',
    expiresAt: Date.now() + 3600000, // 1 hour from now
  };

  const expiredSession: Session = {
    ...mockSession,
    expiresAt: Date.now() - 3600000, // 1 hour ago
  };

  beforeEach(() => {
    localStorageMock.clear();
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('saveSession', () => {
    it('saves session to localStorage without sensitive tokens', () => {
      authClient.saveSession(mockSession);

      // SECURITY: Only user info and expiry stored, NOT access tokens
      const expectedSafeSession = {
        user: mockSession.user,
        expiresAt: mockSession.expiresAt,
      };
      expect(localStorageMock.setItem).toHaveBeenCalledWith(
        'meridian_session',
        JSON.stringify(expectedSafeSession)
      );
      // accessToken should NOT be stored (XSS prevention)
      expect(localStorageMock.setItem).not.toHaveBeenCalledWith(
        'meridian_ws_token',
        expect.any(String)
      );
    });

    it('stores wsToken if provided', () => {
      const sessionWithWsToken = {
        ...mockSession,
        wsToken: 'ws-auth-token',
      };
      authClient.saveSession(sessionWithWsToken);

      expect(localStorageMock.setItem).toHaveBeenCalledWith(
        'meridian_ws_token',
        'ws-auth-token'
      );
    });
  });

  describe('getSession', () => {
    it('returns session from localStorage', () => {
      localStorageMock.getItem.mockImplementation((key: string) => {
        if (key === 'meridian_session') {
          return JSON.stringify(mockSession);
        }
        return null;
      });

      const session = authClient.getSession();
      expect(session).not.toBeNull();
      expect(session?.user.email).toBe(mockUser.email);
    });

    it('returns null when no session exists', () => {
      localStorageMock.getItem.mockReturnValue(null);

      const session = authClient.getSession();
      expect(session).toBeNull();
    });

    it('returns null and clears session when expired', () => {
      localStorageMock.getItem.mockImplementation((key: string) => {
        if (key === 'meridian_session') {
          return JSON.stringify(expiredSession);
        }
        return null;
      });

      const session = authClient.getSession();
      expect(session).toBeNull();
      expect(localStorageMock.removeItem).toHaveBeenCalled();
    });

    it('returns null on invalid JSON', () => {
      localStorageMock.getItem.mockReturnValue('invalid-json');

      const session = authClient.getSession();
      expect(session).toBeNull();
    });
  });

  describe('getUser', () => {
    it('returns user from session', () => {
      localStorageMock.getItem.mockImplementation((key: string) => {
        if (key === 'meridian_session') {
          return JSON.stringify(mockSession);
        }
        return null;
      });

      const user = authClient.getUser();
      expect(user).not.toBeNull();
      expect(user?.email).toBe(mockUser.email);
    });

    it('returns null when no session', () => {
      localStorageMock.getItem.mockReturnValue(null);

      const user = authClient.getUser();
      expect(user).toBeNull();
    });
  });

  describe('isAuthenticated', () => {
    it('returns true when valid session exists', () => {
      localStorageMock.getItem.mockImplementation((key: string) => {
        if (key === 'meridian_session') {
          return JSON.stringify(mockSession);
        }
        return null;
      });

      expect(authClient.isAuthenticated()).toBe(true);
    });

    it('returns false when no session exists', () => {
      localStorageMock.getItem.mockReturnValue(null);

      expect(authClient.isAuthenticated()).toBe(false);
    });

    it('returns false when session is expired', () => {
      localStorageMock.getItem.mockImplementation((key: string) => {
        if (key === 'meridian_session') {
          return JSON.stringify(expiredSession);
        }
        return null;
      });

      expect(authClient.isAuthenticated()).toBe(false);
    });
  });

  describe('hasRole', () => {
    it('returns true when user has the specified role', () => {
      localStorageMock.getItem.mockImplementation((key: string) => {
        if (key === 'meridian_session') {
          return JSON.stringify(mockSession);
        }
        return null;
      });

      expect(authClient.hasRole(UserRole.TREASURY)).toBe(true);
    });

    it('returns false when user has different role', () => {
      localStorageMock.getItem.mockImplementation((key: string) => {
        if (key === 'meridian_session') {
          return JSON.stringify(mockSession);
        }
        return null;
      });

      expect(authClient.hasRole(UserRole.COMPLIANCE)).toBe(false);
    });

    it('returns true for any role when user is ADMIN', () => {
      const adminSession: Session = {
        ...mockSession,
        user: { ...mockUser, role: UserRole.ADMIN },
      };

      localStorageMock.getItem.mockImplementation((key: string) => {
        if (key === 'meridian_session') {
          return JSON.stringify(adminSession);
        }
        return null;
      });

      expect(authClient.hasRole(UserRole.TREASURY)).toBe(true);
      expect(authClient.hasRole(UserRole.COMPLIANCE)).toBe(true);
      expect(authClient.hasRole(UserRole.VIEWER)).toBe(true);
    });

    it('returns false when no user is logged in', () => {
      localStorageMock.getItem.mockReturnValue(null);

      expect(authClient.hasRole(UserRole.TREASURY)).toBe(false);
    });
  });

  describe('logout', () => {
    it('removes session and ws token from localStorage', () => {
      authClient.logout();

      expect(localStorageMock.removeItem).toHaveBeenCalledWith('meridian_session');
      expect(localStorageMock.removeItem).toHaveBeenCalledWith('meridian_ws_token');
    });
  });

  describe('createMockSession', () => {
    it('creates a mock session with provided email and unique tokens', () => {
      // Set NODE_ENV to development for this test
      const originalEnv = process.env.NODE_ENV;
      process.env.NODE_ENV = 'development';

      const session = authClient.createMockSession('test@example.com');

      expect(session.user.email).toBe('test@example.com');
      expect(session.user.role).toBe(UserRole.TREASURY);
      // Tokens are now unique per session (security improvement)
      expect(session.accessToken).toMatch(/^dev-only-token-/);
      expect(session.refreshToken).toMatch(/^dev-only-refresh-/);
      expect(session.expiresAt).toBeGreaterThan(Date.now());

      process.env.NODE_ENV = originalEnv;
    });

    it('generates unique tokens for each session', () => {
      const originalEnv = process.env.NODE_ENV;
      process.env.NODE_ENV = 'development';

      const session1 = authClient.createMockSession('user1@example.com');
      const session2 = authClient.createMockSession('user2@example.com');

      expect(session1.accessToken).not.toBe(session2.accessToken);
      expect(session1.refreshToken).not.toBe(session2.refreshToken);

      process.env.NODE_ENV = originalEnv;
    });

    it('creates mock session with specified role', () => {
      const originalEnv = process.env.NODE_ENV;
      process.env.NODE_ENV = 'development';

      const session = authClient.createMockSession('test@example.com', UserRole.ADMIN);

      expect(session.user.role).toBe(UserRole.ADMIN);

      process.env.NODE_ENV = originalEnv;
    });

    it('throws error in production environment', () => {
      const originalEnv = process.env.NODE_ENV;
      process.env.NODE_ENV = 'production';

      expect(() => authClient.createMockSession('test@example.com')).toThrow(
        'Mock sessions are not available in production'
      );

      process.env.NODE_ENV = originalEnv;
    });
  });
});
