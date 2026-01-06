'use client';

import { Skeleton, SacredMetricSkeleton, SacredCardSkeleton } from '@/components/ui/Skeleton';

export function DashboardSkeleton() {
  return (
    <div className="min-h-screen bg-sacred-gray-100">
      {/* Header Skeleton */}
      <header className="bg-sacred-white border-b border-sacred-gray-200">
        <div className="sacred-container px-6">
          <nav className="flex items-center justify-between h-16">
            <div className="flex items-center space-x-8">
              <Skeleton variant="text" className="h-6 w-24 bg-sacred-gray-200" />
              <div className="hidden md:flex items-center space-x-6">
                <Skeleton variant="text" className="h-4 w-20 bg-sacred-gray-200" />
                <Skeleton variant="text" className="h-4 w-20 bg-sacred-gray-200" />
                <Skeleton variant="text" className="h-4 w-24 bg-sacred-gray-200" />
                <Skeleton variant="text" className="h-4 w-20 bg-sacred-gray-200" />
              </div>
            </div>
            <div className="flex items-center space-x-4">
              <div className="text-right space-y-1">
                <Skeleton variant="text" className="h-3 w-16 bg-sacred-gray-200" />
                <Skeleton variant="text" className="h-4 w-24 bg-sacred-gray-200" />
              </div>
              <Skeleton variant="rounded" className="h-8 w-20 bg-sacred-gray-200" />
            </div>
          </nav>
        </div>
      </header>

      {/* Main Content Skeleton */}
      <div className="sacred-container px-6 py-8">
        {/* Welcome Section */}
        <div className="mb-8">
          <Skeleton variant="text" className="h-9 w-80 mb-2 bg-sacred-gray-200" />
          <Skeleton variant="text" className="h-5 w-64 bg-sacred-gray-200" />
        </div>

        {/* Key Metrics Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="p-6 bg-sacred-white border border-sacred-gray-200 rounded">
              <SacredMetricSkeleton />
            </div>
          ))}
        </div>

        {/* Quick Actions & Account Info */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
          {/* Quick Actions Card */}
          <div className="p-6 bg-sacred-white border border-sacred-gray-200 rounded">
            <Skeleton variant="text" className="h-5 w-32 mb-4 bg-sacred-gray-200" />
            <div className="space-y-3">
              <Skeleton variant="rounded" className="h-10 w-full bg-sacred-gray-200" />
              <Skeleton variant="rounded" className="h-10 w-full bg-sacred-gray-200" />
              <Skeleton variant="rounded" className="h-10 w-full bg-sacred-gray-200" />
            </div>
          </div>

          {/* Account Info Card */}
          <div className="p-6 bg-sacred-white border border-sacred-gray-200 rounded">
            <Skeleton variant="text" className="h-5 w-44 mb-4 bg-sacred-gray-200" />
            <div className="space-y-4">
              {[1, 2, 3, 4].map((i) => (
                <div key={i}>
                  <Skeleton variant="text" className="h-3 w-20 mb-1 bg-sacred-gray-200" />
                  <Skeleton variant="text" className="h-4 w-32 bg-sacred-gray-200" />
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Recent Activity Card */}
        <div className="p-6 bg-sacred-white border border-sacred-gray-200 rounded">
          <Skeleton variant="text" className="h-5 w-36 mb-4 bg-sacred-gray-200" />
          <div className="space-y-4">
            {[1, 2, 3].map((i) => (
              <div key={i} className="flex items-center space-x-4 py-3 border-b border-sacred-gray-100 last:border-0">
                <Skeleton variant="circular" className="h-10 w-10 bg-sacred-gray-200" />
                <div className="flex-1">
                  <Skeleton variant="text" className="h-4 w-48 mb-1 bg-sacred-gray-200" />
                  <Skeleton variant="text" className="h-3 w-32 bg-sacred-gray-200" />
                </div>
                <Skeleton variant="text" className="h-4 w-20 bg-sacred-gray-200" />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export function MintPageSkeleton() {
  return (
    <div className="min-h-screen bg-sacred-gray-100">
      {/* Header - same as dashboard */}
      <header className="bg-sacred-white border-b border-sacred-gray-200">
        <div className="sacred-container px-6">
          <nav className="flex items-center justify-between h-16">
            <div className="flex items-center space-x-8">
              <Skeleton variant="text" className="h-6 w-24 bg-sacred-gray-200" />
              <div className="hidden md:flex items-center space-x-6">
                {[1, 2, 3, 4].map((i) => (
                  <Skeleton key={i} variant="text" className="h-4 w-20 bg-sacred-gray-200" />
                ))}
              </div>
            </div>
            <Skeleton variant="rounded" className="h-8 w-20 bg-sacred-gray-200" />
          </nav>
        </div>
      </header>

      <div className="sacred-container px-6 py-8">
        {/* Title */}
        <div className="mb-8">
          <Skeleton variant="text" className="h-9 w-48 mb-2 bg-sacred-gray-200" />
          <Skeleton variant="text" className="h-5 w-96 bg-sacred-gray-200" />
        </div>

        {/* Main Form Card */}
        <div className="max-w-2xl">
          <div className="p-6 bg-sacred-white border border-sacred-gray-200 rounded">
            <div className="space-y-6">
              {/* Currency Select */}
              <div>
                <Skeleton variant="text" className="h-3 w-20 mb-2 bg-sacred-gray-200" />
                <Skeleton variant="rounded" className="h-10 w-full bg-sacred-gray-200" />
              </div>

              {/* Amount Input */}
              <div>
                <Skeleton variant="text" className="h-3 w-16 mb-2 bg-sacred-gray-200" />
                <Skeleton variant="rounded" className="h-10 w-full bg-sacred-gray-200" />
              </div>

              {/* Fee Display */}
              <div className="p-4 bg-sacred-gray-50 rounded">
                <div className="flex justify-between mb-2">
                  <Skeleton variant="text" className="h-4 w-24 bg-sacred-gray-200" />
                  <Skeleton variant="text" className="h-4 w-16 bg-sacred-gray-200" />
                </div>
                <div className="flex justify-between">
                  <Skeleton variant="text" className="h-4 w-28 bg-sacred-gray-200" />
                  <Skeleton variant="text" className="h-4 w-20 bg-sacred-gray-200" />
                </div>
              </div>

              {/* Submit Button */}
              <Skeleton variant="rounded" className="h-12 w-full bg-sacred-gray-200" />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
