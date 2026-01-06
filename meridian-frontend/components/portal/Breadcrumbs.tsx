'use client';

import { usePathname } from 'next/navigation';
import { Fragment } from 'react';

interface BreadcrumbItem {
  label: string;
  href: string;
}

interface BreadcrumbsProps {
  items?: BreadcrumbItem[];
  className?: string;
}

const routeLabels: Record<string, string> = {
  portal: 'Portal',
  dashboard: 'Dashboard',
  mint: 'Mint',
  burn: 'Burn',
  compliance: 'Compliance',
  settings: 'Settings',
  onboarding: 'Onboarding',
};

export function Breadcrumbs({ items, className }: BreadcrumbsProps) {
  const pathname = usePathname();

  // Auto-generate breadcrumbs from pathname if items not provided
  const breadcrumbItems: BreadcrumbItem[] = items || generateBreadcrumbs(pathname);

  if (breadcrumbItems.length <= 1) {
    return null; // Don't show breadcrumbs for root pages
  }

  return (
    <nav
      aria-label="Breadcrumb"
      className={className}
    >
      <ol className="flex items-center space-x-2 text-xs font-mono uppercase tracking-wider">
        {breadcrumbItems.map((item, index) => {
          const isLast = index === breadcrumbItems.length - 1;

          return (
            <Fragment key={item.href}>
              <li>
                {isLast ? (
                  <span
                    className="text-sacred-gray-800"
                    aria-current="page"
                  >
                    {item.label}
                  </span>
                ) : (
                  <a
                    href={item.href}
                    className="text-sacred-gray-500 hover:text-sacred-gray-700 transition-colors"
                  >
                    {item.label}
                  </a>
                )}
              </li>
              {!isLast && (
                <li aria-hidden="true" className="text-sacred-gray-400">
                  /
                </li>
              )}
            </Fragment>
          );
        })}
      </ol>
    </nav>
  );
}

function generateBreadcrumbs(pathname: string): BreadcrumbItem[] {
  const segments = pathname.split('/').filter(Boolean);
  const breadcrumbs: BreadcrumbItem[] = [];

  let currentPath = '';

  for (const segment of segments) {
    currentPath += `/${segment}`;
    const label = routeLabels[segment] || capitalizeFirst(segment);

    breadcrumbs.push({
      label,
      href: currentPath,
    });
  }

  return breadcrumbs;
}

function capitalizeFirst(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1).replace(/-/g, ' ');
}

// Export a hook for programmatic breadcrumb access
export function useBreadcrumbs(): BreadcrumbItem[] {
  const pathname = usePathname();
  return generateBreadcrumbs(pathname);
}
