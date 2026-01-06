'use client';

export function SkipToContent() {
  return (
    <a
      href="#main-content"
      className="
        sr-only focus:not-sr-only
        focus:fixed focus:top-4 focus:left-4 focus:z-[9999]
        focus:px-6 focus:py-3
        focus:bg-emerald-500 focus:text-white
        focus:rounded-lg focus:font-medium
        focus:outline-none focus:ring-2 focus:ring-emerald-300 focus:ring-offset-2 focus:ring-offset-[#0A0A0B]
        focus:shadow-lg
        transition-all duration-200
      "
    >
      Skip to main content
    </a>
  );
}
