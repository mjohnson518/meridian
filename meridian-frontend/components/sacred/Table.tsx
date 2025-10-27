'use client';

import { cn } from '@/lib/utils';

interface Column<T> {
  header: string;
  accessor: (row: T) => React.ReactNode;
  align?: 'left' | 'right' | 'center';
  className?: string;
}

interface TableProps<T> {
  columns: Column<T>[];
  data: T[];
  dense?: boolean;
  className?: string;
  onRowClick?: (row: T) => void;
}

export function SacredTable<T>({
  columns,
  data,
  dense = true,
  className,
  onRowClick,
}: TableProps<T>) {
  return (
    <div className={cn('w-full overflow-x-auto sacred-scrollbar', className)}>
      <table className={cn('w-full border-collapse', dense && 'text-sm')}>
        <thead className="border-b border-sacred-gray-200">
          <tr>
            {columns.map((col, index) => (
              <th
                key={index}
                className={cn(
                  'text-left font-mono uppercase text-xs tracking-wider',
                  'text-sacred-gray-600 font-medium',
                  dense ? 'p-2' : 'p-3',
                  col.align === 'right' && 'text-right',
                  col.align === 'center' && 'text-center',
                  col.className
                )}
              >
                {col.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {data.map((row, rowIndex) => (
            <tr
              key={rowIndex}
              className={cn(
                'border-b border-sacred-gray-100',
                onRowClick && 'cursor-pointer hover:bg-sacred-gray-50 transition-colors'
              )}
              onClick={() => onRowClick && onRowClick(row)}
            >
              {columns.map((col, colIndex) => (
                <td
                  key={colIndex}
                  className={cn(
                    dense ? 'p-2' : 'p-3',
                    'font-mono',
                    col.align === 'right' && 'text-right',
                    col.align === 'center' && 'text-center',
                    col.className
                  )}
                >
                  {col.accessor(row)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {data.length === 0 && (
        <div className="text-center py-8 text-sacred-gray-500">
          No data available
        </div>
      )}
    </div>
  );
}
