import type { NextConfig } from "next";

// FRONTEND-CRIT-003: Security headers configuration
// NOTE: In production, consider using nonce-based CSP with middleware
const isDevelopment = process.env.NODE_ENV === 'development';

const securityHeaders = [
  {
    // Content Security Policy - restrict resource loading
    // Development: Allow eval for hot reload
    // Production: No eval, rely on strict-dynamic
    key: 'Content-Security-Policy',
    value: isDevelopment
      ? [
          "default-src 'self'",
          "script-src 'self' 'unsafe-eval' 'unsafe-inline'", // Dev only
          "style-src 'self' 'unsafe-inline'",
          "img-src 'self' data: https:",
          "font-src 'self' data:",
          "connect-src 'self' ws: wss: http://localhost:* https:",
          "frame-ancestors 'none'",
        ].join('; ')
      : [
          "default-src 'self'",
          // FRONTEND-CRIT-001 FIX: Removed 'unsafe-inline' from script-src
          // 'strict-dynamic' allows trusted scripts to load other scripts
          // Modern browsers ignore 'unsafe-inline' when 'strict-dynamic' is present
          "script-src 'self' 'strict-dynamic'",
          "style-src 'self' 'unsafe-inline'", // Required for styled-jsx/CSS-in-JS
          "img-src 'self' data: https:",
          "font-src 'self' data:",
          // FRONTEND-CRIT-002 FIX: Only allow wss:// (secure WebSocket)
          "connect-src 'self' wss: https: https://*.walletconnect.com https://*.infura.io",
          "frame-ancestors 'none'",
          "base-uri 'self'",
          "form-action 'self'",
          "upgrade-insecure-requests",
        ].join('; '),
  },
  {
    // Prevent clickjacking
    key: 'X-Frame-Options',
    value: 'DENY',
  },
  {
    // Block MIME type sniffing
    key: 'X-Content-Type-Options',
    value: 'nosniff',
  },
  {
    // Enable XSS filter
    key: 'X-XSS-Protection',
    value: '1; mode=block',
  },
  {
    // Control referrer information
    key: 'Referrer-Policy',
    value: 'strict-origin-when-cross-origin',
  },
  {
    // Enable HSTS (1 year, include subdomains, preload)
    key: 'Strict-Transport-Security',
    value: 'max-age=31536000; includeSubDomains; preload',
  },
  {
    // Restrict browser features
    key: 'Permissions-Policy',
    value: 'camera=(), microphone=(), geolocation=(), interest-cohort=()',
  },
];

const nextConfig: NextConfig = {
  // Apply security headers to all routes
  async headers() {
    return [
      {
        source: '/:path*',
        headers: securityHeaders,
      },
    ];
  },
  // Disable x-powered-by header
  poweredByHeader: false,
};

export default nextConfig;
