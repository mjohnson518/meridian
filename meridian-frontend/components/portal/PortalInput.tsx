'use client';

import { forwardRef, InputHTMLAttributes, TextareaHTMLAttributes } from 'react';
import { cn } from '@/lib/utils';

interface PortalInputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  helperText?: string;
  required?: boolean;
  fullWidth?: boolean;
}

export const PortalInput = forwardRef<HTMLInputElement, PortalInputProps>(
  ({ label, error, helperText, required, fullWidth = true, className, id, ...props }, ref) => {
    const inputId = id || label?.toLowerCase().replace(/\s+/g, '-');

    return (
      <div className={cn(fullWidth && 'w-full')}>
        {label && (
          <label
            htmlFor={inputId}
            className="block text-xs font-mono uppercase tracking-wider text-gray-400 mb-2"
          >
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}
        <input
          ref={ref}
          id={inputId}
          className={cn(
            "w-full px-4 py-3 rounded-lg",
            "bg-white/[0.08] backdrop-blur-sm",
            "border border-white/30",
            "text-white placeholder-gray-400",
            "font-mono text-sm",
            "transition-all duration-200",
            "focus:outline-none focus:border-emerald-500/50 focus:ring-2 focus:ring-emerald-500/20",
            "disabled:opacity-50 disabled:cursor-not-allowed",
            error && "border-red-500/50 focus:border-red-500/50 focus:ring-red-500/20",
            className
          )}
          aria-invalid={!!error}
          aria-describedby={error ? `${inputId}-error` : helperText ? `${inputId}-helper` : undefined}
          {...props}
        />
        {error && (
          <p id={`${inputId}-error`} className="mt-2 text-xs text-red-400 font-mono" role="alert">
            {error}
          </p>
        )}
        {helperText && !error && (
          <p id={`${inputId}-helper`} className="mt-2 text-xs text-gray-500 font-mono">
            {helperText}
          </p>
        )}
      </div>
    );
  }
);

PortalInput.displayName = 'PortalInput';

interface PortalTextareaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  error?: string;
  helperText?: string;
  required?: boolean;
  fullWidth?: boolean;
}

export const PortalTextarea = forwardRef<HTMLTextAreaElement, PortalTextareaProps>(
  ({ label, error, helperText, required, fullWidth = true, className, id, rows = 4, ...props }, ref) => {
    const textareaId = id || label?.toLowerCase().replace(/\s+/g, '-');

    return (
      <div className={cn(fullWidth && 'w-full')}>
        {label && (
          <label
            htmlFor={textareaId}
            className="block text-xs font-mono uppercase tracking-wider text-gray-400 mb-2"
          >
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}
        <textarea
          ref={ref}
          id={textareaId}
          rows={rows}
          className={cn(
            "w-full px-4 py-3 rounded-lg",
            "bg-white/[0.08] backdrop-blur-sm",
            "border border-white/30",
            "text-white placeholder-gray-400",
            "font-mono text-sm",
            "transition-all duration-200",
            "focus:outline-none focus:border-emerald-500/50 focus:ring-2 focus:ring-emerald-500/20",
            "disabled:opacity-50 disabled:cursor-not-allowed",
            "resize-none",
            error && "border-red-500/50 focus:border-red-500/50 focus:ring-red-500/20",
            className
          )}
          aria-invalid={!!error}
          aria-describedby={error ? `${textareaId}-error` : helperText ? `${textareaId}-helper` : undefined}
          {...props}
        />
        {error && (
          <p id={`${textareaId}-error`} className="mt-2 text-xs text-red-400 font-mono" role="alert">
            {error}
          </p>
        )}
        {helperText && !error && (
          <p id={`${textareaId}-helper`} className="mt-2 text-xs text-gray-500 font-mono">
            {helperText}
          </p>
        )}
      </div>
    );
  }
);

PortalTextarea.displayName = 'PortalTextarea';

// Large amount input for financial values
interface PortalAmountInputProps extends Omit<PortalInputProps, 'type'> {
  currency?: string;
  onCurrencyChange?: (currency: string) => void;
  currencies?: { code: string; name: string }[];
}

export const PortalAmountInput = forwardRef<HTMLInputElement, PortalAmountInputProps>(
  ({ label, error, currency, onCurrencyChange, currencies, className, ...props }, ref) => {
    return (
      <div className="w-full">
        {label && (
          <label className="block text-xs font-mono uppercase tracking-wider text-gray-400 mb-2">
            {label}
          </label>
        )}
        <div className="flex gap-3">
          <input
            ref={ref}
            type="number"
            className={cn(
              "flex-1 px-6 py-4 rounded-lg",
              "bg-white/[0.08] backdrop-blur-sm",
              "border border-white/30",
              "text-white placeholder-gray-400",
              "font-mono text-3xl text-right",
              "transition-all duration-200",
              "focus:outline-none focus:border-emerald-500/50 focus:ring-2 focus:ring-emerald-500/20",
              "[appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none",
              error && "border-red-500/50",
              className
            )}
            {...props}
          />
          {currencies && onCurrencyChange && (
            <select
              value={currency}
              onChange={(e) => onCurrencyChange(e.target.value)}
              className={cn(
                "px-4 py-2 rounded-lg",
                "bg-white/[0.08] backdrop-blur-sm",
                "border border-white/30",
                "text-white",
                "font-mono text-sm",
                "transition-all duration-200",
                "focus:outline-none focus:border-emerald-500/50 focus:ring-2 focus:ring-emerald-500/20"
              )}
            >
              {currencies.map((c) => (
                <option key={c.code} value={c.code} className="bg-[#0B0C10]">
                  {c.code}
                </option>
              ))}
            </select>
          )}
        </div>
        {error && (
          <p className="mt-2 text-xs text-red-400 font-mono" role="alert">
            {error}
          </p>
        )}
      </div>
    );
  }
);

PortalAmountInput.displayName = 'PortalAmountInput';
