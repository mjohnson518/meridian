'use client';

import { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface Column<T> {
  header: string;
  accessor: (row: T) => ReactNode;
  align?: 'left' | 'center' | 'right';
  className?: string;
}

interface PortalTableProps<T> {
  columns: Column<T>[];
  data: T[];
  onRowClick?: (row: T) => void;
  emptyMessage?: string;
  className?: string;
  dense?: boolean;
}

export function PortalTable<T>({
  columns,
  data,
  onRowClick,
  emptyMessage = 'No data available',
  className,
  dense = false,
}: PortalTableProps<T>) {
  const alignClasses = {
    left: 'text-left',
    center: 'text-center',
    right: 'text-right',
  };

  return (
    <div className={cn("overflow-x-auto", className)}>
      <table className="w-full">
        <thead>
          <tr className="border-b border-white/10">
            {columns.map((column, index) => (
              <th
                key={index}
                className={cn(
                  "font-mono text-xs uppercase tracking-wider text-gray-500 font-medium",
                  dense ? "px-3 py-2" : "px-4 py-3",
                  alignClasses[column.align || 'left'],
                  column.className
                )}
              >
                {column.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {data.length === 0 ? (
            <tr>
              <td
                colSpan={columns.length}
                className="text-center py-12 text-gray-500 font-mono text-sm"
              >
                {emptyMessage}
              </td>
            </tr>
          ) : (
            data.map((row, rowIndex) => (
              <tr
                key={rowIndex}
                onClick={() => onRowClick?.(row)}
                className={cn(
                  "border-b border-white/5 transition-colors",
                  onRowClick && "cursor-pointer hover:bg-white/[0.02]",
                  rowIndex % 2 === 1 && "bg-white/[0.01]"
                )}
              >
                {columns.map((column, colIndex) => (
                  <td
                    key={colIndex}
                    className={cn(
                      dense ? "px-3 py-2" : "px-4 py-4",
                      alignClasses[column.align || 'left'],
                      column.className
                    )}
                  >
                    {column.accessor(row)}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}

// Table wrapper with card styling
interface PortalTableCardProps<T> extends PortalTableProps<T> {
  title?: string;
  action?: ReactNode;
}

export function PortalTableCard<T>({
  title,
  action,
  ...tableProps
}: PortalTableCardProps<T>) {
  return (
    <div className="rounded-2xl bg-white/[0.02] backdrop-blur-xl border border-white/10 overflow-hidden">
      {(title || action) && (
        <div className="flex items-center justify-between px-6 py-4 border-b border-white/5">
          {title && (
            <h3 className="text-sm font-mono uppercase tracking-wider text-gray-300">
              {title}
            </h3>
          )}
          {action}
        </div>
      )}
      <PortalTable {...tableProps} />
    </div>
  );
}
