'use client';

import { forwardRef, SelectHTMLAttributes } from 'react';
import { cn } from '@/lib/utils';

interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

interface PortalSelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  error?: string;
  helperText?: string;
  required?: boolean;
  fullWidth?: boolean;
  options: SelectOption[];
  placeholder?: string;
}

export const PortalSelect = forwardRef<HTMLSelectElement, PortalSelectProps>(
  ({ label, error, helperText, required, fullWidth = true, options, placeholder, className, id, ...props }, ref) => {
    const selectId = id || label?.toLowerCase().replace(/\s+/g, '-');

    return (
      <div className={cn(fullWidth && 'w-full')}>
        {label && (
          <label
            htmlFor={selectId}
            className="block text-xs font-mono uppercase tracking-wider text-gray-400 mb-2"
          >
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}
        <div className="relative">
          <select
            ref={ref}
            id={selectId}
            className={cn(
              "w-full px-4 py-3 rounded-lg appearance-none",
              "bg-white/[0.02] backdrop-blur-sm",
              "border border-white/10",
              "text-white",
              "font-mono text-sm",
              "transition-all duration-200",
              "focus:outline-none focus:border-emerald-500/50 focus:ring-2 focus:ring-emerald-500/20",
              "disabled:opacity-50 disabled:cursor-not-allowed",
              error && "border-red-500/50 focus:border-red-500/50 focus:ring-red-500/20",
              className
            )}
            aria-invalid={!!error}
            aria-describedby={error ? `${selectId}-error` : helperText ? `${selectId}-helper` : undefined}
            {...props}
          >
            {placeholder && (
              <option value="" disabled className="bg-[#0B0C10] text-gray-500">
                {placeholder}
              </option>
            )}
            {options.map((option) => (
              <option
                key={option.value}
                value={option.value}
                disabled={option.disabled}
                className="bg-[#0B0C10]"
              >
                {option.label}
              </option>
            ))}
          </select>
          {/* Dropdown arrow */}
          <div className="absolute right-4 top-1/2 -translate-y-1/2 pointer-events-none">
            <svg
              className="w-4 h-4 text-gray-500"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 9l-7 7-7-7"
              />
            </svg>
          </div>
        </div>
        {error && (
          <p id={`${selectId}-error`} className="mt-2 text-xs text-red-400 font-mono" role="alert">
            {error}
          </p>
        )}
        {helperText && !error && (
          <p id={`${selectId}-helper`} className="mt-2 text-xs text-gray-500 font-mono">
            {helperText}
          </p>
        )}
      </div>
    );
  }
);

PortalSelect.displayName = 'PortalSelect';
