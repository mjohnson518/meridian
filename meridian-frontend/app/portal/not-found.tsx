import Link from 'next/link';

export default function PortalNotFound() {
  return (
    <div className="min-h-screen flex items-center justify-center px-6">
      <div className="text-center max-w-md">
        <p className="text-8xl font-heading font-bold text-gray-800 dark:text-gray-700 mb-4">
          404
        </p>
        <h2 className="text-xl font-heading font-bold text-gray-900 dark:text-white mb-2">
          Page not found
        </h2>
        <p className="text-sm text-gray-500 font-mono mb-8">
          The portal page you&apos;re looking for doesn&apos;t exist or has been moved.
        </p>
        <Link
          href="/portal/dashboard"
          className="inline-flex items-center gap-2 px-5 py-2.5 text-sm font-mono text-white bg-emerald-600 hover:bg-emerald-500 rounded-xl transition-colors"
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 19l-7-7m0 0l7-7m-7 7h18" />
          </svg>
          Back to Dashboard
        </Link>
      </div>
    </div>
  );
}
