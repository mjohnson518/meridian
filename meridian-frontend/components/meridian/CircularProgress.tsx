'use client';

import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';

interface CircularProgressProps {
  value: number;
  max?: number;
  size?: number;
  strokeWidth?: number;
  showValue?: boolean;
  label?: string;
  className?: string;
}

export function CircularProgress({
  value,
  max = 120,
  size = 120,
  strokeWidth = 8,
  showValue = true,
  label,
  className = '',
}: CircularProgressProps) {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    // Animate to value
    const timer = setTimeout(() => {
      setProgress(value);
    }, 100);
    return () => clearTimeout(timer);
  }, [value]);

  const radius = (size - strokeWidth) / 2;
  const circumference = radius * 2 * Math.PI;
  const offset = circumference - (progress / max) * circumference;

  // Determine color based on value
  const getColor = () => {
    if (value >= 100) return '#10B981'; // emerald
    if (value >= 95) return '#F59E0B'; // amber
    return '#EF4444'; // red
  };

  const color = getColor();

  return (
    <div className={`relative inline-flex flex-col items-center ${className}`}>
      <svg
        width={size}
        height={size}
        className="transform -rotate-90"
      >
        {/* Background circle */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          stroke="currentColor"
          strokeWidth={strokeWidth}
          fill="none"
          className="text-sacred-gray-200 dark:text-dark-border"
        />
        
        {/* Progress circle */}
        <motion.circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          stroke={color}
          strokeWidth={strokeWidth}
          fill="none"
          strokeLinecap="round"
          initial={{ strokeDashoffset: circumference }}
          animate={{ strokeDashoffset: offset }}
          transition={{ duration: 1.5, ease: "easeOut" }}
          style={{
            strokeDasharray: circumference,
          }}
        />
      </svg>

      {/* Center value */}
      {showValue && (
        <div className="absolute inset-0 flex flex-col items-center justify-center">
          <motion.div
            initial={{ scale: 0.8, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            transition={{ delay: 0.5, duration: 0.5 }}
            className="text-center"
          >
            <div className="text-2xl md:text-3xl font-mono font-medium tabular-nums" style={{ color }}>
              {value.toFixed(2)}%
            </div>
            {label && (
              <div className="text-xs font-mono uppercase text-sacred-gray-500 dark:text-dark-muted mt-1">
                {label}
              </div>
            )}
          </motion.div>
        </div>
      )}
    </div>
  );
}

