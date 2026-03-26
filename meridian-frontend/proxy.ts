import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

// Portal routes that require authentication
const PROTECTED_PREFIX = '/portal';

// Routes within /portal that are publicly accessible (no auth required)
const PUBLIC_PORTAL_ROUTES = ['/portal/login', '/portal/register'];

export function proxy(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // Only protect /portal routes
  if (!pathname.startsWith(PROTECTED_PREFIX)) {
    return NextResponse.next();
  }

  // Allow public portal routes (login, register)
  if (PUBLIC_PORTAL_ROUTES.some(route => pathname.startsWith(route))) {
    return NextResponse.next();
  }

  // Check for the httpOnly auth cookie set by the backend
  const authCookie = request.cookies.get('meridian_access_token');

  if (!authCookie) {
    const loginUrl = new URL('/portal/login', request.url);
    loginUrl.searchParams.set('redirect', pathname);
    return NextResponse.redirect(loginUrl);
  }

  return NextResponse.next();
}

export const config = {
  matcher: ['/portal/:path*'],
};
